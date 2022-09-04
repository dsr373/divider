use divider::{User, Ledger,
    backend::{LedgerStore, JsonStore},
    transaction::{Amount, AmountPerUser, BenefitPerUser}};

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
        let result = ledger.add_transfer(&user_from, &user_to, self.amount, &self.description);

        match result {
            Err(err) => panic!("Transaction error: {:?}", &err),
            Ok(_) => ()
        }
    }
}

#[derive(Args, Debug)]
struct AddTransaction {
    #[clap(short, long, value_parser)]
    from: Vec<String>,

    #[clap(short, long, value_parser)]
    to: Vec<String>,

    #[clap(short, long, value_parser)]
    description: String
}

impl AddTransaction {
    fn parse_contributors(arguments: Vec<String>) -> AmountPerUser {
        vec![]
    }

    fn parse_beneficiaries(arguments: Vec<String>) -> BenefitPerUser {
        vec![]
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
        }
        _ => todo!("{:?}", &args.action)
    }

    store.save(&ledger);
}
