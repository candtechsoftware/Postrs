
use bytes::Bytes;
use http_body_util::{Empty, BodyExt};
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::*;
use tokio::net::TcpStream;
use std::str::FromStr;
use std::convert::TryFrom;
use hyper::http::Method;

pub enum RestMethod {
    GET,
    POST,
    DELETE,
    PATCH,
}

#[derive(Debug, )]
pub struct RestClientError;

impl std::fmt::Display for RestClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error in rest client")
    }
} 

impl RestClientError {
    fn new() -> Self {
        Self {}
    }
}

impl TryFrom<RestMethod> for Method {
    type Error = hyper::http::Error;

    fn try_from(value: RestMethod) -> std::result::Result<Self, Self::Error> {
        match value {
            RestMethod::GET => Ok(Method::GET),
            RestMethod::POST => Ok(Method::POST) ,
            RestMethod::DELETE => Ok(Method::DELETE),
            RestMethod::PATCH => Ok(Method::PATCH),
        }
    }
}
impl FromStr for RestMethod {
    type Err = RestClientError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "GET" => Ok(RestMethod::GET),
            "POST" => Ok(RestMethod::POST),
            "DELETE" => Ok(RestMethod::DELETE),
            "PATCH" => Ok(RestMethod::PATCH),
            _ => {
                eprintln!("[REST REQUEST]: Error: Invalid or unsuppored method {} ", s);
                Err(RestClientError::new())
            }
        }
    }
}

pin_project! {
    pub struct TokioIo<T> {
        #[pin]
        inner: T,
    }
}

impl<T> TokioIo<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn inner(self) -> T {
        self.inner
    }
}

impl<T> hyper::rt::Read for TokioIo<T>
where
    T: tokio::io::AsyncRead,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        let n = unsafe {
            let mut buffer = tokio::io::ReadBuf::uninit(buf.as_mut());
            match tokio::io::AsyncRead::poll_read(self.project().inner, cx, &mut buffer) {
                Poll::Ready(Ok(())) => buffer.filled().len(),
                other => return other,
            }
        };
        unsafe {
            buf.advance(n);
        }
        Poll::Ready(Ok(()))
    }
}

impl<T> hyper::rt::Write for TokioIo<T>
where
    T: tokio::io::AsyncWrite,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        tokio::io::AsyncWrite::poll_write(self.project().inner, cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::result::Result<(), std::io::Error>> {
        tokio::io::AsyncWrite::poll_flush(self.project().inner, cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        tokio::io::AsyncWrite::poll_shutdown(self.project().inner, cx)
    }

    fn is_write_vectored(&self) -> bool {
        tokio::io::AsyncWrite::is_write_vectored(&self.inner)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[std::io::IoSlice<'_>],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        tokio::io::AsyncWrite::poll_write_vectored(self.project().inner, cx, bufs)
    }
}

impl<T> tokio::io::AsyncRead for TokioIo<T>
where
    T: hyper::rt::Read,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let filled = buf.filled().len();
        let sub_filled = unsafe {
            let mut buffer = hyper::rt::ReadBuf::uninit(buf.unfilled_mut());

            match hyper::rt::Read::poll_read(self.project().inner, cx, buffer.unfilled()) {
                Poll::Ready(Ok(())) => buffer.filled().len(),
                other => return other,
            }
        };
        let n_filled = filled + sub_filled;
        let n_init = sub_filled;
        unsafe {
            buf.assume_init(n_init);
            buf.set_filled(n_filled);
        }
        Poll::Ready(Ok(()))
    }
}

impl<T> tokio::io::AsyncWrite for TokioIo<T>
where
    T: hyper::rt::Write,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        hyper::rt::Write::poll_write(self.project().inner, cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::result::Result<(), std::io::Error>> {
        hyper::rt::Write::poll_flush(self.project().inner, cx)
    }
    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        hyper::rt::Write::poll_shutdown(self.project().inner, cx)
    }

    fn is_write_vectored(&self) -> bool {
        hyper::rt::Write::is_write_vectored(&self.inner)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[std::io::IoSlice<'_>],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        hyper::rt::Write::poll_write_vectored(self.project().inner, cx, bufs)
    }
}
type RequestResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command


pub async fn make_request(full_url: &str, method_str: &str) ->  RequestResult<String>{
    let url = full_url.parse::<hyper::Uri>().unwrap();
    let host = url.host().expect("uri has not host");
    let port = url.port_u16().unwrap_or(80);
    let addr = format!("{}:{}", host, port);
    let stream = TcpStream::connect(addr).await?;
    let io = TokioIo::new(stream);
    let method = RestMethod::from_str(method_str).expect("Should send valid request method");

    match method {
        RestMethod::GET => { println!("Method is GET"); },
        RestMethod::POST => { println!("Method is POST"); },
        RestMethod::DELETE => { println!("Method is DELETE"); },
        RestMethod::PATCH => { println!("Method is PATCH"); },
    }
 
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection Failed: {:?}", err);
        }
    });

    let authority = url.authority().unwrap().clone();

    let req = hyper::Request::builder()
        .uri(url)
        .method(method)
        .header(hyper::header::HOST, authority.as_str())
        .body(Empty::<Bytes>::new())?;
    
    let mut res = sender.send_request(req).await?;
    println!("Response: {}", res.status());
    println!("Headers: {:#?}\n", res.headers());
    let mut byte_arr = Vec::new();
    while let Some(next) = res.frame().await {
        let frame = next?;
        if let Some(chunk) = frame.data_ref() {
           for i in &mut chunk.into_iter() {
                byte_arr.push(*i);
           }
           
           tokio::io::stdout().write_all(chunk).await?;
        }
    }
    println!("\n\nDone!");
    Ok(String::from_utf8(byte_arr).unwrap())
}