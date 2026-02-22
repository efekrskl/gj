use crate::utils::{to_date_key, to_sqlite_timestamp};
use anyhow::{Context, Result};
use log::debug;
use rusqlite::OptionalExtension;
use rusqlite::types::Value;
use rusqlite::{Connection, params};
use rusqlite_migration::{M, Migrations};
use std::collections::{HashMap, HashSet};

pub struct Database {
    connection: Connection,
}

pub enum SourceType {
    Log,
    Draft,
}

impl SourceType {
    fn as_str(&self) -> &'static str {
        match self {
            SourceType::Draft => "draft",
            SourceType::Log => "log",
        }
    }
}

#[derive(Debug)]
pub struct Log {
    pub id: i64,
    pub content: String,
    source_type: String,
    pub created_at: String,
    updated_at: String,
}

#[derive(Debug)]
pub struct LogPreview {
    pub id: i64,
    created_at: String,
}

// todo: improve the dtos

const MIGRATIONS_SLICE: &[M<'_>] = &[
    M::up(
        r#"
    CREATE TABLE logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    source_type TEXT NOT NULL, -- 'manual' / 'draft'
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );
    "#,
    ),
    M::up(
        r#"
    CREATE TABLE sync_units (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    adapter TEXT NOT NULL,       -- 'notion'
    unit_type TEXT NOT NULL,     -- 'day' / 'log'
    local_key TEXT NOT NULL,     -- date / id i.e. '2026-02-03' / '4212323'
    remote_key TEXT,
    content_hash TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'success', -- 'success' / 'fail',
    last_error TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_synced_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(adapter, unit_type, local_key)
    );
    "#,
    ),
    M::up(
        r#"
    CREATE TABLE sync_state (
    adapter TEXT PRIMARY KEY,      -- 'notion'
    last_synced_at DATETIME
    );
        "#,
    ),
];

const MIGRATIONS: Migrations<'_> = Migrations::from_slice(MIGRATIONS_SLICE);

impl Database {
    pub fn open(sqlite_filename: &String) -> Result<Database> {
        debug!("[db] opening sqlite connection file={}", sqlite_filename);
        let mut connection =
            Connection::open(sqlite_filename).context("Couldn't open the sqlite file.")?;

        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "synchronous", "NORMAL")?;
        connection.pragma_update(None, "foreign_keys", "ON")?;

        debug!("[db] sqlite pragmas applied");

        MIGRATIONS
            .to_latest(&mut connection)
            .context("Couldn't apply the database migrations.")?;

        debug!("[db] migrations complete");

        Ok(Self { connection })
    }

    pub fn update_log(&self, id: i64, content: &str) -> Result<()> {
        let query = r#"
        UPDATE logs
        SET content = ?1, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?2"#;

        debug!("[db] updating log id={} content_len={}", id, content.len());

        self.connection.execute(query, (content, id))?;

        debug!("[db] updated log id={}", id);

        Ok(())
    }

    pub fn add_log(
        &self,
        content: &str,
        date: Option<String>,
        source_type: SourceType,
    ) -> Result<()> {
        let query = r#"
         INSERT INTO logs (content, source_type, created_at)
         VALUES (?1, ?2, COALESCE(?3, CURRENT_TIMESTAMP))"#;

        let date = match date {
            Some(d) => Some(to_sqlite_timestamp(&d)?),
            None => None,
        };

        debug!(
            "[db] adding log source_type={} has_custom_date={} content_len={}",
            source_type.as_str(),
            date.is_some(),
            content.len()
        );

        self.connection
            .execute(query, (content, source_type.as_str(), date))?;

        debug!("[db] log inserted");

        Ok(())
    }

    pub fn get_latest_draft_log_date(&self) -> Option<String> {
        let query = r#"SELECT created_at FROM logs
                   WHERE source_type = ?1
                   ORDER BY created_at DESC
                   LIMIT 1"#;
        let log: Option<String> = self
            .connection
            .query_row(query, [SourceType::Draft.as_str()], |row| row.get(0))
            .optional()
            .ok()
            .flatten();

        log
    }

    pub fn get_logs_by_year(&self, year: i32) -> Result<Vec<Log>> {
        let query = "SELECT id, content, source_type, created_at, updated_at FROM logs WHERE strftime('%Y', created_at) = ?1";
        debug!("[db] loading logs for year={}", year);

        let mut statement = self.connection.prepare(query)?;
        let rows = statement
            .query_map([year.to_string()], |row| {
                Ok(Log {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    source_type: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to fetch logs.")?;

        debug!("[db] loaded logs count={} year={}", rows.len(), year);

        Ok(rows)
    }
}
#[derive(Debug)]
pub struct SyncState {
    adapter: String,
    last_synced_at: Option<String>,
}
#[derive(Debug)]
pub struct SyncUnit {
    pub id: i64,
    pub adapter: String,
    pub unit_type: String,
    pub local_key: String,
    pub remote_key: Option<String>,
    pub content_hash: String,
    pub status: String,
    pub last_error: Option<String>,
    pub created_at: String,
    pub last_synced_at: Option<String>,
}

#[derive(Debug)]
pub struct SyncUnitPreview {
    pub local_key: String,
    pub remote_key: Option<String>,
    pub content_hash: String,
    pub status: String,
    pub last_synced_at: Option<String>,
}

pub struct CreateSyncUnit {
    pub adapter: String,
    pub unit_type: String,
    pub local_key: String,
    pub remote_key: Option<String>,
    pub content_hash: String,
    pub status: String,
    pub last_error: Option<String>,
}

pub struct UpdateSyncUnit {
    pub local_key: String,
    pub adapter: String,
    pub unit_type: String,
    pub remote_key: Option<String>,
    pub content_hash: String,
    pub status: String,
    pub last_error: Option<String>,
}

// Sync Push
impl Database {
    // todo: from / to
    pub fn get_logs_to_push(
        &self,
        adapter: String,
        unit_type: String,
    ) -> Result<(HashMap<String, Vec<Log>>, HashMap<String, SyncUnitPreview>)> {
        debug!(
            "[push] start adapter='{}' unit_type='{}'",
            adapter, unit_type
        );

        let cursor = self.get_sync_cursor(&adapter)?;
        let changed_logs = self.fetch_changed_logs(cursor.as_deref())?;

        let (groups, local_keys) = self.group_logs_by_day(changed_logs)?;

        if local_keys.is_empty() {
            debug!("[push] no logs found");
            return Ok((groups, HashMap::new()));
        }

        let sync_units = self.fetch_sync_units_by_keys(adapter, unit_type, &local_keys)?;
        let sync_units_map = self.to_sync_units_map(sync_units);

        Ok((groups, sync_units_map))
    }

    fn get_sync_cursor(&self, adapter: &str) -> Result<Option<String>> {
        let query = "SELECT last_synced_at FROM sync_state WHERE adapter = ?1;";

        let cursor: Option<String> = self
            .connection
            .query_row(query, params![adapter], |row| row.get(0))
            .optional()
            .context("Failed to fetch sync cursor")?;

        debug!("[push] cursor={:?}", cursor);
        Ok(cursor)
    }

    fn fetch_changed_logs(&self, cursor: Option<&str>) -> Result<Vec<Log>> {
        // todo: ideally we should get from/to and query with them instead. this is a hacky solution to update past logs
        let logs_query = r#"
            SELECT id, content, source_type, created_at, updated_at
            FROM logs
            WHERE updated_at > COALESCE(
                datetime(?1, '-7 days'),
                datetime('now', '-7 days')
            )
            ORDER BY updated_at ASC;
        "#;

        debug!(
            "[push] loading changed logs cursor_present={}",
            cursor.is_some()
        );

        let mut statement = self.connection.prepare(logs_query)?;
        let logs = statement
            .query_map(params![cursor], |row| {
                Ok(Log {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    source_type: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("Failed to fetch changed logs")?;

        debug!("[push] changed_logs={}", logs.len());
        Ok(logs)
    }

    fn group_logs_by_day(
        &self,
        logs: Vec<Log>,
    ) -> Result<(HashMap<String, Vec<Log>>, Vec<String>)> {
        let mut groups: HashMap<String, Vec<Log>> = HashMap::new();
        let mut keys: HashSet<String> = HashSet::new();
        let mut skipped: usize = 0;

        for log in logs {
            let day_key = match to_date_key(&log.created_at) {
                Ok(key) => key,
                Err(err) => {
                    skipped += 1;
                    debug!(
                        "[push] skip log id={} created_at='{}' to_date_key_error={}",
                        log.id, log.created_at, err
                    );
                    continue;
                }
            };

            keys.insert(day_key.clone());
            groups.entry(day_key).or_default().push(log);
        }

        for (day, logs) in groups.iter_mut() {
            logs.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            debug!("[push] day={} logs={}", day, logs.len());
        }

        let mut local_keys: Vec<String> = keys.into_iter().collect();
        local_keys.sort();

        debug!(
            "[push] grouped_days={} keys={} skipped={}",
            groups.len(),
            local_keys.len(),
            skipped
        );

        Ok((groups, local_keys))
    }

    fn fetch_sync_units_by_keys(
        &self,
        adapter: String,
        unit_type: String,
        local_keys: &[String],
    ) -> Result<Vec<SyncUnitPreview>> {
        let in_filter = std::iter::repeat("?")
            .take(local_keys.len())
            .collect::<Vec<_>>()
            .join(",");

        let query = format!(
            r#"
            SELECT local_key, remote_key, content_hash, status, last_synced_at
            FROM sync_units
            WHERE adapter = ?
              AND unit_type = ?
              AND local_key IN ({})
            "#,
            in_filter
        );

        debug!(
            "[push] loading existing sync units keys_count={}",
            local_keys.len()
        );

        let mut params_vec: Vec<Value> = Vec::with_capacity(2 + local_keys.len());
        params_vec.push(adapter.into());
        params_vec.push(unit_type.into());
        for k in local_keys {
            params_vec.push(k.clone().into());
        }

        let mut statement = self.connection.prepare(&query)?;
        let rows = statement
            .query_map(rusqlite::params_from_iter(params_vec), |row| {
                Ok(SyncUnitPreview {
                    local_key: row.get(0)?,
                    remote_key: row.get(1)?,
                    content_hash: row.get(2)?,
                    status: row.get(3)?,
                    last_synced_at: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("Failed to fetch sync_units")?;

        debug!("[push] sync_units_fetched={}", rows.len());
        Ok(rows)
    }

    fn to_sync_units_map(
        &self,
        sync_units: Vec<SyncUnitPreview>,
    ) -> HashMap<String, SyncUnitPreview> {
        let map: HashMap<String, SyncUnitPreview> = sync_units
            .into_iter()
            .map(|su| (su.local_key.clone(), su))
            .collect();

        debug!("[push] sync_units_map_size={}", map.len());
        map
    }
    pub fn create_sync_unit(&self, unit: &CreateSyncUnit) -> Result<()> {
        let query = r#"
            INSERT INTO sync_units (adapter, unit_type, local_key, remote_key, content_hash, status, last_error)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#;

        self.connection.execute(
            query,
            params![
                unit.adapter,
                unit.unit_type,
                unit.local_key,
                unit.remote_key,
                unit.content_hash,
                unit.status,
                unit.last_error,
            ],
        )?;

        debug!(
            "[push] created sync unit local_key={} status={} remote_key_present={}",
            unit.local_key,
            unit.status,
            unit.remote_key.is_some()
        );

        Ok(())
    }

    pub fn update_sync_unit(&self, unit: &UpdateSyncUnit) -> Result<()> {
        let query = r#"
            UPDATE sync_units
            SET remote_key = ?1,
                content_hash = ?2,
                status = ?3,
                last_error = ?4,
                last_synced_at = CURRENT_TIMESTAMP
            WHERE adapter = ?5 AND unit_type = ?6 AND local_key = ?7
        "#;

        debug!(
            "[push] updating sync unit local_key={} status={} remote_key_present={}",
            unit.local_key,
            unit.status,
            unit.remote_key.is_some()
        );

        self.connection.execute(
            query,
            params![
                unit.remote_key,
                unit.content_hash,
                unit.status,
                unit.last_error,
                unit.adapter,
                unit.unit_type,
                unit.local_key,
            ],
        )?;

        Ok(())
    }
}

impl Database {
    pub fn upsert_sync_state(&self, adapter: &str) -> Result<()> {
        let query = r#"
        INSERT INTO sync_state (adapter, last_synced_at)
        VALUES (?1, CURRENT_TIMESTAMP)
        ON CONFLICT(adapter) DO UPDATE SET
            last_synced_at = CURRENT_TIMESTAMP
        "#;

        self.connection.execute(query, params![adapter])?;

        Ok(())
    }
}
