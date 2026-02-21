use anyhow::{Context, Result};
use config::builder::DefaultState;
use config::{ConfigBuilder, File, FileFormat};
use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Database {
    #[serde(default = "database_filename")]
    pub filename: String,
}
#[rustfmt::skip]
fn database_filename() -> String { String::from("gj.sqlite3") }

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Draft {
    #[serde(default)]
    pub git_email: String,

    #[serde()]
    pub from_git: Option<bool>,

    #[serde()]
    pub refine_with_ollama: Option<bool>,

    #[serde()]
    pub redact_sensitive_with_ollama: Option<bool>,

    #[serde()]
    pub ollama_model: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct NotionConfig {
    #[serde()]
    pub api_key: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Push {
    #[serde(default)]
    pub notion: NotionConfig,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub database: Database,

    #[serde(default)]
    pub draft: Draft,

    #[serde(default)]
    pub push: Push,
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

        let config = config_raw
            .try_deserialize()
            .context("Failed to deserialize the config.");

        debug!("Parsed Config: {:?}", config);

        config
    }
}
