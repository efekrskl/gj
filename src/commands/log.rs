use crate::database::Database;
use clap::Args;

#[derive(Args, Debug)]
pub struct LogCommand {
    pub message: String,
    // pub date: Option<String>,
}

impl LogCommand {
    pub fn execute(&self, db: &Database) -> anyhow::Result<()> {
        db.add_log(&self.message)?;
        println!("good job, done.");
        Ok(())
    }
}
