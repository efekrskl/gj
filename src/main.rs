use crate::ai::AiClient;
use crate::commands::LogCommand;
use crate::commands::draft::DraftCommand;
use crate::commands::view::ViewCommand;
use crate::configuration::AppConfig;
use crate::database::Database;
use anyhow::Result;
use clap::{Parser, Subcommand};

mod ai;
mod commands;
mod configuration;
mod database;
mod utils;

#[derive(Parser)]
#[command(name = "gj", version, about = "gj \nTerminal-first journaling.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Log {
        #[arg(required = true)]
        message: String,

        #[arg(long, short)]
        date: Option<String>,
    },
    Draft {
        #[arg(long, short)]
        date: Option<String>,
    },
    View,
}

pub struct Context {
    pub db: Database,
    pub config: AppConfig,
    pub ai_client: AiClient,
}

fn main() -> Result<()> {
    env_logger::init();
    let config = AppConfig::load()?;
    let db = Database::open(&config.database.filename)?;
    let ai_client = AiClient::new(config.options.ollama_model.clone());

    let ctx = Context {
        db,
        config,
        ai_client,
    };

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => e.exit(),
    };

    match cli.command {
        Command::Log { message, date } => {
            LogCommand { message, date }.execute(&ctx)?;
        }
        Command::Draft { date } => {
            DraftCommand { date }.execute(&ctx)?;
        }
        Command::View => {
            let _ = ViewCommand {}.execute(&ctx);

            return Ok(());
        }
    }

    Ok(())
}
