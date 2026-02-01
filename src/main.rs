use crate::commands::draft::DraftCommand;
use crate::commands::{Context, GjCommand, LogCommand};
use crate::configuration::AppConfig;
use crate::database::Database;
use anyhow::Result;
use clap::{CommandFactory, Parser};
use crate::ai::AiClient;

mod commands;
mod configuration;
mod database;
mod utils;
mod ai;

#[derive(Parser)]
#[command(name = "gj", version, about = "gj \nTerminal-first journaling.")]
struct Cli {
    // Simple mode: gj "hello world"
    #[arg(value_name = "LOG")]
    log: Option<String>,

    #[arg(long, short, value_name = "DATE")]
    date: Option<String>,

    #[command(subcommand)]
    command: Option<GjCommand>,
}

fn main() -> Result<()> {
    env_logger::init();
    let config = AppConfig::load()?;
    let db = Database::open(&config.database.filename)?;
    let ai_client =  AiClient::new(config.options.ollama_model.clone());

    let cli = Cli::parse();
    let ctx = Context { db, config, ai_client };

    let cmd_to_run = if let Some(cmd) = cli.command {
        Some(cmd)
    } else if let Some(log_text) = cli.log {
        Some(GjCommand::Log(LogCommand {
            message: log_text,
            date: cli.date,
        }))
    } else {
        None
    };

    match cmd_to_run {
        Some(cmd) => cmd.execute(&ctx)?,
        None => {
            DraftCommand::default().execute(&ctx)?;
            let _ = Cli::command().print_help();
        }
    }

    Ok(())
}
