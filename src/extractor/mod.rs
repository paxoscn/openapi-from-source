//! Route extraction module for parsing web framework route definitions.
//!
//! This module provides a unified interface for extracting route information from different
//! web frameworks. Each framework has its own extractor implementation that knows how to
//! parse framework-specific route definitions.
//!
//! # Supported Frameworks
//!
//! - **Axum**: See [`axum::AxumExtractor`]
//! - **Actix-Web**: See [`actix::ActixExtractor`]
//!
//! # Example
//!
//! ```no_run
//! use rust_openapi_generator::extractor::{RouteExtractor, axum::AxumExtractor};
//! use rust_openapi_generator::parser::AstParser;
//! use std::path::Path;
//!
//! let parsed = AstParser::parse_file(Path::new("src/main.rs")).unwrap();
//! let extractor = AxumExtractor;
//! let routes = extractor.extract_routes(&[parsed]);
//! println!("Found {} routes", routes.len());
//! ```

pub mod axum;
pub mod actix;

use crate::parser::ParsedFile;

/// Trait for extracting route information from parsed Rust files.
///
/// Implementations of this trait know how to analyze the AST of a specific web framework
/// and extract route definitions, including paths, HTTP methods, parameters, and type information.
pub trait RouteExtractor {
    /// Extracts all route information from parsed Rust files.
    ///
    /// # Arguments
    ///
    /// * `parsed_files` - All successfully parsed Rust source files in the project
    ///
    /// # Returns
    ///
    /// Returns a vector of `RouteInfo` structs, one for each discovered route across all files.
    fn extract_routes(&self, parsed_files: &[ParsedFile]) -> Vec<RouteInfo>;
}

/// Complete information about a single API endpoint.
///
/// This structure contains all the metadata needed to generate an OpenAPI operation,
/// including the path, HTTP method, parameters, and request/response types.
#[derive(Debug, Clone)]
pub struct RouteInfo {
    /// The URL path pattern (e.g., "/users/:id" or "/users/{id}")
    pub path: String,
    /// The HTTP method for this route
    pub method: HttpMethod,
    /// The name of the handler function
    pub handler_name: String,
    /// List of parameters extracted from the handler signature
    pub parameters: Vec<Parameter>,
    /// Type information for the request body, if present
    pub request_body: Option<TypeInfo>,
    /// Type information for the response, if it can be determined
    pub response_type: Option<TypeInfo>,
}

/// HTTP methods supported by route extractors.
///
/// These correspond to standard HTTP methods used in RESTful APIs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    /// HTTP GET method
    Get,
    /// HTTP POST method
    Post,
    /// HTTP PUT method
    Put,
    /// HTTP DELETE method
    Delete,
    /// HTTP PATCH method
    Patch,
    /// HTTP OPTIONS method
    Options,
    /// HTTP HEAD method
    Head,
}

/// Information about a single parameter in a route handler.
///
/// Parameters can come from different locations (path, query string, headers)
/// and have associated type information for schema generation.
#[derive(Debug, Clone)]
pub struct Parameter {
    /// The parameter name
    pub name: String,
    /// Where the parameter is extracted from (path, query, header)
    pub location: ParameterLocation,
    /// Type information for generating the parameter schema
    pub type_info: TypeInfo,
    /// Whether the parameter is required (non-optional)
    pub required: bool,
}

/// The location where a parameter value is extracted from in an HTTP request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterLocation {
    /// Path parameter embedded in the URL (e.g., `/users/:id`)
    Path,
    /// Query string parameter (e.g., `?page=1&limit=10`)
    Query,
    /// HTTP header parameter
    Header,
}

/// Type information extracted from Rust code for OpenAPI schema generation.
///
/// This structure captures the essential information about a Rust type needed to
/// generate an appropriate OpenAPI schema, including generic arguments and wrapper types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeInfo {
    /// The base type name (e.g., "String", "User", "i32")
    pub name: String,
    /// Whether this is a generic type with type parameters
    pub is_generic: bool,
    /// Generic type arguments (e.g., for `Vec<String>`, contains TypeInfo for String)
    pub generic_args: Vec<TypeInfo>,
    /// Whether this type is wrapped in `Option<T>`
    pub is_option: bool,
    /// Whether this type is a `Vec<T>` (array type)
    pub is_vec: bool,
}

impl TypeInfo {
    /// Create a new TypeInfo for a simple type
    pub fn new(name: String) -> Self {
        Self {
            name,
            is_generic: false,
            generic_args: Vec::new(),
            is_option: false,
            is_vec: false,
        }
    }

    /// Create a TypeInfo for an `Option<T>` type
    pub fn option(inner: TypeInfo) -> Self {
        Self {
            name: inner.name.clone(),
            is_generic: false,
            generic_args: vec![inner],
            is_option: true,
            is_vec: false,
        }
    }

    /// Create a TypeInfo for a `Vec<T>` type
    pub fn vec(inner: TypeInfo) -> Self {
        Self {
            name: inner.name.clone(),
            is_generic: false,
            generic_args: vec![inner],
            is_option: false,
            is_vec: true,
        }
    }
}

impl RouteInfo {
    /// Create a new RouteInfo with minimal required fields
    pub fn new(path: String, method: HttpMethod, handler_name: String) -> Self {
        Self {
            path,
            method,
            handler_name,
            parameters: Vec::new(),
            request_body: None,
            response_type: None,
        }
    }
}

impl Parameter {
    /// Create a new Parameter
    pub fn new(name: String, location: ParameterLocation, type_info: TypeInfo, required: bool) -> Self {
        Self {
            name,
            location,
            type_info,
            required,
        }
    }
}
