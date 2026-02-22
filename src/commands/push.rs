use crate::AppContext;
use crate::database::{CreateSyncUnit, UpdateSyncUnit};
use crate::sync::notion::NotionClient;
use crate::utils::{build_day_content_string, hash_content};
use anyhow::{Context, Result};
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
    pub async fn execute(&self, ctx: &AppContext) -> Result<()> {
        debug!("[push] command start target={:?}", self.target);

        match self.target {
            PushTarget::Notion => {
                let api_key = ctx
                    .config
                    .push
                    .notion
                    .api_key
                    .clone()
                    .context("Notion API key missing")?;

                let database_id = ctx
                    .config
                    .push
                    .notion
                    .database_id
                    .clone()
                    .context("Notion database_id missing")?;

                let notion_client = NotionClient::new(api_key, database_id)?;

                let (groups, sync_unit_maps) = ctx
                    .db
                    .get_logs_to_push("notion".to_string(), "day".to_string())?;
                debug!(
                    "[push] candidate_days={} existing_sync_units={}",
                    groups.len(),
                    sync_unit_maps.len()
                );

                for (day_key, logs) in groups {
                    let maybe_sync_unit = sync_unit_maps.get(&day_key);
                    let content_string = build_day_content_string(&day_key, &logs);
                    let content_hash = hash_content(&content_string);
                    debug!(
                        "[push] evaluating day={} logs={} has_sync_unit={}",
                        day_key,
                        logs.len(),
                        maybe_sync_unit.is_some()
                    );

                    if let Some(sync_unit) = maybe_sync_unit {
                        // If there's a sync unit, but the status is failed -> sync
                        // If the hash doesn't match with the sync unit -> sync
                        if sync_unit.status == "fail"
                            || sync_unit.remote_key.is_none()
                            || sync_unit.content_hash != content_hash
                        {
                            debug!(
                                "[push] sync_unit with local_key {} and status {} needs repush",
                                sync_unit.local_key, sync_unit.status
                            );

                            let res = match &sync_unit.remote_key {
                                // Has an existing page, update content (more like erase + append)
                                Some(remote_key) => {
                                    debug!(
                                        "[push] updating remote page local_key={} remote_key={}",
                                        sync_unit.local_key, remote_key
                                    );
                                    notion_client
                                        .update(&ctx, remote_key, &content_string)
                                        .await
                                }
                                None => {
                                    // todo: Can this ever happen? ("success" state without a remote_key)
                                    debug!(
                                        "[push] skipping update local_key={} reason=missing_remote_key",
                                        sync_unit.local_key
                                    );
                                    continue;
                                }
                            };

                            let update = match res {
                                // Well it's probably not a new key, but still
                                Ok(new_remote_key) => {
                                    debug!(
                                        "[push] update success local_key={} remote_key={}",
                                        sync_unit.local_key, new_remote_key
                                    );
                                    UpdateSyncUnit {
                                        content_hash,
                                        status: "success".to_string(),
                                        last_error: None,
                                        remote_key: Some(new_remote_key),
                                        local_key: sync_unit.local_key.clone(),
                                        adapter: "notion".to_string(),
                                        unit_type: "day".to_string(),
                                    }
                                }
                                Err(err) => {
                                    debug!(
                                        "[push] update failed local_key={} error={}",
                                        sync_unit.local_key, err
                                    );
                                    UpdateSyncUnit {
                                        content_hash,
                                        status: "fail".to_string(),
                                        last_error: Some(err.to_string()),
                                        remote_key: sync_unit.remote_key.clone(),
                                        local_key: sync_unit.local_key.clone(),
                                        adapter: "notion".to_string(),
                                        unit_type: "day".to_string(),
                                    }
                                }
                            };

                            ctx.db.update_sync_unit(&update)?;
                        } else {
                            debug!(
                                "[push] no sync needed local_key={} status={}",
                                sync_unit.local_key, sync_unit.status
                            );
                        }
                    } else {
                        debug!("[push] creating remote page day={}", day_key);
                        let res = notion_client.create(&ctx, &day_key, &content_string).await;

                        let data = match res {
                            Ok(remote_key) => {
                                debug!(
                                    "[push] create success day={} remote_key={}",
                                    day_key, remote_key
                                );
                                CreateSyncUnit {
                                    adapter: "notion".to_string(),
                                    unit_type: "day".to_string(),
                                    local_key: day_key.clone(),
                                    remote_key: Some(remote_key),
                                    last_error: None,
                                    content_hash,
                                    status: "success".to_string(),
                                }
                            }
                            Err(err) => {
                                debug!("[push] create failed day={} error={}", day_key, err);
                                CreateSyncUnit {
                                    adapter: "notion".to_string(),
                                    unit_type: "day".to_string(),
                                    local_key: day_key.clone(),
                                    remote_key: None,
                                    last_error: Some(err.to_string()),
                                    content_hash,
                                    status: "fail".to_string(),
                                }
                            }
                        };

                        ctx.db.create_sync_unit(&data)?;
                    }
                }

                debug!("[push] notion push complete")
            }
        }

        ctx.db.upsert_sync_state("notion")?;
        debug!("[push] sync cursor updated adapter=notion");

        Ok(())
    }
}
