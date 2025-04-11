use anyhow::{Context, Result};
use gj::emoji::apply_emoji_prefix;
use gj::notion::NotionClient;

pub async fn log(notion_client: NotionClient, entry: String, database_id: String) -> Result<()> {
    let page_title = chrono::Utc::now().format("%B %Y").to_string();
    let page_id = get_or_create_page(&notion_client, &page_title, &database_id)
        .await
        .context("Failed to get or create page")?;

    let date_as_subtitle = chrono::Utc::now().format("%A (%d/%m)").to_string();

    let needs_header = match notion_client
        .get_last_page_header_block_content(&page_id)
        .await
    {
        Some(last_date) => last_date != date_as_subtitle,
        None => true,
    };

    let entries = build_entries(entry);
    let page_subtitle = if needs_header {
        Some(date_as_subtitle)
    } else {
        None
    };

    notion_client
        .add_entries(&page_id, entries, page_subtitle)
        .await?;

    Ok(())
}

fn build_entries(entry: String) -> Vec<String> {
    entry
        .split(";")
        .map(|entry| apply_emoji_prefix(entry))
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
