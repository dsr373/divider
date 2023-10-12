use std::{fs, path::{Path, PathBuf}, collections::HashMap};
use serde::{Serialize, Deserialize};
use toml;
use anyhow::{self, Context};

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageConfig {
    prefix: PathBuf,
    ledgers_map: PathBuf,
    users_map: PathBuf
}

impl StorageConfig {
    pub fn read(filepath: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file_content = fs::read_to_string(filepath)
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
    pub fn read(filepath: impl AsRef<Path>) -> anyhow::Result<Self> {
        let storage = StorageConfig::read(filepath)?;

        let file = fs::File::open(&storage.ledgers_map)
            .with_context(|| "failed to open ledgers file")?;
        let ledgers = serde_json::from_reader(file)
            .with_context(|| "failed to parse ledgers file")?;

        return Ok(AppConfig{ storage, ledgers });
    }
}
