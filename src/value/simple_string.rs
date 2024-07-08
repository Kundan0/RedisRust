use super::deserialize::{Deserialize, WithIndex};
use super::serialize::Serialize;
use crate::check_crlf;
use crate::constants::CRLF;
use crate::error::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleString(pub String);

impl Serialize for SimpleString {
    fn serialize(&self) -> String {
        format!("+{}{CRLF}", self.0)
    }
}
pub struct SimpleStringWithIndex {
    pub value: SimpleString,
    pub index: (usize, usize),
}
impl SimpleStringWithIndex {
    fn index(self) -> (usize, usize) {
        self.index
    }
}
impl WithIndex for SimpleStringWithIndex {
    fn get_index(self) -> (usize, usize) {
        self.index()
    }
}
impl Deserialize for SimpleString {
    type Value = SimpleStringWithIndex;
    fn deserialize(bytes: &[u8]) -> Result<Self::Value> {
        match bytes.get(0) {
            Some(b'+') => {
                let mut string: Vec<u8> = Vec::new();
                let mut i = 1 as usize;
                loop {
                    if check_crlf(bytes, i)? {
                        break;
                    }
                    if let Some(val) = bytes.get(i) {
                        string.push(*val);
                    } else {
                        return Result::Err(RedisError::SimpleStringParseError(
                            ParseError::InvalidFormat,
                        ));
                    }
                    i += 1;
                }
                let value = SimpleString(std::str::from_utf8(&string).unwrap().to_owned());
                // Start index is always 0
                //    i i+1
                //    | |
                // +Ok\r\n
                //
                let index = (0, i + 1);
                Result::Ok(SimpleStringWithIndex { value, index })
            }
            _ => Result::Err(RedisError::SimpleStringParseError(
                ParseError::InvalidFormat,
            )),
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
    fn test_serialize_simple_string() {
        let value = Value::SimpleString(SimpleString("OK".to_owned()));
        let expected = format!("+OK{CRLF}");
        serialize_and_assert(value, &expected);
    }

    #[test]
    fn test_deserialize_simple_string() {
        let test_cases = vec![
            // Valid test cases
            (b"+OK\r\n".to_vec(), "OK".to_string(), true),
            (b"+PONG\r\n".to_vec(), "PONG".to_string(), true),
            (
                b"+Hello, world!\r\n".to_vec(),
                "Hello, world!".to_string(),
                true,
            ),
            (b"+1234567890\r\n".to_vec(), "1234567890".to_string(), true),
            (
                b"+A simple test\r\n".to_vec(),
                "A simple test".to_string(),
                true,
            ),
            (b"+\r\n".to_vec(), "".to_string(), true),
            (
                b"+This is a test.\r\n".to_vec(),
                "This is a test.".to_string(),
                true,
            ),
            // Cases that should return Err
            // Missing CRLF
            (b"+Hello\n".to_vec(), "Hello".to_string(), false),
            // Incomplete simple string
            (b"+Hello".to_vec(), "Hello".to_string(), false),
            (b"+".to_vec(), "".to_string(), false),
            (b"+H".to_vec(), "H".to_string(), false),
        ];

        for (input, expected, should_succeed) in test_cases {
            let result = SimpleString::deserialize(input.as_slice());
            if should_succeed {
                assert_eq!(result.unwrap().value.0, expected);
            } else {
                assert!(result.is_err());
            }
        }
    }
}
