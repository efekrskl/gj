use anyhow::{Context, Result};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderValue};

pub struct NotionClient {
    client: reqwest::Client,
    base_url: String,
}

impl NotionClient {
    pub fn new(api_key: String) -> Result<Self> {
        let mut headers = reqwest::header::HeaderMap::with_capacity(3);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_key))
                .context("Failed to parse Notion api key.")?,
        );
        headers.insert("Notion-Version", HeaderValue::from_static("2025-09-03"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to build reqwest client.")?;

        Ok(NotionClient {
            client,
            base_url: "https://api.notion.com".to_string(),
        })
    }

    pub fn push() {
    }
}
