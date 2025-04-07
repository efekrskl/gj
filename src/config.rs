use dirs::home_dir;
use serde::Deserialize;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Config {
    pub notion_token: String,
    pub database_id: String,
}

pub const CONFIG_PATH: &str = ".gj/config.json";

pub fn config_exists() -> bool {
    let path = get_config_path();
    path.exists()
}

pub fn load_config() -> Config {
    let path = get_config_path();
    let raw = fs::read_to_string(path).expect("⚠️ Missing config. Run `gj setup` first.");
    serde_json::from_str(&raw).expect("⚠️ Invalid config file format.")
}

pub fn save_config(token: String, db: String) {
    let config = json!({
        "notion_token": token,
        "database_id": db
    });
    let path = get_config_path();
    let parent = path.parent().unwrap();
    fs::create_dir_all(parent).unwrap();
    fs::write(path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
    println!("✅ Config saved to ~/{CONFIG_PATH}");
}

fn get_config_path() -> PathBuf {
    let mut path = home_dir().expect("No home dir found");
    path.push(CONFIG_PATH);
    path
}
