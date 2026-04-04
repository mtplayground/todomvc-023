use std::sync::atomic::{AtomicU64, Ordering};

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::sqlite::SqliteConnectOptions;
use tower::ServiceExt;

use server::models::Todo;
use server::routes::api_routes;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

async fn setup_app() -> Router {
    let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let db_name = format!("file:test_db_{n}?mode=memory&cache=shared");

    let options = SqliteConnectOptions::new()
        .filename(&db_name)
        .create_if_missing(true);

    let pool = sqlx::pool::PoolOptions::<sqlx::Sqlite>::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("failed to create in-memory pool");

    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    api_routes().with_state(pool)
}

async fn json_body(body: Body) -> Value {
    let bytes = body.collect().await.expect("body collect").to_bytes();
    serde_json::from_slice(&bytes).expect("parse json")
}

fn json_request(method: Method, uri: &str, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(b) = body {
        builder = builder.header("content-type", "application/json");
        builder.body(Body::from(serde_json::to_vec(&b).expect("serialize"))).expect("build request")
    } else {
        builder.body(Body::empty()).expect("build request")
    }
}

#[tokio::test]
async fn test_list_todos_empty() {
    let app = setup_app().await;

    let resp = app
        .oneshot(json_request(Method::GET, "/api/todos", None))
        .await
        .expect("request failed");

    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp.into_body()).await;
    assert_eq!(body, json!([]));
}

#[tokio::test]
async fn test_create_todo() {
    let app = setup_app().await;

    let resp = app
        .oneshot(json_request(
            Method::POST,
            "/api/todos",
            Some(json!({"title": "Buy milk"})),
        ))
        .await
        .expect("request failed");

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = json_body(resp.into_body()).await;
    assert_eq!(body["title"], "Buy milk");
    assert_eq!(body["completed"], false);
    assert_eq!(body["display_order"], 0);
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn test_create_todo_trims_whitespace() {
    let app = setup_app().await;

    let resp = app
        .oneshot(json_request(
            Method::POST,
            "/api/todos",
            Some(json!({"title": "  trimmed  "})),
        ))
        .await
        .expect("request failed");

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = json_body(resp.into_body()).await;
    assert_eq!(body["title"], "trimmed");
}

#[tokio::test]
async fn test_create_todo_empty_title_rejected() {
    let app = setup_app().await;

    let resp = app
        .oneshot(json_request(
            Method::POST,
            "/api/todos",
            Some(json!({"title": "   "})),
        ))
        .await
        .expect("request failed");

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_and_list() {
    let app = setup_app().await;

    // Create
    let resp = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/todos",
            Some(json!({"title": "First"})),
        ))
        .await
        .expect("request failed");
    assert_eq!(resp.status(), StatusCode::CREATED);

    // List
    let resp = app
        .oneshot(json_request(Method::GET, "/api/todos", None))
        .await
        .expect("request failed");

    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp.into_body()).await;
    let todos: Vec<Todo> = serde_json::from_value(body).expect("parse todos");
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].title, "First");
}

#[tokio::test]
async fn test_update_todo() {
    let app = setup_app().await;

    // Create
    let resp = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/todos",
            Some(json!({"title": "Original"})),
        ))
        .await
        .expect("request failed");
    let body = json_body(resp.into_body()).await;
    let id = body["id"].as_str().expect("id");

    // Update title
    let resp = app
        .clone()
        .oneshot(json_request(
            Method::PATCH,
            &format!("/api/todos/{id}"),
            Some(json!({"title": "Updated"})),
        ))
        .await
        .expect("request failed");

    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp.into_body()).await;
    assert_eq!(body["title"], "Updated");

    // Toggle completed
    let resp = app
        .oneshot(json_request(
            Method::PATCH,
            &format!("/api/todos/{id}"),
            Some(json!({"completed": true})),
        ))
        .await
        .expect("request failed");

    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp.into_body()).await;
    assert_eq!(body["completed"], true);
    assert_eq!(body["title"], "Updated");
}

#[tokio::test]
async fn test_update_nonexistent_todo() {
    let app = setup_app().await;

    let resp = app
        .oneshot(json_request(
            Method::PATCH,
            "/api/todos/nonexistent-id",
            Some(json!({"title": "nope"})),
        ))
        .await
        .expect("request failed");

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_todo() {
    let app = setup_app().await;

    // Create
    let resp = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/todos",
            Some(json!({"title": "Delete me"})),
        ))
        .await
        .expect("request failed");
    let body = json_body(resp.into_body()).await;
    let id = body["id"].as_str().expect("id");

    // Delete
    let resp = app
        .clone()
        .oneshot(json_request(Method::DELETE, &format!("/api/todos/{id}"), None))
        .await
        .expect("request failed");
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let resp = app
        .oneshot(json_request(Method::GET, "/api/todos", None))
        .await
        .expect("request failed");
    let body = json_body(resp.into_body()).await;
    assert_eq!(body, json!([]));
}

#[tokio::test]
async fn test_delete_nonexistent_todo() {
    let app = setup_app().await;

    let resp = app
        .oneshot(json_request(Method::DELETE, "/api/todos/nonexistent-id", None))
        .await
        .expect("request failed");

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_toggle_all() {
    let app = setup_app().await;

    // Create two todos
    app.clone()
        .oneshot(json_request(Method::POST, "/api/todos", Some(json!({"title": "A"}))))
        .await
        .expect("request failed");
    app.clone()
        .oneshot(json_request(Method::POST, "/api/todos", Some(json!({"title": "B"}))))
        .await
        .expect("request failed");

    // Toggle all completed
    let resp = app
        .clone()
        .oneshot(json_request(
            Method::PATCH,
            "/api/todos",
            Some(json!({"completed": true})),
        ))
        .await
        .expect("request failed");
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify all completed
    let resp = app
        .clone()
        .oneshot(json_request(Method::GET, "/api/todos", None))
        .await
        .expect("request failed");
    let body = json_body(resp.into_body()).await;
    let todos: Vec<Todo> = serde_json::from_value(body).expect("parse");
    assert!(todos.iter().all(|t| t.completed));

    // Toggle all back
    let resp = app
        .clone()
        .oneshot(json_request(
            Method::PATCH,
            "/api/todos",
            Some(json!({"completed": false})),
        ))
        .await
        .expect("request failed");
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let resp = app
        .oneshot(json_request(Method::GET, "/api/todos", None))
        .await
        .expect("request failed");
    let body = json_body(resp.into_body()).await;
    let todos: Vec<Todo> = serde_json::from_value(body).expect("parse");
    assert!(todos.iter().all(|t| !t.completed));
}

#[tokio::test]
async fn test_clear_completed() {
    let app = setup_app().await;

    // Create two todos
    let resp = app
        .clone()
        .oneshot(json_request(Method::POST, "/api/todos", Some(json!({"title": "Keep"}))))
        .await
        .expect("request failed");
    let keep_body = json_body(resp.into_body()).await;
    let keep_id = keep_body["id"].as_str().expect("id").to_string();

    app.clone()
        .oneshot(json_request(Method::POST, "/api/todos", Some(json!({"title": "Remove"}))))
        .await
        .expect("request failed");

    // Mark second as completed via toggle all, then uncomplete the first
    app.clone()
        .oneshot(json_request(
            Method::PATCH,
            "/api/todos",
            Some(json!({"completed": true})),
        ))
        .await
        .expect("request failed");

    app.clone()
        .oneshot(json_request(
            Method::PATCH,
            &format!("/api/todos/{keep_id}"),
            Some(json!({"completed": false})),
        ))
        .await
        .expect("request failed");

    // Clear completed
    let resp = app
        .clone()
        .oneshot(json_request(Method::DELETE, "/api/todos/completed", None))
        .await
        .expect("request failed");
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Only "Keep" should remain
    let resp = app
        .oneshot(json_request(Method::GET, "/api/todos", None))
        .await
        .expect("request failed");
    let body = json_body(resp.into_body()).await;
    let todos: Vec<Todo> = serde_json::from_value(body).expect("parse");
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].title, "Keep");
}

#[tokio::test]
async fn test_display_order_increments() {
    let app = setup_app().await;

    app.clone()
        .oneshot(json_request(Method::POST, "/api/todos", Some(json!({"title": "First"}))))
        .await
        .expect("request failed");
    app.clone()
        .oneshot(json_request(Method::POST, "/api/todos", Some(json!({"title": "Second"}))))
        .await
        .expect("request failed");

    let resp = app
        .oneshot(json_request(Method::GET, "/api/todos", None))
        .await
        .expect("request failed");
    let body = json_body(resp.into_body()).await;
    let todos: Vec<Todo> = serde_json::from_value(body).expect("parse");
    assert_eq!(todos.len(), 2);
    assert_eq!(todos[0].title, "First");
    assert_eq!(todos[0].display_order, 0);
    assert_eq!(todos[1].title, "Second");
    assert_eq!(todos[1].display_order, 1);
}
