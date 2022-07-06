use std::collections::{HashSet, HashMap};

use crate::core::user::User;

type Amount = f32;

pub enum Benefit {
    Sum(Amount),
    Even
}

type ContributionsMap<'a> = HashMap<&'a User, Amount>;
type BenefitsMap<'a> = HashMap<&'a User, Benefit>;

pub struct Transaction<'a> {
    contributions: ContributionsMap<'a>,
    benefits: BenefitsMap<'a>
}

pub struct Ledger<'a> {
    balances: HashMap<User, Amount>,
    transactions: Vec<Transaction<'a>>,
}

impl<'a> Ledger<'a> {
    pub fn new(user_names: Vec<String>) -> Ledger<'a> {
        let mut balances = HashMap::new();

        for name in user_names {
            balances.insert(User::new(name), 0f32 as Amount);
        }

        return Ledger { balances, transactions: Vec::new() };
    }

    pub fn get_users(&self) -> HashSet<&User> {
        let mut users = HashSet::new();
        for (user, _balance) in &self.balances {
            users.insert(user);
        }
        return users;
    }

    pub fn get_user_by_name(&self, name: String) -> Option<&User> {
        for (user, _balance) in &self.balances {
            if user.name == name {
                return Some(user);
            }
        }
        return None;
    }

    // TODO: add transactions
    pub fn add_transaction(&mut self, contributions: ContributionsMap, benefits: BenefitsMap) -> Result<(), String> {
        Ok(())
    }
}
