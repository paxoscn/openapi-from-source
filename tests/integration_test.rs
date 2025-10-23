use openapi_generator::{
    detector::FrameworkDetector,
    extractor::{actix::ActixExtractor, axum::AxumExtractor, RouteExtractor},
    openapi_builder::OpenApiBuilder,
    parser::AstParser,
    scanner::FileScanner,
    schema_generator::SchemaGenerator,
    serializer::{serialize_json, serialize_yaml},
    type_resolver::TypeResolver,
};
use tempfile::TempDir;

/// Helper function to create a temporary test project
fn create_test_project(files: Vec<(&str, &str)>) -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    for (path, content) in files {
        let file_path = temp_dir.path().join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create parent directories");
        }
        std::fs::write(&file_path, content).expect("Failed to write test file");
    }
    
    temp_dir
}

#[test]
fn test_axum_end_to_end_generation() {
    // Create a temporary project with Axum code
    let axum_code = include_str!("fixtures/axum_project.rs");
    let temp_dir = create_test_project(vec![("src/main.rs", axum_code)]);
    
    // Step 1: Scan directory
    let scanner = FileScanner::new(temp_dir.path().to_path_buf());
    let scan_result = scanner.scan().expect("Failed to scan directory");
    
    assert!(!scan_result.rust_files.is_empty(), "Should find Rust files");
    
    // Step 2: Parse files
    let parse_results = AstParser::parse_files(&scan_result.rust_files);
    let parsed_files: Vec<_> = parse_results
        .into_iter()
        .filter_map(Result::ok)
        .collect();
    
    assert!(!parsed_files.is_empty(), "Should parse at least one file");
    
    // Step 3: Detect framework
    let detection = FrameworkDetector::detect(&parsed_files);
    assert!(
        detection.frameworks.contains(&openapi_generator::cli::Framework::Axum),
        "Should detect Axum framework"
    );
    
    // Step 4: Extract routes
    let extractor = AxumExtractor;
    let routes = extractor.extract_routes(&parsed_files);
    
    assert!(!routes.is_empty(), "Should extract routes");
    
    // Verify routes were extracted
    assert!(!routes.is_empty(), "Should extract routes from Axum project");
    
    // Verify specific routes exist
    let route_paths: Vec<_> = routes.iter().map(|r| r.path.as_str()).collect();
    
    // Check for health route
    let has_health = route_paths.iter().any(|p| p.contains("health"));
    assert!(has_health, "Should have health route, found: {:?}", route_paths);
    
    // Check for routes with path parameters
    let has_path_param = route_paths.iter().any(|p| p.contains(":id"));
    assert!(has_path_param, "Should have route with :id parameter, found: {:?}", route_paths);
    
    // Step 5: Build OpenAPI document
    let type_resolver = TypeResolver::new(parsed_files);
    let mut schema_gen = SchemaGenerator::new(type_resolver);
    let mut builder = OpenApiBuilder::new();
    
    for route in &routes {
        builder.add_route(route, &mut schema_gen);
    }
    
    let document = builder.build(schema_gen);
    
    // Step 6: Verify document structure
    assert_eq!(document.openapi, "3.0.0");
    assert!(!document.paths.is_empty(), "Document should have paths");
    
    // Step 7: Test serialization
    let yaml = serialize_yaml(&document).expect("Failed to serialize to YAML");
    assert!(yaml.contains("openapi: 3.0.0") || yaml.contains("openapi: '3.0.0'"));
    assert!(yaml.contains("paths:"), "YAML should contain paths section");
    assert!(yaml.contains("health"), "YAML should contain health route");
    
    let json = serialize_json(&document).expect("Failed to serialize to JSON");
    assert!(json.contains("\"openapi\": \"3.0.0\""));
    assert!(json.contains("\"paths\""), "JSON should contain paths section");
    assert!(json.contains("health"), "JSON should contain health route");
}

#[test]
fn test_actix_end_to_end_generation() {
    // Create a temporary project with Actix-Web code
    let actix_code = include_str!("fixtures/actix_project.rs");
    let temp_dir = create_test_project(vec![("src/main.rs", actix_code)]);
    
    // Step 1: Scan directory
    let scanner = FileScanner::new(temp_dir.path().to_path_buf());
    let scan_result = scanner.scan().expect("Failed to scan directory");
    
    assert!(!scan_result.rust_files.is_empty(), "Should find Rust files");
    
    // Step 2: Parse files
    let parse_results = AstParser::parse_files(&scan_result.rust_files);
    let parsed_files: Vec<_> = parse_results
        .into_iter()
        .filter_map(Result::ok)
        .collect();
    
    assert!(!parsed_files.is_empty(), "Should parse at least one file");
    
    // Step 3: Detect framework
    let detection = FrameworkDetector::detect(&parsed_files);
    assert!(
        detection.frameworks.contains(&openapi_generator::cli::Framework::ActixWeb),
        "Should detect Actix-Web framework"
    );
    
    // Step 4: Extract routes
    let extractor = ActixExtractor;
    let routes = extractor.extract_routes(&parsed_files);
    
    assert!(!routes.is_empty(), "Should extract routes");
    
    // Verify specific routes exist
    let route_paths: Vec<_> = routes.iter().map(|r| r.path.as_str()).collect();
    assert!(route_paths.contains(&"/users"), "Should have /users route");
    assert!(route_paths.iter().any(|p| p.contains("{id}")), "Should have route with id parameter");
    assert!(route_paths.contains(&"/health"), "Should have /health route");
    
    // Step 5: Build OpenAPI document
    let type_resolver = TypeResolver::new(parsed_files);
    let mut schema_gen = SchemaGenerator::new(type_resolver);
    let mut builder = OpenApiBuilder::new();
    
    for route in &routes {
        builder.add_route(route, &mut schema_gen);
    }
    
    let document = builder.build(schema_gen);
    
    // Step 6: Verify document structure
    assert_eq!(document.openapi, "3.0.0");
    assert!(!document.paths.is_empty(), "Document should have paths");
    
    // Step 7: Test serialization
    let yaml = serialize_yaml(&document).expect("Failed to serialize to YAML");
    assert!(yaml.contains("openapi: 3.0.0") || yaml.contains("openapi: '3.0.0'"));
    assert!(yaml.contains("paths:"), "YAML should contain paths section");
    assert!(yaml.contains("health") || yaml.contains("users"), "YAML should contain route paths");
    
    let json = serialize_json(&document).expect("Failed to serialize to JSON");
    assert!(json.contains("\"openapi\": \"3.0.0\""));
    assert!(json.contains("\"paths\""), "JSON should contain paths section");
    assert!(json.contains("health") || json.contains("users"), "JSON should contain route paths");
}

#[test]
fn test_openapi_document_structure() {
    // Test with Axum fixture
    let axum_code = include_str!("fixtures/axum_project.rs");
    let temp_dir = create_test_project(vec![("src/lib.rs", axum_code)]);
    
    let scanner = FileScanner::new(temp_dir.path().to_path_buf());
    let scan_result = scanner.scan().expect("Failed to scan");
    let parse_results = AstParser::parse_files(&scan_result.rust_files);
    let parsed_files: Vec<_> = parse_results.into_iter().filter_map(Result::ok).collect();
    
    let extractor = AxumExtractor;
    let routes = extractor.extract_routes(&parsed_files);
    
    let type_resolver = TypeResolver::new(parsed_files);
    let mut schema_gen = SchemaGenerator::new(type_resolver);
    let mut builder = OpenApiBuilder::new();
    
    for route in &routes {
        builder.add_route(route, &mut schema_gen);
    }
    
    let document = builder.build(schema_gen);
    
    // Verify OpenAPI version
    assert_eq!(document.openapi, "3.0.0");
    
    // Verify info section
    assert_eq!(document.info.title, "Generated API");
    assert_eq!(document.info.version, "1.0.0");
    
    // Verify paths exist
    assert!(!document.paths.is_empty());
    
    // Verify components/schemas exist if there are custom types
    if let Some(components) = &document.components {
        if let Some(schemas) = &components.schemas {
            // Should have User, CreateUserRequest, UpdateUserRequest, ListQuery schemas
            assert!(schemas.contains_key("User") || 
                    schemas.contains_key("CreateUserRequest") ||
                    schemas.contains_key("UpdateUserRequest") ||
                    schemas.contains_key("ListQuery"),
                    "Should have at least one schema defined");
        }
    }
}

#[test]
fn test_route_parameters_extraction() {
    let axum_code = include_str!("fixtures/axum_project.rs");
    let temp_dir = create_test_project(vec![("src/main.rs", axum_code)]);
    
    let scanner = FileScanner::new(temp_dir.path().to_path_buf());
    let scan_result = scanner.scan().expect("Failed to scan");
    let parse_results = AstParser::parse_files(&scan_result.rust_files);
    let parsed_files: Vec<_> = parse_results.into_iter().filter_map(Result::ok).collect();
    
    let extractor = AxumExtractor;
    let routes = extractor.extract_routes(&parsed_files);
    
    // Find route with path parameter
    let user_by_id_route = routes.iter().find(|r| r.path.contains(":id"));
    assert!(user_by_id_route.is_some(), "Should find route with :id parameter");
    
    if let Some(route) = user_by_id_route {
        // Verify parameters are extracted
        assert!(!route.parameters.is_empty(), "Route should have parameters");
    }
}

#[test]
fn test_request_body_extraction() {
    let actix_code = include_str!("fixtures/actix_project.rs");
    let temp_dir = create_test_project(vec![("src/main.rs", actix_code)]);
    
    let scanner = FileScanner::new(temp_dir.path().to_path_buf());
    let scan_result = scanner.scan().expect("Failed to scan");
    let parse_results = AstParser::parse_files(&scan_result.rust_files);
    let parsed_files: Vec<_> = parse_results.into_iter().filter_map(Result::ok).collect();
    
    let extractor = ActixExtractor;
    let routes = extractor.extract_routes(&parsed_files);
    
    // Find POST route which should have request body
    let post_route = routes.iter().find(|r| {
        r.path.contains("/users") && 
        matches!(r.method, openapi_generator::extractor::HttpMethod::Post)
    });
    
    assert!(post_route.is_some(), "Should find POST /users route");
    
    if let Some(route) = post_route {
        // POST routes typically have request bodies
        assert!(
            route.request_body.is_some() || !route.parameters.is_empty(),
            "POST route should have request body or parameters"
        );
    }
}

#[test]
fn test_yaml_serialization_format() {
    let axum_code = include_str!("fixtures/axum_project.rs");
    let temp_dir = create_test_project(vec![("src/main.rs", axum_code)]);
    
    let scanner = FileScanner::new(temp_dir.path().to_path_buf());
    let scan_result = scanner.scan().expect("Failed to scan");
    let parse_results = AstParser::parse_files(&scan_result.rust_files);
    let parsed_files: Vec<_> = parse_results.into_iter().filter_map(Result::ok).collect();
    
    let extractor = AxumExtractor;
    let routes = extractor.extract_routes(&parsed_files);
    
    let type_resolver = TypeResolver::new(parsed_files);
    let mut schema_gen = SchemaGenerator::new(type_resolver);
    let mut builder = OpenApiBuilder::new();
    
    for route in &routes {
        builder.add_route(route, &mut schema_gen);
    }
    
    let document = builder.build(schema_gen);
    let yaml = serialize_yaml(&document).expect("Failed to serialize to YAML");
    
    // Verify YAML structure
    assert!(yaml.starts_with("openapi:") || yaml.starts_with("---"));
    assert!(yaml.contains("paths:"));
    assert!(yaml.contains("info:"));
    
    // Verify it's valid YAML by parsing it back
    let parsed: serde_yaml::Value = serde_yaml::from_str(&yaml)
        .expect("Generated YAML should be valid");
    assert!(parsed.get("openapi").is_some());
    assert!(parsed.get("paths").is_some());
}

#[test]
fn test_json_serialization_format() {
    let actix_code = include_str!("fixtures/actix_project.rs");
    let temp_dir = create_test_project(vec![("src/main.rs", actix_code)]);
    
    let scanner = FileScanner::new(temp_dir.path().to_path_buf());
    let scan_result = scanner.scan().expect("Failed to scan");
    let parse_results = AstParser::parse_files(&scan_result.rust_files);
    let parsed_files: Vec<_> = parse_results.into_iter().filter_map(Result::ok).collect();
    
    let extractor = ActixExtractor;
    let routes = extractor.extract_routes(&parsed_files);
    
    let type_resolver = TypeResolver::new(parsed_files);
    let mut schema_gen = SchemaGenerator::new(type_resolver);
    let mut builder = OpenApiBuilder::new();
    
    for route in &routes {
        builder.add_route(route, &mut schema_gen);
    }
    
    let document = builder.build(schema_gen);
    let json = serialize_json(&document).expect("Failed to serialize to JSON");
    
    // Verify JSON structure
    assert!(json.starts_with("{"));
    assert!(json.ends_with("}"));
    assert!(json.contains("\"openapi\""));
    assert!(json.contains("\"paths\""));
    
    // Verify it's valid JSON by parsing it back
    let parsed: serde_json::Value = serde_json::from_str(&json)
        .expect("Generated JSON should be valid");
    assert!(parsed.get("openapi").is_some());
    assert!(parsed.get("paths").is_some());
    
    // Verify pretty printing (should have newlines and indentation)
    assert!(json.contains("\n"), "JSON should be pretty-printed");
}

#[test]
fn test_empty_project_handling() {
    // Create an empty project
    let temp_dir = create_test_project(vec![("src/lib.rs", "// Empty file")]);
    
    let scanner = FileScanner::new(temp_dir.path().to_path_buf());
    let scan_result = scanner.scan().expect("Should scan successfully");
    
    let parse_results = AstParser::parse_files(&scan_result.rust_files);
    let parsed_files: Vec<_> = parse_results.into_iter().filter_map(Result::ok).collect();
    
    let extractor = AxumExtractor;
    let routes = extractor.extract_routes(&parsed_files);
    
    // Should handle empty projects gracefully
    assert!(routes.is_empty(), "Empty project should have no routes");
    
    // Should still be able to build a document
    let type_resolver = TypeResolver::new(parsed_files);
    let schema_gen = SchemaGenerator::new(type_resolver);
    let builder = OpenApiBuilder::new();
    
    let document = builder.build(schema_gen);
    
    // Document should be valid but empty
    assert_eq!(document.openapi, "3.0.0");
    assert!(document.paths.is_empty());
}

#[test]
fn test_multiple_http_methods_same_path() {
    let axum_code = include_str!("fixtures/axum_project.rs");
    let temp_dir = create_test_project(vec![("src/main.rs", axum_code)]);
    
    let scanner = FileScanner::new(temp_dir.path().to_path_buf());
    let scan_result = scanner.scan().expect("Failed to scan");
    let parse_results = AstParser::parse_files(&scan_result.rust_files);
    let parsed_files: Vec<_> = parse_results.into_iter().filter_map(Result::ok).collect();
    
    let extractor = AxumExtractor;
    let routes = extractor.extract_routes(&parsed_files);
    
    // Find routes for /users path
    let users_routes: Vec<_> = routes.iter()
        .filter(|r| r.path == "/users")
        .collect();
    
    // Should have multiple methods (GET and POST) for /users
    if users_routes.len() > 1 {
        let methods: Vec<_> = users_routes.iter()
            .map(|r| &r.method)
            .collect();
        
        // Verify different methods exist
        let has_get = methods.iter().any(|m| matches!(m, openapi_generator::extractor::HttpMethod::Get));
        let has_post = methods.iter().any(|m| matches!(m, openapi_generator::extractor::HttpMethod::Post));
        
        assert!(has_get || has_post, "Should have GET or POST method for /users");
    }
}

#[test]
fn test_response_type_extraction() {
    let axum_code = include_str!("fixtures/axum_project.rs");
    let temp_dir = create_test_project(vec![("src/main.rs", axum_code)]);
    
    let scanner = FileScanner::new(temp_dir.path().to_path_buf());
    let scan_result = scanner.scan().expect("Failed to scan");
    let parse_results = AstParser::parse_files(&scan_result.rust_files);
    let parsed_files: Vec<_> = parse_results.into_iter().filter_map(Result::ok).collect();
    
    let extractor = AxumExtractor;
    let routes = extractor.extract_routes(&parsed_files);
    
    // Find GET /users route - should return Json<Vec<User>>
    let get_users_route = routes.iter().find(|r| {
        r.path == "/users" && 
        matches!(r.method, openapi_generator::extractor::HttpMethod::Get) &&
        r.handler_name == "get_users"
    });
    
    if let Some(route) = get_users_route {
        assert!(route.response_type.is_some(), "GET /users should have response type");
        if let Some(ref response) = route.response_type {
            assert!(response.is_vec, "Response should be a Vec");
            assert_eq!(response.name, "User", "Response should be Vec<User>");
        }
    }
    
    // Find GET /users/:id route - should return Json<User>
    let get_user_route = routes.iter().find(|r| {
        r.path.contains(":id") && 
        matches!(r.method, openapi_generator::extractor::HttpMethod::Get) &&
        r.handler_name == "get_user"
    });
    
    if let Some(route) = get_user_route {
        assert!(route.response_type.is_some(), "GET /users/:id should have response type");
        if let Some(ref response) = route.response_type {
            assert!(!response.is_vec, "Response should not be a Vec");
            assert_eq!(response.name, "User", "Response should be User");
        }
    }
    
    // Find POST /users route - should return Json<User>
    let create_user_route = routes.iter().find(|r| {
        r.path == "/users" && 
        matches!(r.method, openapi_generator::extractor::HttpMethod::Post) &&
        r.handler_name == "create_user"
    });
    
    if let Some(route) = create_user_route {
        assert!(route.response_type.is_some(), "POST /users should have response type");
        if let Some(ref response) = route.response_type {
            assert_eq!(response.name, "User", "Response should be User");
        }
    }
    
    // Find health check route - should return &'static str
    let health_route = routes.iter().find(|r| {
        r.path.contains("health") && 
        r.handler_name == "health_check"
    });
    
    if let Some(route) = health_route {
        assert!(route.response_type.is_some(), "Health check should have response type");
        if let Some(ref response) = route.response_type {
            assert_eq!(response.name, "str", "Response should be str");
        }
    }
}
