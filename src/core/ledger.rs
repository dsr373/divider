use std::collections::{HashMap, HashSet};

use crate::core::user::User;
use crate::core::transaction::{
    Transaction, TransactionResult, TransactionError,
    AmountPerUserRef, BenefitPerUserRef,
    Benefit, Amount};

type UserAmountMap = HashMap<User, Amount>;

pub struct Ledger {
    balances: UserAmountMap,
    transactions: Vec<Transaction>,
    total_spend: Amount
}

impl Ledger {
    const CONSISTENCY_CHECK_INTERVAL: usize = 100;

    pub fn new(user_names: Vec<&str>) -> Ledger {
        let mut balances = HashMap::new();

        for name in user_names {
            balances.insert(User::new(&name), 0f32 as Amount);
        }

        return Ledger { balances, transactions: Vec::new(), total_spend: 0f32 as Amount };
    }

    pub fn get_users(&self) -> HashSet<&User> {
        return self.balances.keys()
            .map(|user| user).collect();
    }

    pub fn get_user_by_name(&self, name: &str) -> Option<&User> {
        self.balances.keys()
            .find_map(|user| if user.name == name { Some(user) } else {None})
    }

    fn update_balances(balances: &mut UserAmountMap, changes: UserAmountMap) -> TransactionResult<()> {
        for (user, delta) in &changes {
            match balances.get_mut(user) {
                Some(val) => *val += delta,
                None => return Err(TransactionError::UnrecognisedUser(user.clone()))
            }
        }
        return Ok(());
    }

    fn consistency_check(&mut self) -> TransactionResult<()> {
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

    // TODO: separate into smaller functions
    fn apply_transaction(total_spend: &mut Amount, balances: &mut UserAmountMap, transaction: &Transaction) -> TransactionResult<()> {
        if transaction.is_direct {
            *total_spend += transaction.total_spending();
        }
        let balance_updates = transaction.balance_updates()?;
        return Ledger::update_balances(balances, balance_updates);
    }

    fn add_transaction(&mut self, transaction: Transaction) -> TransactionResult<()> {
        Ledger::apply_transaction(&mut self.total_spend, &mut self.balances, &transaction)?;
        self.transactions.push(transaction);

        if self.needs_consistency_check() {
            self.consistency_check()?;
        }
        return Ok(());
    }

    pub fn add_expense(&mut self, contributions: AmountPerUserRef, benefits: BenefitPerUserRef, description: &str) -> TransactionResult<()> {
        let transaction = Transaction::new(contributions, benefits, description);
        self.add_transaction(transaction)
    }

    pub fn add_transfer(&mut self, from: &User, to: &User, amount: Amount, description: &str) -> TransactionResult<()> {
        let transaction = Transaction::new_direct(
            vec![(from, amount)],
            vec![(to, Benefit::Sum(amount))],
            description
        );
        self.add_transaction(transaction)
    }
}


#[cfg(test)]
mod tests {
    use crate::core::{Ledger, User};

    use crate::core::transaction::Benefit;

    #[test]
    fn create_and_get_users() {
        let ledger = Ledger::new(vec!["Bilbo", "Frodo", "Legolas", "Gimli"]);

        let users = ledger.get_users();

        assert_eq!(users.len(), 4);
        assert!(users.contains(&User::new("Bilbo")));
        assert!(users.contains(&User::new("Frodo")));
        assert!(users.contains(&User::new("Legolas")));
        assert!(users.contains(&User::new("Gimli")));
        assert!(!users.contains(&User::new("Merry")));
    }

    #[test]
    fn create_and_find_user() {
        let ledger = Ledger::new(vec!["Bilbo", "Frodo", "Legolas", "Gimli"]);
        let frodo = ledger.get_user_by_name("Frodo").unwrap();
        assert_eq!(frodo, &User::new("Frodo"));
    }

    #[test]
    fn simple_transfer() {
        let mut ledger = Ledger::new(vec!["Bilbo", "Frodo", "Legolas", "Gimli"]);

        let bilbo = ledger.get_user_by_name("Bilbo").unwrap().to_owned();
        let frodo = ledger.get_user_by_name("Frodo").unwrap().to_owned();

        ledger.add_transfer(&bilbo, &frodo, 32.0, "").unwrap();

        assert_eq!(*ledger.balances.get(&bilbo).unwrap(), 32.0);
        assert_eq!(*ledger.balances.get(&bilbo).unwrap(), 32.0);
        assert_eq!(*ledger.balances.get(&User::new("Gimli")).unwrap(), 0.0);
    }

    #[test]
    fn unrecognised_user() {
        let mut ledger = Ledger::new(vec!["Bilbo", "Frodo", "Legolas", "Gimli"]);

        let bilbo = User::new("Bilbo");
        let merry = User::new("Merry");

        let res = ledger.add_transfer(&bilbo, &merry, 32.0, "");

        assert!(res.is_err());
    }

    #[test]
    fn complex_expense() {
        let mut ledger = Ledger::new(vec!["Bilbo", "Frodo", "Legolas", "Gimli"]);

        let bilbo = ledger.get_user_by_name("Bilbo").unwrap().to_owned();
        let frodo = ledger.get_user_by_name("Frodo").unwrap().to_owned();
        let legolas = ledger.get_user_by_name("Legolas").unwrap().to_owned();
        let gimli = ledger.get_user_by_name("Gimli").unwrap().to_owned();

        ledger.add_expense(vec![
            (&bilbo, 60.0)
        ], vec![
            (&frodo, Benefit::Even),
            (&legolas, Benefit::Even),
            (&bilbo, Benefit::Even)
        ], "").unwrap();

        /* here:
            Bilbo +60 -20 = +40
            Legolas         -20
            Frodo           -20
         */

        ledger.add_expense(vec![
            (&frodo, 30.0)
        ], vec![
            (&frodo, Benefit::Even),
            (&legolas, Benefit::Sum(6.0)),
            (&gimli, Benefit::Even)
        ], "").unwrap();

        assert_eq!(ledger.transactions.len(), 2);
        assert_eq!(*ledger.balances.get(&bilbo).unwrap(), 40.0);
        assert_eq!(*ledger.balances.get(&frodo).unwrap(), -2.0);
        assert_eq!(*ledger.balances.get(&legolas).unwrap(), -26.0);
        assert_eq!(*ledger.balances.get(&gimli).unwrap(), -12.0);
    }
}