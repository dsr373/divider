use std::path::{Path, PathBuf};
use std::fs;

use crate::backend::{LedgerStore, Result};
use crate::Ledger;

pub struct JsonStore {
    file_path: PathBuf
}

impl JsonStore {
    pub fn new(path: &Path) -> JsonStore {
        return JsonStore { file_path: path.to_owned() };
    }
}

impl LedgerStore for JsonStore {
    fn read(&self) -> Result<Ledger> {
        let file_contents = fs::read_to_string(&self.file_path)?;
        return serde_json::from_str::<Ledger>(&file_contents)
            .map_err(|err| err.into());
    }

    fn save(&self, ledger: &Ledger) -> Result<()> {
        let ledger_str = serde_json::to_string_pretty(ledger)?;
        fs::write(&self.file_path, ledger_str)?;
        return Ok(());
    }
}
