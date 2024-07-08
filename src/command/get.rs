use crate::command::Execute;
use crate::storage::get;
use crate::value::bulk_string::BulkString;
use crate::value::nulls::Nulls;
use crate::value::serialize::Serialize;
use crate::value::simple_error::{ErrorType, SimpleError};
use crate::value::Value;
use core::panic;
pub struct GetCommand;
impl Execute for GetCommand {
    fn execute(self, options: Vec<BulkString>) -> Box<dyn Serialize> {
        if options.len() != 1 {
            return Box::new(Value::SimpleError(SimpleError {
                error_type: ErrorType::try_from("ERR").unwrap(),
                message: String::from("wrong number of arguments for 'echo' command"),
            }));
        }
        if let Some(val) = options.get(0).cloned() {
            if let Ok(va) = get(val.0) {
                Box::new(Value::BulkString(BulkString(va)))
            } else {
                Box::new(Value::Nulls(Nulls))
            }
        } else {
            panic!("invalid arguments");
        }
    }
}
