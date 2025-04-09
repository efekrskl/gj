use crate::emoji::apply_emoji_prefix;
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

    pub async fn create_page(&self, title: String, database_id: &str) -> Result<String> {
        let timestamp = Utc::now().to_rfc3339();

        let payload = json!({
            "parent": { "database_id": database_id },
            "properties": {
                "Name": {
                    "title": [
                        {
                            "type": "text",
                            "text": {
                                "content": title,
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

        println!("✅ Page created with ID: {}", id);
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
            todo!()
        }

        let response_json: Value = response
            .json()
            .await
            .expect("Failed to parse response as JSON");

        let results = response_json["results"].as_array().unwrap();
        if results.is_empty() {
            println!("No page found with title: {}", title);
            return None;
        }

        let page_id = results[0]["id"]
            .as_str()
            .expect("Response missing page ID")
            .to_string();
        println!("✅ Found page ID: {}", page_id);
        Some(page_id)
    }

    pub async fn get_last_page_header_block_content(&self, page_id: &str) -> Option<String> {
        let res = self
            .client
            .get(format!("{}/v1/blocks/{}/children", self.api_url, page_id))
            .send()
            .await;

        match res {
            Ok(r) if r.status().is_success() => {
                let response: Value = r.json().await.unwrap();

                let results = response["results"].as_array().unwrap();

                if results.is_empty() {
                    None
                } else {
                    let headers = results.iter().filter(|block| block["type"] == "heading_1");
                    let last_header = headers.last();

                    let last_header_content = last_header?
                        .get("heading_1")?
                        .get("rich_text")?
                        .get(0)?
                        .get("text")?
                        .get("content")?
                        .as_str()
                        .map(|s| s.to_string());

                    if let Some(content) = last_header_content {
                        println!("Found last header content: {}", content);
                        Some(content)
                    } else {
                        None
                    }
                }
            }
            Ok(r) => {
                let err = r.text().await.unwrap_or_default();
                eprintln!("❌ Sync failed: {}", err);

                None
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);

                None
            }
        }
    }

    pub async fn append_entry(
        &self,
        page_id: &str,
        entry: String,
        header: Option<String>,
    ) -> Result<()> {
        let mut logs = vec![];
        if header.is_some() {
            logs.push(json!({
                "type": "heading_1",
                "heading_1": {
                    "rich_text": [
                        {
                            "type": "text",
                            "text": {
                                "content": header,
                            }
                        }
                    ]
                }
            }))
        }

        let entry_logs = entry.split(";").map(|message| {
            json!({
                "type": "bulleted_list_item",
                "bulleted_list_item": {
                    "rich_text": [
                        {
                            "type": "text",
                            "text": {
                                "content": apply_emoji_prefix(message),
                            }
                        }
                    ]
                }
            })
        });

        logs.extend(entry_logs);

        let payload = json!({
            "children": logs
        });

        let res = self
            .client
            .patch(format!("{}/v1/blocks/{}/children", self.api_url, page_id))
            .json(&payload)
            .send()
            .await;

        match res {
            Ok(r) if r.status().is_success() => println!("✅ Messages appended to page."),
            Ok(r) => {
                let err = r.text().await.unwrap_or_default();
                eprintln!("❌ Sync failed: {}", err);
            }
            Err(e) => eprintln!("❌ Error: {}", e),
        }

        Ok(())
    }

    // todo use a single function to create a page
    pub async fn create_parent_page(&self) -> Result<String> {
        let payload = json!({
            "parent": { "type": "workspace", "workspace": true },
            "properties": {
                "title": [
                    {
                        "type": "text",
                        "text": { "content": "Work Journal" }
                    }
                ]
            }
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

        println!("✅ Page created with ID: {}", id);
        Ok(id)
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
                "text": { "content": "Work Log (gj)" }
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

        let json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse database creation response")?;
        let db_id = json["id"]
            .as_str()
            .context("Missing database ID from response")?
            .to_string();

        println!("✅ Created GJ database with ID: {}", db_id);
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

            if title_text == "Work Log (gj)" {
                if let Some(id) = db["id"].as_str() {
                    println!("✅ Found GJ database by title: {}", id);
                    return Ok(Some(id.to_string()));
                }
            }
        }

        Ok(None)
    }
}
