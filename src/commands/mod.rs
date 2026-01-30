pub mod draft;
pub mod log;

pub use self::log::LogCommand;
use crate::commands::draft::DraftCommand;
use crate::database::Database;
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum GjCommand {
    Log(LogCommand),
    Draft(DraftCommand),
}

pub struct Context {
    pub db: Database,
}

impl GjCommand {
    pub fn execute(&self, ctx: Context) -> Result<()> {
        match self {
            GjCommand::Log(cmd) => cmd.execute(ctx),
            GjCommand::Draft(cmd) => cmd.execute(ctx),
        }
    }
}
