#![allow(clippy::never_loop)]
use std::{
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::{self, AsyncRead};

use tokio::io::AsyncWrite;

use crate::http::*;
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct Connection {
    pub tcp: TcpStream,
    pub addr: SocketAddr,
}

impl Connection {
    pub fn new((tcp, addr): (TcpStream, SocketAddr)) -> Self {
        Self { tcp, addr }
    }

    pub async fn handle(mut self) -> Result<(), tokio::io::Error> {
        loop {
            let mut buf = Vec::with_capacity(1024);
            let n = self.read_buf(&mut buf).await?;
            let request = HttpRequest::decode(&buf[..n]).unwrap();
            let response = match request.url {
                b"/" => HttpResponse::new_ok(),
                _ => HttpResponse::new_not_found(),
            };
            self.write_all(&response.encode()).await?;
            break;
        }
        Ok(())
    }
}

impl AsyncWrite for Connection {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let tcp = Pin::new(&mut self.tcp);
        TcpStream::poll_write(tcp, cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let tcp = Pin::new(&mut self.tcp);
        TcpStream::poll_flush(tcp, cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        let tcp = Pin::new(&mut self.tcp);
        TcpStream::poll_shutdown(tcp, cx)
    }
}

impl AsyncRead for Connection {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let tcp = Pin::new(&mut self.tcp);
        TcpStream::poll_read(tcp, cx, buf)
    }
}
