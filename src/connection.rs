#![allow(clippy::never_loop)]
use std::{
    env::args,
    net::SocketAddr,
    path::PathBuf,
    pin::Pin,
    str::FromStr,
    task::{Context, Poll},
};
use tokio::io::AsyncWriteExt;
use tokio::io::{self, AsyncRead};
use tokio::{fs::File, io::AsyncReadExt};

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
        println!("Accepted new connection {}", self.addr);
        loop {
            let mut buf = Vec::with_capacity(1024);
            let n = self.read_buf(&mut buf).await?;
            let request = HttpRequest::decode(&buf[..n]).unwrap();
            let response = match request.url {
                "/" => HttpResponse::new_ok(),
                "/user-agent" => HttpResponse::new_ok().with_body(
                    request
                        .headers
                        .get("User-Agent")
                        .unwrap()
                        .to_string()
                        .as_bytes()
                        .to_vec(),
                ),
                url if url.starts_with("/files/") && request.method == HttpMethod::Get => {
                    let _ = args().nth(1).expect("To have passed --directory flag");
                    let path = args().nth(2).expect("To have passed directory");
                    let mut files = std::fs::read_dir(path.clone()).unwrap();
                    let file = files.find(|entry| {
                        entry.as_ref().unwrap().path()
                            == PathBuf::from_str(&path).unwrap().join(&url[7..])
                    });
                    if let Some(Ok(file)) = file {
                        let mut file = File::open(file.path()).await.unwrap();
                        let mut buf = vec![];
                        file.read_buf(&mut buf).await.unwrap();
                        HttpResponse::new_ok()
                            .with_body(buf)
                            .set_header("Content-Type", "application/octet-stream")
                    } else {
                        HttpResponse::new_not_found()
                    }
                }

                url if url.starts_with("/files/") && request.method == HttpMethod::Post => {
                    let _ = args().nth(1).expect("To have passed --directory flag");
                    let path = args().nth(2).expect("To have passed directory");
                    let mut file =
                        File::create(PathBuf::from_str(&path).unwrap().join(&url[7..])).await?;
                    let buf = request.body.unwrap();
                    file.write_all(buf).await?;
                    HttpResponse::new_created()
                }
                url if url.starts_with("/echo/") => {
                    HttpResponse::new_ok().with_body(url[6..].as_bytes().to_vec())
                }
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
