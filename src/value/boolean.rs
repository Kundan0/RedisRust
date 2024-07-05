use super::deserialize::{Deserialize, WithIndex};
use super::serialize::Serialize;
use crate::check_crlf;
use crate::constants::CRLF;
use crate::error::*;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Boolean(pub bool);

impl Serialize for Boolean {
    fn serialize(self) -> String {
        if self.0 {
            format!("#t{CRLF}")
        } else {
            format!("#f{CRLF}")
        }
    }
}
pub struct BooleanWithIndex {
    pub value: Boolean,
    pub index: (usize, usize),
}
impl BooleanWithIndex {
    fn index(self) -> (usize, usize) {
        self.index
    }
}
impl WithIndex for BooleanWithIndex {
    fn get_index(self) -> (usize, usize) {
        self.index()
    }
}
impl Deserialize for Boolean {
    type Value = BooleanWithIndex;

    fn deserialize(bytes: &[u8]) -> Result<Self::Value> {
        if bytes.get(0) != Some(&b'#') {
            return Err(RedisError::BooleanParseError(ParseError::InvalidFormat));
        }

        let value = match bytes.get(1) {
            Some(b't') => true,
            Some(b'f') => false,
            _ => return Err(RedisError::BooleanParseError(ParseError::InvalidFormat)),
        };
        // Start index is always 0
        //    2 3
        //    | |
        //  #t\r\n
        //

        match check_crlf(bytes, 2) {
            Ok(true) => Ok(BooleanWithIndex {
                value: Boolean(value),
                index: (0, 3),
            }),
            _ => Err(RedisError::BooleanParseError(ParseError::InvalidFormat)),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::CRLF;
    use crate::value::Value;

    fn serialize_and_assert(value: Value, expected: &str) {
        assert_eq!(expected, value.serialize());
    }

    #[test]
    fn test_serialize_boolean() {
        let value_t = Value::Boolean(Boolean(true));
        let value_f = Value::Boolean(Boolean(false));
        let expected_t = format!("#t{CRLF}");
        let expected_f = format!("#f{CRLF}");
        serialize_and_assert(value_t, &expected_t);
        serialize_and_assert(value_f, &expected_f);
    }

    #[test]
    fn test_deserialize_boolean() {
        let test_cases = vec![
            // Valid test cases
            (b"#t\r\n".to_vec(), Boolean(true), true),
            (b"#f\r\n".to_vec(), Boolean(false), true),
            // Cases that should return Err
            // Missing CRLF
            (b"#t\n".to_vec(), Boolean(true), false),
            // Invalid boolean values
            (b"#x\r\n".to_vec(), Boolean(false), false),
            (b"#\r\n".to_vec(), Boolean(false), false),
        ];

        for (input, expected, should_succeed) in test_cases {
            let result = Boolean::deserialize(input.as_slice());
            if should_succeed {
                assert_eq!(result.unwrap().value, expected);
            } else {
                assert!(result.is_err());
            }
        }
    }
}
