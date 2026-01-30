use crate::commands::Context;
use clap::Args;

#[derive(Args, Debug)]
pub struct LogCommand {
    #[arg(long, short, value_name = "DATE")]
    pub date: Option<String>,
    pub message: String,
}

impl LogCommand {
    pub fn execute(&self, ctx: &Context) -> anyhow::Result<()> {
        ctx.db.add_log(&self.message, self.date.clone())?;
        println!("good job, done.");
        Ok(())
    }
}
