use crate::Context;
use clap::{Args, ValueEnum};
use log::debug;

#[derive(Debug, Clone, ValueEnum)]
pub enum PushTarget {
    Notion,
}

#[derive(Args, Debug)]
pub struct PushCommand {
    #[arg(value_enum)]
    pub target: PushTarget,
}

impl PushCommand {
    pub fn execute(&self, ctx: &Context) -> anyhow::Result<()> {
        debug!("Executing gj push");

        match self.target {
            PushTarget::Notion => {
                // let notion_client = NotionClient::new();
                let logs_to_push = ctx
                    .db
                    .get_logs_to_push("notion".to_string(), "day".to_string())?;
            
                println!("{:?}", logs_to_push);
            }
        }

        Ok(())
    }
}
