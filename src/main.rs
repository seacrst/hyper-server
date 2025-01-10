use std::{convert::Infallible, net::SocketAddr, sync::Arc};

use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::{body::Incoming, service::service_fn, Request, Response};
use hyper_server::{Proxy, RwProxy, Worker};
use hyper_util::rt::TokioIo;
use tokio::{net::TcpListener, sync::RwLock};
use collect_args::Args;

const PORT: u16 = 3000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], PORT));
    let listener = TcpListener::bind(addr).await?;

    let args = Args::collect();
    let (_, ports) = args.input("ports");

    let empty_port_list_msg = "Enter at least one port";

    let ports: Vec<u16> = match ports {
      Some(ports) if ports != "" => {
        ports.trim()
          .split(" ")
          .map(|port| port.parse::<u16>().expect("Port value is not a number"))
          .collect()
      },
      None => {
        panic!("{empty_port_list_msg}")
      },
      _ => {
        panic!("{empty_port_list_msg}")
      }
    };

    if ports[0] == PORT {
      panic!("Cannot use port {}", PORT);
    } 

    let workers = ports.iter()
      .map(|port| Worker::new(format!("http://localhost:{}", port)))
      .collect();

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
