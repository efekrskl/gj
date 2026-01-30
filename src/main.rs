use crate::configuration::AppConfig;
use crate::database::Database;
use anyhow::Result;
use rusqlite::{Connection, params};
use rusqlite_migration::{M, Migrations};

mod configuration;
mod database;

fn main() -> Result<()> {
    env_logger::init();
    let config = AppConfig::load()?;
    let db = Database::open(config.database.filename)?;

    db.connection.execute(
        "CREATE TABLE IF NOT EXISTS project(
            project_name TEXT PRIMARY KEY,
            description  TEXT,
            deadline     DATE
        )",
        (),
    )?;

    Ok(())
}
