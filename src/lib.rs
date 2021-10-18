pub mod accounts_base;
mod amount_type;
pub mod client_account;
mod transactions;

use crate::accounts_base::AccountsBase;
use crate::transactions::{Transaction, TransactionType};
use csv::{ReaderBuilder, Trim};
use std::error::Error;

/// Processes the transaction in a CSV file given as path
pub struct TransactionEngine<'a> {
    transactions_path: &'a str,
    accounts: AccountsBase,
}

impl<'a> TransactionEngine<'a> {
    /// Creates new engine
    /// # Arguments:
    /// * `path` - path to the CSV file with transactions
    pub fn new(path: &'a str) -> Self {
        TransactionEngine {
            transactions_path: path,
            accounts: AccountsBase::new(),
        }
    }

    /// Processes the transactions.
    /// Returns AccountsBase object or an error.
    pub fn process(mut self) -> Result<AccountsBase, Box<dyn Error>> {
        let mut reader = ReaderBuilder::new()
            .trim(Trim::All)
            .from_path(self.transactions_path)?;
        for result in reader.deserialize() {
            let transaction: Transaction = result?;

            let account = self.accounts.entry(transaction.client).or_default();
            match transaction {
                Transaction {
                    transaction_type: TransactionType::Deposit,
                    client: _,
                    tx: _,
                    amount,
                } => account.deposit(amount)?,
                Transaction {
                    transaction_type: TransactionType::Withdrawal,
                    client: _,
                    tx: _,
                    amount,
                } => account.withdraw(amount),
                _ => unimplemented!(),
            }
        }
        Ok(self.accounts)
    }
}
