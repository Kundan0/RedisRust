mod echo;

use crate::error::*;
use crate::{
    error::RedisError,
    value::{bulk_string::BulkString, serialize::Serialize},
};
use echo::EchoCommand;
pub enum Command {
    ECHO(EchoCommand),
    PING,
    GET,
    SET,
}
impl TryFrom<BulkString> for Command {
    type Error = RedisError;
    fn try_from(s: BulkString) -> Result<Self> {
        match s {
            BulkString(x) if x.to_lowercase() == String::from("echo") => {
                Ok(Self::ECHO(EchoCommand))
            }
            BulkString(x) if x.to_lowercase() == String::from("ping") => Ok(Self::PING),
            BulkString(x) if x.to_lowercase() == String::from("get") => Ok(Self::GET),
            BulkString(x) if x.to_lowercase() == String::from("set") => Ok(Self::SET),
            _ => Err(RedisError::UnknownCommand),
        }
    }
}
pub trait Execute {
    fn execute(self, options: Vec<BulkString>) -> impl Serialize;
}

impl Execute for Command {
    fn execute(self, options: Vec<BulkString>) -> impl Serialize {
        match self {
            Self::ECHO(x) => x.execute(options),
            _ => todo!(),
        }
    }
}
