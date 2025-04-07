mod commands;
mod config;

use crate::commands::log::log;
use crate::commands::setup::setup;
use clap::{Parser, Subcommand};
use serde::Deserialize;

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

#[derive(Deserialize)]
struct Config {
    notion_token: String,
    database_id: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Log => log().await,
        Commands::Setup => setup(),
    }
}
