use dialoguer::Input;
use gj::notion::NotionClient;

pub async fn log(notion_client: NotionClient) {
    let messages_raw: String = Input::new()
        .with_prompt("📝 What did you work on today?")
        .interact_text()
        .unwrap();
    let message_rows = messages_raw
        .split(";")
        .enumerate()
        .map(|(i, message)| format!("{} - {}", i + 1, message.trim()))
        .collect::<Vec<String>>();

    let page_title = chrono::Utc::now().format("%d-%m-%Y").to_string();
    let existing_page = notion_client.get_page_id_by_title(&page_title).await;

    if (existing_page != None) {
        println!("Page already exists for today. Updating...");
        // let page_id = existing_page.unwrap();
        // notion_client.update_page(page_id, message_rows).await;
        todo!("Not implemented yet");
    }

    println!("Existing page: {:?}", existing_page);

    notion_client.create_page(page_title, message_rows).await
}
