use axum::{extract::Path, response::IntoResponse};

pub async fn get_all() -> impl IntoResponse {

}

pub async fn get_one(Path(path): Path<String>) -> impl IntoResponse {
  println!("{path}")

}