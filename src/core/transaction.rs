use std::collections::{HashSet, HashMap};

use crate::core::user::User;

enum Benefit {
    Amount(f32),
    Even
}

pub struct Transaction<'a> {
    contributions: HashMap<&'a User, f32>,
    benefits: HashMap<&'a User, Benefit>
}

pub struct Ledger<'a> {
    balances: HashMap<User, f32>,
    transactions: Vec<Transaction<'a>>,
}

impl<'a> Ledger<'a> {
    pub fn new(user_names: Vec<String>) -> Ledger<'a> {
        let mut balances = HashMap::new();

        for name in user_names {
            balances.insert(User::new(name), 0f32);
        }

        return Ledger { balances, transactions: Vec::new() };
    }

    // TODO: make a getter for the users and balances

    pub fn add_transaction(&mut self, )

}
