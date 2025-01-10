use std::error::Error;

use tokio::net::TcpListener;
use axum::{routing::get, serve::Serve, Router};
use handlers::todos::{
  get_all
};

mod handlers;
mod parts;

pub struct App {
  server: Serve<TcpListener, Router, Router>,
  addr: String
}

impl App {
  pub async fn build(addr: &str) -> Result<App, Box<dyn Error>> {
    let router = Router::new()
      .route("todos", get(get_all));

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