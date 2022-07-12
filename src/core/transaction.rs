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

pub type AmountPerUser = Vec<(User, Amount)>;
pub type BenefitPerUser = Vec<(User, Benefit)>;

pub struct Transaction {
    contributions: AmountPerUser,
    benefits: BenefitPerUser
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

#[derive(Debug)]
pub enum TransactionError {
    InsufficientBenefits{specified: Amount, spent: Amount},
    ExcessBenefits{specified: Amount, spent: Amount},
    UnrecognisedUser(User)
}

pub type TransactionResult<T> = Result<T, TransactionError>;

impl Transaction {
    pub fn new(contributions: AmountPerUser, benefits: BenefitPerUser) -> Transaction {
        Transaction { contributions, benefits }
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
    use std::collections::{HashSet, HashMap};
    use crate::core::User;
    use crate::core::Transaction;
    use crate::core::transaction::{AmountPerUser, Benefit, BenefitPerUser};

    #[test]
    fn can_print() {
        let bilbo = User::new("Bilbo");
        let legolas = User::new("Legolas");
        let gimli = User::new("Gimli");

        let contrib: AmountPerUser = vec![(bilbo, 32.0)];

        let benefit: BenefitPerUser = vec![
            (legolas, Benefit::Even),
            (gimli, Benefit::Sum(10.0))
        ];

        let transaction = Transaction::new(contrib, benefit);
        let repr = transaction.to_string();

        assert_eq!(repr, "Contributions: Bilbo: 32; Beneficiaries: Legolas: Even; Gimli: 10; ");
    }

    #[test]
    fn total_spent() {
        let bilbo = User::new("Bilbo");
        let frodo = User::new("Frodo");
        let legolas = User::new("Legolas");
        let gimli = User::new("Gimli");

        let contrib: AmountPerUser = vec![
            (bilbo, 32.0),
            (frodo, 12.0)
        ];

        let benefit: BenefitPerUser = vec![
            (legolas, Benefit::Even),
            (gimli, Benefit::Sum(10.0))
        ];

        let transaction = Transaction::new(contrib, benefit);
        assert_eq!(transaction.total_spending(), 44.0);
    }

    #[test]
    fn balance_distribution() {
        let bilbo = User::new("Bilbo");
        let frodo = User::new("Frodo");
        let legolas = User::new("Legolas");
        let gimli = User::new("Gimli");

        let contrib: AmountPerUser = vec![
            (bilbo.clone(), 32.0),
            (frodo.clone(), 12.0)
        ];

        let benefit: BenefitPerUser = vec![
            (legolas.clone(), Benefit::Even),
            (frodo.clone(), Benefit::Even),
            (gimli.clone(), Benefit::Sum(10.0))
        ];

        let transaction = Transaction::new(contrib, benefit);
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
