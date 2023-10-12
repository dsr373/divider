use std::{path::{Path, PathBuf}, collections::HashMap};

use anyhow::{self, Context};
use serde::{Serialize, Deserialize};
use rocket::tokio::fs;
use toml;

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageConfig {
    prefix: PathBuf,
    ledgers_map: PathBuf,
    users_map: PathBuf
}

impl StorageConfig {
    pub async fn read(filepath: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file_content = fs::read_to_string(filepath)
            .await
            .with_context(|| "failed to read config file")?;
        let config = toml::from_str(&file_content)
            .with_context(|| "failed to parse config file")?;
        return Ok(config);
    }
}

type LedgerLocationById = HashMap<String, PathBuf>;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    storage: StorageConfig,
    pub ledgers: LedgerLocationById
}

impl AppConfig {
    pub async fn read(filepath: impl AsRef<Path>) -> anyhow::Result<Self> {
        let storage = StorageConfig::read(filepath).await?;

        let ledgers_map_content = fs::read_to_string(&storage.ledgers_map)
            .await
            .with_context(|| "failed to read ledgers file")?;
        let ledgers = serde_json::from_str(&ledgers_map_content)
            .with_context(|| "failed to parse ledgers file")?;

        return Ok(AppConfig{ storage, ledgers });
    }
}
