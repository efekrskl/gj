use crate::configuration::AppConfig;
use anyhow::Result;

mod configuration;

fn main() -> Result<()> {
    env_logger::init();
    let _config = AppConfig::load()?;

    Ok(())
}
