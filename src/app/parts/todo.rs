use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Default)]
pub struct Todo {
  pub id: String,
  pub title: String,
  pub description: String
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone)]
pub struct CreateTodo {
  pub title: String,
  pub description: String
}

pub type UpdateTodo = CreateTodo;