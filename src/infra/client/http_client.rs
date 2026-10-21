use reqwest::{Client, Response};
use std::time::Duration;

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        Self { client }
    }

    pub async fn get(&self, url: &str) -> Result<Response, reqwest::Error> {
        let response = self.client.get(url).send().await?;
        Ok(response)
    }

    pub async fn post(&self, url: &str, json_payload: &str) -> Result<Response, reqwest::Error> {
        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .body(json_payload.to_string())
            .send()
            .await?;

        Ok(response)
    }

    pub async fn put() {}
    pub async fn delete() {}
    pub async fn query() {}
}
