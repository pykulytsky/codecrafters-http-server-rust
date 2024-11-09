#![allow(clippy::never_loop)]
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
mod connection;
use connection::Connection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Logs from your program will appear here!");
    let listener = TcpListener::bind("127.0.0.1:4221").await?;

    loop {
        let mut connection = Connection::new(listener.accept().await?);
        let n = connection.write(b"HTTP/1.1 200 OK\r\n\r\n").await?;
        println!("Written {n} bytes to {}", connection.addr);
        break;
    }
    Ok(())
}
