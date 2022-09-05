use divider::{User, Ledger,
    backend::{LedgerStore, JsonStore},
    transaction::{Amount, BenefitPerUser, Benefit, AmountPerUser, ToBorrowedUsers, TransactionError}};

use std::path::PathBuf;
use colored::Colorize;
use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(version, about, propagate_version = true)]
struct Cli {
   /// Path to ledger file to operate on
   #[clap(value_parser)]
    path: PathBuf,

   /// Action to perform
   #[clap(subcommand)]
   action: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    /// Read and display balances
    Balances,
    /// List all transactions
    List,
    /// Add a new user
    AddUser(AddUser),
    /// Add a new direct transfer
    AddTransfer(AddTransfer),
    /// Add a new expense
    AddTransaction(AddTransaction)
}

fn print_balances(ledger: &Ledger) {
    for (user, balance) in ledger.get_balances() {
        let color = if balance < 0.0 {
            colored::ColoredString::bright_red
        } else if balance > 0.0 {
            colored::ColoredString::green
        } else {
            colored::ColoredString::normal
        };
        let fmt_balance = color(format!("{}", balance).white());
        println!("{}: {}", user, fmt_balance);
    }
}

#[derive(Args, Debug)]
struct AddUser {
    /// Name of the user to be added to the ledger
    #[clap(value_parser)]
    name: String
}

#[derive(Args, Debug)]
struct AddTransfer {
    /// Name of user that paid
    #[clap(short='f', long, value_parser)]
    from: String,

    /// Name of user that got paid
    #[clap(short='t', long, value_parser)]
    to: String,

    #[clap(short='a', long, value_parser)]
    amount: Amount,

    #[clap(short='d', long, value_parser, default_value_t = String::from("Transfer"))]
    description: String
}

impl AddTransfer {
    fn add_transfer(&self, ledger: &mut Ledger) {
        let user_from = ledger.get_user_by_name(&self.from)
            .expect(&format!("No such user {}", &self.from)).to_owned();
        let user_to = ledger.get_user_by_name(&self.to)
            .expect(&format!("No such user {}", &self.to)).to_owned();
        let result = ledger.add_transfer(
            &user_from, &user_to, self.amount, &self.description);

        match result {
            Err(err) => panic!("Transaction error: {:?}", &err),
            Ok(_) => ()
        };
    }
}

#[derive(Args, Debug)]
struct AddTransaction {
    /// Pairs of: (name, amount) contributed to this expense
    #[clap(short, long, value_parser)]
    from: Vec<String>,

    /// Names of beneficiaries of the expense. Specifying
    /// amount benefitted is optional, and if omitted "even"
    /// benefit will be assumed.
    ///
    /// Examples:
    /// `Ben George Mike` -> split evenly between all three
    ///
    /// `Ben 14 George Mike` -> Ben spent 14 and the rest is split evenly between George and Mike
    #[clap(short, long, value_parser)]
    to: Vec<String>,

    #[clap(short, long, value_parser)]
    description: String
}

impl AddTransaction {
    pub fn add_transaction(&self, ledger: &mut Ledger) {
        let contributions = AddTransaction::parse_contributors(&self.from);
        let benefits = AddTransaction::parse_beneficiaries(&self.to);

        let result = ledger.add_expense(
            (&contributions).to_borrowed_users(),
            (&benefits).to_borrowed_users(), &self.description);

        if let Err(transaction_error) = result {
            panic!("{}", transaction_error.to_string());
        }
    }

    fn parse_contributors(arguments: &Vec<String>) -> AmountPerUser {
        let mut contributions = AmountPerUser::new();

        for slice in arguments.chunks(2) {
            if slice.len() < 2 {
                panic!("Contributions must be pairs of name and amount");
            }
            let user = User::new(&slice[0]);
            let amount: Amount = slice[1].parse().expect("Must be a number");
            contributions.push((user, amount));
        }
        return contributions;
    }

    fn parse_beneficiaries(arguments: &Vec<String>) -> BenefitPerUser {
        let mut beneficiaries = BenefitPerUser::new();
        let mut prev_user: Option<User> = None;

        for val in arguments {
            match val.parse::<Amount>() {
                Ok(amount) => {
                    // this is the amount that the previous user benefitted
                    match &prev_user {
                        None => panic!("Expected a user before {}", amount),
                        Some(user) => {
                            beneficiaries.push((user.to_owned(), Benefit::Sum(amount)));
                            prev_user = None;
                        }
                    }
                },
                Err(_) => {
                    // this is not a number so it must be a user
                    // if we have a prev_user, its benefit is Even
                    if let Some(user) = prev_user {
                        beneficiaries.push((user, Benefit::Even));
                    }
                    prev_user = Some(User::new(val));
                }
            }
        }
        if let Some(user) = &prev_user {
            beneficiaries.push((user.to_owned(), Benefit::Even));
        }

        return beneficiaries;
    }
}

fn main() {
    let args = Cli::parse();

    let store = JsonStore::new(&args.path);
    let mut ledger = store.read();

    match args.action {
        Subcommands::Balances => {
            print_balances(&ledger);
        },
        Subcommands::List => {
            for t in ledger.get_transactions() {
                println!("{}", t);
            }
        }
        Subcommands::AddUser(add_user) => {
            ledger.add_user(&add_user.name);
        },
        Subcommands::AddTransfer(add_transfer) => {
            add_transfer.add_transfer(&mut ledger);
        },
        Subcommands::AddTransaction(add_transaction) => {
            add_transaction.add_transaction(&mut ledger);
        }
        _ => todo!("{:?}", &args.action)
    }

    store.save(&ledger);
}

#[cfg(test)]
mod parser_tests {
    use divider::{User, transaction::Benefit};
    use rstest::rstest;
    use crate::AddTransaction;

    #[rstest]
    fn parse_contributions_correct() {
        let cmdline = "Bilbo 12 Legolas 20";
        let arguments = cmdline.split(' ')
            .map(|s| s.to_string()).collect::<Vec<String>>();
        let parsed = AddTransaction::parse_contributors(&arguments);

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], (User::new("Bilbo"), 12.0));
        assert_eq!(parsed[1], (User::new("Legolas"), 20.0));
    }

    #[rstest]
    #[should_panic(expected = "Contributions must be pairs of name and amount")]
    fn parse_contributions_odd_arguments() {
        let cmdline = "Bilbo 12 Legolas";
        let arguments = cmdline.split(' ')
            .map(|s| s.to_string()).collect::<Vec<String>>();
        let _ = AddTransaction::parse_contributors(&arguments);
    }

    #[rstest]
    #[should_panic(expected = "Must be a number")]
    fn parse_contributions_not_a_number() {
        let cmdline = "Bilbo 12 Legolas abcdef";
        let arguments = cmdline.split(' ')
            .map(|s| s.to_string()).collect::<Vec<String>>();
        let _ = AddTransaction::parse_contributors(&arguments);
    }

    #[rstest]
    fn parse_beneficiaries_single() {
        let cmdline = "Aragorn";
        let arguments = cmdline.split(' ')
            .map(|s| s.to_string()).collect::<Vec<String>>();
        let beneficiaries = AddTransaction::parse_beneficiaries(&arguments);

        assert_eq!(beneficiaries.len(), 1);
        assert_eq!(beneficiaries[0], (User::new("Aragorn"), Benefit::Even));
    }

    #[rstest]
    fn parse_beneficiaries_all_even() {
        let cmdline = "Bilbo Legolas";
        let arguments = cmdline.split(' ')
            .map(|s| s.to_string()).collect::<Vec<String>>();
        let beneficiaries = AddTransaction::parse_beneficiaries(&arguments);

        assert_eq!(beneficiaries.len(), 2);
        assert_eq!(beneficiaries[0], (User::new("Bilbo"), Benefit::Even));
        assert_eq!(beneficiaries[1], (User::new("Legolas"), Benefit::Even));
    }

    #[rstest]
    fn parse_beneficiaries_some_specific() {
        let cmdline = "Bilbo Legolas 24";
        let arguments = cmdline.split(' ')
            .map(|s| s.to_string()).collect::<Vec<String>>();
        let beneficiaries = AddTransaction::parse_beneficiaries(&arguments);

        assert_eq!(beneficiaries.len(), 2);
        assert_eq!(beneficiaries[0], (User::new("Bilbo"), Benefit::Even));
        assert_eq!(beneficiaries[1], (User::new("Legolas"), Benefit::Sum(24.0)));
    }

    #[rstest]
    #[should_panic(expected = "Expected a user before 30")]
    fn parse_beneficiaries_two_numbers() {
        let cmdline = "Bilbo 24 30 Legolas";
        let arguments = cmdline.split(' ')
            .map(|s| s.to_string()).collect::<Vec<String>>();
        let _ = AddTransaction::parse_beneficiaries(&arguments);
    }

    #[rstest]
    #[should_panic(expected = "Expected a user before 31")]
    fn parse_beneficiaries_starts_with_number() {
        let cmdline = "31 Bilbo Legolas";
        let arguments = cmdline.split(' ')
            .map(|s| s.to_string()).collect::<Vec<String>>();
        let _ = AddTransaction::parse_beneficiaries(&arguments);
    }
}
