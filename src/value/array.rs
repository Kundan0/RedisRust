use super::deserialize::{Deserialize, WithIndex};
use super::serialize::Serialize;
use crate::constants::CRLF;
use crate::error::*;
use crate::read_until_crlf;
use crate::value::Value;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Array(pub Vec<Value>);

impl Serialize for Array {
    fn serialize(self) -> String {
        let mut serialized_array = format!("*{}{CRLF}", self.0.len());
        for value in self.0 {
            serialized_array = format!("{serialized_array}{}", value.serialize())
        }
        serialized_array
    }
}
pub struct ArrayWithIndex {
    pub value: Array,
    pub index: (usize, usize),
}
impl ArrayWithIndex {
    fn index(self) -> (usize, usize) {
        self.index
    }
}
impl WithIndex for ArrayWithIndex {
    fn get_index(self) -> (usize, usize) {
        self.index()
    }
}

impl Deserialize for Array {
    type Value = ArrayWithIndex;
    fn deserialize(bytes: &[u8]) -> Result<Self::Value> {
        match bytes.get(0) {
            Some(b'*') => {
                let first_crlf_index = read_until_crlf(&bytes[..], 1)?;
                dbg!(first_crlf_index);
                let count_int = std::str::from_utf8(&bytes[1..first_crlf_index])?
                    .to_owned()
                    .parse::<usize>()?;
                dbg!(count_int);
                let mut value_vec: Vec<Value> = Vec::new();
                let mut upto_index = first_crlf_index + 1;
                for _i in 0..count_int {
                    dbg!(_i);
                    if _i == 0 {
                        dbg!(std::str::from_utf8(&bytes[upto_index + 1..]).unwrap());
                    }
                    let value_with_index = Value::deserialize(&bytes[upto_index + 1..])?;
                    dbg!(&value_with_index.value);
                    value_vec.push(value_with_index.value);
                    upto_index = upto_index + (value_with_index.index.1 + 1);
                    dbg!(upto_index);
                }
                Result::Ok(ArrayWithIndex {
                    value: Array(value_vec),
                    index: (0, upto_index),
                })
            }
            _ => Result::Err(RedisError::CommandParseError(ParseError::InvalidFormat)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::CRLF;
    use crate::value::{
        array::Array,
        boolean::Boolean,
        bulk_string::BulkString,
        integer::Integer,
        serialize::Serialize,
        simple_error::{ErrorType, SimpleError},
        simple_string::SimpleString,
        Value,
    };
    fn serialize_and_assert(value: Value, expected: &str) {
        assert_eq!(expected, value.serialize());
    }

    #[test]
    fn test_serialize_array() {
        let value1 = Value::SimpleString(SimpleString("OK".to_owned()));
        let value2 = Value::SimpleError(SimpleError {
            error_type: ErrorType::ERR,
            message: "Unknown Command".to_owned(),
        });
        let value3 = Value::Integer(Integer(56_i64));
        let value4 = Value::BulkString(BulkString("bulk string".to_owned()));
        let value5 = Value::Boolean(Boolean(true));

        let nested_array1 = Value::Array(Array(vec![value1.clone(), value2.clone()]));
        let nested_array2 =
            Value::Array(Array(vec![value3.clone(), value4.clone(), value5.clone()]));

        let outer_array = Value::Array(Array(vec![nested_array1.clone(), nested_array2.clone()]));

        let expected1 = format!("+OK{CRLF}");
        let expected2 = format!("-ERR Unknown Command{CRLF}");
        let expected3 = format!(":56{CRLF}");
        let expected4 = format!("${}{CRLF}bulk string{CRLF}", "bulk string".len());
        let expected5 = format!("#t{CRLF}");

        let expected_nested_array1 = format!("*2{CRLF}{}{}", expected1, expected2);
        let expected_nested_array2 = format!("*3{CRLF}{}{}{}", expected3, expected4, expected5);
        let expected_outer_array = format!(
            "*2{CRLF}{}{}",
            expected_nested_array1, expected_nested_array2
        );

        serialize_and_assert(outer_array, &expected_outer_array);
    }

    fn deserialize_and_assert(bytes: &[u8], expected: Option<Value>, should_succeed: bool) {
        let result = Value::deserialize(bytes);
        if should_succeed {
            assert!(result.is_ok());
            assert_eq!(result.unwrap().value, expected.unwrap());
        } else {
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_deserialize_array() {
        let test_cases = vec![
            // Valid test cases
            (
                b"*2\r\n+OK\r\n-ERR unknown command\r\n".to_vec(),
                Some(Value::Array(Array(vec![
                    Value::SimpleString(SimpleString("OK".to_owned())),
                    Value::SimpleError(SimpleError {
                        error_type: ErrorType::ERR,
                        message: "unknown command".to_owned(),
                    }),
                ]))),
                true,
            ),
            (
                b"*3\r\n:1000\r\n$6\r\nfoobar\r\n#t\r\n".to_vec(),
                Some(Value::Array(Array(vec![
                    Value::Integer(Integer(1000)),
                    Value::BulkString(BulkString("foobar".to_owned())),
                    Value::Boolean(Boolean(true)),
                ]))),
                true,
            ),
            (
                b"*2\r\n*1\r\n:123\r\n*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n".to_vec(),
                Some(Value::Array(Array(vec![
                    Value::Array(Array(vec![Value::Integer(Integer(123))])),
                    Value::Array(Array(vec![
                        Value::BulkString(BulkString("foo".to_owned())),
                        Value::BulkString(BulkString("bar".to_owned())),
                    ])),
                ]))),
                true,
            ),
            // Failing test cases
            // Unknown type
            (b"*1\r\n~unknown\r\n".to_vec(), None, false),
            // Incomplete Array header
            (b"*2\r\n+OK\r\n".to_vec(), None, false),
            // Incomplete nested Array
            (b"*2\r\n*1\r\n:123".to_vec(), None, false),
        ];

        for (input, expected, should_succeed) in test_cases {
            deserialize_and_assert(&input, expected, should_succeed);
        }
    }
}
