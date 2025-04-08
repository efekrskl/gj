use anyhow::{Context, Result};
use dialoguer::Input;
use gj::notion::NotionClient;

pub async fn log(notion_client: NotionClient) -> Result<()> {
    let messages: String = Input::new()
        .with_prompt("📝 What did you work on today?")
        .interact_text()?;
    let page_title = chrono::Utc::now().format("%m-%Y").to_string();

    match notion_client.get_page_id_by_title(&page_title).await {
        Some(page_id) => {
            println!("Page already exists for today. Updating...");
            update_page_with_messages(notion_client, &page_id, messages).await?;
        }
        None => {
            println!("No page found for today. Creating a new one...");
            let page_id = notion_client.create_page(page_title).await
                .context("Failed to create page")?;
            update_page_with_messages(notion_client, &page_id, messages).await?;
        }
    }
    
    Ok(())
}

async fn update_page_with_messages(
    notion_client: NotionClient,
    page_id: &str,
    messages: String,
) -> Result<()> {
    let today_date = chrono::Utc::now().format("%d-%m").to_string();

    let needs_header = match notion_client
        .get_last_page_header_block_content(page_id)
        .await
    {
        Some(last_date) => last_date != today_date,
        None => true,
    };

    if needs_header {
        println!("Adding date header to page...");
        notion_client.append_header(page_id, today_date).await?;
    }

    notion_client.append_messages(page_id, messages).await?;

    Ok(())
}
