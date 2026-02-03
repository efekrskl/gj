use crate::utils::to_iso8601_timestamp;
use anyhow::{Context, Result};
use log::debug;
use rusqlite::Connection;
use rusqlite::OptionalExtension;
use rusqlite_migration::{M, Migrations};

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
    // raw_context: String,
    // tags: String,
    pub created_at: String,
    // updated_at: String,
}

const MIGRATIONS_SLICE: &[M<'_>] = &[M::up(
    r#"
    CREATE TABLE logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    raw_context TEXT,
    tags TEXT,
    source_type TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);"#,
)];

const MIGRATIONS: Migrations<'_> = Migrations::from_slice(MIGRATIONS_SLICE);

impl Database {
    pub fn open(sqlite_filename: &String) -> Result<Database> {
        let mut connection =
            Connection::open(sqlite_filename).context("Couldn't open the sqlite file.")?;

        debug!("SQLite connection is open.");

        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "synchronous", "NORMAL")?;
        connection.pragma_update(None, "foreign_keys", "ON")?;

        debug!("Applied PRAGMAs.");

        MIGRATIONS
            .to_latest(&mut connection)
            .context("Couldn't apply the database migrations.")?;

        debug!("Migrations are complete.");

        Ok(Self { connection })
    }

    pub fn update_log(
        &self,
        id: i64,
        content: &str,
    ) -> Result<()> {
        let query = r#"
        UPDATE logs
        SET content = ?1
        WHERE id = ?2"#;

        debug!(
            "Executing {query} with args {:?}",
            (id, content)
        );

        self.connection
            .execute(query, (content, id))?;

        debug!("Query complete.");

        Ok(())
    }

    pub fn add_log(
        &self,
        content: &str,
        date: Option<String>,
        source_type: SourceType,
    ) -> Result<()> {
        let query = r#"
         INSERT INTO logs (content, raw_context, tags, source_type, created_at)
         VALUES (?1, '{}', '[]', ?2, COALESCE(?3, CURRENT_TIMESTAMP))"#;

        let date = match date {
            Some(d) => Some(to_iso8601_timestamp(&d)?),
            None => None,
        };

        debug!(
            "Executing {query} with args {:?}",
            (content, source_type.as_str(), &date)
        );

        self.connection
            .execute(query, (content, source_type.as_str(), date))?;

        debug!("Query complete.");

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
        let query =
            "SELECT id, content, source_type, created_at FROM logs WHERE strftime('%Y', created_at) = ?1";

        debug!("Preparing {query}");

        let mut statement = self.connection.prepare(query)?;
        let rows = statement
            .query_map([year.to_string()], |row| {
                Ok(Log {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    source_type: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to fetch logs.")?;

        debug!("Query complete.");

        Ok(rows)
    }
}
