use super::deserialize::{Deserialize, WithIndex};
use super::serialize::Serialize;
use crate::check_crlf;
use crate::constants::CRLF;
use crate::error::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorType {
    ERR,
    WRONGTYPE,
}
impl ToString for ErrorType {
    fn to_string(&self) -> String {
        match self {
            ErrorType::ERR => "ERR".to_string(),
            ErrorType::WRONGTYPE => "WRONGTYPE".to_string(),
        }
    }
}

impl TryFrom<&str> for ErrorType {
    type Error = RedisError;
    fn try_from(value: &str) -> Result<Self> {
        match value.to_uppercase().as_str() {
            "ERR" => Result::Ok(ErrorType::ERR),
            "WRONGTYPE" => Result::Ok(ErrorType::WRONGTYPE),
            _ => Err(RedisError::SimpleErrorParseError(ParseError::UnknownValue)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleError {
    pub message: String,
    pub error_type: ErrorType,
}

impl Serialize for SimpleError {
    fn serialize(&self) -> String {
        format!("-{:?} {}{CRLF}", self.error_type, self.message)
    }
}
pub struct SimpleErrorWithIndex {
    pub value: SimpleError,
    pub index: (usize, usize),
}
impl SimpleErrorWithIndex {
    fn index(self) -> (usize, usize) {
        self.index
    }
}
impl WithIndex for SimpleErrorWithIndex {
    fn get_index(self) -> (usize, usize) {
        self.index()
    }
}
impl Deserialize for SimpleError {
    type Value = SimpleErrorWithIndex;
    fn deserialize(bytes: &[u8]) -> Result<Self::Value> {
        match bytes.get(0) {
            Some(b'-') => {
                let mut string: Vec<u8> = Vec::new();
                let mut i = 1 as usize;
                loop {
                    if check_crlf(bytes, i)? {
                        break;
                    }
                    if let Some(val) = bytes.get(i) {
                        string.push(*val);
                    } else {
                        return Result::Err(RedisError::SimpleErrorParseError(
                            ParseError::InvalidFormat,
                        ));
                    }
                    i += 1;
                }
                let value = std::str::from_utf8(&string)
                    .unwrap()
                    .split_once(|c| c == ' ' || c == '\n');
                if value.is_none() {
                    return Result::Err(RedisError::SimpleErrorParseError(
                        ParseError::InvalidFormat,
                    ));
                }
                let value = value.unwrap();
                let value = SimpleError {
                    error_type: ErrorType::try_from(value.0)?,
                    message: value.1.to_string(),
                };
                // Start index is always 0
                //    i i+1
                //    | |
                // -Err\r\n
                //
                let index = (0, i + 1);
                Result::Ok(SimpleErrorWithIndex { value, index })
            }
            _ => Result::Err(RedisError::SimpleErrorParseError(ParseError::InvalidFormat)),
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
    fn test_serialize_simple_error() {
        let value = Value::SimpleError(SimpleError {
            error_type: ErrorType::ERR,
            message: "Unknown Command".to_owned(),
        });
        let expected = format!("-ERR Unknown Command{CRLF}");
        serialize_and_assert(value, &expected);

        let value = Value::SimpleError(SimpleError {
            error_type: ErrorType::WRONGTYPE,
            message: "Operation against a key holding the wrong kind of value".to_owned(),
        });
        let expected =
            format!("-WRONGTYPE Operation against a key holding the wrong kind of value{CRLF}");
        serialize_and_assert(value, &expected);

        let value = Value::SimpleError(SimpleError {
            error_type: ErrorType::ERR,
            message: "Some other error".to_owned(),
        });
        let expected = format!("-ERR Some other error{CRLF}");
        serialize_and_assert(value, &expected);
    }

    #[test]
    fn test_deserialize_simple_error() {
        let test_cases = vec![
            // Valid test cases
            (
                b"-ERR message\r\n".to_vec(),
                SimpleError {
                    error_type: ErrorType::ERR,
                    message: "message".to_string(),
                },
                true,
            ),
            (
                b"-ERR unknown command 'foobar'\r\n".to_vec(),
                SimpleError {
                    error_type: ErrorType::ERR,
                    message: "unknown command 'foobar'".to_string(),
                },
                true,
            ),
            (
                b"-WRONGTYPE Operation against a key holding the wrong kind of value\r\n".to_vec(),
                SimpleError {
                    error_type: ErrorType::WRONGTYPE,
                    message: "Operation against a key holding the wrong kind of value".to_string(),
                },
                true,
            ),
            (
                b"-ERR syntax error\r\n".to_vec(),
                SimpleError {
                    error_type: ErrorType::ERR,
                    message: "syntax error".to_string(),
                },
                true,
            ),
            (
                b"-ERR unknown command\r\n".to_vec(),
                SimpleError {
                    error_type: ErrorType::ERR,
                    message: "unknown command".to_string(),
                },
                true,
            ),
            (
                b"-ERR no such key\r\n".to_vec(),
                SimpleError {
                    error_type: ErrorType::ERR,
                    message: "no such key".to_string(),
                },
                true,
            ),
            (
                b"-ERR wrong number of arguments\r\n".to_vec(),
                SimpleError {
                    error_type: ErrorType::ERR,
                    message: "wrong number of arguments".to_string(),
                },
                true,
            ),
            (
                b"-ERR value is not an integer or out of range\r\n".to_vec(),
                SimpleError {
                    error_type: ErrorType::ERR,
                    message: "value is not an integer or out of range".to_string(),
                },
                true,
            ),
            // Unknown error types
            (
                b"-UNKNOWN error type\r\n".to_vec(),
                SimpleError {
                    error_type: ErrorType::ERR,
                    message: "error type".to_string(),
                },
                false,
            ),
            (
                b"-SOMEERR some unknown error\r\n".to_vec(),
                SimpleError {
                    error_type: ErrorType::ERR,
                    message: "some unknown error".to_string(),
                },
                false,
            ),
            // Cases that should return Err
            // Missing CRLF
            (
                b"-Error message\n".to_vec(),
                SimpleError {
                    error_type: ErrorType::ERR,
                    message: "Error message".to_string(),
                },
                false,
            ),
            // Incomplete simple error
            (
                b"-Error message".to_vec(),
                SimpleError {
                    error_type: ErrorType::ERR,
                    message: "Error message".to_string(),
                },
                false,
            ),
            (
                b"-".to_vec(),
                SimpleError {
                    error_type: ErrorType::ERR,
                    message: "".to_string(),
                },
                false,
            ),
            (
                b"-E".to_vec(),
                SimpleError {
                    error_type: ErrorType::ERR,
                    message: "E".to_string(),
                },
                false,
            ),
        ];

        for (input, expected, should_succeed) in test_cases {
            let result = SimpleError::deserialize(input.as_slice());
            if should_succeed {
                assert_eq!(result.unwrap().value, expected);
            } else {
                assert!(result.is_err());
            }
        }
    }
}
