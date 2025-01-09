use std::{convert::Infallible, sync::Arc};

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{
  body::Incoming, HeaderMap, Method, Request, Response
};
use reqwest::{Client, Error};
use tokio::sync::RwLock;

const THRESHOLD: u64 = 5000;
pub struct Worker {
  pub addr: String,
  pub cons_num: u64,
  pub available: bool
}

impl Worker {
  pub fn new(addr: String) -> Self {
    Self {
      addr,
      cons_num: 0,
      available: true
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
  pub current: usize,
  pub available: bool
}

impl Proxy {
  pub fn new(workers: Vec<Worker>) -> Self {
    Self {
      workers,
      current: 0,
      available: true
    }
  }

  pub fn decide(&mut self) -> &mut Self {
    if let None = self.workers.iter().find(|w| w.available) {
      self.available = false;
      return self;
    }

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

      let mut set_next = || self.current = (self.current + 1) % self.workers.len();

      match index_worker_tup {
        Some((index, worker)) if worker.available => self.current = index,
        Some((_, _)) => set_next(),
        None => set_next()
      }

      self
  }

  pub async fn execute(&mut self, req: Request<Incoming>) -> Result<reqwest::Response, reqwest::Error> {
    if !self.available {
      panic!("Services not responding");
    }
    
    let client = Client::new();
    let worker = self.workers.get_mut(self.current).unwrap();
    let mut url = worker.addr.clone();

    if let Some(rest) = req.uri().path_and_query() {
      url = url + rest.as_str();
    }
  
    let method = req.method().clone();
    let headers = req.headers().clone();
    let body = req.into_body().collect().await.expect("Collecting bytes error");
  
    worker.increase();
  
    let res = client.request(method, url)
      .headers(headers)
      .body(body.to_bytes())
      .send()
      .await;
  
    worker.decrease();
  
    res
  }
}


