# Transaction Engine

This project implements a simple simulator of a transaction engine which takes a chronological transactions list
as a CSV file.

The output of the application is clients' accounts in CSV format printed to the stdout.

## How to run tests

```bash
cargo test
```

Runs the unit tests and the integration tests. The integration tests are kept in **tests** directory.

## How to run application

```bash
cargo run -- transactions.csv > accounts.csv
```

## Client's account

Each client account keeps the following data:

- **available funds** - these funds can be either credited or debited,
- **held funds** - these funds are being frozen due to a **dispute**,
- **total funds** - this is a sum of **available** and **held** funds,
- **locked** flag - if account is locked no transaction takes effect.

## Supported transactions

Each **Deposit** and **Withdrawal** transaction has a unique (u32) transaction id.
Other transactions are used to reverse a **Deposit** and should have the same transaction id.

- **Deposit** - increases the **available** funds if account isn't locked and creates a new account if doesn't exist.
- **Withdrawal** - decreases the **available** funds if account isn't locked or there is sufficient amount of funds, 
                   and creates a new account if doesn't exist.
- **Dispute** - moves the amount of funds from the **Deposit** with the same id from the **available** funds
                to the **held** funds.
- **Resolve** - concludes a **Dispute** and moves the amount of funds from the **Deposit** with the same id
                from the **held** funds to the **available** funds.
- **Chargeback** - concludes a **Dispute** and withdraws the amount of funds from the **Deposit** with the same id
                   from the **held** funds.

## Supported scenarios

All described scenarios are tested by the integration tests in **tests** directory.

1. Only deposits can be disputed, trying to dispute a different kind of transaction will take no effect.
2. A dispute can be concluded either by resolve or chargeback, never both.
   If a resolve will be followed back by a chargeback, and vice versa, the first transaction take prevail.
3. A duplicated dispute transaction is ignored, but the first takes effect.
4. A duplicated chargeback transaction is ignored, but the first takes effect.
5. A duplicated resolve transaction is ignored, but the first takes effect.
6. A transaction that is a dispute, resolve or a chargeback and has a unique transaction id is dropped.
7. Either a chargeback or a resolve transaction called after by more than one dispute transaction with the same id
   don't take effect.
8. Either a chargeback or a resolve transaction called when there hasn't been a dispute transaction with the same id
   don't take effect.
9. Executing a withdrawal between a deposit and a dispute transaction that reverses it
   can cause a negative balance in the available funds.

## Implementation details

### Amount type

The amount in transactions and in printed accounts has to be a decimal with a precision of four places past the decimal.
The amount is kept in i64, value of the integer represents a multiple of 0.0001. It is better to keep the amount in an
integer than in a float, because it gives a better accuracy. This way the accounts and transactions can keep values from
~ **-9.22E-14** to ~ **9.22E-14**. If a fund in an account gets overflown, the application panics!

### Using the input CSV as a history of transactions

To properly handle a dispute, resolve or a chargeback, the history of previous transactions has to be checked. Each time
a before-mentioned transaction is executed the application searches for previous transactions in the CSV file.
Keeping a history of all transactions or even only the deposits in memory would be a good idea for a small data set.
Keeping in mind that the input CSV file can keep up to 2^32 deposits would need gigabytes of RAM to handle it. Thus,
the input file serves as a history.