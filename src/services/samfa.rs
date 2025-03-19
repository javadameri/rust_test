use reqwest::{header::HeaderValue, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use anyhow::{Result, Context};


#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub success: bool,
    pub data: serde_json::Value, // برای پردازش داده‌های متغیر JSON
}

pub struct ApiClient {
    client: Client,
    base_url: String,
    token:String
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://seller.samfaa.ir/api/v1/".to_string(),
            token: "token".to_string(),
        }
    }

    pub async fn get_kinds(&self) -> Result<Value> { 
        let url = format!("{}{}", self.base_url, "show/kind");

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::ACCEPT, HeaderValue::from_static("application/json"));

        let auth_header = HeaderValue::from_str(&self.token)
            .context("Invalid Authorization Token")?;
        headers.insert(reqwest::header::AUTHORIZATION, auth_header);

        let response = self.client.get(url).headers(headers).send().await
            .context("Failed to send request")?;

        let json = response.json::<Value>().await
            .context("Failed to parse JSON response")?;

        Ok(json)
    }
}
