use crate::command::Execute;
use crate::value::bulk_string::BulkString;
use crate::value::serialize::Serialize;
use crate::value::simple_error::{ErrorType, SimpleError};
use crate::value::Value;
pub struct EchoCommand;
impl Execute for EchoCommand {
    fn execute(self, options: Vec<BulkString>) -> impl Serialize {
        if let Some(val) = options.get(0).cloned() {
            Value::BulkString(val)
        } else {
            Value::SimpleError(SimpleError {
                error_type: ErrorType::try_from("ERR").unwrap(),
                message: String::from("At least one option required"),
            })
        }
    }
}
