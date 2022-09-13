use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use colored::Colorize;

use crate::core::user::{UserName, Amount};

pub type UserAmountMap = HashMap<UserName, Amount>;

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
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

pub type AmountPerUser<T> = Vec<(T, Amount)>;
pub type BenefitPerUser<T> = Vec<(T, Benefit)>;

/// Trait turning a type with user borrows (e.g. `&'a User` or ids as &str)
/// into an equivalent type with owned users or ids (as String).
/// Maybe not the best solution, potentially shared ownership of users
/// will be required in the future.
trait ToOwnedUsers {
    type WithOwnedUsers;
    fn to_owned_users(&self) -> Self::WithOwnedUsers;
}

impl<T: Copy> ToOwnedUsers for Vec<(&str, T)> {
    type WithOwnedUsers = Vec<(UserName, T)>;

    fn to_owned_users(&self) -> Self::WithOwnedUsers {
        self.iter().map(|pair| (pair.0.to_owned(), pair.1)).collect()
    }
}


#[derive(Serialize, Deserialize)]
pub struct Transaction {
    contributions: AmountPerUser<UserName>,
    benefits: BenefitPerUser<UserName>,
    pub is_direct: bool,
    pub description: String
}

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: ", "From".bold())?;
        for (user, amount) in &self.contributions {
            write!(f, "{}: {}; ", user, amount)?;
        }

        write!(f, "{}: ", "To".bold())?;
        for (user, benefit) in &self.benefits {
            write!(f, "{}: {}; ", user, benefit)?;
        }
        return Ok(());
    }
}

#[derive(Debug)]
pub enum TransactionError {
    InsufficientBenefits{specified: Amount, spent: Amount},
    ExcessBenefits{specified: Amount, spent: Amount},
    UnrecognisedUser(UserName)
}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg: String = match self {
            TransactionError::InsufficientBenefits { specified, spent } => {
                format!("{} spent, but {} used", spent, specified)
            },
            TransactionError::ExcessBenefits { specified, spent } => {
                format!("{} spent, but {} used", spent, specified)
            },
            TransactionError::UnrecognisedUser(username) => {
                format!("No such user: {}", username)
            }
        };
        write!(f, "Transaction error: {}", msg)
    }
}

pub type TransactionResult<T> = Result<T, TransactionError>;

impl Transaction {
    pub fn new(contributions: AmountPerUser<&str>, benefits: BenefitPerUser<&str>, description: &str, direct: bool) -> Transaction {
        Transaction {
            contributions: contributions.to_owned_users(),
            benefits: benefits.to_owned_users(),
            is_direct: direct,
            description: description.to_owned() }
    }

    pub fn total_spending(&self) -> Amount {
        return self.contributions.iter()
            .map(|contrib| contrib.1).sum();
    }

    fn specified_benefits(&self) -> Amount {
        return self.benefits.iter()
            .map(|user_benefit| match user_benefit.1 {
                Benefit::Sum(val) => val,
                _ => 0.0
            }).sum();
    }

    fn num_even_benefits(&self) -> usize {
        return self.benefits.iter()
            .fold(0,|count, user_benefit| match user_benefit.1 {
                Benefit::Even => count + 1,
                _ => count
            });
    }

    pub fn balance_updates(&self) -> TransactionResult<UserAmountMap> {
        let mut balance_delta: UserAmountMap = HashMap::new();

        let spending = self.total_spending();
        let specified_benefits = self.specified_benefits();
        if specified_benefits > spending {
            return Err(TransactionError::ExcessBenefits{specified: specified_benefits, spent: spending})
        }

        let num_evens = self.num_even_benefits();
        let total_amount_evens = spending - specified_benefits;
        if total_amount_evens > 0.0 && num_evens == 0 {
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


#[cfg(test)]
mod tests {
    use crate::{Transaction, transaction::Benefit};
    use colored;
    use rstest::{fixture, rstest};

    #[rstest]
    fn can_print() {
        colored::control::set_override(false);

        let contrib = vec![("Bilbo", 32.0)];

        let benefit = vec![
            ("Legolas", Benefit::Even),
            ("Gimli", Benefit::Sum(10.0))
        ];

        let transaction = Transaction::new(contrib, benefit, "", false);
        let repr = transaction.to_string();

        assert_eq!(repr, "From: Bilbo: 32; To: Legolas: Even; Gimli: 10; ");
    }

    #[fixture]
    fn transaction() -> Transaction {
        let contrib = vec![
            ("Bilbo", 32.0),
            ("Frodo", 12.0)
        ];

        let benefit = vec![
            ("Legolas", Benefit::Even),
            ("Frodo", Benefit::Even),
            ("Gimli", Benefit::Sum(10.0))
        ];

        return Transaction::new(contrib, benefit, "", false);
    }

    #[rstest]
    fn total_spent(transaction: Transaction) {
        assert_eq!(transaction.total_spending(), 44.0);
    }

    #[rstest]
    fn balance_distribution(transaction: Transaction) {
        let balance_delta = transaction.balance_updates().unwrap();

        assert_eq!(balance_delta.keys().len(), 4);

        assert_eq!(transaction.num_even_benefits(), 2);
        assert_eq!(transaction.total_spending(), 44.0);
        assert_eq!(transaction.specified_benefits(), 10.0);

        assert_eq!(*balance_delta.get("Bilbo").unwrap(), 32.0);
        assert_eq!(*balance_delta.get("Legolas").unwrap(), -17.0);
        assert_eq!(*balance_delta.get("Frodo").unwrap(), -5.0);
        assert_eq!(*balance_delta.get("Gimli").unwrap(), -10.0);
    }
}
