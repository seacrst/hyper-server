use serde::Serialize;

#[derive(serde::Deserialize, Serialize)]
pub struct Todo {
  pub id: String,
  pub title: String,
  pub description: String
}

#[derive(serde::Deserialize, Serialize)]
pub struct CreateTodo {
  pub title: String,
  pub description: String
}