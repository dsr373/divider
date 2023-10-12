use chrono::{Utc, DateTime};
use divider::{Ledger, Amount,
    backend::{LedgerStore, JsonStore},
    transaction::{BenefitPerUser, Benefit, AmountPerUser, TransactionResult}};

use std::path::PathBuf;
use std::error;
use std::result;
use std::process::ExitCode;

use anyhow::anyhow;
use colored::Colorize;
use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, propagate_version = true)]
struct Cli {
   /// Path to ledger file to operate on
   #[arg(value_parser)]
    path: PathBuf,

   /// Action to perform
   #[command(subcommand)]
   action: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    /// Create new ledger
    New {
        /// Names of the users on the ledger
        #[arg(required=true)]
        names: Vec<String>
    },
    /// Read and display balances
    Balances,
    /// List all transactions
    List,
    /// Add a new user
    AddUser {
        /// Name of the user to be added to the ledger
        #[arg()]
        name: String
    },
    /// Add a new direct transfer
    AddDirect(AddDirect),
    /// Add a new expense
    AddExpense(AddExpense),
    /// Undo an existing transaction
    Undo  {
        /// Id of the transaction to undo (as appears in output of 'list')
        #[arg(value_parser = parse_hex_to_int, required=true)]
        id: usize
    }
}

fn parse_hex_to_int(arg: &str) -> Result<usize, std::num::ParseIntError> {
    usize::from_str_radix(arg, 16)
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
        let fmt_balance = color(format!("{:.2}", balance).white());
        println!("{}: {}", user, fmt_balance);
    }
}

fn parse_time_minutes(arg: &str) -> anyhow::Result<DateTime<Utc>> {
    let naive_t = chrono::NaiveDateTime::parse_from_str(arg, "%F %R")?;
    let local_t = match naive_t.and_local_timezone(chrono::offset::Local) {
        chrono::LocalResult::None => Err(anyhow!("no such date time in local timezone: {}", naive_t)),
        chrono::LocalResult::Single(t) => Ok(t),
        chrono::LocalResult::Ambiguous(_, _) => Err(anyhow!("time is ambiguous in local timezone: {}", naive_t)),
    }?;
    return Ok(local_t.with_timezone(&Utc));
}

#[derive(Args, Debug)]
struct AddDirect {
    /// Name of user that paid
    #[arg(short='f', long)]
    from: String,

    /// Name of user that got paid
    #[arg(short='t', long)]
    to: String,

    #[arg(short='a', long)]
    amount: Amount,

    /// Describe the purpose of the transfer
    #[arg(short='d', long, default_value_t = String::from("Transfer"))]
    description: String,

    /// The time the transaction happened. Example format: "2022-05-01 12:21". Default is now.
    #[arg(short='T', long, value_parser = parse_time_minutes)]
    time: Option<DateTime<Utc>>
}

impl AddDirect {
    fn add_direct(&self, ledger: &mut Ledger) -> TransactionResult<()> {
        ledger.add_transfer(&self.from, &self.to, self.amount, &self.description, self.time)
    }
}

#[derive(Args, Debug)]
struct AddExpense {
    /// Pairs of: (name, amount) contributed to this expense. space separated.
    /// Example: `Donald 5 Will 29`
    #[arg(short, long, required=true, num_args=2..)]
    from: Vec<String>,

    /// Names of beneficiaries of the expense. Specifying
    /// amount benefitted is optional, and if omitted "even"
    /// benefit will be assumed.
    ///
    /// Examples:
    /// `Ben George Mike` -> split evenly between all three.
    /// `Ben 14 George Mike` -> Ben spent 14 and the rest is split evenly between George and Mike.
    #[arg(short, long, required=true)]
    to: Vec<String>,

    /// Describe the purpose of the expense
    #[arg(short, long, default_value_t = String::from(""))]
    description: String,

    /// The time the transaction happened. Example format: "2022-05-01 12:21". Default is now.
    #[arg(short='T', long, value_parser = parse_time_minutes)]
    time: Option<DateTime<Utc>>
}

impl AddExpense {
    pub fn add_expense(&self, ledger: &mut Ledger) -> TransactionResult<()> {
        let contributions: AmountPerUser<&str> = AddExpense::parse_contributors(&self.from);
        let benefits: BenefitPerUser<&str> = AddExpense::parse_beneficiaries(&self.to);

        ledger.add_expense(contributions, benefits, &self.description, self.time)
    }

    fn parse_contributors(arguments: &Vec<String>) -> AmountPerUser<&str> {
        let mut contributions: AmountPerUser<&str> = AmountPerUser::new();

        for slice in arguments.chunks(2) {
            if slice.len() < 2 {
                panic!("Contributions must be pairs of name and amount");
            }
            let user_name = &slice[0];
            let amount: Amount = slice[1].parse().expect("Must be a number");
            contributions.push((user_name, amount));
        }
        return contributions;
    }

    fn parse_beneficiaries(arguments: &Vec<String>) -> BenefitPerUser<&str> {
        let mut beneficiaries: BenefitPerUser<&str> = BenefitPerUser::new();
        let mut prev_user: Option<&str> = None;

        for val in arguments {
            match val.parse::<Amount>() {
                Ok(amount) => {
                    // this is the amount that the previous user benefitted
                    match prev_user {
                        None => panic!("Expected a user before {}", amount),
                        Some(user) => {
                            beneficiaries.push((user, Benefit::Sum(amount)));
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
                    prev_user = Some(val);
                }
            }
        }
        if let Some(user) = &prev_user {
            beneficiaries.push((user.to_owned(), Benefit::Even));
        }

        return beneficiaries;
    }
}

type ActionResult = result::Result<(), Box<dyn error::Error>>;

fn execute_action(action: Subcommands, store: &dyn LedgerStore) -> ActionResult {
    match action {
        Subcommands::New{ names } => {
            let ledger = Ledger::new(names);
            store.save(&ledger)
        }
        Subcommands::Balances => {
            let ledger = store.read()?;
            print_balances(&ledger);
            Ok(())
        },
        Subcommands::List => {
            let ledger = store.read()?;
            for t in ledger.get_transactions() {
                println!("{}", t);
            };
            Ok(())
        }
        Subcommands::AddUser{ name } => {
            let mut ledger = store.read()?;
            ledger.add_user(&name);
            store.save(&ledger)
        },
        Subcommands::AddDirect(add_direct) => {
            let mut ledger = store.read()?;
            add_direct.add_direct(&mut ledger)?;
            store.save(&ledger)
        },
        Subcommands::AddExpense(add_expense) => {
            let mut ledger = store.read()?;
            add_expense.add_expense(&mut ledger)?;
            store.save(&ledger)
        },
        Subcommands::Undo{ id } => {
            let mut ledger = store.read()?;
            ledger.reverse_by_id(id)?;
            store.save(&ledger)
        }
    }
}

fn main() -> ExitCode {
    let args = Cli::parse();

    let store = JsonStore::new(&args.path);
    let action_result: ActionResult = execute_action(args.action, &store);

    match action_result {
        Ok(()) => return ExitCode::SUCCESS,
        Err(err) => {
            println!("{}: {}", "Error".bright_red().bold(), err);
            return ExitCode::FAILURE;
        }
    }
}

#[cfg(test)]
mod parser_tests {
    use divider::transaction::Benefit;
    use rstest::rstest;
    use crate::AddExpense;
    use crate::parse_hex_to_int;

    #[rstest]
    fn parse_contributions_correct() {
        let cmdline = "Bilbo 12 Legolas 20";
        let arguments = cmdline.split(' ')
            .map(|s| s.to_string()).collect::<Vec<String>>();
        let parsed = AddExpense::parse_contributors(&arguments);

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], ("Bilbo", 12.0));
        assert_eq!(parsed[1], ("Legolas", 20.0));
    }

    #[rstest]
    #[should_panic(expected = "Contributions must be pairs of name and amount")]
    fn parse_contributions_odd_arguments() {
        let cmdline = "Bilbo 12 Legolas";
        let arguments = cmdline.split(' ')
            .map(|s| s.to_string()).collect::<Vec<String>>();
        let _ = AddExpense::parse_contributors(&arguments);
    }

    #[rstest]
    #[should_panic(expected = "Must be a number")]
    fn parse_contributions_not_a_number() {
        let cmdline = "Bilbo 12 Legolas abcdef";
        let arguments = cmdline.split(' ')
            .map(|s| s.to_string()).collect::<Vec<String>>();
        let _ = AddExpense::parse_contributors(&arguments);
    }

    #[rstest]
    fn parse_beneficiaries_single() {
        let cmdline = "Aragorn";
        let arguments = cmdline.split(' ')
            .map(|s| s.to_string()).collect::<Vec<String>>();
        let beneficiaries = AddExpense::parse_beneficiaries(&arguments);

        assert_eq!(beneficiaries.len(), 1);
        assert_eq!(beneficiaries[0], ("Aragorn", Benefit::Even));
    }

    #[rstest]
    fn parse_beneficiaries_all_even() {
        let cmdline = "Bilbo Legolas";
        let arguments = cmdline.split(' ')
            .map(|s| s.to_string()).collect::<Vec<String>>();
        let beneficiaries = AddExpense::parse_beneficiaries(&arguments);

        assert_eq!(beneficiaries.len(), 2);
        assert_eq!(beneficiaries[0], ("Bilbo", Benefit::Even));
        assert_eq!(beneficiaries[1], ("Legolas", Benefit::Even));
    }

    #[rstest]
    fn parse_beneficiaries_some_specific() {
        let cmdline = "Bilbo Legolas 24";
        let arguments = cmdline.split(' ')
            .map(|s| s.to_string()).collect::<Vec<String>>();
        let beneficiaries = AddExpense::parse_beneficiaries(&arguments);

        assert_eq!(beneficiaries.len(), 2);
        assert_eq!(beneficiaries[0], ("Bilbo", Benefit::Even));
        assert_eq!(beneficiaries[1], ("Legolas", Benefit::Sum(24.0)));
    }

    #[rstest]
    #[should_panic(expected = "Expected a user before 30")]
    fn parse_beneficiaries_two_numbers() {
        let cmdline = "Bilbo 24 30 Legolas";
        let arguments = cmdline.split(' ')
            .map(|s| s.to_string()).collect::<Vec<String>>();
        let _ = AddExpense::parse_beneficiaries(&arguments);
    }

    #[rstest]
    #[should_panic(expected = "Expected a user before 31")]
    fn parse_beneficiaries_starts_with_number() {
        let cmdline = "31 Bilbo Legolas";
        let arguments = cmdline.split(' ')
            .map(|s| s.to_string()).collect::<Vec<String>>();
        let _ = AddExpense::parse_beneficiaries(&arguments);
    }

    #[rstest]
    fn parse_hex() {
        assert_eq!(parse_hex_to_int("00c0").unwrap(), 12 * 16);
        assert_eq!(parse_hex_to_int("00c5").unwrap(), 12 * 16 + 5);
        assert_eq!(parse_hex_to_int("0ad8").unwrap(), 10 * 256 + 13 * 16 + 8);
        assert!(   parse_hex_to_int("00ga").is_err());
    }
}
