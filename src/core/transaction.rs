use std::collections::{HashSet, HashMap};

use crate::core::user::User;

type Amount = f32;

#[derive(Clone, Copy, PartialEq)]
pub enum Benefit {
    Sum(Amount),
    Even
}

impl std::fmt::Display for Benefit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let disp = match self {
            Self::Sum(amt) => amt.to_string(),
            Self::Even => "Even".to_string()
        };
        write!(f, "{}", disp)
    }
}

type AmountPerUser<'a> = HashMap<&'a User, Amount>;
type BenefitsMap<'a> = HashMap<&'a User, Benefit>;

pub struct Transaction<'a> {
    contributions: AmountPerUser<'a>,
    benefits: BenefitsMap<'a>
}

impl<'a> std::fmt::Display for Transaction<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Contributions: ")?;
        for (user, amount) in &self.contributions {
            write!(f, "{}: {}; ", user, amount)?;
        }

        write!(f, "Beneficiaries: ");
        for (user, benefit) in &self.benefits {
            write!(f, "{}: {}; ", user, benefit)?;
        }
        return Ok(());
    }
}


enum TransactionError {
    InsufficientBenefits{specified: Amount, spent: Amount},
    ExcessBenefits{specified: Amount, spent: Amount},
    UnrecognisedUser(String)
}

type TransactionResult = Result<(), TransactionError>;

pub struct Ledger<'a> {
    balances: HashMap<User, Amount>,
    transactions: Vec<Transaction<'a>>,
    total_spend: Amount
}

impl<'a> Ledger<'a> {
    pub fn new(user_names: Vec<String>) -> Ledger<'a> {
        let mut balances = HashMap::new();

        for name in user_names {
            balances.insert(User::new(name), 0f32 as Amount);
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

    fn update_balances(&mut self, changes: AmountPerUser) -> TransactionResult {
        for (user, delta) in &changes {
            match self.balances.get_mut(*user) {
                Some(val) => *val += delta,
                None => return Err(TransactionError::UnrecognisedUser(user.name.clone()))
            }
        }
        return Ok(());
    }

    // TODO: separate into smaller functions
    fn apply_transaction(&mut self, transaction: &'a Transaction) -> TransactionResult {
        let mut balance_delta: AmountPerUser = self.balances.keys()
            .map(|user| (user, 0.0)).collect();

        // increase balance to contributors
        for (user, contrib) in &transaction.contributions {
            match balance_delta.get_mut(user) {
                Some(val) => *val += contrib,
                None => return Err(TransactionError::UnrecognisedUser(user.name.clone()))
            }
        }

        // calculate total spending
        let spending: Amount = transaction.contributions.values().sum();
        self.total_spend += spending;

        // calculate which beneficiary amounts are already specified
        let specified_benefits: Amount = transaction.benefits.values()
            .map(|benefit| match *benefit {
                Benefit::Sum(val) => val,
                _ => 0.0
            }).sum();

        if specified_benefits > spending {
            return Err(TransactionError::ExcessBenefits{specified: specified_benefits, spent: spending})
        }

        // count the number of "even" benefits
        let num_evens = transaction.benefits.values()
            .map(|benefit| match benefit {
                Benefit::Even => true,
                _ => false
            }).count();

        let even_benefit_amt = spending - specified_benefits;
        if even_benefit_amt > 0f32 && num_evens == 0 {
            return Err(TransactionError::InsufficientBenefits{specified: specified_benefits, spent: spending})
        }

        let benefit_per_even = spending / (num_evens as f32);

        // update beneficiaries' balances
        for (user, benefit) in &transaction.benefits {
            let amount_benefit: Amount = match *benefit {
                Benefit::Sum(val) => val,
                Benefit::Even => benefit_per_even
            };

            match balance_delta.get_mut(user) {
                Some(val) => *val -= amount_benefit,
                None => return Err(TransactionError::UnrecognisedUser(user.name.clone()))
            }
        }

        return self.update_balances(balance_delta);
    }

    // TODO: add transactions
    pub fn add_transaction(&mut self, contributions: AmountPerUser<'a>, benefits: BenefitsMap<'a>) -> TransactionResult {
        let transaction = Transaction{contributions, benefits};
        self.apply_transaction(&transaction)?;
        self.transactions.push(transaction);
        Ok(())
    }
}
