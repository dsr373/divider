use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use colored::Colorize;
use chrono::{DateTime, offset::Local, Utc};

use crate::core::user::{UserName, Amount};
use crate::core::error::TransactionError;

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
    pub id: usize,
    #[serde(with = "datetime_serialization")]
    pub datetime: DateTime<Utc>,
    contributions: AmountPerUser<UserName>,
    benefits: BenefitPerUser<UserName>,
    pub is_direct: bool,
    pub description: String
}

mod datetime_serialization {
    use serde::{de, Serializer, Deserializer, Deserialize};
    use chrono::{DateTime, Utc};

    pub fn serialize<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let s = dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        s.parse::<DateTime<Utc>>()
            .map_err(de::Error::custom)
    }
}

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04x}\t", self.id)?;

        let dt_string = format!("{}", self.datetime.with_timezone(&Local).format("%F %R %:z"));
        write!(f, "{} ", dt_string.dimmed())?;

        write!(f, "{}: ", "From".bold())?;
        for (user, amount) in &self.contributions {
            write!(f, "{}: {}; ", user, amount)?;
        }

        write!(f, "{}: ", "To".bold())?;
        for (user, benefit) in &self.benefits {
            write!(f, "{}: {}; ", user, benefit)?;
        }

        write!(f, "{}: {}", "Description".bold(), &self.description)?;
        return Ok(());
    }
}

pub type TransactionResult<T> = Result<T, TransactionError>;

impl Transaction {
    pub fn new(contributions: AmountPerUser<&str>, benefits: BenefitPerUser<&str>,
        description: &str, direct: bool, opt_id: Option<usize>, opt_time: Option<DateTime<Utc>>) -> Transaction
    {
        let datetime = match opt_time {
            None => Utc::now(),
            Some(time) => time
        };
        let id = match opt_id {
            None => 0,
            Some(number) => number
        };
        Transaction {
            id,
            datetime,
            contributions: contributions.to_owned_users(),
            benefits: benefits.to_owned_users(),
            is_direct: direct,
            description: description.to_string() }
    }

    pub fn total_spending(&self) -> Amount {
        return self.contributions.iter()
            .map(|contrib| contrib.1).sum();
    }

    pub fn reverse(&self) -> TransactionResult<Transaction> {
        let benefit_per_even = self.benefits_per_even()?;

        let contributions = self.benefits.iter().map(|(user, benefit)| {
            match benefit {
                Benefit::Sum(number) => (user.clone(), *number),
                Benefit::Even => (user.clone(), benefit_per_even)
            }
        }).collect();

        let benefits = self.contributions.iter()
            .map(|(user, contrib)| (user.clone(), Benefit::Sum(*contrib))).collect();

        return Ok(Transaction {
            id: 0,
            datetime: Utc::now(),
            contributions,
            benefits,
            is_direct: false,
            description: format!("Undo {:04x}", self.id) });
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

    fn benefits_per_even(&self) -> TransactionResult<Amount> {
        let spending = self.total_spending();
        let specified_benefits = self.specified_benefits();
        if specified_benefits > spending {
            return Err(TransactionError::ExcessBenefits{specified: specified_benefits, spent: spending})
        }

        let num_evens = self.num_even_benefits();
        let total_amount_evens = spending - specified_benefits;
        if total_amount_evens > 0.0 && num_evens == 0 {
            return Err(TransactionError::InsufficientBenefits{specified: specified_benefits, spent: spending})
        } else if total_amount_evens == 0.0 && num_evens == 0 {
            return Ok(0.0);
        }

        return Ok(total_amount_evens / (num_evens as f32));
    }

    pub fn balance_updates(&self) -> TransactionResult<UserAmountMap> {
        let mut balance_delta: UserAmountMap = HashMap::new();

        let benefit_per_even = self.benefits_per_even()?;

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
    use crate::{Transaction, transaction::Benefit, core::TransactionError};
    use chrono::{TimeZone, Local, Utc};
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

        let time = Local.ymd(2022, 5, 1).and_hms(12, 0, 0);

        let transaction = Transaction::new(contrib, benefit,
            "Dinner", false, Some(214), Some(time.with_timezone(&Utc)));

        let repr = transaction.to_string();

        assert_eq!(repr, "00d6\t2022-05-01 12:00 +01:00 From: Bilbo: 32; To: Legolas: Even; Gimli: 10; Description: Dinner");
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

        let time = Utc.ymd(2022, 5, 1).and_hms(12, 0, 0);

        return Transaction::new(contrib, benefit, "", false,
            None, Some(time));
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

    #[rstest]
    fn reverse_transaction(transaction: Transaction) {
        let reversed = transaction.reverse().unwrap();

        assert_eq!(reversed.specified_benefits(), 44.0);

        let reversed_delta = reversed.balance_updates().unwrap();
        let original_delta = transaction.balance_updates().unwrap();
        assert_eq!(reversed_delta.len(), original_delta.len());

        for (user, delta) in &reversed_delta {
            assert!(original_delta.contains_key(user));
            assert_eq!(original_delta.get_key_value(user).unwrap(), (user, &-delta));
        }
    }

    #[rstest]
    fn insufficient_benefits() {
        let contrib = vec![
            ("Bilbo", 32.0)
        ];

        let benefit = vec![
            ("Gimli", Benefit::Sum(10.0)),
            ("Frodo", Benefit::Sum(12.0))
        ];

        let result = Transaction::new(contrib, benefit, "", false, None, None).balance_updates();

        match result {
            Err(TransactionError::InsufficientBenefits { specified, spent }) if specified == 22.0 && spent == 32.0 => {},
            _ => panic!("Result does not match InsufficientBenefits: {:?}", &result)
        }
    }

    #[rstest]
    fn excess_benefits() {
        let contrib = vec![
            ("Bilbo", 32.0)
        ];

        let benefit = vec![
            ("Gimli", Benefit::Sum(40.0)),
            ("Frodo", Benefit::Even),
            ("Legolas", Benefit::Even)
        ];

        let result = Transaction::new(contrib, benefit, "", false, None, None).balance_updates();

        match result {
            Err(TransactionError::ExcessBenefits { specified, spent }) if specified == 40.0 && spent == 32.0 => {},
            _ => panic!("Result does not match ExcessBenefits: {:?}", &result)
        }
    }

    #[rstest]
    fn no_evens() {
        let contrib = vec![
            ("Bilbo", 32.0)
        ];

        let benefit = vec![
            ("Gimli", Benefit::Sum(22.0)),
            ("Frodo", Benefit::Sum(10.0))
        ];

        let balance_delta = Transaction::new(contrib, benefit, "", false, None, None).balance_updates().unwrap();

        assert_eq!(*balance_delta.get("Bilbo").unwrap(), 32.0);
        assert_eq!(*balance_delta.get("Gimli").unwrap(), -22.0);
        assert_eq!(*balance_delta.get("Frodo").unwrap(), -10.0);
        assert!(!balance_delta.contains_key("Legolas"));
    }

    // TODO: test reverse
}
