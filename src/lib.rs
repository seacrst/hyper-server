use std::{convert::Infallible, sync::Arc};

use bytes::Bytes;
use http_body_util::Full;
use hyper::{
  body::Incoming, HeaderMap, Method, Request, Response
};
use reqwest::Client;
use tokio::sync::RwLock;

const THRESHOLD: u64 = 5000;
pub struct Worker {
  pub addr: String,
  pub cons_num: u64
}

impl Worker {
  pub fn new(addr: String) -> Self {
    Self {
      addr,
      cons_num: 0
    }
  }

  pub fn increase(&mut self) {
    self.cons_num += 1;
  }

  pub fn decrease(&mut self) {
    if self.cons_num > 0 {
      self.cons_num -= 1;
    }
  }

  pub fn is_busy(&self) -> bool {
    self.cons_num > THRESHOLD
  }
}

pub type RwProxy = Arc<RwLock<Proxy>>;

pub struct Proxy {
  workers: Vec<Worker>,
  pub current: usize
}

impl Proxy {
  pub fn new(workers: Vec<Worker>) -> Self {
    Self {
      workers,
      current: 0
    }
  }

  pub async fn handle_req(&mut self, req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let index_worker_tup = self.workers.iter()
      .enumerate()
      .find(|(i, w)| {
        match self.workers.get(i + 1) {
          Some(next_w) => {
            if w.is_busy() && next_w.is_busy() {
              w.cons_num < next_w.cons_num
            } else {
              !w.is_busy()
            }
          },
          None => !w.is_busy()
        }
      });
    
    match index_worker_tup {
        Some((index, _)) => {
          self.current = index;
          let worker = self.workers.get_mut(index).unwrap();
          execute(worker.addr.clone(), req);
        },
        None => {
          hhdh
        }
    }
    
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
  }
}


async fn execute(host: String, req: Request<Incoming>) -> Result<reqwest::Response, reqwest::Error> {
  let client = Client::new();
  client.request(req.method(), url)
    .headers(req.headers())
    .body(req.body())
    .send()
    .await
}