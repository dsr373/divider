use std::error;
use std::result;

use crate::core::Ledger;

pub type BackendError = Box<dyn error::Error>;

pub type Result<T> = result::Result<T, BackendError>;

pub trait LedgerStore {
    fn read(&self) -> Result<Ledger>;
    fn save(&self, ledger: &Ledger) -> Result<()>;
}
