use anyhow::{Context, Result};
use config::builder::DefaultState;
use config::{ConfigBuilder, File, FileFormat};
use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct User {
    #[serde(default)]
    pub email: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Database {
    #[serde(default = "database_filename")]
    pub filename: String,
}
#[rustfmt::skip]
fn database_filename() -> String { String::from("gj.sqlite3") }

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub user: User,

    #[serde(default)]
    pub database: Database,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config_raw = ConfigBuilder::<DefaultState>::default()
            .add_source(
                File::with_name(".gj.toml")
                    .format(FileFormat::Toml)
                    .required(false),
            )
            .build()
            .context("Failed to build the config hierarchy.")?;

        debug!("Loaded Config: {:?}", config_raw);

        config_raw
            .try_deserialize()
            .context("Failed to deserialize the config.")
    }
}
