use std::sync::Arc;

use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::{body::Incoming, HeaderMap, Method, Request};
use reqwest::Client;
use tokio::sync::RwLock;

pub type ExecResult = Result<reqwest::Response, reqwest::Error>;
const CONNECTION_LIMIT: u64 = 2;
pub struct Worker {
    pub addr: String,
    pub cons_num: u64,
    pub available: bool,
}

impl Worker {
    pub fn new(addr: String) -> Self {
        Self {
            addr,
            cons_num: 0,
            available: true,
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

    pub fn is_busy(&self, limit: u64) -> bool {
        self.cons_num > limit
    }
}

pub type RwProxy = Arc<RwLock<Proxy>>;

pub struct Proxy {
    workers: Vec<Worker>,
    pub current: usize,
    pub available: bool,
    pub limit: u64
}

impl Proxy {
    pub fn new(workers: Vec<Worker>) -> Self {
        Self {
            workers,
            current: 0,
            available: true,
            limit: CONNECTION_LIMIT
        }
    }

    pub fn decide(&mut self) -> &mut Self {
      
        if let None = self.workers.iter().find(|w| w.available) {
            self.available = false;
            return self;
        }

        let workers_cons = self.workers.iter().filter(|w| w.cons_num > self.limit).count();

        if workers_cons == self.workers.len() {
          self.limit = self.limit + CONNECTION_LIMIT;
        }

        let index_worker_tup = self.workers
                .iter()
                .enumerate()
                .find(|(i, w)| match self.workers.get(i + 1) {
                    Some(next_w) => {
                        if w.is_busy(self.limit) && next_w.is_busy(self.limit) {
                            w.cons_num < next_w.cons_num
                        } else {
                            !w.is_busy(self.limit)
                        }
                    }
                    None => !w.is_busy(self.limit),
                });

        let mut set_next = || self.current = (self.current + 1) % self.workers.len();

        match index_worker_tup {
            Some((index, worker)) if worker.available => self.current = index,
            None => set_next(),
            _ => set_next(),
        }

        self
    }

    pub async fn execute(&mut self, req: Request<Incoming>) -> ExecResult {
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
        let body = req
            .into_body()
            .collect()
            .await
            .expect("Collecting bytes error");

        let next_req = ForwardReq {
            url: url.clone(),
            headers: headers.clone(),
            body: body.to_bytes(),
            method: method.clone(),
        };

        worker.increase();
        worker.available = true;

        let res = client
            .request(method, url)
            .headers(headers)
            .body(next_req.body.clone())
            .send()
            .await;

        worker.decrease();

        if let Err(err) = &res {
            err.status()
                .inspect(|status| worker.available = status.is_server_error());
        }

        if !worker.available {
            return self.decide().retry_exec(next_req).await;
        }

        res
    }

    async fn retry_exec(&mut self, req: ForwardReq) -> ExecResult {
        if !self.available {
            panic!("Services not responding");
        }

        let next_req = req.clone();

        let ForwardReq {
            url,
            method,
            headers,
            body,
        } = req;

        let client = Client::new();
        let worker = self.workers.get_mut(self.current).unwrap();

        worker.increase();
        worker.available = true;

        let res = client
            .request(method, url)
            .headers(headers)
            .body(body)
            .send()
            .await;

        worker.decrease();

        if let Err(err) = &res {
            err.status()
                .inspect(|status| worker.available = status.is_server_error());
        }

        if !worker.available {
            return Box::pin(self.decide().retry_exec(next_req)).await;
        }

        res
    }
}

#[derive(Clone)]
struct ForwardReq {
    url: String,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
}
