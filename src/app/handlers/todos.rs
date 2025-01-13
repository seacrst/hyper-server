use axum::{extract::{Path, State}, response::IntoResponse, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::app::{parts::todo::CreateTodo, TodoState};

pub async fn get_all(State(state): State<TodoState>) -> impl IntoResponse {
  let todos = state.write().await
    .get_all()
    .unwrap_or(Vec::new());

  Json(json!(todos))
}

pub async fn get_one(Path(id): Path<String>, State(state): State<TodoState>) -> impl IntoResponse {
  match state.write().await.get_one(id) {
    Ok(todo) => (StatusCode::OK, Json(json!(todo))),
    Err(_) => (StatusCode::NO_CONTENT, Json(json!(())))
  }
}

pub async fn create_todo(State(state): State<TodoState>, Json(todo): Json<CreateTodo>) -> impl IntoResponse {
  let created = state.write().await
    .set(todo);

  match created {
    Ok(todo) => (StatusCode::CREATED, Json(json!(todo))),
    Err(_) => (StatusCode::BAD_REQUEST, Json(json!(())))
  }
}

pub async fn update_todo(
  State(state): State<TodoState>, 
  Path(id): Path<String>, 
  Json(todo): Json<CreateTodo>
) -> impl IntoResponse {
  let r = state.write().await
    .update(id, todo);

  match r {
      Ok(todo) => (StatusCode::OK, Json(json!(todo))),
      Err(_) => (StatusCode::BAD_REQUEST, Json(json!(())))
  }
}

pub async fn remove_todo(State(state): State<TodoState>, Path(id): Path<String>) -> impl IntoResponse {
  state.write().await
    .remove(id)
}