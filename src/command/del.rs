use crate::command::Execute;
use crate::storage::delete;
use crate::value::bulk_string::BulkString;
use crate::value::integer::Integer;
use crate::value::serialize::Serialize;
use crate::value::Value;
pub struct DelCommand;
impl Execute for DelCommand {
    fn execute(self, options: Vec<BulkString>) -> Box<dyn Serialize> {
        let mut keys = Vec::new();
        for key in options {
            keys.push(key.0);
        }
        Box::new(Value::Integer(Integer(delete(keys) as i64)))
    }
}
