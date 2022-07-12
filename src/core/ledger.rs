use std::collections::{HashMap, HashSet};

use crate::core::user::User;
use crate::core::transaction::{Transaction, Amount, TransactionResult, TransactionError, AmountPerUser, BenefitPerUser};

pub struct Ledger {
    balances: HashMap<User, Amount>,
    transactions: Vec<Transaction>,
    total_spend: Amount
}

impl Ledger {
    pub fn new(user_names: Vec<String>) -> Ledger {
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

    pub fn get_user_by_name(&self, name: String) -> Option<&User> {
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
        self.total_spend += transaction.total_spending();
        let balance_updates = transaction.balance_updates()?;
        return self.update_balances(balance_updates);
    }

    // TODO: add transactions
    pub fn add_transaction(&mut self, contributions: AmountPerUser, benefits: BenefitPerUser) -> TransactionResult<()> {
        let transaction = Transaction::new(contributions, benefits);
        self.apply_transaction(&transaction)?;
        self.transactions.push(transaction);
        Ok(())
    }
}
