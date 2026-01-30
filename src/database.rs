use crate::utils::to_iso8601_timestamp;
use anyhow::{Context, Result};
use log::debug;
use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};

pub struct Database {
    connection: Connection,
}

#[derive(Debug)]
pub struct Log {
    pub id: i64,
    pub content: String,
    // raw_context: String,
    // tags: String,
    // source_type: String,
    // created_at: i64,
    // updated_at: i64,
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

    pub fn add_log(&self, content: &str, date: Option<String>) -> Result<()> {
        let query = r#"
         INSERT INTO logs (content, raw_context, tags, source_type, created_at)
         VALUES (?1, '{}', '[]', ?2, ?3)"#;

        debug!("Executing {query}");

        let date = match date {
            Some(d) => Some(to_iso8601_timestamp(&d)?),
            None => None,
        };
        self.connection.execute(query, (content, "manual", date))?;

        debug!("Query complete.");

        Ok(())
    }

    pub fn get_recent_logs(&self) -> Result<Vec<Log>> {
        let query = "SELECT id, content FROM logs LIMIT 10";

        debug!("Preparing {query}");

        let mut statement = self.connection.prepare(query)?;
        let rows = statement
            .query_map([], |row| {
                Ok(Log {
                    id: row.get(0)?,
                    content: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to fetch logs.")?;

        debug!("Query complete.");

        Ok(rows)
    }
}
