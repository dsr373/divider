#[cfg(test)]
mod tests {
    use crate::core::{User, Transaction, Ledger};
    use crate::core::transaction::Benefit;

    use rstest::{fixture, rstest};
    use serde_json::json;

    #[fixture]
    fn users() -> (User, User, User, User) {
        let bilbo = User::new("Bilbo");
        let frodo = User::new("Frodo");
        let legolas = User::new("Legolas");
        let gimli = User::new("Gimli");
        return (bilbo, frodo, legolas, gimli);
    }

    #[fixture]
    fn transaction(users: (User, User, User, User)) -> Transaction {
        let (bilbo, frodo, legolas, gimli) = users;
        let contrib = vec![
            (&bilbo, 32.0),
            (&frodo, 12.0)
        ];

        let benefit = vec![
            (&legolas, Benefit::Even),
            (&frodo, Benefit::Even),
            (&gimli, Benefit::Sum(10.0))
        ];

        return Transaction::new(contrib, benefit, "");
    }

    #[fixture]
    fn transaction_json() -> serde_json::Value {
        json!({
            "contributions": [
                [{"name": "Bilbo"}, 32.0],
                [{"name": "Frodo"}, 12.0]
            ],
            "benefits": [
                [{"name": "Legolas"}, "Even"],
                [{"name": "Frodo"}, "Even"],
                [{"name": "Gimli"}, {"Sum": 10.0}],
            ],
            "is_direct": false,
            "description": ""
        })
    }

    #[fixture]
    fn ledger_json(transaction_json: serde_json::Value) -> serde_json::Value {
        json!({
            "balances": [
                [{"name": "Bilbo"}, 32.0],
                [{"name": "Frodo"}, -5.0],
                [{"name": "Legolas"}, -17.0],
                [{"name": "Gimli"}, -10.0],
            ],
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
        for v in ledger_json["balances"].as_array().unwrap() {
            assert!(serialised["balances"].as_array().unwrap().contains(&v));
        }
    }

    #[rstest]
    fn ledger_deserialize(ledger: Ledger, ledger_json: serde_json::Value) {
        let deserialised = serde_json::from_value::<Ledger>(ledger_json).unwrap();
        assert_eq!(deserialised.get_users(), ledger.get_users());
        assert_eq!(deserialised.get_balances(), ledger.get_balances());
    }
}