mod commands;

use crate::commands::log::log;
use crate::commands::setup::setup;
use anyhow::Result;
use clap::{Parser, Subcommand};
use gj::config::load_config;
use gj::notion::NotionClient;

#[derive(Parser)]
#[command(name = "gj", version, about = "Dead simple CLI for journaling")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Setup,

    Log { entry: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Setup => {
            setup().await?;
            Ok(())
        }
        Commands::Log { entry } => {
            let config = load_config();
            let notion_client = NotionClient::new(config.notion_token);
            log(notion_client, entry, config.database_id).await
        }
    }
}
