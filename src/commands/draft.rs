use crate::commands::Context;
use clap::Args;

#[derive(Args, Debug)]
pub struct DraftCommand {
    #[arg(long, short, value_name = "DATE")]
    pub date: Option<String>
}

impl DraftCommand {
    pub fn execute(&self, ctx: Context) -> anyhow::Result<()> {
        let template = "Lorem ipsum";
        let edited = edit::edit(template)?;

        ctx.db.add_log(&edited, self.date.clone())?;

        Ok(())
    }
}

impl Default for DraftCommand {
    fn default() -> Self {
        Self {
            date: None
        }
    }
}
