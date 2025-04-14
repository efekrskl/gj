use anyhow::Result;
use dialoguer::{Confirm, Input, Password};
use gj::config::CONFIG_PATH;
use gj::config::{config_exists, save_config};
use gj::notion::NotionClient;

pub async fn setup() -> Result<()> {
    if config_exists() {
        let confirm = Confirm::new()
            .with_prompt("⚠️ Config already exists. Are you sure you want to overwrite it?")
            .default(false)
            .interact()?;

        if !confirm {
            println!("❌ Aborted");
            return Ok(());
        }
    }

    let token: String = Password::new()
        .with_prompt("🔑 Notion Integration Token")
        .interact()?;

    let root_page_id: String = Input::new()
        .allow_empty(true)
        .with_prompt("📄 Root Page ID (Leave empty if the database already exists)")
        .interact_text()?;

    let confirm = Confirm::new()
        .with_prompt(format!("Save to ~/{CONFIG_PATH} ?"))
        .default(true)
        .interact()?;

    let notion_client = NotionClient::new(token.clone());

    // todo deduplicate by a different way
    let database_id = match notion_client.find_gj_database_by_title().await? {
        Some(id) => id,
        None => notion_client.create_gj_database(&root_page_id).await?,
    };
    
    if confirm {
        save_config(token, database_id);
        print_post_setup_guide();
        Ok(())
    } else {
        println!("❌ Aborted");
        Ok(())
    }
}

pub fn print_post_setup_guide() {
    println!(
        r#"

  /$$$$$$     /$$$$$ /$$
 /$$__  $$   |__  $$| $$
| $$  \__/      | $$| $$
| $$ /$$$$      | $$| $$
| $$|_  $$ /$$  | $$|__/
| $$  \ $$| $$  | $$    
|  $$$$$$/|  $$$$$$/ /$$
 \______/  \______/ |__/
                        
📌 Final touches in Notion (highly recommended):

1. ➕ Enable Full Width — click the ⋯ menu → "Full width"
2. 🗓️ Switch the database view to "Calendar" — click the ⋯ menu on the auto generated database → "Layout" → "Calendar"
3. 🏷️ Under the same ⋯ menu, click "Properties" and make "Tags" visible
4. 🚨 DO NOT rename "GJ Dead Simple Journal"
   (this is how gj finds your database)

You're all set. Try:
  gj "first log entry"
"#
    );
}
