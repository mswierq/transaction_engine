use regex::Regex;
use serde::de::Error;
use serde::{Deserialize, Deserializer};

/// This is an alias of the actual type that holds the amount.
/// The type is i64, the value represents a multiple of 0.0001.
pub type AmountType = i64;

/// Deserializes the amount from string.
/// Returns an error if the format of the string is invalid or value is overflown!
pub fn deserialize_amount<'de, D>(deserializer: D) -> Result<AmountType, D::Error>
where
    D: Deserializer<'de>,
{
    const PRECISION: usize = 4;

    let amount_str = String::deserialize(deserializer)?;
    let re = Regex::new(r"^(\d+)(?:\.{0,1})(\d{0,4})$").unwrap();

    if let Some(capture) = re.captures_iter(&amount_str).next() {
        let mut result = capture[1].parse::<AmountType>().map_err(D::Error::custom)?
            * ((10 as AmountType).pow(PRECISION as u32)); //decimal
        if !&capture[2].is_empty() {
            let fractional_len = capture[2].len();
            let fractional = capture[2].to_owned()
                + &(0..PRECISION - fractional_len)
                    .map(|_| "0")
                    .collect::<String>();
            result += fractional.parse::<AmountType>().map_err(D::Error::custom)?;
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
    use rstest::rstest;
    use serde::Deserialize;
    use serde_json;

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestStruct {
        #[serde(deserialize_with = "deserialize_amount")]
        amount: AmountType,
    }

    #[rstest]
    #[case(".0")]
    #[case("A")]
    #[case("1.3434.233")]
    #[case(".3434.233")]
    #[case("a.233")]
    fn test_deserialize_invalid_amount(#[case] invalid_amount: &str) {
        let data = r#"{"amount": ""#.to_owned() + invalid_amount + r#""}"#;
        let result: Result<TestStruct, _> = serde_json::from_str(&data);
        assert!(result
            .unwrap_err()
            .to_string()
            .contains(&format!("Invalid amount format! {}", invalid_amount)));
    }

    #[rstest]
    #[case("999999999999999999999999999999999999999999999999999999999")]
    fn test_deserialize_too_large_number(#[case] invalid_amount: &str) {
        let data = r#"{"amount": ""#.to_owned() + invalid_amount + r#""}"#;
        let result: Result<TestStruct, _> = serde_json::from_str(&data);
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("number too large to fit in target type"));
    }

    #[rstest]
    #[case("1.0", 10000)]
    #[case("21.001", 210010)]
    #[case("1323.3434", 13233434)]
    #[case("233", 2330000)]
    fn test_deserialize_valid_amount(#[case] valid_amount: &str, #[case] expected: AmountType) {
        let data = r#"{"amount": ""#.to_owned() + valid_amount + r#""}"#;
        let result: Result<TestStruct, _> = serde_json::from_str(&data);
        assert_eq!(result.unwrap().amount, expected);
    }
}
