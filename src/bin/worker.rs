use collect_args::Args;
use hyper_server::app::App;

#[tokio::main]
async fn main() {
  let args = Args::collect();
  let app = App::build("127.0.0.1:3000").await.expect("Failed to build app");
  app.run().await.expect("Failed to run app")
}