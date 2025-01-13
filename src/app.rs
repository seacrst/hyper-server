use std::{error::Error, sync::Arc};

use services::{RedisStore, TodoStore};
use tokio::net::TcpListener;
use axum::{routing::get, serve::Serve, Router};

use handlers::todos::{
  get_all, get_one
};

pub mod services;
mod handlers;
mod parts;

const REDIS_HOST: &str = "127.0.0.1";

pub struct App {
  server: Serve<TcpListener, Router, Router>,
  addr: String
}

impl App {
  pub async fn build(addr: &str) -> Result<App, Box<dyn Error>> {
    let todo_store: Arc<tokio::sync::RwLock<dyn TodoStore + Send + Sync>> = Arc::new(
      tokio::sync::RwLock::new(
        RedisStore::try_new(REDIS_HOST.to_string())
      )
    );
    
    let router = Router::new()
      .route("/todos", get(get_all))
      .route("/todos/{id}", get(get_one))
      .with_state(todo_store);

      let tcp = TcpListener::bind(addr).await?;
      let addr = tcp.local_addr()?.to_string();
      let server = axum::serve(tcp, router);

      Ok(App { server, addr })
  }

  pub async fn run(self) -> Result<(), std::io::Error> {
    println!("listening on {}", &self.addr);
    self.server.await
  }
}