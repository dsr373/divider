use divider::{User, Ledger, transaction::{Amount, Benefit}};

use std::path::PathBuf;
use std::fs;

use serde_json;
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

// TODO: subcommands for `new`, `add`, `add-user`, `read`s
#[derive(Debug, Subcommand)]
enum Subcommands {
    /// Read and display balances
    Balances,
    /// Add a new user
    AddUser(AddUser),
    /// Add a new direct transfer
    AddTransfer(AddTransfer),
    /// Add a new expense
    AddTransaction(AddTransaction)
}

#[derive(Args, Debug)]
struct AddUser {
    /// The name of the user to be added to the ledger
    #[clap(short, long, value_parser)]
    name: String
}

#[derive(Args, Debug)]
struct AddTransfer {
    #[clap(short, long, value_parser)]
    from: String,

    #[clap(short, long, value_parser)]
    to: String,

    #[clap(short, long, value_parser)]
    amount: Amount
}

#[derive(Args, Debug)]
struct AddTransaction {
    #[clap(short, long, value_parser)]
    from: Vec<String>
}

fn main() {
    let args = Cli::parse();

    let file_contents = fs::read_to_string(args.path).
        expect("File could not be read");
    let ledger: Ledger = serde_json::from_str(&file_contents).
        expect("File is not valid JSON");

    match args.action {
        Subcommands::Balances => {
            for (u, b) in ledger.get_balances() {
                println!("{}: {}", u, b);
            }
        },
        _ => panic!("Not implemented")
    }
}
