// Copyright 2019 Mathew Odden <mathewrodden@gmail.com>
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

use serde::{Deserialize, Serialize};
use serde_json;
use tracing_subscriber;
use url::form_urlencoded;

pub struct IAM {
    api_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
}

impl IAM {
    pub fn new(api_key: &str) -> Self {
        IAM {
            api_key: api_key.to_string(),
        }
    }

    pub fn token(&mut self) -> String {
        self.request_token().access_token
    }

    fn request_token(&mut self) -> TokenResponse {
        let encoded: String = form_urlencoded::Serializer::new(String::new())
            .append_pair("grant_type", "urn:ibm:params:oauth:grant-type:apikey")
            .append_pair("apikey", &self.api_key)
            .finish();

        let c = reqwest::blocking::Client::new();

        let resp = c.post("https://iam.cloud.ibm.com/identity/token")
            .header("Authorization", "Basic Yng6Yng=")
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(encoded)
            .send()
            .expect("Get token failed");

        let text = resp.text().expect("Getting body text failed");
        let token_resp: TokenResponse = serde_json::from_str(&text).unwrap();
        token_resp
    }
}

impl Default for IAM {
    fn default() -> Self {
        let env_key = "IBMCLOUD_API_KEY";
        let api_key = match std::env::var(env_key) {
            Ok(k) => k,
            _ => {
                panic!("'IBMCLOUD_API_KEY' not set or invalid");
            }
        };

        Self { api_key }
    }
}

pub fn main() {
    tracing_subscriber::fmt::init();

    println!("AccessToken: {}", IAM::default().token());
}
