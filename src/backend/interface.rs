use crate::core::Ledger;

pub trait LedgerStore {
    fn read(&self) -> anyhow::Result<Ledger>;
    fn save(&self, ledger: &Ledger) -> anyhow::Result<()>;
}
