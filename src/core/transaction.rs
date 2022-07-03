use std::collections::{HashSet, HashMap};

use crate::core::user::User;

enum AmountPaid {
    Sum(f32),
    Even
}

pub struct Transaction<'a> {
    participations: HashMap<&'a User, AmountPaid>
}

pub struct Ledger<'a> {
    users: HashSet<User>,
    transactions: Vec<Transaction<'a>>
}

impl Ledger {
    pub fn add_transaction() {}

}
