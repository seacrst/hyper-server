use std::{error::Error, sync::Arc};

use services::{RedisStore, TodoStore};
use tokio::{net::TcpListener, sync::RwLock};
use axum::{routing::{delete, get, patch, post}, serve::Serve, Router};

use handlers::todos::{create_todo, get_all, get_one, remove_todo, update_todo};

pub mod services;
mod handlers;
mod parts;

pub type TodoState = Arc<RwLock<dyn TodoStore + Send + Sync>>;

const REDIS_HOST: &str = "127.0.0.1";

pub struct App {
  server: Serve<TcpListener, Router, Router>,
  addr: String
}

impl App {
  pub async fn build(config: AppConfig<'_>) -> Result<App, Box<dyn Error>> {
    let todo_store: TodoState = Arc::new(
      RwLock::new(
        RedisStore::try_new(REDIS_HOST.to_string(), config.clear_store)
      )
    );
    
    let router = Router::new()
    .route("/todo", post(create_todo))
      .route("/todos", get(get_all))
      .route("/todos/{id}", get(get_one))
      .route("/todo/{id}", patch(update_todo))
      .route("/todo/{id}", delete(remove_todo))
      .with_state(todo_store);

      let tcp = TcpListener::bind(config.addr).await?;
      let addr = tcp.local_addr()?.to_string();
      let server = axum::serve(tcp, router);

      Ok(App { server, addr })
  }

  pub async fn run(self) -> Result<(), std::io::Error> {
    println!("listening on {}", &self.addr);
    self.server.await
  }
}

pub struct AppConfig<'a> {
  pub addr: &'a str,
  pub clear_store: bool
}

