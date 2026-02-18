use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct NotionAdapterConfigRaw {
    #[serde(default = "notion_cfg_v1")]
    pub version: u32,
    pub database_id: Option<String>,
    pub root_page_id: Option<String>,
    pub notion_version: Option<String>,
}

fn notion_cfg_v1() -> u32 {
    1
}

#[derive(Debug, Clone)]
pub struct NotionAdapterConfig {
    pub version: u32,
    pub database_id: String,
    pub root_page_id: String,
    pub notion_version: String,
}

// todo: migrations

impl NotionAdapterConfigRaw {
    pub fn from_json(json: &str) -> Result<NotionAdapterConfig> {
        let raw: Self = serde_json::from_str(json).context("Invalid Notion adapter config JSON")?;

        raw.validate()
    }

    fn validate(self) -> Result<NotionAdapterConfig> {
        match self.version {
            1 => {}
            v => bail!("Unsupported Notion config version: {}", v),
        }

        let database_id = match self.database_id {
            Some(id) if !id.trim().is_empty() => id,
            Some(_) => bail!("notion.database_id is empty"),
            None => bail!("notion.database_id is missing"),
        };

        let root_page_id = match self.root_page_id {
            Some(id) if !id.trim().is_empty() => id,
            Some(_) => bail!("notion.root_page_id is empty"),
            None => bail!("notion.root_page_id is missing"),
        };

        let notion_version = match self.notion_version {
            Some(v) if !v.trim().is_empty() => v,
            Some(_) => bail!("notion.notion_version is empty"),
            None => "2022-06-28".to_string(),
        };

        Ok(NotionAdapterConfig {
            version: self.version,
            database_id,
            root_page_id,
            notion_version,
        })
    }
}
