mod commands;

use crate::commands::log::log;
use crate::commands::setup::setup;
use anyhow::Result;
use clap::Parser;
use gj::config::load_config;
use gj::notion::NotionClient;

#[derive(Parser)]
#[command(name = "gj", version, about = "Dead simple CLI for journaling", long_about = None)]
struct Cli {
    #[arg(long)]
    setup: bool,

    #[arg()]
    entry: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.setup {
        setup();
        return Ok(());
    }

    if let Some(entry) = cli.entry {
        let config = load_config();
        let notion_client = NotionClient::new(config.notion_token, config.database_id);
        log(notion_client, entry).await
    } else {
        println!("Please provide a journal entry or use `--setup` to configure.");
        Ok(())
    }
}
