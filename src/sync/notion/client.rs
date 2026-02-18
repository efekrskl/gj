use crate::AppContext;
use crate::sync::notion::adapter_config::{NotionAdapterConfig, NotionAdapterConfigRaw};
use crate::sync::notion::types::{
    CreatePageParent, CreatePageProperties, CreatePageRequest, DateProperty, DateValue, IdResponse,
    RichText, RichTextType, TextContent, TitleProperty,
};
use anyhow::{Context, Result, bail};
use chrono::Utc;
use log::debug;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderValue};

pub struct NotionClient {
    client: reqwest::Client,
    base_url: String,
}

impl NotionClient {
    pub fn new(api_key: Option<String>) -> Result<Self> {
        // todo: improve the message and call for action
        let api_key = api_key.context("Notion API key is missing.")?;

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

    fn create_notion_db() {
        todo!()
    }

    pub async fn create_page(&self, page_title: &str, database_id: &str) -> Result<String> {
        let timestamp = Utc::now().to_rfc3339();

        let request = CreatePageRequest {
            parent: CreatePageParent { database_id },
            properties: CreatePageProperties {
                name: TitleProperty {
                    title: vec![RichText {
                        kind: RichTextType::Text,
                        text: TextContent {
                            content: page_title,
                        },
                    }],
                },
                date: DateProperty {
                    date: DateValue { start: &timestamp },
                },
            },
        };

        let response = self
            .client
            .post(format!("{}/v1/pages", self.base_url))
            .json(&request)
            .send()
            .await
            .context("Failed to send page creation request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown Error".to_string());

            bail!("Notion create_page failed ({}): {}", status, body);
        }

        let created: IdResponse = response
            .json()
            .await
            .context("Failed to parse create page response JSON")?;

        Ok(created.id)
    }

    fn get_adapter_config(&self, ctx: &AppContext) -> Result<NotionAdapterConfig> {
        let raw_notion_adapter_config = ctx.db.get_raw_sync_adapter_config("notion")?;

        debug!("raw_notion_adapter_config {:?}", raw_notion_adapter_config);

        match raw_notion_adapter_config {
            Some(config_json_str) => {
                NotionAdapterConfigRaw::from_json(&config_json_str)
            },
            None => bail!("Notion adapter config was not found."),
        }
    }

    pub async fn push(&self, ctx: &AppContext, date: &str, content: &str) -> Result<()> {
        let notion_adapter_config = self.get_adapter_config(&ctx)?;

        // todo: check if the page exists first
        debug!("[push] pushing to notion key: {}", date);

        let result = self.create_page(date, &notion_adapter_config.database_id).await?;

        Ok(())
    }
}
