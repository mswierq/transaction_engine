use crate::amount_type::{amount_serde, AmountType};
use crate::client_account::ClientAccount;
use csv::IntoInnerError;
use csv::{Writer, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;

pub type AccountsBase = HashMap<u16, ClientAccount>;

#[derive(Serialize, Deserialize)]
pub struct AccountRecord {
    pub client: u16,
    #[serde(with = "amount_serde")]
    pub available: AmountType,
    #[serde(with = "amount_serde")]
    pub held: AmountType,
    #[serde(with = "amount_serde")]
    total: AmountType,
    pub locked: bool,
}

pub fn serialize_accounts_base<W>(
    accounts: &AccountsBase,
    writer: W,
) -> Result<W, IntoInnerError<Writer<W>>>
where
    W: Write,
{
    let mut csv_writer = WriterBuilder::new().from_writer(writer);
    for (client, account) in accounts {
        let record = AccountRecord {
            client: *client,
            available: account.available,
            held: account.held,
            total: account.total(),
            locked: account.locked,
        };
        csv_writer.serialize(&record).unwrap();
    }
    csv_writer.into_inner()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_accounts_base_single_record() {
        let mut accounts = AccountsBase::new();
        accounts.insert(1, ClientAccount::default());
        let output = serialize_accounts_base(&accounts, vec![]).unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "client,available,held,total,locked\n1,0.0,0.0,0.0,false\n"
        );
    }
}
