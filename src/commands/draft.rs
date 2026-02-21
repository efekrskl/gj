use crate::AppContext;
use crate::database::SourceType;
use crate::utils::get_git_activity;
use log::debug;

pub struct DraftCommand {
    pub date: Option<String>,
}

impl DraftCommand {
    pub async fn execute(&self, ctx: &AppContext) -> anyhow::Result<()> {
        debug!(
            "[draft] command start has_custom_date={} from_git={} refine_with_ollama={}",
            self.date.is_some(),
            ctx.config.draft.from_git.unwrap_or(false),
            ctx.config.draft.refine_with_ollama.unwrap_or(false)
        );

        let draft_from_git = ctx.config.draft.from_git.unwrap_or(false);
        let mut template = if draft_from_git {
            let latest_draft_date = ctx.db.get_latest_draft_log_date();
            debug!(
                "[draft] collecting git activity latest_draft_date={:?}",
                latest_draft_date
            );
            get_git_activity(&ctx.config.draft.git_email, latest_draft_date)
        } else {
            Ok(String::new())
        }?;
        let draft_with_ollama = ctx.config.draft.refine_with_ollama.unwrap_or(false);
        if draft_with_ollama {
            if let Ok(response) = ctx
                .ai_client
                .summarize(&template, ctx.config.draft.redact_sensitive_with_ollama)
                .await
            {
                template = response;
            }
        }
        let edited = edit::edit(template)?;
        debug!("[draft] editor completed edited_len={}", edited.len());

        ctx.db
            .add_log(&edited, self.date.clone(), SourceType::Draft)?;
        debug!("[draft] command success");

        Ok(())
    }
}
