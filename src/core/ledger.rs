use std::collections::HashMap;

use crate::core::user::{User, UserName, Amount};
use crate::core::transaction::{
    Transaction, TransactionResult,
    Benefit, AmountPerUser, BenefitPerUser, UserAmountMap};
use crate::core::error::TransactionError;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};


type UserMap = HashMap<UserName, User>;

#[derive(Serialize, Deserialize)]
pub struct Ledger {
    next_id: usize,
    balances: UserAmountMap,
    users: UserMap,
    transactions: Vec<Transaction>,
    total_spend: Amount
}


impl Ledger {
    const CONSISTENCY_CHECK_INTERVAL: usize = 100;

    pub fn new<T: AsRef<str>>(user_names: Vec<T>) -> Ledger {
        let users = user_names.iter()
            .map(|user_name| (String::from(user_name.as_ref()), User::new(user_name.as_ref())))
            .collect();

        let balances = user_names.iter()
            .map(|user_name| (String::from(user_name.as_ref()), 0.0 as Amount))
            .collect();

        return Ledger { next_id: 1, balances, users, transactions: Vec::new(), total_spend: 0.0 };
    }

    pub fn get_users(&self) -> Vec<&User> {
        return self.users.values().collect();
    }

    pub fn get_balances(&self) -> UserAmountMap {
        return self.balances.iter()
            .map(|pair| (pair.0.to_owned(), pair.1.to_owned()))
            .collect();
    }

    pub fn get_transactions(&self) -> &Vec<Transaction> {
        return &self.transactions;
    }

    pub fn add_user(&mut self, name: &str) {
        self.users.insert(name.to_owned(), User::new(name));
    }

    pub fn add_expense(&mut self, contributions: AmountPerUser<&str>, benefits: BenefitPerUser<&str>,
        description: &str, time: Option<DateTime<Utc>>) -> TransactionResult<()> {
        let transaction = Transaction::new(
            contributions,
            benefits,
            description,
            false,
            None,
            time);
        self.add_transaction(transaction)
    }

    pub fn add_transfer(&mut self, from: &str, to: &str, amount: Amount,
        description: &str, time: Option<DateTime<Utc>>) -> TransactionResult<()> {
        let transaction = Transaction::new(
            vec![(from, amount)],
            vec![(to, Benefit::Sum(amount))],
            description,
            true,
            None,
            time
        );
        self.add_transaction(transaction)
    }

    pub fn add_transaction(&mut self, mut transaction: Transaction) -> TransactionResult<()> {
        Ledger::apply_transaction(&mut self.total_spend, &mut self.balances, &transaction)?;
        self.assign_transaction_id(&mut transaction);
        self.transactions.push(transaction);

        if self.needs_consistency_check() {
            self.reapply_all()?;
        }
        return Ok(());
    }

    fn assign_transaction_id(&mut self, transaction: &mut Transaction) {
        transaction.id = self.next_id;
        self.next_id += 1;
    }

    fn apply_transaction(total_spend: &mut Amount, balances: &mut UserAmountMap, transaction: &Transaction) -> TransactionResult<()> {
        if !transaction.is_direct {
            *total_spend += transaction.total_spending();
        }
        let balance_updates = transaction.balance_updates()?;
        return Ledger::update_balances(balances, balance_updates);
    }

    fn update_balances(balances: &mut UserAmountMap, changes: UserAmountMap) -> TransactionResult<()> {
        for (user, delta) in &changes {
            match balances.get_mut(user) {
                Some(val) => *val += delta,
                None => return Err(TransactionError::UnknownUser(user.clone()))
            }
        }
        return Ok(());
    }

    fn reapply_all(&mut self) -> TransactionResult<()> {
        let mut new_balances: UserAmountMap =
            self.balances.keys().map(|user| (user.clone(), 0.0)).collect();
        let mut new_total: Amount = 0.0;

        for transaction in &self.transactions {
            Ledger::apply_transaction(&mut new_total, &mut new_balances, transaction)?;
        }

        self.total_spend = new_total;
        self.balances = new_balances;
        return Ok(());
    }

    fn needs_consistency_check(&self) -> bool {
        return self.transactions.len() % Self::CONSISTENCY_CHECK_INTERVAL == 0;
    }
}


#[cfg(test)]
mod tests {
    use crate::core::{Ledger, User, UserName};
    use crate::core::transaction::Benefit;
    use crate::core::error::TransactionError;
    use crate::transaction::{AmountPerUser, BenefitPerUser};

    use rstest::{fixture, rstest};

    type UserNames4 = (UserName, UserName, UserName, UserName);

    #[fixture]
    fn ledger() -> Ledger {
        return Ledger::new(vec!["Bilbo", "Frodo", "Legolas", "Gimli"]);
    }

    #[fixture]
    fn user_names() -> UserNames4 {
        let bilbo = String::from("Bilbo");
        let frodo = String::from("Frodo");
        let legolas = String::from("Legolas");
        let gimli = String::from("Gimli");

        return (bilbo, frodo, legolas, gimli);
    }

    #[rstest]
    fn create_and_get_users(ledger: Ledger, user_names: UserNames4) {
        let (bilbo, frodo, legolas, gimli) = user_names;

        let users = ledger.get_users();

        assert_eq!(users.len(), 4);
        assert!(users.contains(&&User::new(&bilbo)));
        assert!(users.contains(&&User::new(&frodo)));
        assert!(users.contains(&&User::new(&legolas)));
        assert!(users.contains(&&User::new(&gimli)));
        assert!(!users.contains(&&User::new("Merry")));
    }

    #[rstest]
    fn simple_transfer(mut ledger: Ledger, user_names: UserNames4) {
        let (bilbo, frodo, _, gimli) = user_names;

        ledger.add_transfer(&bilbo, &frodo, 32.0, "", None).unwrap();

        assert_eq!(ledger.total_spend, 0.0);
        assert_eq!(*ledger.balances.get(&bilbo).unwrap(), 32.0);
        assert_eq!(*ledger.balances.get(&frodo).unwrap(), -32.0);
        assert_eq!(*ledger.balances.get(&gimli).unwrap(), 0.0);
    }

    #[rstest]
    fn unrecognised_user(mut ledger: Ledger, user_names: UserNames4) {
        let bilbo = user_names.0;
        let merry = String::from("Merry");

        let res = ledger.add_transfer(&bilbo, &merry, 32.0, "", None);

        assert!(res.is_err());
        assert!(matches!(res, Err(TransactionError::UnknownUser(..))));
    }

    fn add_transaction_bilbo(ledger: &mut Ledger, user_names: &UserNames4) {
        let (bilbo, frodo, legolas, _) = user_names;
        let contributions: AmountPerUser<&str> = vec![(bilbo, 60.0)];
        let benefits: BenefitPerUser<&str> = vec![
            (frodo, Benefit::Even),
            (legolas, Benefit::Even),
            (bilbo, Benefit::Even)
        ];
        ledger.add_expense(contributions, benefits, "", None).unwrap()
    }

    fn add_transaction_frodo(ledger: &mut Ledger, user_names: &UserNames4) {
        let (_, frodo, legolas, gimli) = user_names;
        let contributions: AmountPerUser<&str> = vec![(frodo, 30.0)];
        let benefits: BenefitPerUser<&str> = vec![
            (frodo, Benefit::Even),
            (legolas, Benefit::Sum(6.0)),
            (gimli, Benefit::Even)
        ];
        ledger.add_expense(contributions, benefits, "", None).unwrap()
    }

    #[rstest]
    fn complex_expense(mut ledger: Ledger, user_names: UserNames4) {
        let (bilbo, frodo, legolas, gimli) = &user_names;

        add_transaction_bilbo(&mut ledger, &user_names);
        add_transaction_frodo(&mut ledger, &user_names);

        assert_eq!(ledger.transactions.len(), 2);
        assert_eq!(ledger.total_spend, 90.0);
        assert_eq!(*ledger.balances.get(bilbo).unwrap(), 40.0);
        assert_eq!(*ledger.balances.get(frodo).unwrap(), -2.0);
        assert_eq!(*ledger.balances.get(legolas).unwrap(), -26.0);
        assert_eq!(*ledger.balances.get(gimli).unwrap(), -12.0);
    }

    #[rstest]
    fn consistency_check(mut ledger: Ledger, user_names: UserNames4) {
        const INTERVAL: usize = Ledger::CONSISTENCY_CHECK_INTERVAL;
        let (bilbo, frodo, legolas, gimli) = &user_names;

        let repeated_transactions = (INTERVAL - 1)/2;

        for _ in 0..repeated_transactions {
            add_transaction_bilbo(&mut ledger, &user_names);
            add_transaction_frodo(&mut ledger, &user_names);
        }

        // before reapplying all
        assert_eq!(*ledger.balances.get(bilbo).unwrap(), (repeated_transactions as f32) * 40.0);
        assert_eq!(*ledger.balances.get(frodo).unwrap(), (repeated_transactions as f32) * -2.0);
        assert_eq!(*ledger.balances.get(legolas).unwrap(), (repeated_transactions as f32) * -26.0);
        assert_eq!(*ledger.balances.get(gimli).unwrap(), (repeated_transactions as f32) * -12.0);

        // mess with one of the values
        *ledger.balances.get_mut(bilbo).unwrap() += 100.0;

        // one of these should do the consistency check
        add_transaction_bilbo(&mut ledger, &user_names);
        add_transaction_frodo(&mut ledger, &user_names);

        // after reapplying all
        assert_eq!(*ledger.balances.get(bilbo).unwrap(), ((repeated_transactions + 1) as f32) * 40.0);
        assert_eq!(*ledger.balances.get(frodo).unwrap(), ((repeated_transactions + 1) as f32) * -2.0);
        assert_eq!(*ledger.balances.get(legolas).unwrap(), ((repeated_transactions + 1) as f32) * -26.0);
        assert_eq!(*ledger.balances.get(gimli).unwrap(), ((repeated_transactions + 1) as f32) * -12.0);
    }
}


#[cfg(test)]
mod serialise_tests {
    use crate::UserName;
    use crate::core::{Transaction, Ledger};
    use crate::core::transaction::Benefit;
    use crate::transaction::{AmountPerUser, BenefitPerUser};

    use rstest::{fixture, rstest};
    use serde_json::json;
    use chrono::{Utc, TimeZone};

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

        let time = Utc.ymd(2022, 5, 1).and_hms(11, 0, 0);

        return Transaction::new(contrib, benefit, "", false,
            Some(3), Some(time));
    }

    #[fixture]
    fn transaction_json() -> serde_json::Value {
        json!({
            "id": 3,
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
            "description": "",
            "datetime": "2022-05-01T11:00:00+00:00"
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
