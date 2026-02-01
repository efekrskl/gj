use crate::commands::Context;
use crate::utils::get_git_activity;
use clap::Args;
use log::debug;
use crate::database::SourceType;

#[derive(Args, Debug)]
pub struct DraftCommand {
    #[arg(long, short, value_name = "DATE")]
    pub date: Option<String>,
}

impl DraftCommand {
    pub fn execute(&self, ctx: &Context) -> anyhow::Result<()> {
        debug!("Executing gj draft");

        let draft_from_git = ctx.config.options.draft_from_git.unwrap_or(false);
        let mut template = if draft_from_git {
            let latest_draft_date = ctx.db.get_latest_draft_log_date();
            get_git_activity(&ctx.config.user.email, latest_draft_date)
        } else {
            Ok(String::new())
        }?;
        let draft_with_ollama = ctx.config.options.draft_with_ollama.unwrap_or(false);
        if draft_with_ollama {
            if let Ok(response) = ctx.ai_client.summarize(&template) {
                template = response
            };
        }
        let edited = edit::edit(template)?;

        ctx.db.add_log(&edited, self.date.clone(), SourceType::Draft)?;

        Ok(())
    }
}

impl Default for DraftCommand {
    fn default() -> Self {
        Self { date: None }
    }
}
