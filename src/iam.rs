// Copyright 2022 Mathew Odden <mathewrodden@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json;
use tracing::debug;
use tracing_subscriber;
use url::form_urlencoded;

pub struct Client {
    api_key: String,
    token: Arc<Mutex<Option<Token>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
    pub refresh_token: String,
    pub expiry: Instant,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<TokenResponse> for Token {
    fn from(tr: TokenResponse) -> Self {
        Token {
            access_token: tr.access_token,
            token_type: tr.token_type,
            refresh_token: tr.refresh_token.unwrap_or_else(|| "".to_string()),
            expiry: Instant::now() + Duration::from_secs(tr.expires_in.unwrap_or_else(|| 1200)),
        }
    }
}

impl Token {
    pub fn valid(&self) -> bool {
        Instant::now().checked_duration_since(self.expiry).is_none()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
}

impl Client {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            token: Arc::new(Mutex::new(None)),
        }
    }

    pub fn token(&self) -> Result<Token, Box<dyn std::error::Error>> {
        let mut token = self.token.lock().unwrap();

        if let Some(t) = token.clone() {
            if t.valid() {
                return Ok(t);
            }
        }

        *token = Some(self.request_token());

        Ok(token.as_ref().unwrap().clone())
    }

    fn request_token(&self) -> Token {
        let encoded: String = form_urlencoded::Serializer::new(String::new())
            .append_pair("grant_type", "urn:ibm:params:oauth:grant-type:apikey")
            .append_pair("apikey", &self.api_key)
            .finish();

        let c = reqwest::blocking::Client::new();

        let resp = c
            .post("https://iam.cloud.ibm.com/identity/token")
            .header("Authorization", "Basic Yng6Yng=")
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(encoded)
            .send()
            .expect("Get token failed");

        let text = resp.text().expect("Getting body text failed");
        let token_resp: TokenResponse = serde_json::from_str(&text).unwrap();

        token_resp.into()
    }
}

impl Default for Client {
    fn default() -> Self {
        let env_key = "IBMCLOUD_API_KEY";
        let api_key = match std::env::var(env_key) {
            Ok(k) => k,
            _ => {
                panic!("'IBMCLOUD_API_KEY' not set or invalid");
            }
        };

        Self::new(&api_key)
    }
}

pub fn main() {
    tracing_subscriber::fmt::init();

    let iam = Client::default();

    let token = iam.token().unwrap();
    debug!("Token: {:?}", token);
    println!("AccessToken: {}", token.access_token);
}

#[cfg(test)]
mod tests {
    use super::{Client, Token};

    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    fn get_test_token() -> Token {
        let access_token = String::from("");
        let refresh_token = String::from("");
        let token_type = String::from("test");

        Token {
            access_token,
            refresh_token,
            token_type,
            expiry: Instant::now() + Duration::from_secs(1200),
        }
    }

    #[test]
    fn token_expiry() {
        let mut token = get_test_token();
        token.expiry = Instant::now() + Duration::from_secs(10);
        assert!(token.valid());

        token.expiry = Instant::now() - Duration::from_secs(10);
        assert!(!token.valid());
    }

    #[test]
    fn token_caching() {
        let iam = Client::new("".into());
        *iam.token.lock().unwrap() = Some(get_test_token());

        let token = iam.token().unwrap();
        let token2 = iam.token().unwrap();
        assert_eq!(token, token2);
    }

    #[test]
    fn threadsafe_cache() {
        let iam = Client::new("".into());
        *iam.token.lock().unwrap() = Some(get_test_token());

        let c = Arc::new(iam);
        let c1 = c.clone();
        let c2 = c.clone();

        let t1 = thread::spawn(move || c1.token().unwrap());

        let t2 = thread::spawn(move || c2.token().unwrap());

        let res1 = t1.join().unwrap();
        let res2 = t2.join().unwrap();

        assert_eq!(res1, res2);
    }
}
