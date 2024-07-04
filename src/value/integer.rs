use super::deserialize::{Deserialize, WithIndex};
use super::serialize::Serialize;
use crate::check_crlf;
use crate::constants::CRLF;
use crate::error::*;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Integer(pub i64);

impl Serialize for Integer {
    fn serialize(self) -> String {
        format!(":{}{CRLF}", self.0)
    }
}
#[derive(Debug)]
pub struct IntegerWithIndex {
    pub value: Integer,
    pub index: (usize, usize),
}
impl IntegerWithIndex {
    fn index(self) -> (usize, usize) {
        self.index
    }
}
impl WithIndex for IntegerWithIndex {
    fn get_index(self) -> (usize, usize) {
        self.index()
    }
}
impl Deserialize for Integer {
    type Value = IntegerWithIndex;
    fn deserialize(bytes: &[u8]) -> Result<Self::Value> {
        match bytes.get(0) {
            Some(b':') => {
                let mut string: Vec<u8> = Vec::new();
                let mut i = 1 as usize;
                loop {
                    if check_crlf(bytes, i)? {
                        break;
                    }
                    if let Some(val) = bytes.get(i) {
                        string.push(*val);
                    } else {
                        return Result::Err(RedisError::IntegerParseError(
                            ParseError::InvalidFormat,
                        ));
                    }
                    i += 1;
                }
                let value = Integer(std::str::from_utf8(&string)?.parse()?);
                // Start index is always 0
                //    i i+1
                //    | |
                // :123\r\n
                //
                let index = (0, i + 1);
                Result::Ok(IntegerWithIndex { value, index })
            }
            _ => Result::Err(RedisError::IntegerParseError(ParseError::InvalidFormat)),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::CRLF;
    use crate::value::Value;

    fn serialize_and_assert(value: Value, expected: &str) {
        assert_eq!(expected, Value::serialize(value));
    }

    #[test]
    fn test_serialize_integer() {
        let value = Value::Integer(Integer(56_i64));
        let expected = format!(":56{CRLF}");
        serialize_and_assert(value, &expected);
    }

    #[test]
    fn test_deserialize_integer() {
        let test_cases = vec![
            // Valid test cases
            (b":123\r\n".to_vec(), Integer(123), true),
            (b":0\r\n".to_vec(), Integer(0), true),
            (b":-456\r\n".to_vec(), Integer(-456), true),
            // Cases that should return Err
            // Missing CRLF
            (b":789\n".to_vec(), Integer(789), false),
            // Incomplete integer
            (b":789".to_vec(), Integer(789), false),
            (b":\r\n".to_vec(), Integer(0), false),
            (b":".to_vec(), Integer(0), false),
            (b":abc\r\n".to_vec(), Integer(0), false),
        ];

        for (input, expected, should_succeed) in test_cases {
            let result = Integer::deserialize(input.as_slice());
            if should_succeed {
                assert_eq!(result.unwrap().value, expected);
            } else {
                assert!(result.is_err());
            }
        }
    }
}
