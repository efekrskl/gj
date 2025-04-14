use crate::capitalize_first::capitalize_first;
use crate::config::GJ_TITLE_MARKER;
use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::{Value, json};

pub struct NotionClient {
    client: reqwest::Client,
    api_url: String,
}

impl NotionClient {
    pub fn new(token: String) -> Self {
        let mut headers = reqwest::header::HeaderMap::with_capacity(3);
        headers.insert(
            "Authorization",
            format!("Bearer {}", token)
                .parse()
                .expect("Invalid token format"),
        );
        headers.insert(
            "Notion-Version",
            "2022-06-28".parse().expect("Invalid Notion API version"),
        );
        headers.insert(
            "Content-Type",
            "application/json"
                .parse()
                .expect("Invalid Content-Type header"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client");

        NotionClient {
            client,
            api_url: "https://api.notion.com".to_string(),
        }
    }

    pub async fn create_page(&self, page_title: &str, database_id: &str) -> Result<String> {
        let timestamp = Utc::now().to_rfc3339();

        let payload = json!({
            "parent": { "database_id": database_id },
            "properties": {
                "Name": {
                    "title": [
                        {
                            "type": "text",
                            "text": {
                                "content": page_title,
                            }
                        }
                    ]
                },
                "Date": { "date": { "start": timestamp }},
            },
        });

        let response = self
            .client
            .post(format!("{}/v1/pages", self.api_url))
            .json(&payload)
            .send()
            .await
            .context("Failed to send create page request")?;

        if !response.status().is_success() {
            let error = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown Error".to_string());
            anyhow::bail!("Failed to create page: {}", error);
        }

        let response_json: Value = response
            .json()
            .await
            .context("Failed to parse response as JSON")?;

        let id = response_json["id"]
            .as_str()
            .context("Response missing page ID")?
            .to_string();

        Ok(id)
    }

    pub async fn get_page_id_by_title(&self, title: &str, database_id: &str) -> Option<String> {
        let payload = json!({
            "filter": {
            "property": "Name",
            "title": {
                "equals": title
            }
        }
        });

        let response = self
            .client
            .post(format!(
                "{}/v1/databases/{}/query",
                self.api_url, database_id
            ))
            .json(&payload)
            .send()
            .await
            .expect("Failed request during get_page_id_by_title");

        if !response.status().is_success() {
            let error = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown Error".to_string());
            eprintln!("❌ Failed to get page ID: {}", error);
            return None;
        }

        let response_json: Value = response
            .json()
            .await
            .expect("Failed to parse response as JSON");

        let results = response_json["results"].as_array().unwrap();
        if results.is_empty() {
            return None;
        }

        let page_id = results[0]["id"]
            .as_str()
            .expect("Response missing page ID")
            .to_string();
        Some(page_id)
    }

    pub async fn add_entries(&self, page_id: &str, entries: Vec<String>) -> Result<()> {
        let entries_property = entries
            .iter()
            .map(|message| {
                json!({
                    "type": "bulleted_list_item",
                    "bulleted_list_item": {
                        "rich_text": [
                            {
                                "type": "text",
                                "text": {
                                    "content": message,
                                }
                            }
                        ]
                    }
                })
            })
            .collect::<Vec<Value>>();

        let payload = json!({
            "children": entries_property,
        });

        let res = self
            .client
            .patch(format!("{}/v1/blocks/{}/children", self.api_url, page_id))
            .json(&payload)
            .send()
            .await;

        match res {
            Ok(r) if r.status().is_success() => (),
            Ok(r) => {
                let err = r.text().await.unwrap_or_default();
                eprintln!("❌ Sync failed: {}", err);
            }
            Err(e) => eprintln!("❌ Error: {}", e),
        }

        Ok(())
    }

    pub async fn create_gj_database(&self, root_page_id: &str) -> Result<String> {
        let payload = json!({
            "is_inline": true,
            "parent": {
                "type": "page_id",
                "page_id": root_page_id
            },
            "title": [{
                "type": "text",
                "text": { "content": GJ_TITLE_MARKER }
            }],
            "properties": {
                "Name": { "title": {} },
                "Date": { "date": {} },
                "Highlights": { "rich_text": {} },
                "Tag": {
                    "multi_select": {
                        "options": [
                            { "name": "Momentum", "color": "green" },
                            { "name": "Burnout", "color": "gray" },
                            { "name": "Learning", "color": "pink" },
                            { "name": "Deep Work", "color": "orange" },
                            { "name": "Heavy Context Switching", "color": "purple" },
                            { "name": "Blocked", "color": "yellow" },
                            { "name": "Planning", "color": "default" },
                            { "name": "Collaboration", "color": "brown" },
                            { "name": "Maintenance", "color": "blue" },
                            { "name": "Building", "color": "red" }
                        ]
                    }
                },
            },
        });

        let response = self
            .client
            .post(format!("{}/v1/databases", self.api_url))
            .json(&payload)
            .send()
            .await
            .context("Failed to send create database request")?;

        if !response.status().is_success() {
            let err_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to create database: {}", err_text);
        }

        let json: Value = response
            .json()
            .await
            .context("Failed to parse database creation response")?;
        let db_id = json["id"]
            .as_str()
            .context("Missing database ID from response")?
            .to_string();

        Ok(db_id)
    }

    pub async fn find_gj_database_by_title(&self) -> Result<Option<String>> {
        let payload = json!({
            "filter": {
                "value": "database",
                "property": "object"
            }
        });

        let res = self
            .client
            .post(format!("{}/v1/search", self.api_url))
            .json(&payload)
            .send()
            .await
            .context("Failed to search for GJ database")?;

        let json: Value = res
            .json()
            .await
            .context("Failed to parse search response")?;

        let default_vec = vec![];
        let results = json["results"].as_array().unwrap_or(&default_vec);

        for db in results {
            let title_text = db["title"]
                .get(0)
                .and_then(|t| t["plain_text"].as_str())
                .unwrap_or("");

            if title_text == GJ_TITLE_MARKER {
                if let Some(id) = db["id"].as_str() {
                    return Ok(Some(id.to_string()));
                }
            }
        }

        Ok(None)
    }

    pub async fn get_tags_from_page(&self, page_id: &str) -> Result<Vec<String>> {
        let response = self
            .client
            .get(format!("{}/v1/pages/{}", self.api_url, page_id))
            .send()
            .await
            .context("Failed to fetch page")?;

        if !response.status().is_success() {
            let err = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to get page tags: {}", err);
        }

        let json: Value = response.json().await.context("Failed to parse page JSON")?;

        let tags = json["properties"]["Tag"]["multi_select"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v["name"].as_str().map(|s| s.to_string()))
            .collect::<Vec<String>>();

        Ok(tags)
    }

    pub async fn add_tags_to_page(&self, page_id: &str, tags: Vec<String>) -> Result<()> {
        if tags.is_empty() {
            return Ok(());
        }

        let mut existing_tags = self.get_tags_from_page(page_id).await?;
        existing_tags.extend(tags);

        let multi_select = existing_tags
            .into_iter()
            .map(|tag| json!({ "name": capitalize_first(&tag) }))
            .collect::<Vec<Value>>();

        let payload = json!({
            "properties": {
                "Tag": {
                    "multi_select": multi_select
                }
            }
        });

        let res = self
            .client
            .patch(format!("{}/v1/pages/{}", self.api_url, page_id))
            .json(&payload)
            .send()
            .await
            .context("Failed to update page tags")?;

        if !res.status().is_success() {
            let err = res
                .text()
                .await
                .unwrap_or_else(|_| "Unknown Error".to_string());
            anyhow::bail!("❌ Failed to update tags: {}", err);
        }

        Ok(())
    }
}
