use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub completed: bool,
    pub display_order: i64,
}

#[derive(Debug, Serialize)]
pub struct CreateTodo {
    pub title: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateTodo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_order: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct BulkUpdateTodos {
    pub completed: bool,
}

const BASE: &str = "/api/todos";

pub async fn fetch_todos() -> Result<Vec<Todo>, String> {
    Request::get(BASE)
        .send()
        .await
        .map_err(|e| format!("fetch error: {e}"))?
        .json::<Vec<Todo>>()
        .await
        .map_err(|e| format!("parse error: {e}"))
}

pub async fn create_todo(title: &str) -> Result<Todo, String> {
    Request::post(BASE)
        .json(&CreateTodo {
            title: title.to_string(),
        })
        .map_err(|e| format!("serialize error: {e}"))?
        .send()
        .await
        .map_err(|e| format!("fetch error: {e}"))?
        .json::<Todo>()
        .await
        .map_err(|e| format!("parse error: {e}"))
}

pub async fn update_todo(id: &str, update: &UpdateTodo) -> Result<Todo, String> {
    Request::patch(&format!("{BASE}/{id}"))
        .json(update)
        .map_err(|e| format!("serialize error: {e}"))?
        .send()
        .await
        .map_err(|e| format!("fetch error: {e}"))?
        .json::<Todo>()
        .await
        .map_err(|e| format!("parse error: {e}"))
}

pub async fn delete_todo(id: &str) -> Result<(), String> {
    let resp = Request::delete(&format!("{BASE}/{id}"))
        .send()
        .await
        .map_err(|e| format!("fetch error: {e}"))?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("delete failed: {}", resp.status()))
    }
}

pub async fn toggle_all(completed: bool) -> Result<(), String> {
    let resp = Request::patch(BASE)
        .json(&BulkUpdateTodos { completed })
        .map_err(|e| format!("serialize error: {e}"))?
        .send()
        .await
        .map_err(|e| format!("fetch error: {e}"))?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("toggle all failed: {}", resp.status()))
    }
}

pub async fn clear_completed() -> Result<(), String> {
    let resp = Request::delete(&format!("{BASE}/completed"))
        .send()
        .await
        .map_err(|e| format!("fetch error: {e}"))?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("clear completed failed: {}", resp.status()))
    }
}
