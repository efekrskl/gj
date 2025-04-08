use chrono::Utc;
use serde_json::{Value, json};
use std::fmt::Pointer;

pub struct NotionClient {
    client: reqwest::Client,
    token: String,
    database_id: String,
    api_url: String,
}

#[derive(serde::Deserialize, Debug)]
struct SearchByTitleResult {
    id: String,
}

#[derive(serde::Deserialize, Debug)]
struct SearchByTitleResponse {
    results: Vec<SearchByTitleResult>,
}

impl NotionClient {
    pub fn new(token: String, database_id: String) -> Self {
        let client = reqwest::Client::new();

        NotionClient {
            client,
            token,
            database_id,
            api_url: "https://api.notion.com".to_string(),
        }
    }

    pub async fn create_page(&self, title: String, messages: Vec<String>) {
        let messages: Value = messages
            .into_iter()
            .map(|message| {
                json!({
                    "object": "block",
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
            })
            .collect();

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
            "children": messages,
        });

        let res = self
            .client
            .post(format!("{}{}", self.api_url, "/v1/pages"))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .header("Notion-Version", "2022-06-28")
            .json(&payload)
            .send()
            .await;

        match res {
            Ok(r) if r.status().is_success() => println!("✅ Log synced to Notion."),
            Ok(r) => {
                let err = r.text().await.unwrap_or_default();
                eprintln!("❌ Sync failed: {}", err);
            }
            Err(e) => eprintln!("❌ Error: {}", e),
        }
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

        let res = self
            .client
            .post(format!(
                "{}/v1/databases/{}/query",
                self.api_url, self.database_id
            ))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .header("Notion-Version", "2022-06-28")
            .json(&payload)
            .send()
            .await;

        match res {
            Ok(r) if r.status().is_success() => {
                let response: SearchByTitleResponse = r.json().await.unwrap();
                println!("Response: {:?}", response);

                if response.results.is_empty() {
                    None
                } else {
                    let first_item = response.results.first().unwrap();
                    let id = first_item.id.clone();
                    println!("Found page with ID: {}", id);
                    Some(id)
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
}
