use std::{path::{Path, PathBuf}, collections::HashMap};

use serde::{Serialize, Deserialize};
use rocket::tokio::fs;
use thiserror::Error;
use toml;

#[derive(Error, Debug)]
pub enum ServerConfigError {
    #[error("failed to read config file: {0:?}")]
    FailedToRead(#[from] std::io::Error),

    #[error("failed to parse config file: {0:?}")]
    FailedToParse(#[from] toml::de::Error),

    #[error("failed to parse ledgers file: {0:?}")]
    FailedToParseLedgers(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageConfig {
    prefix: PathBuf,
    ledgers_map: PathBuf,
    users_map: PathBuf
}

impl StorageConfig {
    pub async fn read(filepath: impl AsRef<Path>) -> Result<Self, ServerConfigError> {
        let file_content = fs::read_to_string(filepath)
            .await?;
        let config = toml::from_str(&file_content)?;
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
    pub async fn read(filepath: impl AsRef<Path>) -> Result<Self, ServerConfigError> {
        let storage = StorageConfig::read(filepath).await?;

        let ledgers_map_content = fs::read_to_string(&storage.ledgers_map)
            .await?;
        let ledgers = serde_json::from_str(&ledgers_map_content)?;

        return Ok(AppConfig{ storage, ledgers });
    }
}
