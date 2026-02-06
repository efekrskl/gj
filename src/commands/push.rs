use crate::Context;
use crate::database::{CreateSyncUnit, UpdateSyncUnit};
use crate::utils::{build_day_content_string, hash_content};
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
                let (groups, sync_unit_maps) = ctx
                    .db
                    .get_logs_to_push("notion".to_string(), "day".to_string())?;

                for (day_key, logs) in groups {
                    let maybe_sync_unit = sync_unit_maps.get(&day_key);
                    let content_string = build_day_content_string(&day_key, &logs);
                    let content_hash = hash_content(&content_string);

                    if let Some(sync_unit) = maybe_sync_unit {
                        debug!(
                            "[push] found sync_unit for local_key {}",
                            sync_unit.local_key
                        );
                        // If there's a sync unit, but the status is failed -> sync
                        // If the hash doesn't match with the sync unit -> sync
                        if sync_unit.status == "fail" || sync_unit.content_hash != content_hash {
                            debug!(
                                "[push] sync_unit with local_key {} and status {} needs repush",
                                sync_unit.local_key, sync_unit.status
                            );

                            // todo: notion call

                            let update = UpdateSyncUnit {
                                content_hash,
                                status: "success".to_string(),
                                last_error: None,
                                remote_key: Some("temp_id".to_string()),
                                local_key: sync_unit.local_key.to_owned(),
                                adapter: "notion".to_string(),
                                unit_type: "day".to_string(),
                            };
                            ctx.db.update_sync_unit(&update)?;
                        }
                    } else {
                        // If there's no sync unit -> sync
                        let data = CreateSyncUnit {
                            adapter: "notion".to_string(),
                            unit_type: "day".to_string(),
                            local_key: day_key,
                            remote_key: Some("temp_key".to_string()),
                            last_error: None,
                            content_hash,
                            status: "success".to_string(),
                        };

                        // todo: notion call

                        ctx.db.create_sync_unit(&data)?;
                    }
                }

                debug!("[push] notion push complete.")
            }
        }

        ctx.db.upsert_sync_state("notion")?;
        
        Ok(())
    }
}
