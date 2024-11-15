
use std::future::Future;

use hyper::rt::Executor;

#[derive(Default, Debug, Clone)]
pub struct TokioExecutor;

impl<F> Executor<F> for TokioExecutor
where F: Future + Send + 'static, F::Output: Send + 'static {
    fn execute(&self, fut: F) {
        tokio::spawn(fut);
    }
}

impl TokioExecutor {
  pub fn new() -> Self {
    Self {}
  }
}