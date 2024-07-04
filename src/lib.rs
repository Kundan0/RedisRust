use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
pub mod constants;
mod error;
use crate::constants::CRLF;
mod value;
use crate::error::{RedisError, Result};
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

pub async fn handle_connection(mut stream: TcpStream) {
    println!("Handling new connection");
    loop {
        let mut buf = vec![0; 512];
        let bytes_read = stream.read(&mut buf).await.unwrap();
        if bytes_read == 0 {
            break;
        }
        println!("Got: {:?}\n\nTotal bytes: {bytes_read}", buf);
        let _ = stream.write_all(format!("+PONG{}", CRLF).as_bytes()).await;
    }
}
