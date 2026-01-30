use crate::commands::Context;
use crate::utils::get_git_activity;
use clap::Args;
use log::debug;

#[derive(Args, Debug)]
pub struct DraftCommand {
    #[arg(long, short, value_name = "DATE")]
    pub date: Option<String>,
}

impl DraftCommand {
    pub fn execute(&self, ctx: &Context) -> anyhow::Result<()> {
        debug!("Executing gj draft");
        
        let draft_from_git = ctx.config.options.draft_from_git.unwrap_or(false);
        let template = if draft_from_git {
            get_git_activity(&ctx.config.user.email)
        } else {
            Ok(String::new())
        }?;
        let edited = edit::edit(template)?;

        ctx.db.add_log(&edited, self.date.clone())?;

        Ok(())
    }
}

impl Default for DraftCommand {
    fn default() -> Self {
        Self { date: None }
    }
}
