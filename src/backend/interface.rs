use crate::core::Ledger;

pub trait LedgerStore {
    fn read(&self) -> Ledger;
    fn save(&self, ledger: &Ledger);
}
