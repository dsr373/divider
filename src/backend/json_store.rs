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

#[cfg(test)]
mod tests {
    use crate::UserName;
    use crate::core::{Transaction, Ledger};
    use crate::core::transaction::Benefit;
    use crate::transaction::{AmountPerUser, BenefitPerUser};

    use rstest::{fixture, rstest};
    use serde_json::json;

    type UserNames4 = (UserName, UserName, UserName, UserName);

    #[fixture]
    fn users() -> UserNames4 {
        let bilbo = String::from("Bilbo");
        let frodo = String::from("Frodo");
        let legolas = String::from("Legolas");
        let gimli = String::from("Gimli");
        return (bilbo, frodo, legolas, gimli);
    }

    #[fixture]
    fn transaction(users: UserNames4) -> Transaction {
        let (bilbo, frodo, legolas, gimli) = users;
        let contrib: AmountPerUser<&str> = vec![
            (&bilbo, 32.0),
            (&frodo, 12.0)
        ];

        let benefit: BenefitPerUser<&str> = vec![
            (&legolas, Benefit::Even),
            (&frodo, Benefit::Even),
            (&gimli, Benefit::Sum(10.0))
        ];

        return Transaction::new(contrib, benefit, "", false);
    }

    #[fixture]
    fn transaction_json() -> serde_json::Value {
        json!({
            "contributions": [
                ["Bilbo", 32.0],
                ["Frodo", 12.0]
            ],
            "benefits": [
                ["Legolas", "Even"],
                ["Frodo", "Even"],
                ["Gimli", {"Sum": 10.0}],
            ],
            "is_direct": false,
            "description": ""
        })
    }

    #[fixture]
    fn ledger_json(transaction_json: serde_json::Value) -> serde_json::Value {
        json!({
            "balances": {
                "Bilbo": 32.0,
                "Frodo": -5.0,
                "Legolas": -17.0,
                "Gimli": -10.0,
            },
            "users": {
                "Bilbo": {"name": "Bilbo"},
                "Frodo": {"name": "Frodo"},
                "Legolas": {"name": "Legolas"},
                "Gimli": {"name": "Gimli"},
            },
            "total_spend": 44.0,
            "transactions": [transaction_json]
        })
    }

    #[rstest]
    fn transaction_serialize(transaction: Transaction, transaction_json: serde_json::Value) {
        let value = serde_json::to_value(&transaction).unwrap();
        assert_eq!(value, transaction_json);
    }

    #[rstest]
    fn transaction_deserialize(transaction: Transaction, transaction_json: serde_json::Value) {
        let parsed = serde_json::from_value::<Transaction>(transaction_json).unwrap();
        assert_eq!(transaction.description, parsed.description);
        assert_eq!(transaction.is_direct, parsed.is_direct);
        assert_eq!(transaction.total_spending(), parsed.total_spending());
        assert_eq!(transaction.balance_updates().unwrap(), parsed.balance_updates().unwrap());
    }

    #[fixture]
    fn ledger(transaction: Transaction) -> Ledger {
        let mut ledger = Ledger::new(vec!["Bilbo", "Frodo", "Legolas", "Gimli"]);
        ledger.add_transaction(transaction).unwrap();
        return ledger;
    }

    #[rstest]
    fn ledger_serialize(ledger: Ledger, ledger_json: serde_json::Value) {
        let serialised = serde_json::to_value(&ledger).unwrap();
        assert_eq!(serialised["transactions"], ledger_json["transactions"]);
        assert_eq!(serialised["total_spend"], ledger_json["total_spend"]);
        for v in ledger_json["balances"].as_object().unwrap() {
            assert!(serialised["balances"].as_object().unwrap().contains_key(v.0));
        }
    }

    #[rstest]
    fn ledger_deserialize(ledger: Ledger, ledger_json: serde_json::Value) {
        let deserialised = serde_json::from_value::<Ledger>(ledger_json).unwrap();

        let users_ledger = ledger.get_users();
        for user in deserialised.get_users() {
            assert!(users_ledger.contains(&user));
        }

        let balances_ledger = ledger.get_balances();
        for (name, balance) in deserialised.get_balances() {
            assert!(balances_ledger.contains_key(&name));
            assert_eq!(balances_ledger.get(&name).unwrap(), &balance);
        }
    }
}