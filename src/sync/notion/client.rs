use crate::AppContext;
use crate::sync::notion::adapter_config::{NotionAdapterConfig, NotionAdapterConfigRaw};
use crate::sync::notion::types::{
    Block, BlockType, BulletedListItem, CreatePageParent, CreatePageProperties, CreatePageRequest,
    DateProperty, DateValue, IdResponse, RichText, RichTextType, TextContent, TitleProperty,
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

    pub async fn create_page(
        &self,
        page_title: &str,
        date: &str,
        content: &str,
        database_id: &str,
    ) -> Result<String> {
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
                    date: DateValue { start: &date },
                },
            },
            children: vec![Block {
                kind: BlockType::BulletedListItem,
                bulleted_list_item: BulletedListItem {
                    rich_text: vec![RichText {
                        kind: RichTextType::Text,
                        text: TextContent { content },
                    }],
                },
            }],
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
            Some(config_json_str) => NotionAdapterConfigRaw::from_json(&config_json_str),
            None => bail!("Notion adapter config was not found."),
        }
    }

    pub async fn create(&self, ctx: &AppContext, date: &str, content: &str) -> Result<String> {
        let notion_adapter_config = self.get_adapter_config(&ctx)?;

        // todo: check if the page exists first
        debug!(
            "[push] pushing to notion key: {}, content length: {}",
            date,
            content.len()
        );

        self.create_page(date, date, content, &notion_adapter_config.database_id)
            .await
    }

    pub async fn update(
        &self,
        _ctx: &AppContext,
        remote_key: &str,
        content: &str,
    ) -> Result<String> {
        // Erase existing content, properties etc. will be preserved so the user can keep some metadata safely
        let erase_response = self
            .client
            .patch(format!("{}/v1/pages/{}", self.base_url, remote_key))
            .json(&serde_json::json!({ "erase_content": true }))
            .send()
            .await
            .context("Failed to send erase_content request")?;

        if !erase_response.status().is_success() {
            let status = erase_response.status();
            let body = erase_response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown Error".to_string());

            bail!("Notion erase_content failed ({}): {}", status, body);
        }

        // Append the new content
        let append_response = self
            .client
            .patch(format!(
                "{}/v1/blocks/{}/children",
                self.base_url, remote_key
            ))
            .json(&serde_json::json!({
                "children": [
                    {
                        "object": "block",
                        "type": "paragraph",
                        "paragraph": {
                            "rich_text": [
                                {
                                    "type": "text",
                                    "text": { "content": content }
                                }
                            ]
                        }
                    }
                ]
            }))
            .send()
            .await
            .context("Failed to send append children request")?;

        if !append_response.status().is_success() {
            let status = append_response.status();
            let body = append_response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown Error".to_string());

            bail!("Notion append children failed ({}): {}", status, body);
        }

        Ok(remote_key.to_string())
    }
}
