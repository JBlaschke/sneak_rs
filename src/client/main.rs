use http_body_util::{BodyExt, Empty, Full};
use hyper::{Method, Request};
use hyper::body::Bytes;
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt as _, self};


type Result<T> = std::result::Result<
    T, Box<dyn std::error::Error + Send + Sync>
>;


#[tokio::main]
async fn main() -> Result<()> {
    let url = "http://httpbin.org/ip".parse::<hyper::Uri>()?;
    fetch_url1(url).await?;

    let url2 = "http://0.0.0.0:3000/echo/reversed".parse::<hyper::Uri>()?;
    fetch_url2(url2).await
}


async fn fetch_url1(url: hyper::Uri) -> Result<()> {

    let host = url.host().expect("URI has no host");
    let port = url.port_u16().unwrap_or(80);
    let address = format!("{}:{}", host, port);
    
    let stream = TcpStream::connect(address).await?;
    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed {:?}", err);
        }
    });

    let authority = url.authority().unwrap().clone();
    let req = Request::builder()
        .method(Method::GET)
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .body(Empty::<Bytes>::new())?;

    let mut res = sender.send_request(req).await?;
    println!("Response status: {:?}", res.status());

    while let Some(next) = res.frame().await {
        let frame = next?;
        if let Some(chunk) = frame.data_ref() {
            io::stdout().write_all(chunk).await?;
        }
    }

    Ok(())
}


async fn fetch_url2(url: hyper::Uri) -> Result<()> {

    let host = url.host().expect("URI has no host");
    let port = url.port_u16().unwrap_or(80);
    let address = format!("{}:{}", host, port);
    
    let stream = TcpStream::connect(address).await?;
    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed {:?}", err);
        }
    });

    let authority = url.authority().unwrap().clone();
    let data = "Hi there";
    let body: Full<Bytes> = Full::from(Bytes::from(data));
    let req = Request::builder()
        .method(Method::POST)
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .body(body)?;

    let mut res = sender.send_request(req).await?;
    println!("Response status: {:?}", res.status());

    while let Some(next) = res.frame().await {
        let frame = next?;
        if let Some(chunk) = frame.data_ref() {
            io::stdout().write_all(chunk).await?;
        }
    }

    Ok(())
}
