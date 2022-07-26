#[cfg(test)]
mod tests {
    use crate::core::{User, Transaction, Ledger};
    use crate::core::transaction::Benefit;

    use rstest::{fixture, rstest};
    use serde_json::{to_string, json};

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

    #[rstest]
    fn transaction_serialize(transaction: Transaction) {
        let repr = serde_json::to_string_pretty(&transaction).unwrap();
        println!("{}", repr);

        let parsed = serde_json::from_str::<Transaction>(&repr).unwrap();

        assert_eq!(transaction.description, parsed.description);
        assert_eq!(transaction.is_direct, parsed.is_direct);
        assert_eq!(transaction.total_spending(), parsed.total_spending());
        assert_eq!(transaction.balance_updates().unwrap(), parsed.balance_updates().unwrap());
    }
}