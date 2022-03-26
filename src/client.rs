use serde::{Deserialize, Serialize};
use serde_json;
use tracing::debug;

use crate::iam;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const DEFAULT_ENDPOINT: &str = "https://xenobuilds.mattbuilt.com";
const DEFAULT_EU_ENDPOINT: &str = "https://hapctl-eu.kp-ops.net";

#[derive(Debug, Clone)]
struct InvalidServerNameError;

impl std::fmt::Display for InvalidServerNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid server name given")
    }
}

impl std::error::Error for InvalidServerNameError {}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct SetWeightRequest {
    weight: u32,
    reason: String,
}

pub struct Client {
    endpoint: String,
}

impl Client {
    pub fn new(servername: &str, endpoint: Option<&str>) -> Self {
        match endpoint {
            Some(e) => Client { endpoint: e.into() },
            None => {
                if servername.contains("eu-de") {
                    Client {
                        endpoint: DEFAULT_EU_ENDPOINT.into(),
                    }
                } else {
                    Client {
                        endpoint: DEFAULT_ENDPOINT.into(),
                    }
                }
            }
        }
    }

    pub fn get_weight(&self, server_name: &str) -> Result<String> {
        let parts: Vec<&str> = server_name.split("/").collect();
        if parts.len() != 2 {
            return Err(InvalidServerNameError.into());
        }

        let token = iam::Client::default().token()?;

        let uri = format!(
            "{}/v1/backends/{}/servers/{}/weight",
            self.endpoint, parts[0], parts[1]
        );

        let c = reqwest::blocking::Client::new();
        let body = c
            .get(uri)
            .header("Authorization", format!("Bearer {}", token.access_token))
            .send()?
            .text()?;

        debug!("body: {:?}", body);
        Ok(body)
    }

    pub fn set_weight(&self, server_name: &str, weight: u32, reason: &str) -> Result<String> {
        let parts: Vec<&str> = server_name.split("/").collect();
        if parts.len() != 2 {
            return Err(InvalidServerNameError.into());
        }

        let token = iam::Client::default().token()?;

        let uri = format!(
            "{}/v1/backends/{}/servers/{}/weight",
            self.endpoint, parts[0], parts[1]
        );
        let reqdata = SetWeightRequest {
            weight,
            reason: reason.to_string(),
        };

        let request = serde_json::to_string(&reqdata)?;

        let c = reqwest::blocking::Client::new();
        let body = c
            .post(uri)
            .header("Authorization", format!("Bearer {}", token.access_token))
            .header("Content-Type", "application/json")
            .body(request)
            .send()?
            .text()?;

        debug!("body: {:?}", body);
        Ok(body.into())
    }
}
