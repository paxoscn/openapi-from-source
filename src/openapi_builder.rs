use crate::extractor::{HttpMethod, RouteInfo};
use crate::schema_generator::{Schema, SchemaGenerator};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenAPI document builder
pub struct OpenApiBuilder {
    /// OpenAPI info section
    info: Info,
    /// Paths collection (URL path -> PathItem)
    paths: HashMap<String, PathItem>,
    /// Components section (schemas, etc.)
    components: Components,
}

/// OpenAPI Info object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    /// API title
    pub title: String,
    /// API version
    pub version: String,
    /// API description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// OpenAPI PathItem object - represents all operations for a single path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathItem {
    /// GET operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    /// POST operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
    /// PUT operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,
    /// DELETE operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,
    /// PATCH operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,
    /// OPTIONS operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Operation>,
    /// HEAD operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<Operation>,
}

/// OpenAPI Operation object - represents a single API operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Operation summary
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Operation description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Operation ID
    #[serde(rename = "operationId", skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// Parameters (path, query, header)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<Parameter>>,
    /// Request body
    #[serde(rename = "requestBody", skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBody>,
    /// Responses
    pub responses: HashMap<String, Response>,
}

/// OpenAPI Parameter object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter location (path, query, header)
    #[serde(rename = "in")]
    pub location: String,
    /// Whether the parameter is required
    pub required: bool,
    /// Parameter schema
    pub schema: Schema,
    /// Parameter description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// OpenAPI RequestBody object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    /// Request body description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the request body is required
    pub required: bool,
    /// Content types and their schemas
    pub content: HashMap<String, MediaType>,
}

/// OpenAPI MediaType object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaType {
    /// Schema for this media type
    pub schema: Schema,
}

/// OpenAPI Response object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Response description
    pub description: String,
    /// Response content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<HashMap<String, MediaType>>,
}

/// OpenAPI Components object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
    /// Schema definitions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<HashMap<String, Schema>>,
}

/// Complete OpenAPI document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiDocument {
    /// OpenAPI version
    pub openapi: String,
    /// API info
    pub info: Info,
    /// API paths
    pub paths: HashMap<String, PathItem>,
    /// Components (schemas, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,
}

impl OpenApiBuilder {
    /// Create a new OpenApiBuilder with default info
    pub fn new() -> Self {
        debug!("Initializing OpenApiBuilder");
        Self {
            info: Info {
                title: "Generated API".to_string(),
                version: "1.0.0".to_string(),
                description: Some("API documentation generated from Rust code".to_string()),
            },
            paths: HashMap::new(),
            components: Components { schemas: None },
        }
    }

    /// Set custom info for the API
    pub fn with_info(mut self, title: String, version: String, description: Option<String>) -> Self {
        self.info = Info {
            title,
            version,
            description,
        };
        self
    }

    /// Add a route to the OpenAPI document
    pub fn add_route(&mut self, route: &RouteInfo, schema_gen: &mut SchemaGenerator) {
        debug!("Adding route: {} {}", route.method_str(), route.path);

        // Convert path parameters from :param to {param} format
        let openapi_path = Self::convert_path_format(&route.path);

        // Generate parameters
        let parameters = if route.parameters.is_empty() {
            None
        } else {
            let params: Vec<Parameter> = route
                .parameters
                .iter()
                .map(|p| {
                    let param_schema = schema_gen.generate_parameter_schema(p);
                    Parameter {
                        name: param_schema.name,
                        location: param_schema.location,
                        required: param_schema.required,
                        schema: param_schema.schema,
                        description: None,
                    }
                })
                .collect();
            Some(params)
        };

        // Generate request body if present
        let request_body = route.request_body.as_ref().map(|type_info| {
            let schema = schema_gen.generate_schema(type_info);
            RequestBody {
                description: Some("Request body".to_string()),
                required: true,
                content: {
                    let mut content = HashMap::new();
                    content.insert(
                        "application/json".to_string(),
                        MediaType { schema },
                    );
                    content
                },
            }
        });

        // Generate response
        let response = if let Some(response_type) = &route.response_type {
            let schema = schema_gen.generate_schema(response_type);
            Response {
                description: "Successful response".to_string(),
                content: Some({
                    let mut content = HashMap::new();
                    content.insert(
                        "application/json".to_string(),
                        MediaType { schema },
                    );
                    content
                }),
            }
        } else {
            // Default response when type is unknown
            Response {
                description: "Successful response".to_string(),
                content: None,
            }
        };

        let mut responses = HashMap::new();
        responses.insert("200".to_string(), response);

        // Create the operation
        let operation = Operation {
            summary: Some(format!("{} {}", route.method_str(), route.path)),
            description: None,
            operation_id: Some(route.handler_name.clone()),
            parameters,
            request_body,
            responses,
        };

        // Add operation to the appropriate path and method
        let path_item = self.paths.entry(openapi_path).or_insert_with(|| PathItem {
            get: None,
            post: None,
            put: None,
            delete: None,
            patch: None,
            options: None,
            head: None,
        });

        match route.method {
            HttpMethod::Get => path_item.get = Some(operation),
            HttpMethod::Post => path_item.post = Some(operation),
            HttpMethod::Put => path_item.put = Some(operation),
            HttpMethod::Delete => path_item.delete = Some(operation),
            HttpMethod::Patch => path_item.patch = Some(operation),
            HttpMethod::Options => path_item.options = Some(operation),
            HttpMethod::Head => path_item.head = Some(operation),
        }
    }

    /// Convert path format from :param or {param} to OpenAPI {param} format
    fn convert_path_format(path: &str) -> String {
        // Handle both Axum style (:param) and Actix style ({param})
        // Convert :param to {param}
        let parts: Vec<&str> = path.split('/').collect();
        let converted_parts: Vec<String> = parts
            .iter()
            .map(|part| {
                if part.starts_with(':') {
                    format!("{{{}}}", &part[1..])
                } else {
                    part.to_string()
                }
            })
            .collect();
        
        converted_parts.join("/")
    }

    /// Build the final OpenAPI document
    pub fn build(self, schema_gen: SchemaGenerator) -> OpenApiDocument {
        debug!("Building final OpenAPI document");

        // Collect all schemas from the schema generator
        let schemas = schema_gen.get_schemas();
        let components = if !schemas.is_empty() {
            Some(Components {
                schemas: Some(schemas.clone()),
            })
        } else {
            None
        };

        OpenApiDocument {
            openapi: "3.0.0".to_string(),
            info: self.info,
            paths: self.paths,
            components,
        }
    }
}

impl Default for OpenApiBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RouteInfo {
    /// Get the HTTP method as a string
    fn method_str(&self) -> &str {
        match self.method {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Head => "HEAD",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::{HttpMethod, Parameter, ParameterLocation, RouteInfo, TypeInfo};
    use crate::parser::AstParser;
    use crate::type_resolver::TypeResolver;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper function to create a temporary file with content
    fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
        let file_path = dir.path().join(name);
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file_path
    }

    /// Helper function to create a SchemaGenerator from code
    fn create_generator_from_code(code: &str) -> SchemaGenerator {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file(&temp_dir, "test.rs", code);
        let parsed = AstParser::parse_file(&file_path).unwrap();
        let type_resolver = TypeResolver::new(vec![parsed]);
        SchemaGenerator::new(type_resolver)
    }

    #[test]
    fn test_new_builder() {
        let builder = OpenApiBuilder::new();
        
        assert_eq!(builder.info.title, "Generated API");
        assert_eq!(builder.info.version, "1.0.0");
        assert!(builder.info.description.is_some());
        assert!(builder.paths.is_empty());
    }

    #[test]
    fn test_with_info() {
        let builder = OpenApiBuilder::new()
            .with_info(
                "My API".to_string(),
                "2.0.0".to_string(),
                Some("Custom description".to_string()),
            );
        
        assert_eq!(builder.info.title, "My API");
        assert_eq!(builder.info.version, "2.0.0");
        assert_eq!(builder.info.description, Some("Custom description".to_string()));
    }

    #[test]
    fn test_add_simple_get_route() {
        let mut builder = OpenApiBuilder::new();
        let mut schema_gen = create_generator_from_code("");
        
        let route = RouteInfo::new(
            "/users".to_string(),
            HttpMethod::Get,
            "get_users".to_string(),
        );
        
        builder.add_route(&route, &mut schema_gen);
        
        assert_eq!(builder.paths.len(), 1);
        assert!(builder.paths.contains_key("/users"));
        
        let path_item = &builder.paths["/users"];
        assert!(path_item.get.is_some());
        assert!(path_item.post.is_none());
        
        let operation = path_item.get.as_ref().unwrap();
        assert_eq!(operation.operation_id, Some("get_users".to_string()));
        assert!(operation.parameters.is_none());
        assert!(operation.request_body.is_none());
        assert!(operation.responses.contains_key("200"));
    }

    #[test]
    fn test_add_post_route_with_request_body() {
        let code = r#"
            pub struct User {
                pub id: u32,
                pub name: String,
            }
        "#;
        
        let mut builder = OpenApiBuilder::new();
        let mut schema_gen = create_generator_from_code(code);
        
        let mut route = RouteInfo::new(
            "/users".to_string(),
            HttpMethod::Post,
            "create_user".to_string(),
        );
        route.request_body = Some(TypeInfo::new("User".to_string()));
        
        builder.add_route(&route, &mut schema_gen);
        
        let path_item = &builder.paths["/users"];
        assert!(path_item.post.is_some());
        
        let operation = path_item.post.as_ref().unwrap();
        assert!(operation.request_body.is_some());
        
        let request_body = operation.request_body.as_ref().unwrap();
        assert!(request_body.required);
        assert!(request_body.content.contains_key("application/json"));
    }

    #[test]
    fn test_add_route_with_path_parameter() {
        let mut builder = OpenApiBuilder::new();
        let mut schema_gen = create_generator_from_code("");
        
        let mut route = RouteInfo::new(
            "/users/:id".to_string(),
            HttpMethod::Get,
            "get_user".to_string(),
        );
        route.parameters.push(Parameter::new(
            "id".to_string(),
            ParameterLocation::Path,
            TypeInfo::new("u32".to_string()),
            true,
        ));
        
        builder.add_route(&route, &mut schema_gen);
        
        // Path should be converted to OpenAPI format
        assert!(builder.paths.contains_key("/users/{id}"));
        
        let path_item = &builder.paths["/users/{id}"];
        let operation = path_item.get.as_ref().unwrap();
        
        assert!(operation.parameters.is_some());
        let parameters = operation.parameters.as_ref().unwrap();
        assert_eq!(parameters.len(), 1);
        assert_eq!(parameters[0].name, "id");
        assert_eq!(parameters[0].location, "path");
        assert!(parameters[0].required);
    }

    #[test]
    fn test_add_route_with_query_parameter() {
        let mut builder = OpenApiBuilder::new();
        let mut schema_gen = create_generator_from_code("");
        
        let mut route = RouteInfo::new(
            "/users".to_string(),
            HttpMethod::Get,
            "list_users".to_string(),
        );
        route.parameters.push(Parameter::new(
            "page".to_string(),
            ParameterLocation::Query,
            TypeInfo::new("i32".to_string()),
            false,
        ));
        
        builder.add_route(&route, &mut schema_gen);
        
        let path_item = &builder.paths["/users"];
        let operation = path_item.get.as_ref().unwrap();
        
        assert!(operation.parameters.is_some());
        let parameters = operation.parameters.as_ref().unwrap();
        assert_eq!(parameters.len(), 1);
        assert_eq!(parameters[0].name, "page");
        assert_eq!(parameters[0].location, "query");
        assert!(!parameters[0].required);
    }

    #[test]
    fn test_add_route_with_response_type() {
        let code = r#"
            pub struct User {
                pub id: u32,
                pub name: String,
            }
        "#;
        
        let mut builder = OpenApiBuilder::new();
        let mut schema_gen = create_generator_from_code(code);
        
        let mut route = RouteInfo::new(
            "/users/:id".to_string(),
            HttpMethod::Get,
            "get_user".to_string(),
        );
        route.response_type = Some(TypeInfo::new("User".to_string()));
        
        builder.add_route(&route, &mut schema_gen);
        
        let path_item = &builder.paths["/users/{id}"];
        let operation = path_item.get.as_ref().unwrap();
        
        let response = &operation.responses["200"];
        assert_eq!(response.description, "Successful response");
        assert!(response.content.is_some());
        
        let content = response.content.as_ref().unwrap();
        assert!(content.contains_key("application/json"));
    }

    #[test]
    fn test_add_multiple_routes_same_path() {
        let mut builder = OpenApiBuilder::new();
        let mut schema_gen = create_generator_from_code("");
        
        let get_route = RouteInfo::new(
            "/users".to_string(),
            HttpMethod::Get,
            "list_users".to_string(),
        );
        
        let post_route = RouteInfo::new(
            "/users".to_string(),
            HttpMethod::Post,
            "create_user".to_string(),
        );
        
        builder.add_route(&get_route, &mut schema_gen);
        builder.add_route(&post_route, &mut schema_gen);
        
        // Should have only one path entry
        assert_eq!(builder.paths.len(), 1);
        
        let path_item = &builder.paths["/users"];
        assert!(path_item.get.is_some());
        assert!(path_item.post.is_some());
        
        assert_eq!(
            path_item.get.as_ref().unwrap().operation_id,
            Some("list_users".to_string())
        );
        assert_eq!(
            path_item.post.as_ref().unwrap().operation_id,
            Some("create_user".to_string())
        );
    }

    #[test]
    fn test_add_routes_different_methods() {
        let mut builder = OpenApiBuilder::new();
        let mut schema_gen = create_generator_from_code("");
        
        let methods = vec![
            (HttpMethod::Get, "get_handler"),
            (HttpMethod::Post, "post_handler"),
            (HttpMethod::Put, "put_handler"),
            (HttpMethod::Delete, "delete_handler"),
            (HttpMethod::Patch, "patch_handler"),
        ];
        
        for (method, handler) in methods {
            let route = RouteInfo::new(
                "/resource".to_string(),
                method,
                handler.to_string(),
            );
            builder.add_route(&route, &mut schema_gen);
        }
        
        let path_item = &builder.paths["/resource"];
        assert!(path_item.get.is_some());
        assert!(path_item.post.is_some());
        assert!(path_item.put.is_some());
        assert!(path_item.delete.is_some());
        assert!(path_item.patch.is_some());
    }

    #[test]
    fn test_convert_path_format_axum_style() {
        let path = "/users/:id/posts/:post_id";
        let converted = OpenApiBuilder::convert_path_format(path);
        assert_eq!(converted, "/users/{id}/posts/{post_id}");
    }

    #[test]
    fn test_convert_path_format_actix_style() {
        let path = "/users/{id}/posts/{post_id}";
        let converted = OpenApiBuilder::convert_path_format(path);
        assert_eq!(converted, "/users/{id}/posts/{post_id}");
    }

    #[test]
    fn test_convert_path_format_no_params() {
        let path = "/users/list";
        let converted = OpenApiBuilder::convert_path_format(path);
        assert_eq!(converted, "/users/list");
    }

    #[test]
    fn test_build_document_structure() {
        let code = r#"
            pub struct User {
                pub id: u32,
                pub name: String,
            }
        "#;
        
        let mut builder = OpenApiBuilder::new();
        let mut schema_gen = create_generator_from_code(code);
        
        let mut route = RouteInfo::new(
            "/users".to_string(),
            HttpMethod::Post,
            "create_user".to_string(),
        );
        route.request_body = Some(TypeInfo::new("User".to_string()));
        
        builder.add_route(&route, &mut schema_gen);
        
        let document = builder.build(schema_gen);
        
        assert_eq!(document.openapi, "3.0.0");
        assert_eq!(document.info.title, "Generated API");
        assert_eq!(document.info.version, "1.0.0");
        assert_eq!(document.paths.len(), 1);
        assert!(document.components.is_some());
        
        let components = document.components.unwrap();
        assert!(components.schemas.is_some());
        
        let schemas = components.schemas.unwrap();
        assert!(schemas.contains_key("User"));
    }

    #[test]
    fn test_build_document_with_multiple_schemas() {
        let code = r#"
            pub struct User {
                pub id: u32,
                pub profile: Profile,
            }
            
            pub struct Profile {
                pub bio: String,
            }
        "#;
        
        let mut builder = OpenApiBuilder::new();
        let mut schema_gen = create_generator_from_code(code);
        
        let mut route = RouteInfo::new(
            "/users".to_string(),
            HttpMethod::Post,
            "create_user".to_string(),
        );
        route.request_body = Some(TypeInfo::new("User".to_string()));
        
        builder.add_route(&route, &mut schema_gen);
        
        let document = builder.build(schema_gen);
        
        let components = document.components.unwrap();
        let schemas = components.schemas.unwrap();
        
        // Both User and Profile should be in schemas
        assert!(schemas.contains_key("User"));
        assert!(schemas.contains_key("Profile"));
    }

    #[test]
    fn test_build_document_no_schemas() {
        let mut builder = OpenApiBuilder::new();
        let mut schema_gen = create_generator_from_code("");
        
        let route = RouteInfo::new(
            "/health".to_string(),
            HttpMethod::Get,
            "health_check".to_string(),
        );
        
        builder.add_route(&route, &mut schema_gen);
        
        let document = builder.build(schema_gen);
        
        // Components should be None when there are no schemas
        assert!(document.components.is_none());
    }

    #[test]
    fn test_operation_summary_format() {
        let mut builder = OpenApiBuilder::new();
        let mut schema_gen = create_generator_from_code("");
        
        let route = RouteInfo::new(
            "/users/:id".to_string(),
            HttpMethod::Get,
            "get_user".to_string(),
        );
        
        builder.add_route(&route, &mut schema_gen);
        
        let path_item = &builder.paths["/users/{id}"];
        let operation = path_item.get.as_ref().unwrap();
        
        assert_eq!(operation.summary, Some("GET /users/:id".to_string()));
    }

    #[test]
    fn test_default_response_without_type() {
        let mut builder = OpenApiBuilder::new();
        let mut schema_gen = create_generator_from_code("");
        
        let route = RouteInfo::new(
            "/users".to_string(),
            HttpMethod::Delete,
            "delete_user".to_string(),
        );
        
        builder.add_route(&route, &mut schema_gen);
        
        let path_item = &builder.paths["/users"];
        let operation = path_item.delete.as_ref().unwrap();
        
        let response = &operation.responses["200"];
        assert_eq!(response.description, "Successful response");
        assert!(response.content.is_none());
    }

    #[test]
    fn test_complex_route_with_all_features() {
        let code = r#"
            pub struct CreateUserRequest {
                pub name: String,
                pub email: String,
            }
            
            pub struct User {
                pub id: u32,
                pub name: String,
                pub email: String,
            }
        "#;
        
        let mut builder = OpenApiBuilder::new();
        let mut schema_gen = create_generator_from_code(code);
        
        let mut route = RouteInfo::new(
            "/users".to_string(),
            HttpMethod::Post,
            "create_user".to_string(),
        );
        route.request_body = Some(TypeInfo::new("CreateUserRequest".to_string()));
        route.response_type = Some(TypeInfo::new("User".to_string()));
        route.parameters.push(Parameter::new(
            "api_key".to_string(),
            ParameterLocation::Header,
            TypeInfo::new("String".to_string()),
            true,
        ));
        
        builder.add_route(&route, &mut schema_gen);
        
        let path_item = &builder.paths["/users"];
        let operation = path_item.post.as_ref().unwrap();
        
        // Check parameters
        assert!(operation.parameters.is_some());
        let parameters = operation.parameters.as_ref().unwrap();
        assert_eq!(parameters.len(), 1);
        assert_eq!(parameters[0].location, "header");
        
        // Check request body
        assert!(operation.request_body.is_some());
        
        // Check response
        let response = &operation.responses["200"];
        assert!(response.content.is_some());
        
        // Build and check schemas
        let document = builder.build(schema_gen);
        let schemas = document.components.unwrap().schemas.unwrap();
        assert!(schemas.contains_key("CreateUserRequest"));
        assert!(schemas.contains_key("User"));
    }

    #[test]
    fn test_multiple_paths_in_document() {
        let mut builder = OpenApiBuilder::new();
        let mut schema_gen = create_generator_from_code("");
        
        let routes = vec![
            ("/users", HttpMethod::Get, "list_users"),
            ("/users/:id", HttpMethod::Get, "get_user"),
            ("/posts", HttpMethod::Get, "list_posts"),
            ("/posts/:id", HttpMethod::Get, "get_post"),
        ];
        
        for (path, method, handler) in routes {
            let route = RouteInfo::new(
                path.to_string(),
                method,
                handler.to_string(),
            );
            builder.add_route(&route, &mut schema_gen);
        }
        
        let document = builder.build(schema_gen);
        
        assert_eq!(document.paths.len(), 4);
        assert!(document.paths.contains_key("/users"));
        assert!(document.paths.contains_key("/users/{id}"));
        assert!(document.paths.contains_key("/posts"));
        assert!(document.paths.contains_key("/posts/{id}"));
    }
}
