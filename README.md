# Divider
An app to split a series of payments in a group flexibly

## Installation
This app is written in Rust so can be installed with Cargo.
First, obtain Rust for your system [from here](https://www.rust-lang.org/tools/install).

Then you can just install the package directly from github with:
```
cargo install --git https://github.com/dsr373/divider
```

Or you can clone this repository and install from your local version:
```
git clone https://github.com/dsr373/divider
cargo install --path divider
```

## Idea
The app operates on a **ledger** of transactions made between **registered people**.

Imagine you go on holiday with a group of friends. Someone pays for the accommodation, someone else pays for dinner one night, you do two separate grocery runs and different people pay again, somebody else yet again pays for the tickets; you go out for drinks but two of your friends do something else. This app is to help you keep track of who owes who.

The way it works is by giving each participant a balance. Your balance goes up when paying for someone else's stuff (or when you pay someone directly), and it goes down when someone else pays for you. If your balance is negative, you owe money to the people with a positive balance.

Let's take some examples.

Say Alice, Bob and Charlie go out for some drinks, and Alice pays the tab of $75 at the end. Assuming she remembers to record her spending, and that they agree to split it evenly, you have:
> Alice paid $75 for Alice, Bob and Charlie
>
> Each of their share is $25, so Bob and Charlie owe Alice $25 each
>
> The Alice's balance goes up by 50, and Bob and Charlie get -25 each

Say Alex, Ben, Cara and Danielle have dinner together. It costs them $120 all together, but Ben had a fancy steak so his meal was $45. The rest agree to split evenly, and Cara picks up the bill:
> Ben has spent $45
>
> Alex, Cara and Danielle have spent (120 - 45) / 3 = $25 each
>
> Ben's balance decreases by $45, Alex's and Danielle's by $25, and Cara's balance goes up by 120 - 25 (for her own meal) = $95
>
> As a check, $95 is equal to what the other 3 owe Cara: 95 = 25 + 25 + 45

Note that the sum of balances must always be zero, as all the transactions are between the people in the group.

Finally, the app distinguishes between *direct payments*, when one member pays another, and *expenses*, when someone spends money for someone else's goods. This is to be able to calculate how much money has been actually spent (i.e. left the group, as opposed to being moved around).

## Usage
At the moment, the repository provides one executable which is a command line app (though the plan is to provide more options to do the same thing, for different use cases).
The executable installed is `divider-cli`.


The executable is called as
```
divider-cli LEDGER ACTION arguments...
```

To start a ledger for a group consisting of Alex, Ben, Cara and Danielle, create a new `ledger.json` file:
```
divider-cli ledger.json new Alex Ben Cara Danielle
```

To record the transaction in the second example above, you can run:
```
divider-cli ledger.json add-expense --from Cara 120 --to Ben 45 Alex Cara Danielle --description 'dinner at the pub' --time '2022-06-27 23:35'
```
Note that the time and description are optional. If you don't provide a time, the current time is used. The default description is just empty.

If Ben pays Cara back for his meal, you can record this **direct** payment like so:
```
divider-cli ledger.json add-direct --from Ben --to Cara --amount 45
```

Then you can check everyone's balances by doing:
```
divider-cli ledger.json balances
```
To list the history of transactions:
```
divider-cli ledger.json list
```

The first four-character code in the list of transactions is the ID. If you've made a mistake or want to otherwise undo a transaction with ID `0b3f`, you can do that with the `undo` command:
```
divider-cli ledger.json undo 0b3f
```

The executable and each subcommand can be called with `--help` to find out more about their interfaces.
