use crate::commands::{GjCommand, LogCommand};
use crate::configuration::AppConfig;
use crate::database::Database;
use anyhow::Result;
use clap::{CommandFactory, Parser};

mod commands;
mod configuration;
mod database;

#[derive(Parser)]
#[command(name = "gj", version, about = "gj \nTerminal-first journaling.")]
struct Cli {
    // Simple mode: gj "hello world"
    #[arg(value_name = "LOG")]
    log: Option<String>,

    #[command(subcommand)]
    command: Option<GjCommand>,
}

fn main() -> Result<()> {
    env_logger::init();
    let config = AppConfig::load()?;
    let db = Database::open(config.database.filename)?;

    let cli = Cli::parse();

    let cmd_to_run = if let Some(cmd) = cli.command {
        Some(cmd)
    } else if let Some(log_text) = cli.log {
        Some(GjCommand::Log(LogCommand { message: log_text }))
    } else {
        None
    };

    match cmd_to_run {
        Some(cmd) => cmd.execute(&db)?,
        None => {
            // todo: interactive mode

            let _ = Cli::command().print_help();
        }
    }

    Ok(())
}
