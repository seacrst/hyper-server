use collect_args::Args;
use hyper_server::app::{App, AppConfig};

#[tokio::main]
async fn main() {
  let args = Args::collect();

  let (_, flush_all) = args.flag("-f");
  let (_, port) = args.input("-p");

  let config = AppConfig {
    addr: format!("127.0.0.1:{}", port.expect("Port is missing")),
    clear_store: flush_all
  };

  let app = App::build(config).await.expect("Failed to build app");
  app.run().await.expect("Failed to run app")
}