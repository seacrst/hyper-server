use collect_args::Args;
use hyper_server::app::{App, AppConfig};

#[tokio::main]
async fn main() {
  let args = Args::collect();

  let (_, flush_all) = args.flag("-f");

  let config = AppConfig {
    addr: "127.0.0.1:3000",
    clear_store: flush_all
  };

  let app = App::build(config).await.expect("Failed to build app");
  app.run().await.expect("Failed to run app")
}