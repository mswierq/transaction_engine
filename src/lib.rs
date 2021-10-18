pub mod accounts_base;
mod amount_type;
pub mod client_account;
mod transactions;

use crate::accounts_base::AccountsBase;
use crate::amount_type::AmountType;
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

            match transaction {
                Transaction {
                    transaction_type: TransactionType::Deposit,
                    client,
                    tx: _,
                    amount,
                } => {
                    let _ = self.deposit(client, amount)?;
                }
                Transaction {
                    transaction_type: TransactionType::Withdrawal,
                    client,
                    tx: _,
                    amount,
                } => self.withdraw(client, amount),
                Transaction {
                    transaction_type: TransactionType::Dispute,
                    client,
                    tx,
                    amount: _,
                } => {
                    let _ = self.dispute_transaction(client, tx)?;
                }
                Transaction {
                    transaction_type: TransactionType::Resolve,
                    client,
                    tx,
                    amount: _,
                } => {
                    let _ = self.resolve_transactions(client, tx)?;
                }
                Transaction {
                    transaction_type: TransactionType::Chargeback,
                    client,
                    tx,
                    amount: _,
                } => self.chargeback_transactions(client, tx),
                _ => unimplemented!(),
            }
        }
        Ok(self.accounts)
    }

    /// Deposits client founds.
    /// Creates a new account if client's account doesn't exist yet.
    fn deposit(&mut self, client: u16, amount: AmountType) -> Result<(), Box<dyn Error>> {
        let account = self.accounts.entry(client).or_default();
        account.deposit(amount)?;
        Ok(())
    }

    /// Withdraws funds if client's account exists.
    fn withdraw(&mut self, client: u16, amount: AmountType) {
        let account = self.accounts.entry(client).or_default();
        account.withdraw(amount);
    }

    /// Finds a transaction to dispute in the CSV transaction file.
    /// If a deposit transaction is not found then drop the operation.
    fn dispute_transaction(&mut self, client: u16, tx: u32) -> Result<(), Box<dyn Error>> {
        let mut reader = ReaderBuilder::new()
            .trim(Trim::All)
            .from_path(self.transactions_path)?;
        for result in reader.deserialize() {
            let transaction: Transaction = result?;

            if transaction.transaction_type == TransactionType::Deposit
                && transaction.client == client
                && transaction.tx == tx
            {
                if let Some(account) = self.accounts.get_mut(&client) {
                    account.dispute(transaction.amount)?;
                    break;
                }
            }
            // A deposit operation hasn't been found, the dispute transaction that started this
            // processing has been found. Drop the operation.
            if transaction.transaction_type == TransactionType::Dispute
                && transaction.client == client
                && transaction.tx == tx
            {
                break;
            }
        }
        Ok(())
    }

    /// Finds matching deposit and dispute transactions in the CSV transaction file
    /// to resolve the dispute.
    /// If the transactions are not found then drop the operation.
    fn resolve_transactions(&mut self, client: u16, tx: u32) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }

    /// Finds matching deposit and dispute transactions in the CSV transaction file
    /// to chargeback the client.
    /// If the transactions are not found then drop the operation.
    fn chargeback_transactions(&mut self, client: u16, tx: u32) {
        unimplemented!()
    }
}
