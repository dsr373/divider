use std::collections::HashMap;

use crate::core::user::User;

pub type Amount = f32;

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

pub type AmountPerUser = HashMap<User, Amount>;
pub type BenefitsMap = HashMap<User, Benefit>;

pub struct Transaction {
    contributions: AmountPerUser,
    benefits: BenefitsMap
}

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Contributions: ")?;
        for (user, amount) in &self.contributions {
            write!(f, "{}: {}; ", user, amount)?;
        }

        write!(f, "Beneficiaries: ")?;
        for (user, benefit) in &self.benefits {
            write!(f, "{}: {}; ", user, benefit)?;
        }
        return Ok(());
    }
}


pub enum TransactionError {
    InsufficientBenefits{specified: Amount, spent: Amount},
    ExcessBenefits{specified: Amount, spent: Amount},
    UnrecognisedUser(User)
}

pub type TransactionResult<T> = Result<T, TransactionError>;

impl Transaction {
    pub fn new(contributions: AmountPerUser, benefits: BenefitsMap) -> Transaction {
        Transaction { contributions, benefits }
    }

    pub fn total_spending(&self) -> Amount {
        return self.contributions.values().sum();
    }

    pub fn specified_benefits(&self) -> Amount {
        return self.benefits.values()
            .map(|benefit| match *benefit {
                Benefit::Sum(val) => val,
                _ => 0.0
            }).sum();
    }

    pub fn num_even_benefits(&self) -> usize {
        return self.benefits.values()
            .map(|benefit| match benefit {
                Benefit::Even => true,
                _ => false
            }).count();
    }

    pub fn balance_updates(&self) -> TransactionResult<AmountPerUser> {
        let mut balance_delta: AmountPerUser = HashMap::new();

        let spending = self.total_spending();
        let specified_benefits = self.specified_benefits();
        if specified_benefits > spending {
            return Err(TransactionError::ExcessBenefits{specified: specified_benefits, spent: spending})
        }

        let num_evens = self.num_even_benefits();
        let total_amount_evens = spending - specified_benefits;
        if total_amount_evens > 0f32 && num_evens == 0 {
            return Err(TransactionError::InsufficientBenefits{specified: specified_benefits, spent: spending})
        }

        let benefit_per_even = total_amount_evens / (num_evens as f32);

        for (user, contrib) in &self.contributions {
            balance_delta.insert(user.clone(), *contrib);
        }
        for (user, benefit) in &self.benefits {
            let final_benefit = match *benefit {
                Benefit::Sum(val) => val,
                Benefit::Even => benefit_per_even
            };
            *balance_delta.entry(user.clone()).or_insert(0f32) -= final_benefit;
        }

        return Ok(balance_delta);
    }
}
