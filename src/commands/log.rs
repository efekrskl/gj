use crate::database::SourceType;
use clap::Args;
use log::debug;
use crate::Context;

#[derive(Args, Debug)]
pub struct LogCommand {
    #[arg(long, short, value_name = "DATE")]
    pub date: Option<String>,
    pub message: String,
}

impl LogCommand {
    pub fn execute(&self, ctx: &Context) -> anyhow::Result<()> {
        debug!("Executing gj log");

        ctx.db
            .add_log(&self.message, self.date.clone(), SourceType::Log)?;
        
        println!("good job, done.");

        Ok(())
    }
}
