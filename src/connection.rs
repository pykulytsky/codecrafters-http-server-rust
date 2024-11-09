use std::{
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io;

use tokio::io::AsyncWrite;

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
