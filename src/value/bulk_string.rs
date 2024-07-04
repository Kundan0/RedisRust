use super::serialize::Serialize;
use crate::check_crlf;
use crate::constants::CRLF;
use crate::error::*;
use crate::read_until_crlf;
use crate::value::deserialize::{Deserialize, WithIndex};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BulkString(pub String);
impl Serialize for BulkString {
    fn serialize(self) -> String {
        format!("${}{CRLF}{}{CRLF}", self.0.len(), self.0)
    }
}
pub struct BulkStringWithIndex {
    pub value: BulkString,
    pub index: (usize, usize),
}
impl BulkStringWithIndex {
    fn index(self) -> (usize, usize) {
        self.index
    }
}
impl WithIndex for BulkStringWithIndex {
    fn get_index(self) -> (usize, usize) {
        self.index()
    }
}
impl Deserialize for BulkString {
    type Value = BulkStringWithIndex;
    fn deserialize(bytes: &[u8]) -> Result<Self::Value> {
        match bytes.get(0) {
            Some(b'$') => {
                let first_crlf_index = read_until_crlf(&bytes[..], 1)?;
                let count_int = std::str::from_utf8(&bytes[1..first_crlf_index])?
                    .to_owned()
                    .parse::<usize>()?;
                let mut string: Vec<u8> = Vec::new();
                for i in 0..count_int {
                    if let Some(val) = bytes.get(2 + first_crlf_index + i) {
                        string.push(*val);
                    } else {
                        return Result::Err(RedisError::BulkStringParseError(
                            ParseError::InvalidFormat,
                        ));
                    }
                }
                if !check_crlf(bytes, count_int + first_crlf_index + 2)? {
                    return Result::Err(RedisError::BulkStringParseError(
                        ParseError::InvalidFormat,
                    ));
                }
                // Currently the Bulk String holds String instead of Vec<u8>. Bulkstring is
                // serialized to String. This needs to be changed.
                let value = BulkString(std::str::from_utf8(&string).unwrap().to_owned());
                // Start index is always 0
                //   first_crlf_index
                //   |  +1    +count_int
                //   |  |     |  +2
                //   |  |     |  |
                // $5\r\nhello\r\n
                //
                let index = (0, first_crlf_index + 1 + count_int + 2);
                Result::Ok(BulkStringWithIndex { value, index })
            }
            _ => Result::Err(RedisError::BulkStringParseError(ParseError::InvalidFormat)),
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
    fn test_serialize_bulk_string() {
        let value = Value::BulkString(BulkString("bulk string".to_owned()));
        let expected = format!("$11{CRLF}bulk string{CRLF}");
        serialize_and_assert(value, &expected);
    }

    #[test]
    fn test_deseriaize_bulk_string() {
        let test_cases = vec![
            // Valid test cases
            (
                b"$11\r\nbulk string\r\n".to_vec(),
                "bulk string".to_string(),
                true,
            ),
            (
                b"$12\r\nbulk\r\nstring\r\n".to_vec(),
                "bulk\r\nstring".to_string(),
                true,
            ),
            (
                b"$12\r\nbulk\r\nstring\r\n".to_vec(),
                "bulk\r\nstring".to_string(),
                true,
            ),
            (
                b"$12\r\nhello\r\nworld\r\n".to_vec(),
                "hello\r\nworld".to_string(),
                true,
            ),
            (
                b"$13\r\na simple test\r\n".to_vec(),
                "a simple test".to_string(),
                true,
            ),
            (
                b"$10\r\n1234567890\r\n".to_vec(),
                "1234567890".to_string(),
                true,
            ),
            (
                b"$10\r\n1234567890\r\n\r\n".to_vec(),
                "1234567890".to_string(),
                true,
            ),
            (b"$0\r\n\r\n".to_vec(), "".to_string(), true),
            (b"$0\r\n\r\n\r\n".to_vec(), "".to_string(), true),
            // Cases that should return Err
            // Mismatched length
            (
                b"$5\r\nlonger string\r\n".to_vec(),
                "longer string".to_string(),
                false,
            ),
            // Missing CRLF
            (b"$5\r\nhello\n".to_vec(), "hello".to_string(), false),
            // Invalid length
            (b"$-5\r\nhello\r\n".to_vec(), "".to_string(), false),
            // Non-numeric length
            (b"$abc\r\nhello\r\n".to_vec(), "".to_string(), false),
            // Incomplete bulk string
            (b"$5\r\nhel".to_vec(), "hel".to_string(), false),
            (b"$".to_vec(), "".to_string(), false),
            (b"$4".to_vec(), "".to_string(), false),
            (b"$4\r".to_vec(), "".to_string(), false),
            (b"$1\n".to_vec(), "".to_string(), false),
            (b"$23\r\n".to_vec(), "".to_string(), false),
        ];

        for (input, expected, should_succeed) in test_cases {
            let result = BulkString::deserialize(input.as_slice());
            if should_succeed {
                assert_eq!(result.unwrap().value.0, expected);
            } else {
                assert!(result.is_err());
            }
        }
    }
}
