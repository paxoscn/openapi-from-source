// Actix-Web test fixture - a simple API with various route types
use actix_web::{web, HttpResponse, Responder};
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
#[actix_web::get("/users")]
async fn get_users(query: web::Query<ListQuery>) -> impl Responder {
    HttpResponse::Ok().json(Vec::<User>::new())
}

// GET route with path parameter
#[actix_web::get("/users/{id}")]
async fn get_user(path: web::Path<u32>) -> impl Responder {
    let id = path.into_inner();
    HttpResponse::Ok().json(User {
        id,
        name: "Test".to_string(),
        email: "test@example.com".to_string(),
    })
}

// POST route with request body
#[actix_web::post("/users")]
async fn create_user(payload: web::Json<CreateUserRequest>) -> impl Responder {
    HttpResponse::Ok().json(User {
        id: 1,
        name: payload.name.clone(),
        email: payload.email.clone(),
    })
}

// PUT route with path parameter and request body
#[actix_web::put("/users/{id}")]
async fn update_user(
    path: web::Path<u32>,
    payload: web::Json<UpdateUserRequest>,
) -> impl Responder {
    let id = path.into_inner();
    HttpResponse::Ok().json(User {
        id,
        name: payload.name.clone().unwrap_or_default(),
        email: payload.email.clone().unwrap_or_default(),
    })
}

// DELETE route with path parameter
#[actix_web::delete("/users/{id}")]
async fn delete_user(path: web::Path<u32>) -> impl Responder {
    HttpResponse::NoContent()
}

// Health check endpoint
#[actix_web::get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

// Scoped routes example
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(get_users)
            .service(get_user)
            .service(create_user)
            .service(update_user)
            .service(delete_user)
            .service(health_check),
    );
}
