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
        for (position, result) in reader.deserialize().enumerate() {
            let transaction: Transaction = result?;

            match transaction.transaction_type {
                TransactionType::Deposit => {
                    let _ = self.deposit(&transaction)?;
                }
                TransactionType::Withdrawal => self.withdraw(&transaction),
                TransactionType::Dispute => {
                    let _ = self.dispute_transaction(&transaction, position)?;
                }
                TransactionType::Resolve => {
                    let _ = self.resolve_transaction(&transaction, position)?;
                }
                TransactionType::Chargeback => {
                    let _ = self.chargeback_transaction(&transaction, position)?;
                }
            }
        }
        Ok(self.accounts)
    }

    /// Deposits client's founds.
    /// Creates a new account if client's account doesn't exist yet.
    fn deposit(&mut self, transaction: &Transaction) -> Result<(), Box<dyn Error>> {
        let account = self.accounts.entry(transaction.client).or_default();
        account.deposit(transaction.amount)?;
        Ok(())
    }

    /// Withdraws funds if the client's account has sufficient available funds.
    /// Creates a new account if client's account doesn't exist yet.
    fn withdraw(&mut self, transaction: &Transaction) {
        let account = self.accounts.entry(transaction.client).or_default();
        account.withdraw(transaction.amount);
    }

    /// Moves amount from the available funds to the held funds that has been deposited
    /// by a transaction with the same id and for the same client.
    /// If a deposit transaction is not found then drop the operation.
    /// If a dispute is duplicated or the order of transactions
    /// with the same id isn't right then drop.
    fn dispute_transaction(
        &mut self,
        transaction: &Transaction,
        position: usize,
    ) -> Result<(), Box<dyn Error>> {
        let transactions_with_positions =
            self.find_transactions(transaction.client, transaction.tx, position)?;
        if transactions_with_positions.len() == 2 {
            let (deposit, _) = &transactions_with_positions[0];
            let (_, dispute_position) = &transactions_with_positions[1];

            if deposit.transaction_type == TransactionType::Deposit && *dispute_position == position
            {
                if let Some(account) = self.accounts.get_mut(&transaction.client) {
                    account.dispute(deposit.amount)?;
                }
            }
        }
        Ok(())
    }

    /// Moves amount from the held funds to the available funds that has been deposited
    /// by a transaction with the same id and for the same client.
    /// If a deposit transaction is not found then drop the operation.
    /// If a dispute hasn't be executed or the order of transactions
    /// with the same id isn't right then drop.
    fn resolve_transaction(
        &mut self,
        transaction: &Transaction,
        position: usize,
    ) -> Result<(), Box<dyn Error>> {
        let transactions_with_positions =
            self.find_transactions(transaction.client, transaction.tx, position)?;
        if transactions_with_positions.len() == 3 {
            let (deposit, _) = &transactions_with_positions[0];
            let (dispute, _) = &transactions_with_positions[1];
            let (_, resolve_position) = &transactions_with_positions[2];

            if deposit.transaction_type == TransactionType::Deposit
                && dispute.transaction_type == TransactionType::Dispute
                && *resolve_position == position
            {
                if let Some(account) = self.accounts.get_mut(&transaction.client) {
                    account.resolve(deposit.amount)?;
                }
            }
        }
        Ok(())
    }

    /// Withdraws amount from held funds that has been deposited
    /// by a transaction with the same id and for the same client.
    /// If a deposit transaction is not found then drop the operation.
    /// If a dispute hasn't be executed or the order of transactions
    /// with the same id isn't right then drop.
    fn chargeback_transaction(
        &mut self,
        transaction: &Transaction,
        position: usize,
    ) -> Result<(), Box<dyn Error>> {
        let transactions_with_positions =
            self.find_transactions(transaction.client, transaction.tx, position)?;
        if transactions_with_positions.len() == 3 {
            let (deposit, _) = &transactions_with_positions[0];
            let (dispute, _) = &transactions_with_positions[1];
            let (_, chargeback_position) = &transactions_with_positions[2];

            if deposit.transaction_type == TransactionType::Deposit
                && dispute.transaction_type == TransactionType::Dispute
                && *chargeback_position == position
            {
                if let Some(account) = self.accounts.get_mut(&transaction.client) {
                    account.chargeback(deposit.amount);
                }
            }
        }
        Ok(())
    }

    /// Finds up to three first transactions with the given client and transaction id.
    /// Returns the vector of tuples which contains transaction and their positions
    /// in the CSV file.
    /// The search is either ended by finding three transactions or reaching the passed
    /// position.
    /// # Arguments
    /// * `client` - client id
    /// * `tx` - transaction id
    /// * `end_position` - a CSV record position which ends the search
    fn find_transactions(
        &self,
        client: u16,
        tx: u32,
        end_position: usize,
    ) -> Result<Vec<(Transaction, usize)>, Box<dyn Error>> {
        let mut reader = ReaderBuilder::new()
            .trim(Trim::All)
            .from_path(self.transactions_path)?;
        let mut transactions = Vec::with_capacity(3);
        let mut count = 0;

        for (position, result) in reader.deserialize().enumerate() {
            let record: Transaction = result?;
            if record.client == client && record.tx == tx {
                transactions.push((record, position));
                count += 1;
            }
            if count == 3 || position == end_position {
                break;
            }
        }

        Ok(transactions)
    }
}
