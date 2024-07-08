use crate::command::Execute;
use crate::value::bulk_string::BulkString;
use crate::value::serialize::Serialize;
use crate::value::simple_error::{ErrorType, SimpleError};
use crate::value::simple_string::SimpleString;
use crate::value::Value;
pub struct PingCommand;
impl Execute for PingCommand {
    fn execute(self, options: Vec<BulkString>) -> Box<dyn Serialize> {
        let options_count = options.len();
        if options_count > 1 {
            return Box::new(Value::SimpleError(SimpleError {
                error_type: ErrorType::try_from("ERR").unwrap(),
                message: String::from("wrong number of arguments for 'ping' command"),
            }));
        } else if options_count == 1 {
            if let Some(val) = options.get(0).cloned() {
                Box::new(Value::BulkString(val))
            } else {
                unreachable!();
            }
        } else {
            Box::new(Value::SimpleString(SimpleString(String::from("PONG"))))
        }
    }
}
