use crate::command::Execute;
use crate::value::bulk_string::BulkString;
use crate::value::serialize::Serialize;
use crate::value::simple_error::{ErrorType, SimpleError};
use crate::value::Value;
use core::panic;
pub struct EchoCommand;
impl Execute for EchoCommand {
    fn execute(self, options: Vec<BulkString>) -> Box<dyn Serialize> {
        if options.len() != 1 {
            return Box::new(Value::SimpleError(SimpleError {
                error_type: ErrorType::try_from("ERR").unwrap(),
                message: String::from("wrong number of arguments for 'echo' command"),
            }));
        }
        if let Some(val) = options.get(0).cloned() {
            Box::new(Value::BulkString(val))
        } else {
            panic!("invalid arguments");
        }
    }
}
