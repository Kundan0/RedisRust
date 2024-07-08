pub mod command;
pub mod constants;
mod error;
pub mod storage;
pub mod value;

use crate::command::Execute;
use crate::error::{RedisError, Result};
use crate::value::serialize::Serialize;
use command::Command;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use value::array::Array;
use value::bulk_string::BulkString;
use value::deserialize::Deserialize;
use value::simple_error::{ErrorType, SimpleError};
use value::Value;

fn read_until_crlf(item: &[u8], count: usize) -> Result<usize> {
    let mut local_count = 0;
    for index in 0..item.len() {
        if check_crlf(item, index)? {
            local_count += 1;
            if local_count == count {
                return Ok(index);
            } else {
                continue;
            }
        }
    }
    Err(RedisError::CRLFNotFoundError)
}

fn check_crlf(value: &[u8], index: usize) -> Result<bool> {
    match value.get(index) {
        Some(b'\r') => match value.get(index + 1) {
            Some(b'\n') => Ok(true),
            Some(_) => Ok(false),
            None => Err(RedisError::IndexOutOfBoundError),
        },
        Some(_) => Ok(false),
        None => Err(RedisError::IndexOutOfBoundError),
    }
}

fn execute(request: Array) -> String {
    let mut command: Option<Command> = None;
    let mut options: Vec<BulkString> = vec![];
    let mut requested_command: String = String::new();
    for (i, value) in request.0.iter().enumerate() {
        if let Value::BulkString(x) = value.clone() {
            if i == 0 {
                if let Ok(com) = Command::try_from(x.clone()) {
                    command = Some(com);
                } else {
                    requested_command = x.0;
                }
            } else {
                options.push(x);
            }
        }
    }
    if command.is_none() {
        let mut args_string = String::new();
        for arg in options.clone() {
            args_string.push_str(format!("'{}' ", arg.0).as_str())
        }
        return Value::SimpleError(SimpleError {
            error_type: ErrorType::ERR,
            message: format!(
                "unknown command '{}', with args beginning with: {}",
                requested_command, args_string
            ),
        })
        .serialize();
    }
    let response = command.unwrap().execute(options);
    response.serialize()
}

pub async fn handle_connection(mut stream: TcpStream) {
    println!("Handling new connection");
    loop {
        let mut buf = vec![0; 512];
        let bytes_read = stream.read(&mut buf).await.unwrap();
        if bytes_read == 0 {
            break;
        }
        let value = Value::deserialize(&buf[..]);
        match value {
            Err(e) => {
                dbg!(e);
            }
            Ok(val) => {
                if let Value::Array(arr) = val.value {
                    let _ = stream.write_all(execute(arr).as_bytes()).await;
                }
            }
        }
    }
}
