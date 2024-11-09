#![allow(unused_imports)]
use tokio::net::TcpListener;
mod connection;
mod error;
mod http;
use connection::Connection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:4221").await?;

    loop {
        let connection = Connection::new(listener.accept().await?);
        tokio::spawn(async move { connection.handle().await });
    }
}
