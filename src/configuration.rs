use anyhow::{Context, Result};
use config::builder::DefaultState;
use config::{ConfigBuilder, File, FileFormat};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct User {
    #[serde(default)]
    pub email: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub user: User,
}

pub fn get_config() -> Result<AppConfig> {
    let config_raw = ConfigBuilder::<DefaultState>::default()
        .add_source(
            File::with_name(".gj.toml")
                .format(FileFormat::Toml)
                .required(false),
        )
        .build()
        .context("Failed to build the config hierarchy.")?;

    config_raw
        .try_deserialize()
        .context("Failed to deserialize the config.")
}
