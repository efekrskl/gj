use crate::AppContext;
use crate::sync::notion::types::{
    Block, BlockType, BulletedListItem, CreatePageParent, CreatePageProperties, CreatePageRequest,
    DateProperty, DateValue, IdResponse, RichText, RichTextType, TextContent, TitleProperty,
};
use anyhow::{Context, Result, bail};
use log::debug;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderValue};

pub struct NotionClient {
    client: reqwest::Client,
    base_url: String,
    database_id: String,
}

impl NotionClient {
    pub fn new(api_key: String, database_id: String) -> Result<Self> {
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
            base_url: "https://api.notion.com".to_string(),
            client,
            database_id
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
        debug!(
            "[notion] create_page start title={} date={} content_len={}",
            page_title,
            date,
            content.len()
        );
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
        debug!("[notion] create_page success page_id={}", created.id);

        Ok(created.id)
    }

    pub async fn create(&self, date: &str, content: &str) -> Result<String> {
        debug!(
            "[notion] create start day_key={} content_len={}",
            date,
            content.len()
        );

        self.create_page(date, date, content, &self.database_id)
            .await
    }

    pub async fn update(
        &self,
        _ctx: &AppContext,
        remote_key: &str,
        content: &str,
    ) -> Result<String> {
        debug!(
            "[notion] update start remote_key={} content_len={}",
            remote_key,
            content.len()
        );
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
        debug!("[notion] update erase_content success remote_key={}", remote_key);

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
        debug!("[notion] update append success remote_key={}", remote_key);

        Ok(remote_key.to_string())
    }
}
