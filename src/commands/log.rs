use crate::database::SourceType;
use clap::Args;
use log::debug;
use crate::AppContext;

#[derive(Args, Debug)]
pub struct LogCommand {
    #[arg(long, short, value_name = "DATE")]
    pub date: Option<String>,
    pub message: String,
}

impl LogCommand {
    pub fn execute(&self, ctx: &AppContext) -> anyhow::Result<()> {
        debug!(
            "[log] command start has_custom_date={} message_len={}",
            self.date.is_some(),
            self.message.len()
        );

        ctx.db
            .add_log(&self.message, self.date.clone(), SourceType::Log)?;
        debug!("[log] command success");
        
        println!("good job, done.");

        Ok(())
    }
}
