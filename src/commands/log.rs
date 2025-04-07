use chrono::Utc;
use dialoguer::Input;
use reqwest::Client;
use serde_json::{json, Value};
use crate::config::load_config;

pub async fn log() {
    let config = load_config();
    let client = Client::new();

    let messages_raw: String = Input::new()
        .with_prompt("📝 What did you work on today?")
        .interact_text()
        .unwrap();
    let messages: Value = messages_raw
        .split(';')
        .enumerate()
        .map(|(i, message)| {
            json!({
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [
                        {
                            "type": "text",
                            "text": {
                                "content": format!("{} - {}", i+1, message.trim()),
                            }
                        }
                    ]
                }
            })
        })
        .collect();

    let timestamp = Utc::now().to_rfc3339();

    let payload = json!({
        "parent": { "database_id": config.database_id },
        "properties": {
            "Date": { "date": { "start": timestamp }},
        },
        "children": messages,
    });

    let res = client
        .post("https://api.notion.com/v1/pages")
        .header("Authorization", format!("Bearer {}", config.notion_token))
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
