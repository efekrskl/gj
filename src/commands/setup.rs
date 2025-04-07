use crate::config::save_config;
use crate::config::{CONFIG_PATH, config_exists};
use dialoguer::{Confirm, Input};

pub fn setup() {
    if config_exists() {
        let confirm = Confirm::new()
            .with_prompt("⚠️ Config already exists. Are you sure you want to overwrite it?")
            .default(false)
            .interact()
            .unwrap();

        if !confirm {
            println!("❌ Aborted");
            return;
        }
    }

    let token: String = Input::new()
        .with_prompt("🔑 Notion Integration Token")
        .interact_text()
        .unwrap();

    let db: String = Input::new()
        .with_prompt("📄 Notion Database ID")
        .interact_text()
        .unwrap();

    let confirm = Confirm::new()
        .with_prompt(format!("Save to ~/{CONFIG_PATH} ?"))
        .default(true)
        .interact()
        .unwrap();

    if confirm {
        save_config(token, db);
    } else {
        println!("❌ Aborted");
    }
}
