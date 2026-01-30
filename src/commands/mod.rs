pub mod log;

use clap::Subcommand;
use crate::database::Database;
use anyhow::Result;

pub use self::log::LogCommand;

#[derive(Subcommand)]
pub enum GjCommand {
    Log(LogCommand),
}

impl GjCommand {
    pub fn execute(&self, db: &Database) -> Result<()> {
        match self {
            GjCommand::Log(cmd) => cmd.execute(db),
        }
    }
}