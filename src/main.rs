mod commands;

use crate::commands::log::log;
use crate::commands::setup::setup;
use anyhow::Result;
use clap::Parser;
use gj::config::load_config;
use gj::notion::NotionClient;

#[derive(Parser)]
#[command(name = "gj", version, about = "Dead simple CLI for journaling")]
struct Cli {
    #[arg(value_name = "ENTRY", required_unless_present = "setup")]
    entry: Option<String>,

    #[arg(long)]
    setup: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.setup {
        setup().await?;
        return Ok(());
    }

    if let Some(entry) = cli.entry {
        let config = load_config();
        let notion_client = NotionClient::new(config.notion_token);
        log(notion_client, entry, config.database_id).await
    } else {
        eprintln!("Error: No entry provided. Use --help for usage.");
        Ok(())
    }
}
