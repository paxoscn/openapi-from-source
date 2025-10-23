// Test to verify cross-file function resolution works
use openapi_from_source::extractor::{RouteExtractor, axum::AxumExtractor};
use openapi_from_source::parser::ParsedFile;
use std::path::PathBuf;

#[test]
fn test_cross_file_function_resolution() {
    // File 1: Handler definitions
    let handlers_code = r#"
        use axum::Json;
        use serde::Serialize;
        
        #[derive(Serialize)]
        pub struct User {
            id: u32,
            name: String,
        }
        
        pub async fn get_user() -> Json<User> {
            Json(User { id: 1, name: "Test".to_string() })
        }
        
        pub async fn list_users() -> Json<Vec<User>> {
            Json(vec![])
        }
    "#;
    
    // File 2: Route definitions
    let routes_code = r#"
        use axum::{Router, routing::get};
        
        pub fn app() -> Router {
            Router::new()
                .route("/user", get(get_user))
                .route("/users", get(list_users))
        }
    "#;
    
    // Parse both files
    let handlers_ast = syn::parse_file(handlers_code).expect("Failed to parse handlers");
    let routes_ast = syn::parse_file(routes_code).expect("Failed to parse routes");
    
    let parsed_files = vec![
        ParsedFile {
            path: PathBuf::from("handlers.rs"),
            syntax_tree: handlers_ast,
        },
        ParsedFile {
            path: PathBuf::from("routes.rs"),
            syntax_tree: routes_ast,
        },
    ];
    
    // Extract routes - should find functions from handlers.rs
    let extractor = AxumExtractor;
    let routes = extractor.extract_routes(&parsed_files);
    
    println!("Found {} routes", routes.len());
    for route in &routes {
        println!("  Route: {:?} {} -> {}", route.method, route.path, route.handler_name);
        if let Some(ref response) = route.response_type {
            println!("    Response: {} (is_vec: {})", response.name, response.is_vec);
        }
    }
    
    // Verify that response types were resolved from handlers.rs
    assert_eq!(routes.len(), 2, "Should find 2 routes");
    
    let user_route = routes.iter().find(|r| r.path == "/user").expect("Should find /user route");
    assert_eq!(user_route.handler_name, "get_user");
    assert!(user_route.response_type.is_some(), "Should have response type");
    if let Some(ref response) = user_route.response_type {
        assert_eq!(response.name, "User");
        assert!(!response.is_vec);
    }
    
    let users_route = routes.iter().find(|r| r.path == "/users").expect("Should find /users route");
    assert_eq!(users_route.handler_name, "list_users");
    assert!(users_route.response_type.is_some(), "Should have response type");
    if let Some(ref response) = users_route.response_type {
        assert_eq!(response.name, "User");
        assert!(response.is_vec);
    }
    
    println!("\n✅ Cross-file function resolution works correctly!");
}
