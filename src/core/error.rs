use std::error;

use crate::core::{Amount, UserName};

#[derive(Debug)]
pub enum TransactionError {
    /// Occurs when a transaction specifies all user benefits,
    /// but the sum of the benefits is smaller than that of the contributions.
    InsufficientBenefits {
        specified: Amount,
        spent: Amount
    },
    /// Occurs when a transaction's specified benefits
    /// exceed the sum of all the contributions
    ExcessBenefits {
        specified: Amount,
        spent: Amount
    },
    /// Occurs when attempting to register a transaction
    /// involving a user not registered on a ledger.
    UnknownUser(UserName),
    /// Occurs when attempting to reference a transaction
    /// by an id which does not exist on the ledger
    UnknownTransactionId(usize)
}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionError::InsufficientBenefits { specified, spent } => {
                write!(f, "too few benefits specified: {} out of {} spent", specified, spent)
            },
            TransactionError::ExcessBenefits { specified, spent } => {
                write!(f, "too many benefits specified: {} out of {} spent", specified, spent)
            },
            TransactionError::UnknownUser(username) => {
                write!(f, "no such user: {}", username)
            },
            TransactionError::UnknownTransactionId(id) => {
                write!(f, "no such transaction id: {}", id)
            }
        }
    }
}

impl error::Error for TransactionError {}
