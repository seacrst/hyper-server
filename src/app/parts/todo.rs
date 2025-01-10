use uuid::Uuid as Id;

pub struct Todo {
  id: Id,
  title: String,
  description: String
}

pub struct CreateTodo {
  title: String,
  description: String
}