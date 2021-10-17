use regex::Regex;
use serde::de::Error;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize, PartialEq, Debug)]
pub enum TransactionType {
    #[serde(rename = "deposit")]
    Deposit,
    #[serde(rename = "withdrawal")]
    Withdrawal,
    #[serde(rename = "dispute")]
    Dispute,
    #[serde(rename = "resolve")]
    Resolve,
    #[serde(rename = "chargeback")]
    Chargeback,
}

//This struct represents a deserialized transaction record in a CSV file.
#[derive(Deserialize, Debug, PartialEq)]
pub struct Transaction {
    #[serde(rename = "type")]
    transaction_type: TransactionType,
    client: u16,
    tx: u32,
    #[serde(deserialize_with = "deserialize_amount")]
    amount: i64,
}

fn deserialize_amount<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    const PRECISION: usize = 4;

    let amount_str = String::deserialize(deserializer)?;
    let re = Regex::new(r"^(\d+)(?:\.{0,1})(\d{0,4})$").unwrap();

    if let Some(capture) = re.captures_iter(&amount_str).next() {
        let mut result = capture[1].parse::<i64>().map_err(D::Error::custom).unwrap()
            * (10_i64.pow(PRECISION as u32)); //decimal
        if !&capture[2].is_empty() {
            let fractional_len = capture[2].len();
            let fractional = capture[2].to_owned()
                + &(0..PRECISION - fractional_len)
                    .map(|_| "0")
                    .collect::<String>();
            result += fractional.parse::<i64>().map_err(D::Error::custom).unwrap();
        }
        return Ok(result);
    }
    Err(D::Error::custom(format!(
        "Invalid amount format! {}",
        amount_str
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv::{ReaderBuilder, Trim};
    use rstest::rstest;

    #[test]
    fn test_successful_records_read() {
        let data = "\
type,\tclient\t,\ttx,\tamount
deposit,\t1,\t1,\t1.0
withdrawal,\t2,\t2,\t2.1000
dispute,\t3,\t3,\t2.01
resolve,\t4,\t4,\t3.003
chargeback,\t5,\t5,\t0";

        let expected = vec![
            Transaction {
                transaction_type: TransactionType::Deposit,
                client: 1,
                tx: 1,
                amount: 10000,
            },
            Transaction {
                transaction_type: TransactionType::Withdrawal,
                client: 2,
                tx: 2,
                amount: 21000,
            },
            Transaction {
                transaction_type: TransactionType::Dispute,
                client: 3,
                tx: 3,
                amount: 20100,
            },
            Transaction {
                transaction_type: TransactionType::Resolve,
                client: 4,
                tx: 4,
                amount: 30030,
            },
            Transaction {
                transaction_type: TransactionType::Chargeback,
                client: 5,
                tx: 5,
                amount: 0,
            },
        ];

        let mut reader = ReaderBuilder::new()
            .trim(Trim::All)
            .delimiter(b',')
            .from_reader(data.as_bytes());
        let mut expected_iter = expected.iter();

        for result in reader.deserialize() {
            let record: Transaction = result.unwrap();
            assert_eq!(&record, expected_iter.next().unwrap());
        }

        assert_eq!(expected_iter.next(), None);
    }

    #[rstest]
    #[case(".0")]
    #[case("A")]
    #[case("1.3434.233")]
    #[case(".3434.233")]
    #[case("a.233")]
    fn test_read_record_invalid_amount(#[case] invalid_amount: &str) {
        let record_to_read = "deposit, 1, 1, ".to_owned() + invalid_amount;
        let mut reader = ReaderBuilder::new()
            .trim(Trim::All)
            .has_headers(false)
            .delimiter(b',')
            .from_reader((record_to_read).as_bytes());

        let record: Result<Transaction, _> = reader.deserialize().next().unwrap();

        assert_eq!(
            record.unwrap_err().to_string(),
            format!(
                "CSV deserialize error: record 0 (line: 1, byte: 0): Invalid amount format! {}",
                invalid_amount
            )
        );
    }
}
