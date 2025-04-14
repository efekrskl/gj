use anyhow::{Context, Result};
use gj::emoji::apply_emoji_prefix;
use gj::notion::NotionClient;
use gj::with_spinner::with_spinner;

pub async fn log(
    notion_client: NotionClient,
    entry: String,
    tags: Vec<String>,
    database_id: String,
) -> Result<()> {
    let page_title = chrono::Utc::now().format("%B %e, %Y").to_string();
    let page_id = with_spinner(
        get_or_create_page(&notion_client, &page_title, &database_id),
        "🔎 Connecting...".to_string(),
        None,
        None,
    )
    .await
    .context("Failed to get or create page")?;
    let entries = build_entries(entry);

    with_spinner(
        async {
            notion_client.add_entries(&page_id, entries.clone()).await?;
            notion_client.add_tags_to_page(&page_id, tags).await?;

            Ok(())
        },
        "🚀 Pushing Logs...".to_string(),
        Some("🎉 All done, gj!".to_string()),
        Some("❌ Failed to push logs".to_string()),
    )
    .await?;

    Ok(())
}

fn build_entries(entry: String) -> Vec<String> {
    entry
        .split(";")
        .map(|entry| apply_emoji_prefix(entry.trim()))
        .collect::<Vec<String>>()
}

async fn get_or_create_page(
    notion_client: &NotionClient,
    page_title: &str,
    database_id: &str,
) -> Result<String> {
    let page_id = match notion_client
        .get_page_id_by_title(&page_title, &database_id)
        .await
    {
        Some(page_id) => page_id,
        None => {
            let new_page_id = notion_client
                .create_page(page_title, &database_id)
                .await
                .context("Failed to create page")?;
            new_page_id
        }
    };

    Ok(page_id)
}
