use std::collections::{HashMap, HashSet};

use crate::core::user::User;
use crate::core::transaction::{
    Transaction, TransactionResult, TransactionError,
    AmountPerUserRef, BenefitPerUserRef,
    Benefit, Amount};

type UserAmountMap = HashMap<User, Amount>;

pub struct Ledger {
    balances: UserAmountMap,
    transactions: Vec<Transaction>,
    total_spend: Amount
}

impl Ledger {
    const CONSISTENCY_CHECK_INTERVAL: usize = 100;

    pub fn new(user_names: Vec<&str>) -> Ledger {
        let mut balances = HashMap::new();

        for name in user_names {
            balances.insert(User::new(&name), 0f32 as Amount);
        }

        return Ledger { balances, transactions: Vec::new(), total_spend: 0f32 as Amount };
    }

    pub fn get_users(&self) -> HashSet<&User> {
        return self.balances.keys()
            .map(|user| user).collect();
    }

    pub fn get_user_by_name(&self, name: &str) -> Option<&User> {
        self.balances.keys()
            .find_map(|user| if user.name == name { Some(user) } else {None})
    }

    pub fn add_expense(&mut self, contributions: AmountPerUserRef, benefits: BenefitPerUserRef, description: &str) -> TransactionResult<()> {
        let transaction = Transaction::new(contributions, benefits, description);
        self.add_transaction(transaction)
    }

    pub fn add_transfer(&mut self, from: &User, to: &User, amount: Amount, description: &str) -> TransactionResult<()> {
        let transaction = Transaction::new_direct(
            vec![(from, amount)],
            vec![(to, Benefit::Sum(amount))],
            description
        );
        self.add_transaction(transaction)
    }

    fn add_transaction(&mut self, transaction: Transaction) -> TransactionResult<()> {
        Ledger::apply_transaction(&mut self.total_spend, &mut self.balances, &transaction)?;
        self.transactions.push(transaction);

        if self.needs_consistency_check() {
            self.consistency_check()?;
        }
        return Ok(());
    }

    fn update_balances(balances: &mut UserAmountMap, changes: UserAmountMap) -> TransactionResult<()> {
        for (user, delta) in &changes {
            match balances.get_mut(user) {
                Some(val) => *val += delta,
                None => return Err(TransactionError::UnrecognisedUser(user.clone()))
            }
        }
        return Ok(());
    }

    // TODO: separate into smaller functions
    fn apply_transaction(total_spend: &mut Amount, balances: &mut UserAmountMap, transaction: &Transaction) -> TransactionResult<()> {
        if transaction.is_direct {
            *total_spend += transaction.total_spending();
        }
        let balance_updates = transaction.balance_updates()?;
        return Ledger::update_balances(balances, balance_updates);
    }

    fn consistency_check(&mut self) -> TransactionResult<()> {
        let mut new_balances: UserAmountMap =
            self.balances.keys().map(|user| (user.clone(), 0.0)).collect();
        let mut new_total: Amount = 0.0;

        for transaction in &self.transactions {
            Ledger::apply_transaction(&mut new_total, &mut new_balances, transaction)?;
        }

        self.total_spend = new_total;
        self.balances = new_balances;
        return Ok(());
    }

    fn needs_consistency_check(&self) -> bool {
        return self.transactions.len() % Self::CONSISTENCY_CHECK_INTERVAL == 0;
    }
}


#[cfg(test)]
mod tests {
    use crate::core::{Ledger, User};
    use crate::core::transaction::{Benefit, TransactionError, AmountPerUserRef, BenefitPerUserRef, TransactionResult};

    use rstest::{fixture, rstest};

    type User4 = (User, User, User, User);

    #[fixture]
    fn ledger() -> Ledger {
        return Ledger::new(vec!["Bilbo", "Frodo", "Legolas", "Gimli"]);
    }

    #[fixture]
    fn users(ledger: Ledger) -> User4 {
        let bilbo = ledger.get_user_by_name("Bilbo").unwrap().to_owned();
        let frodo = ledger.get_user_by_name("Frodo").unwrap().to_owned();
        let legolas = ledger.get_user_by_name("Legolas").unwrap().to_owned();
        let gimli = ledger.get_user_by_name("Gimli").unwrap().to_owned();

        return (bilbo, frodo, legolas, gimli);
    }

    #[rstest]
    fn create_and_get_users(ledger: Ledger, users: User4) {
        let (bilbo, frodo, legolas, gimli) = users;

        let users_set = ledger.get_users();

        assert_eq!(users_set.len(), 4);
        assert!(users_set.contains(&bilbo));
        assert!(users_set.contains(&frodo));
        assert!(users_set.contains(&legolas));
        assert!(users_set.contains(&gimli));
        assert!(!users_set.contains(&User::new("Merry")));
    }

    #[rstest]
    fn create_and_find_user(ledger: Ledger) {
        let frodo = ledger.get_user_by_name("Frodo").unwrap();
        assert_eq!(frodo, &User::new("Frodo"));
    }

    #[rstest]
    fn simple_transfer(mut ledger: Ledger, users: User4) {
        let (bilbo, frodo, _, gimli) = users;

        ledger.add_transfer(&bilbo, &frodo, 32.0, "").unwrap();

        assert_eq!(*ledger.balances.get(&bilbo).unwrap(), 32.0);
        assert_eq!(*ledger.balances.get(&frodo).unwrap(), -32.0);
        assert_eq!(*ledger.balances.get(&gimli).unwrap(), 0.0);
    }

    #[rstest]
    fn unrecognised_user(mut ledger: Ledger, users: User4) {
        let bilbo = users.0;
        let merry = User::new("Merry");

        let res = ledger.add_transfer(&bilbo, &merry, 32.0, "");

        assert!(res.is_err());
        assert!(matches!(res, Err(TransactionError::UnrecognisedUser(..))));
    }

    fn add_transaction_bilbo(ledger: &mut Ledger, users: &User4) -> TransactionResult<()> {
        let (bilbo, frodo, legolas, _) = users;
        let contributions = vec![(bilbo, 60.0)];
        let benefits = vec![
            (frodo, Benefit::Even),
            (legolas, Benefit::Even),
            (bilbo, Benefit::Even)
        ];
        ledger.add_expense(contributions, benefits, "")
    }

    fn add_transaction_frodo(ledger: &mut Ledger, users: &User4) -> TransactionResult<()> {
        let (_, frodo, legolas, gimli) = users;
        let contributions = vec![(frodo, 30.0)];
        let benefits = vec![
            (frodo, Benefit::Even),
            (legolas, Benefit::Sum(6.0)),
            (gimli, Benefit::Even)
        ];
        ledger.add_expense(contributions, benefits, "")
    }

    #[rstest]
    fn complex_expense(mut ledger: Ledger, users: User4) {
        let (bilbo, frodo, legolas, gimli) = &users;

        add_transaction_bilbo(&mut ledger, &users).unwrap();
        add_transaction_frodo(&mut ledger, &users).unwrap();

        assert_eq!(ledger.transactions.len(), 2);
        assert_eq!(*ledger.balances.get(bilbo).unwrap(), 40.0);
        assert_eq!(*ledger.balances.get(frodo).unwrap(), -2.0);
        assert_eq!(*ledger.balances.get(legolas).unwrap(), -26.0);
        assert_eq!(*ledger.balances.get(gimli).unwrap(), -12.0);
    }

    #[rstest]
    fn consistency_check(mut ledger: Ledger, users: User4) {
        const interval: usize = Ledger::CONSISTENCY_CHECK_INTERVAL;
        let (bilbo, frodo, legolas, gimli) = &users;

        let repeated_transactions = (interval - 1)/2;

        for _ in 0..repeated_transactions {
            add_transaction_bilbo(&mut ledger, &users).unwrap();
            add_transaction_frodo(&mut ledger, &users).unwrap();
        }

        // before reapplying all
        assert_eq!(*ledger.balances.get(bilbo).unwrap(), (repeated_transactions as f32) * 40.0);
        assert_eq!(*ledger.balances.get(frodo).unwrap(), (repeated_transactions as f32) * -2.0);
        assert_eq!(*ledger.balances.get(legolas).unwrap(), (repeated_transactions as f32) * -26.0);
        assert_eq!(*ledger.balances.get(gimli).unwrap(), (repeated_transactions as f32) * -12.0);

        // mess with one of the values
        *ledger.balances.get_mut(&bilbo).unwrap() += 100.0;

        // one of these should do the consistency check
        add_transaction_bilbo(&mut ledger, &users).unwrap();
        add_transaction_frodo(&mut ledger, &users).unwrap();

        // after reapplying all
        assert_eq!(*ledger.balances.get(bilbo).unwrap(), ((repeated_transactions + 1) as f32) * 40.0);
        assert_eq!(*ledger.balances.get(frodo).unwrap(), ((repeated_transactions + 1) as f32) * -2.0);
        assert_eq!(*ledger.balances.get(legolas).unwrap(), ((repeated_transactions + 1) as f32) * -26.0);
        assert_eq!(*ledger.balances.get(gimli).unwrap(), ((repeated_transactions + 1) as f32) * -12.0);
    }
}