use crate::config::save_config;
use dialoguer::{Confirm, Input};

pub fn setup() {
    let token: String = Input::new()
        .with_prompt("🔑 Notion Integration Token")
        .interact_text()
        .unwrap();

    let db: String = Input::new()
        .with_prompt("📄 Notion Database ID")
        .interact_text()
        .unwrap();

    let confirm = Confirm::new()
        .with_prompt("Save to ~/.gj/config.json ?")
        .default(true)
        .interact()
        .unwrap();

    if confirm {
        save_config(token, db);
    } else {
        println!("❌ Aborted");
    }
}
