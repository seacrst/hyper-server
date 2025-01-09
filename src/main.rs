use std::{convert::Infallible, net::SocketAddr, sync::Arc};

use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::{body::Incoming, service::service_fn, Request, Response};
use hyper_server::{Proxy, RwProxy, Worker};
use hyper_util::rt::TokioIo;
use tokio::{net::TcpListener, sync::RwLock};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;

    let workers = vec![
        Worker::new(String::from("http://localhost:3001")),
        Worker::new(String::from("http://localhost:3002")),
        Worker::new(String::from("http://localhost:3003")),
    ];

    let proxy = Arc::new(RwLock::new(Proxy::new(workers)));

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let proxy = proxy.clone();

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(move |req| handle_req(req, proxy.clone())))
                .await
            {
                eprintln!("Error serving connection: {}", err);
            }
        });
    }
}

async fn handle_req(req: Request<Incoming>, proxy: RwProxy) -> Result<Response<Full<Bytes>>, Infallible> {
    let res = proxy.write().await
      .decide()
      .execute(req)
      .await;

    match res {
        Ok(r) => Ok(Response::new(Full::new(Bytes::from(r.bytes().await.unwrap())))),
        Err(_) => panic!("Fail")
    }
}
