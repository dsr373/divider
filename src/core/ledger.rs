use std::collections::{HashMap, HashSet};

use crate::core::user::User;
use crate::core::transaction::{Transaction, Amount, TransactionResult, TransactionError, AmountPerUser, BenefitPerUser, Benefit};

pub struct Ledger {
    balances: HashMap<User, Amount>,
    transactions: Vec<Transaction>,
    total_spend: Amount
}

impl Ledger {
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

    fn update_balances(&mut self, changes: HashMap<User, Amount>) -> TransactionResult<()> {
        for (user, delta) in &changes {
            match self.balances.get_mut(user) {
                Some(val) => *val += delta,
                None => return Err(TransactionError::UnrecognisedUser(user.clone()))
            }
        }
        return Ok(());
    }

    // TODO: separate into smaller functions
    fn apply_transaction(&mut self, transaction: &Transaction) -> TransactionResult<()> {
        if transaction.is_direct {
            self.total_spend += transaction.total_spending();
        }
        let balance_updates = transaction.balance_updates()?;
        return self.update_balances(balance_updates);
    }

    pub fn add_expense(&mut self, contributions: AmountPerUser, benefits: BenefitPerUser) -> TransactionResult<()> {
        let transaction = Transaction::new(contributions, benefits);
        self.apply_transaction(&transaction)?;
        self.transactions.push(transaction);
        return Ok(());
    }

    pub fn add_transfer(&mut self, from: &User, to: &User, amount: Amount) -> TransactionResult<()> {
        let transaction = Transaction::new_direct(
            vec![(from.clone(), amount)],
            vec![(to.clone(), Benefit::Sum(amount))]
        );
        self.apply_transaction(&transaction)?;
        self.transactions.push(transaction);
        return Ok(());
    }
}


#[cfg(test)]
mod tests {
    use crate::core::{Ledger, Transaction, User};

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

        ledger.add_transfer(&bilbo, &frodo, 32.0).unwrap();

        assert_eq!(*ledger.balances.get(&bilbo).unwrap(), 32.0);
        assert_eq!(*ledger.balances.get(&bilbo).unwrap(), 32.0);
        assert_eq!(*ledger.balances.get(&User::new("Gimli")).unwrap(), 0.0);
    }

    #[test]
    fn unrecognised_user() {
        let mut ledger = Ledger::new(vec!["Bilbo", "Frodo", "Legolas", "Gimli"]);

        let bilbo = User::new("Bilbo");
        let merry = User::new("Merry");

        let res = ledger.add_transfer(&bilbo, &merry, 32.0);

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
            (bilbo.clone(), 60.0)
        ], vec![
            (frodo.clone(), Benefit::Even),
            (legolas.clone(), Benefit::Even),
            (bilbo.clone(), Benefit::Even)
        ]).unwrap();

        /* here:
            Bilbo +60 -20 = +40
            Legolas         -20
            Frodo           -20
         */

        ledger.add_expense(vec![
            (frodo.clone(), 30.0)
        ], vec![
            (frodo.clone(), Benefit::Even),
            (legolas.clone(), Benefit::Sum(6.0)),
            (gimli.clone(), Benefit::Even)
        ]).unwrap();

        assert_eq!(ledger.transactions.len(), 2);
        assert_eq!(*ledger.balances.get(&bilbo).unwrap(), 40.0);
        assert_eq!(*ledger.balances.get(&frodo).unwrap(), -2.0);
        assert_eq!(*ledger.balances.get(&legolas).unwrap(), -26.0);
        assert_eq!(*ledger.balances.get(&gimli).unwrap(), -12.0);
    }
}