// Axum test fixture - a simple API with various route types
use axum::{
    extract::{Path, Query},
    routing::{get, post, put, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

// Simple GET route
async fn get_users(Query(query): Query<ListQuery>) -> Json<Vec<User>> {
    Json(vec![])
}

// GET route with path parameter
async fn get_user(Path(id): Path<u32>) -> Json<User> {
    Json(User {
        id,
        name: "Test".to_string(),
        email: "test@example.com".to_string(),
    })
}

// POST route with request body
async fn create_user(Json(payload): Json<CreateUserRequest>) -> Json<User> {
    Json(User {
        id: 1,
        name: payload.name,
        email: payload.email,
    })
}

// PUT route with path parameter and request body
async fn update_user(
    Path(id): Path<u32>,
    Json(payload): Json<UpdateUserRequest>,
) -> Json<User> {
    Json(User {
        id,
        name: payload.name.unwrap_or_default(),
        email: payload.email.unwrap_or_default(),
    })
}

// DELETE route with path parameter
async fn delete_user(Path(id): Path<u32>) -> () {
    ()
}

// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/users", get(get_users).post(create_user))
        .route("/users/:id", get(get_user).put(update_user).delete(delete_user))
}

// Nested routes example
pub fn create_api_router() -> Router {
    let user_routes = Router::new()
        .route("/", get(get_users).post(create_user))
        .route("/:id", get(get_user));

    Router::new()
        .nest("/api/v1/users", user_routes)
        .route("/api/v1/health", get(health_check))
}
