mod del;
mod echo;
mod get;
mod ping;
mod set;

use crate::error::*;
use crate::{
    error::RedisError,
    value::{bulk_string::BulkString, serialize::Serialize},
};
use del::DelCommand;
use echo::EchoCommand;
use get::GetCommand;
use ping::PingCommand;
use set::SetCommand;
pub enum Command {
    ECHO(EchoCommand),
    PING(PingCommand),
    GET(GetCommand),
    SET(SetCommand),
    DEL(DelCommand),
}
impl TryFrom<BulkString> for Command {
    type Error = RedisError;
    fn try_from(s: BulkString) -> Result<Self> {
        match s {
            BulkString(x) if x.to_lowercase() == String::from("echo") => {
                Ok(Self::ECHO(EchoCommand))
            }
            BulkString(x) if x.to_lowercase() == String::from("ping") => {
                Ok(Self::PING(PingCommand))
            }
            BulkString(x) if x.to_lowercase() == String::from("get") => Ok(Self::GET(GetCommand)),
            BulkString(x) if x.to_lowercase() == String::from("set") => Ok(Self::SET(SetCommand)),
            BulkString(x) if x.to_lowercase() == String::from("del") => Ok(Self::DEL(DelCommand)),
            _ => Err(RedisError::UnknownCommand),
        }
    }
}
pub trait Execute {
    fn execute(self, options: Vec<BulkString>) -> Box<dyn Serialize>;
}

impl Execute for Command {
    fn execute(self, options: Vec<BulkString>) -> Box<dyn Serialize> {
        match self {
            Self::ECHO(echo_command) => echo_command.execute(options),
            Self::PING(ping_command) => ping_command.execute(options),
            Self::GET(get_command) => get_command.execute(options),
            Self::SET(set_command) => set_command.execute(options),
            Self::DEL(del_command) => del_command.execute(options),
        }
    }
}
