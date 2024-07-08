use super::deserialize::{Deserialize, WithIndex};
use super::serialize::Serialize;
use crate::check_crlf;
use crate::constants::CRLF;
use crate::error::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nulls;

impl Serialize for Nulls {
    fn serialize(&self) -> String {
        format!("_{CRLF}")
    }
}
pub struct NullsWithIndex {
    pub value: Nulls,
    pub index: (usize, usize),
}
impl NullsWithIndex {
    fn index(self) -> (usize, usize) {
        self.index
    }
}
impl WithIndex for NullsWithIndex {
    fn get_index(self) -> (usize, usize) {
        self.index()
    }
}
impl Deserialize for Nulls {
    type Value = NullsWithIndex;
    fn deserialize(bytes: &[u8]) -> Result<Self::Value> {
        match bytes.get(0) {
            Some(b'_') => {
                if check_crlf(bytes, 1)? {
                    Ok(NullsWithIndex {
                        value: Nulls,
                        index: (0, 2),
                    })
                } else {
                    return Result::Err(RedisError::NullsParseError(ParseError::InvalidFormat));
                }
            }
            _ => Result::Err(RedisError::NullsParseError(ParseError::InvalidFormat)),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::CRLF;
    use crate::value::Value;
    fn serialize_and_assert(value: Value, expected: &str) {
        assert_eq!(expected, Value::serialize(&value));
    }

    #[test]
    fn test_serialize_nulls() {
        let value = Value::Nulls(Nulls);
        let expected = format!("_{CRLF}");
        serialize_and_assert(value, &expected);
    }

    #[test]
    fn test_deserialize_nulls() {
        let test_cases = vec![
            // Valid test cases
            (b"_\r\n".to_vec(), Nulls, true),
        ];

        for (input, expected, should_succeed) in test_cases {
            let result = Nulls::deserialize(input.as_slice());
            if should_succeed {
                assert_eq!(result.unwrap().value, expected);
            } else {
                assert!(result.is_err());
            }
        }
    }
}
