use anyhow::{Context, Result};
use log::debug;
use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};

pub struct Database {
    pub connection: Connection,
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
    pub fn open(sqlite_filename: String) -> Result<Database> {
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
}
