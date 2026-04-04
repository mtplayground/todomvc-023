use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Json, Router,
};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::{BulkUpdateTodos, CreateTodo, Todo, UpdateTodo};

pub fn api_routes() -> Router<SqlitePool> {
    Router::new()
        .route(
            "/api/todos",
            get(list_todos).post(create_todo).patch(toggle_all_todos),
        )
        .route("/api/todos/completed", axum::routing::delete(clear_completed))
        .route(
            "/api/todos/{id}",
            patch(update_todo).delete(delete_todo),
        )
}

async fn list_todos(State(pool): State<SqlitePool>) -> impl IntoResponse {
    match sqlx::query_as::<_, Todo>("SELECT id, title, completed, display_order FROM todos ORDER BY display_order ASC")
        .fetch_all(&pool)
        .await
    {
        Ok(todos) => (StatusCode::OK, Json(todos)).into_response(),
        Err(e) => {
            tracing::error!("failed to list todos: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn create_todo(
    State(pool): State<SqlitePool>,
    Json(input): Json<CreateTodo>,
) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    let title = input.title.trim().to_string();

    if title.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "title must not be empty"}))).into_response();
    }

    let next_order: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(display_order), -1) + 1 FROM todos")
        .fetch_one(&pool)
        .await
        .unwrap_or(0);

    match sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (id, title, completed, display_order) VALUES (?, ?, FALSE, ?) RETURNING id, title, completed, display_order",
    )
    .bind(&id)
    .bind(&title)
    .bind(next_order)
    .fetch_one(&pool)
    .await
    {
        Ok(todo) => (StatusCode::CREATED, Json(todo)).into_response(),
        Err(e) => {
            tracing::error!("failed to create todo: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn update_todo(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
    Json(input): Json<UpdateTodo>,
) -> impl IntoResponse {
    let existing = sqlx::query_as::<_, Todo>(
        "SELECT id, title, completed, display_order FROM todos WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await;

    let existing = match existing {
        Ok(Some(todo)) => todo,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("failed to fetch todo: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let title = input
        .title
        .map(|t| t.trim().to_string())
        .unwrap_or(existing.title);
    let completed = input.completed.unwrap_or(existing.completed);
    let display_order = input.display_order.unwrap_or(existing.display_order);

    match sqlx::query_as::<_, Todo>(
        "UPDATE todos SET title = ?, completed = ?, display_order = ? WHERE id = ? RETURNING id, title, completed, display_order",
    )
    .bind(&title)
    .bind(completed)
    .bind(display_order)
    .bind(&id)
    .fetch_one(&pool)
    .await
    {
        Ok(todo) => (StatusCode::OK, Json(todo)).into_response(),
        Err(e) => {
            tracing::error!("failed to update todo: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn delete_todo(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(&id)
        .execute(&pool)
        .await
    {
        Ok(result) => {
            if result.rows_affected() == 0 {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::NO_CONTENT
            }
        }
        Err(e) => {
            tracing::error!("failed to delete todo: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn toggle_all_todos(
    State(pool): State<SqlitePool>,
    Json(input): Json<BulkUpdateTodos>,
) -> impl IntoResponse {
    match sqlx::query("UPDATE todos SET completed = ?")
        .bind(input.completed)
        .execute(&pool)
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(e) => {
            tracing::error!("failed to toggle all todos: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn clear_completed(State(pool): State<SqlitePool>) -> impl IntoResponse {
    match sqlx::query("DELETE FROM todos WHERE completed = TRUE")
        .execute(&pool)
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(e) => {
            tracing::error!("failed to clear completed todos: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
