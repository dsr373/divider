use std::path::{Path, PathBuf};
use std::fs;

use crate::backend::LedgerStore;
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
    fn read(&self) -> Ledger {
        let file_contents = fs::read_to_string(&self.file_path)
            .expect("File could not be read.");
        return serde_json::from_str(&file_contents)
            .expect(&format!("Parsing JSON ledger failed from file: {}", self.file_path.display()));
    }

    fn save(&self, ledger: &Ledger) {
        let ledger_str = serde_json::to_string(ledger)
            .expect("Failed to serialise ledger to string.");
        fs::write(&self.file_path, ledger_str)
            .expect(&format!("Failed to write JSON ledger to path: {}", self.file_path.display()));
    }
}
