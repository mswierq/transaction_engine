use csv::{ReaderBuilder, Trim};
use rstest::rstest;
use std::path::Path;
use transaction_engine::accounts_base::{AccountRecord, AccountsBase};
use transaction_engine::client_account::ClientAccount;
use transaction_engine::TransactionEngine;

fn read_expected_accounts(path: &Path) -> AccountsBase {
    let mut reader = ReaderBuilder::new()
        .trim(Trim::All)
        .from_path(path)
        .unwrap();
    let mut accounts = AccountsBase::new();

    for result in reader.deserialize() {
        let record: AccountRecord = result.unwrap();
        accounts.insert(
            record.client,
            ClientAccount {
                available: record.available,
                held: record.held,
                locked: record.locked,
            },
        );
    }
    accounts
}

#[rstest]
#[case(
    "basic_deposit_and_withdrawal_tx.csv",
    "basic_deposit_and_withdrawal_accounts.csv"
)]
#[case("basic_withdrawals_tx.csv", "basic_withdrawals_accounts.csv")]
#[case("basic_dispute_tx.csv", "basic_dispute_accounts.csv")]
fn test_transaction_engine(#[case] input: &str, #[case] expected: &str) {
    let transactions_path = Path::new(file!()).parent().unwrap().join(input);
    let expected_path = Path::new(file!()).parent().unwrap().join(expected);
    let engine = TransactionEngine::new(transactions_path.to_str().unwrap());
    let accounts = engine.process();
    assert_eq!(accounts.unwrap(), read_expected_accounts(&expected_path));
}
