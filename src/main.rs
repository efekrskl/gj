mod commands;

use crate::commands::log::log;
use crate::commands::setup::setup;
use clap::{Parser, Subcommand};
use gj::config::load_config;
use gj::notion::NotionClient;

#[derive(Parser)]
#[command(name = "gj", version, about = "Dead simple CLI for journaling", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Log,
    Setup,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config = load_config();
    let notion_client = NotionClient::new(config.notion_token, config.database_id);

    match cli.command {
        Commands::Log => log(notion_client).await,
        Commands::Setup => setup(),
    }
}
