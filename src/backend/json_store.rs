use std::path::{Path, PathBuf};
use std::fs;

use crate::backend::LedgerStore;
use crate::Ledger;

pub struct JsonStore {
    file_path: PathBuf
}

impl JsonStore {
    pub fn new<P: AsRef<Path>>(path: P) -> JsonStore {
        return JsonStore { file_path: path.as_ref().to_owned() };
    }
}

impl LedgerStore for JsonStore {
    fn read(&self) -> anyhow::Result<Ledger> {
        let file_contents = fs::read_to_string(&self.file_path)?;
        return serde_json::from_str::<Ledger>(&file_contents)
            .map_err(|err| err.into());
    }

    fn save(&self, ledger: &Ledger) -> anyhow::Result<()> {
        let ledger_str = serde_json::to_string_pretty(ledger)?;
        fs::write(&self.file_path, ledger_str)?;
        return Ok(());
    }
}
