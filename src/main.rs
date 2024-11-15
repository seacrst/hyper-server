use std::{convert::Infallible, net::SocketAddr};

use bytes::Bytes;
use http_body_util::Full;
use hyper::{server::conn::http1::Builder, service::service_fn, Request, Response};
use tokio::net::TcpListener;
use hyper_util::rt::{TokioExecutor, TokioIo};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
  let listener  = TcpListener::bind(addr).await?;
  
  loop {
    let (stream, _) = listener .accept().await?;
    let io = TokioIo::new(stream);
    
    tokio::task::spawn(async move {
      if let Err(err) = hyper::server::conn::http2::Builder::new(TokioExecutor::new())
        .serve_connection(io, service_fn(hello))
        .await {
            eprintln!("Error serving connection: {}", err);
        }
    });
  }
  
}
async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
  Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}
async fn handle_req(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {

  let json_data = serde_json::to_string("").unwrap();

  let res = Response::new(Full::new(Bytes::from(json_data)));

  Ok(res)
}