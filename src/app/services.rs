use redis::{Client, Commands, Connection};

use super::parts::todo::{CreateTodo, Todo, UpdateTodo};

pub trait TodoStore {
    fn set(&mut self, todo: CreateTodo) -> Result<Todo, String>;
    fn get_one(&mut self, id: String) -> Result<Todo, String>;
    fn get_all(&mut self) -> Result<Vec<Todo>, String>;
    fn remove(&mut self, id: String) -> Result<(), String>;
    fn update(&mut self, id: String, todo: CreateTodo) -> Result<Todo, String>;
}

pub struct RedisStore {
  con: Connection
}

impl RedisStore {
  pub fn try_new(host: String, flush_all: bool) -> Self {
    let addr = format!("redis://{}/", host);
    let mut con = Client::open(String::from(addr))
      .expect("Failed to get Redis client")
      .get_connection()
      .expect("Failed to get Redis connection");

      if flush_all {
        let _: () = redis::cmd("FLUSHALL").query(&mut con).unwrap();
      }

    Self { con }
  }
}

impl TodoStore for RedisStore {
    fn get_all(&mut self) -> Result<Vec<Todo>, String> {
      let r: Result<Vec<String>, String> = match self.con.scan() {
          Err(_) => Err("No todos".to_string()),
          Ok(iter) => Ok(iter.collect())
      };
      
      let r = r.map(|keys| {
        let todos: Vec<Todo> = keys.iter().map(|k| self.get_one(k.to_string()).unwrap()).collect();
        todos
      });

      r
    }
    
    fn get_one(&mut self, id: String) -> Result<Todo, String> {
        let result: Result<String, redis::RedisError> = self.con.get(id);

        result
          .map(|s| serde_json::from_str(&s).expect("Could not deserialize todo"))
          .map_err(|_| String::from("none"))
    }

    fn set(&mut self, CreateTodo {title, description} : CreateTodo) -> Result<Todo, String> {
      let id = uuid::Uuid::new_v4().to_string();

      let todo = Todo {id: id.clone(), title, description};

      let r: Result<(), String> = self.con.set(id.clone(), serde_json::to_string(&todo).expect("Not set"))
        .map_err(|_| String::from("Not set"));

      if r.is_err() {
        r.map(|_| Todo::default())
      } else {
        self.get_one(id)
      }
    }
    
    fn remove(&mut self, id: String) -> Result<(), String> {
        self.con.del(id)
          .map_err(|_| String::from("Not removed"))
    }
    
    fn update(&mut self, id: String, UpdateTodo {description, title}: UpdateTodo) -> Result<Todo, String> {
      let new_todo = Todo {
        id: id.clone(),
        title,
        description
      };

      let r: Result<(), String> = self.con.set(id.clone(), serde_json::to_string(&new_todo).expect("Not updated"))
        .map_err(|_| String::from("Not updated"));

      if r.is_err() {
        r.map(|_| Todo::default())
      } else {
        self.get_one(id)
      }
    }
}