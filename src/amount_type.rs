/// This is an alias of the actual type that holds the amount.
/// The type is i64, the value represents a multiple of 0.0001.
pub type AmountType = i64;

#[warn(clippy::unnecessary_cast)]
pub mod amount_serde {
    use super::AmountType;
    use regex::Regex;
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    const PRECISION: usize = 4;
    #[allow(clippy::unnecessary_cast)]
    const WHOLE_NUMBER: AmountType = (10 as AmountType).pow(PRECISION as u32);

    /// Serializes the amount to string.
    /// Always returns an OK with result.
    pub fn serialize<S>(amount: &AmountType, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut amount_str = format!(
            "{}.{:0>4}",
            amount / WHOLE_NUMBER,
            amount - (amount / WHOLE_NUMBER) * WHOLE_NUMBER
        );
        //trim trailing zeros, but no more than 3
        let mut counter = 0;
        while amount_str.ends_with('0') && counter < 3 {
            amount_str.truncate(amount_str.len() - 1);
            counter += 1;
        }
        amount_str.serialize(serializer)
    }

    /// Deserializes the amount from string.
    /// Returns an error if the format of the string is invalid or value is overflown!
    pub fn deserialize<'de, D>(deserializer: D) -> Result<AmountType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let amount_str = String::deserialize(deserializer)?;

        if amount_str.is_empty() {
            return Ok(0);
        }

        let re = Regex::new(r"^(\-?)(\d+)(?:\.?)(\d{0,4})$").unwrap();

        if let Some(capture) = re.captures_iter(&amount_str).next() {
            let sign: AmountType = if !capture[1].is_empty() { -1 } else { 1 };
            let mut result =
                capture[2].parse::<AmountType>().map_err(D::Error::custom)? * WHOLE_NUMBER; //decimal
            if !&capture[3].is_empty() {
                let fractional_len = capture[3].len();
                let fractional = capture[3].to_owned()
                    + &(0..PRECISION - fractional_len)
                        .map(|_| "0")
                        .collect::<String>();
                result += fractional.parse::<AmountType>().map_err(D::Error::custom)?;
            }
            return Ok(sign * result);
        }
        Err(D::Error::custom(format!(
            "Invalid amount format! {}",
            amount_str
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde::{Deserialize, Serialize};
    use serde_json;

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct TestStruct {
        #[serde(with = "amount_serde")]
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
    #[case("-233.01", -2330100)]
    #[case("-233", -2330000)]
    #[case("", 0)]
    fn test_deserialize_valid_amount(#[case] valid_amount: &str, #[case] expected: AmountType) {
        let data = r#"{"amount": ""#.to_owned() + valid_amount + r#""}"#;
        let result: Result<TestStruct, _> = serde_json::from_str(&data);
        assert_eq!(result.unwrap().amount, expected);
    }

    #[rstest]
    #[case(10000, "1.0")]
    #[case(210010, "21.001")]
    #[case(13233434, "1323.3434")]
    #[case(2330200, "233.02")]
    #[case(0, "0.0")]
    fn test_serialize_amount(#[case] input: AmountType, #[case] expected: &str) {
        let test_struct = TestStruct { amount: input };
        assert_eq!(
            serde_json::to_string(&test_struct).unwrap(),
            r#"{"amount":""#.to_owned() + expected + r#""}"#
        )
    }
}
