use anyhow::{Result};
mod configuration;

use crate::configuration::get_config;

fn main() -> Result<()> {
    let config = get_config()?;

    println!("{:?}", config);
    
    Ok(())
}
