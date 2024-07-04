mod array;
mod boolean;
mod bulk_string;
mod deserialize;
mod integer;
mod serialize;
mod simple_error;
mod simple_string;
use array::Array;
use boolean::Boolean;
use bulk_string::BulkString;
use deserialize::{Deserialize, WithIndex};
use integer::Integer;
use serialize::Serialize;
use simple_error::SimpleError;
use simple_string::SimpleString;

use crate::error::{ParseError, RedisError};

use self::{
    array::ArrayWithIndex, boolean::BooleanWithIndex, bulk_string::BulkStringWithIndex,
    integer::IntegerWithIndex, simple_error::SimpleErrorWithIndex,
    simple_string::SimpleStringWithIndex,
};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    SimpleString(SimpleString),
    SimpleError(SimpleError),
    Integer(Integer),
    BulkString(BulkString),
    Array(Array),
    Boolean(Boolean),
}

pub struct ValueWithIndex {
    value: Value,
    index: (usize, usize),
}
impl ValueWithIndex {
    pub fn index(self) -> (usize, usize) {
        self.index
    }
}
impl From<SimpleStringWithIndex> for ValueWithIndex {
    fn from(item: SimpleStringWithIndex) -> Self {
        Self {
            value: Value::SimpleString(item.value),
            index: item.index,
        }
    }
}
impl From<SimpleErrorWithIndex> for ValueWithIndex {
    fn from(item: SimpleErrorWithIndex) -> Self {
        Self {
            value: Value::SimpleError(item.value),
            index: item.index,
        }
    }
}
impl From<IntegerWithIndex> for ValueWithIndex {
    fn from(item: IntegerWithIndex) -> Self {
        Self {
            value: Value::Integer(item.value),
            index: item.index,
        }
    }
}
impl From<BulkStringWithIndex> for ValueWithIndex {
    fn from(item: BulkStringWithIndex) -> Self {
        Self {
            value: Value::BulkString(item.value),
            index: item.index,
        }
    }
}
impl From<ArrayWithIndex> for ValueWithIndex {
    fn from(item: ArrayWithIndex) -> Self {
        Self {
            value: Value::Array(item.value),
            index: item.index,
        }
    }
}
impl From<BooleanWithIndex> for ValueWithIndex {
    fn from(item: BooleanWithIndex) -> Self {
        Self {
            value: Value::Boolean(item.value),
            index: item.index,
        }
    }
}
impl WithIndex for ValueWithIndex {
    fn get_index(self) -> (usize, usize) {
        self.index()
    }
}
impl Serialize for Value {
    fn serialize(self) -> String {
        match self {
            Value::SimpleString(simple_string) => simple_string.serialize(),
            Value::SimpleError(simple_error) => simple_error.serialize(),
            Value::Integer(integer) => integer.serialize(),
            Value::BulkString(bulk_string) => bulk_string.serialize(),
            Value::Array(array) => array.serialize(),
            Value::Boolean(boolean) => boolean.serialize(),
        }
    }
}
impl Deserialize for Value {
    type Value = ValueWithIndex;
    fn deserialize(bytes: &[u8]) -> crate::error::Result<Self::Value> {
        match bytes.get(0) {
            Some(b'+') => Ok(ValueWithIndex::from(SimpleString::deserialize(bytes)?)),
            Some(b'-') => Ok(ValueWithIndex::from(SimpleError::deserialize(bytes)?)),
            Some(b':') => Ok(ValueWithIndex::from(Integer::deserialize(bytes)?)),
            Some(b'$') => Ok(ValueWithIndex::from(BulkString::deserialize(bytes)?)),
            Some(b'*') => Ok(ValueWithIndex::from(Array::deserialize(bytes)?)),
            Some(b'#') => Ok(ValueWithIndex::from(Boolean::deserialize(bytes)?)),
            _ => Err(RedisError::ValueParseError(ParseError::InvalidFormat)),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::ErrorType;

    fn deserialize_and_assert(bytes: &[u8], expected: Value) {
        let result = Value::deserialize(bytes).unwrap().value;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deserialize_all_values() {
        let test_cases = vec![
            // SimpleString
            (
                b"+OK\r\n".to_vec(),
                Value::SimpleString(SimpleString("OK".to_owned())),
            ),
            // SimpleError
            (
                b"-ERR unknown command\r\n".to_vec(),
                Value::SimpleError(SimpleError {
                    error_type: ErrorType::ERR,
                    message: "unknown command".to_owned(),
                }),
            ),
            // Integer
            (b":1000\r\n".to_vec(), Value::Integer(Integer(1000))),
            // BulkString
            (
                b"$6\r\nfoobar\r\n".to_vec(),
                Value::BulkString(BulkString("foobar".to_owned())),
            ),
            // Boolean True
            (b"#t\r\n".to_vec(), Value::Boolean(Boolean(true))),
            // Boolean False
            (b"#f\r\n".to_vec(), Value::Boolean(Boolean(false))),
        ];

        for (input, expected) in test_cases {
            deserialize_and_assert(&input, expected);
        }
    }
}
