mod json_store;
mod interface;

pub use interface::{LedgerStore, Result, BackendError};
pub use json_store::JsonStore;