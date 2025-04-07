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

    notion_client.create_page(message_rows).await
}
