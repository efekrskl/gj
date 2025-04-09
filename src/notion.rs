use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::{Value, json};

pub struct NotionClient {
    client: reqwest::Client,
    database_id: String,
    api_url: String,
}

impl NotionClient {
    pub fn new(token: String, database_id: String) -> Self {
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
            database_id,
            api_url: "https://api.notion.com".to_string(),
        }
    }

    pub async fn create_page(&self, title: String) -> Result<String> {
        let timestamp = Utc::now().to_rfc3339();

        let payload = json!({
            "parent": { "database_id": self.database_id },
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

    pub async fn get_page_id_by_title(&self, title: &str) -> Option<String> {
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
                self.api_url, self.database_id
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

        let entry_logs = entry
            .split("--")
            .map(|message| {
                json!({
                    "type": "paragraph",
                    "paragraph": {
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
}
