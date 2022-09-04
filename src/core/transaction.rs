use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use colored::Colorize;

use crate::core::user::User;

pub type Amount = f32;

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

pub type AmountPerUser = Vec<(User, Amount)>;
pub type AmountPerUserRef<'a> = Vec<(&'a User, Amount)>;
pub type BenefitPerUser = Vec<(User, Benefit)>;
pub type BenefitPerUserRef<'a> = Vec<(&'a User, Benefit)>;

/// Trait turning a type with user borrows (e.g. `&'a User`)
/// into an equivalent type with owned users. It's helpful
/// to avoid complex reference structures. Maybe not the
/// best solution, potentially shared ownership of users will
/// be required in the future.
trait ToOwnedUsers {
    type WithOwnedUsers;
    fn to_owned_users(&self) -> Self::WithOwnedUsers;
}

impl<'a, T: Copy> ToOwnedUsers for Vec<(&'a User, T)> {
    type WithOwnedUsers = Vec<(User, T)>;
    fn to_owned_users(&self) -> Self::WithOwnedUsers {
        self.iter().map(|pair| (pair.0.to_owned(), pair.1)).collect()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Transaction {
    contributions: AmountPerUser,
    benefits: BenefitPerUser,
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
    UnrecognisedUser(User)
}

pub type TransactionResult<T> = Result<T, TransactionError>;

impl Transaction {
    pub fn new(contributions: AmountPerUserRef, benefits: BenefitPerUserRef, description: &str) -> Transaction {
        Transaction {
            contributions: contributions.to_owned_users(),
            benefits: benefits.to_owned_users(),
            is_direct: false,
            description: description.to_owned() }
    }

    pub fn new_direct(contributions: AmountPerUserRef, benefits: BenefitPerUserRef, description: &str) -> Transaction {
        Transaction {
            contributions: contributions.to_owned_users(),
            benefits: benefits.to_owned_users(),
            is_direct: true,
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

    pub fn balance_updates(&self) -> TransactionResult<HashMap<User, Amount>> {
        let mut balance_delta: HashMap<User, Amount> = HashMap::new();

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


#[cfg(test)]
mod tests {
    use crate::{User, Transaction, transaction::Benefit};
    use colored;
    use rstest::{fixture, rstest};

    #[fixture]
    fn users() -> (User, User, User, User) {
        let bilbo = User::new("Bilbo");
        let frodo = User::new("Frodo");
        let legolas = User::new("Legolas");
        let gimli = User::new("Gimli");
        return (bilbo, frodo, legolas, gimli);
    }

    #[rstest]
    fn can_print(users: (User, User, User, User)) {
        colored::control::set_override(false);
        let (bilbo, _, legolas, gimli) = users;

        let contrib = vec![(&bilbo, 32.0)];

        let benefit = vec![
            (&legolas, Benefit::Even),
            (&gimli, Benefit::Sum(10.0))
        ];

        let transaction = Transaction::new(contrib, benefit, "");
        let repr = transaction.to_string();

        assert_eq!(repr, "From: Bilbo: 32; To: Legolas: Even; Gimli: 10; ");
    }

    #[fixture]
    fn transaction(users: (User, User, User, User)) -> Transaction {
        let (bilbo, frodo, legolas, gimli) = users;
        let contrib = vec![
            (&bilbo, 32.0),
            (&frodo, 12.0)
        ];

        let benefit = vec![
            (&legolas, Benefit::Even),
            (&frodo, Benefit::Even),
            (&gimli, Benefit::Sum(10.0))
        ];

        return Transaction::new(contrib, benefit, "");
    }

    #[rstest]
    fn total_spent(transaction: Transaction) {
        assert_eq!(transaction.total_spending(), 44.0);
    }

    #[rstest]
    fn balance_distribution(users: (User, User, User, User), transaction: Transaction) {
        let (bilbo, frodo, legolas, gimli) = users;
        let balance_delta = transaction.balance_updates().unwrap();

        assert_eq!(balance_delta.keys().len(), 4);

        assert_eq!(transaction.num_even_benefits(), 2);
        assert_eq!(transaction.total_spending(), 44.0);
        assert_eq!(transaction.specified_benefits(), 10.0);

        assert_eq!(*balance_delta.get(&bilbo).unwrap(), 32.0);
        assert_eq!(*balance_delta.get(&legolas).unwrap(), -17.0);
        assert_eq!(*balance_delta.get(&frodo).unwrap(), -5.0);
        assert_eq!(*balance_delta.get(&gimli).unwrap(), -10.0);
    }
}
