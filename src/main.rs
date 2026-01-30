use crate::configuration::AppConfig;
use crate::database::Database;
use anyhow::Result;

mod configuration;
mod database;

fn main() -> Result<()> {
    env_logger::init();
    let config = AppConfig::load()?;
    let db = Database::open(config.database.filename)?;

    Ok(())
}
