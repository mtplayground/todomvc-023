use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub completed: bool,
    pub display_order: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateTodo {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodo {
    pub title: Option<String>,
    pub completed: Option<bool>,
    pub display_order: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct BulkUpdateTodos {
    pub completed: bool,
}
