use my_redis::constants::{IP, PORT};
use my_redis::handle_connection;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(format!("{IP}:{PORT}"))
        .await
        .expect(format!("Cannot bind to {IP}:{PORT}").as_str());
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            handle_connection(stream).await;
        });
    }
}
