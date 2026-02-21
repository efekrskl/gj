use crate::ai::AiClient;
use crate::commands::log::LogCommand;
use crate::commands::draft::DraftCommand;
use crate::commands::push::{PushCommand, PushTarget};
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
mod sync;

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
    Push {
        #[arg(value_enum)]
        target: PushTarget,
    },
}

pub struct AppContext {
    pub db: Database,
    pub config: AppConfig,
    pub ai_client: AiClient,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let config = AppConfig::load()?;
    let db = Database::open(&config.database.filename)?;
    let ai_client = AiClient::new(config.draft.ollama_model.clone());

    let ctx = AppContext {
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
            ViewCommand {}.execute(&ctx)?;
        }
        Command::Push { target } => {
            PushCommand { target }.execute(&ctx).await?;
        }
    }

    Ok(())
}
