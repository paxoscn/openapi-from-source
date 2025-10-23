//! Rust OpenAPI Generator - Automatic OpenAPI documentation from Rust web projects.
//!
//! This library provides tools to automatically generate OpenAPI 3.0 documentation by analyzing
//! Rust source code. It uses static code analysis to extract route definitions, type information,
//! and handler signatures from web framework code.
//!
//! # Supported Frameworks
//!
//! - **Axum**: Extracts routes from `Router` definitions and method chains
//! - **Actix-Web**: Extracts routes from route macros like `#[get]`, `#[post]`, etc.
//!
//! # Architecture
//!
//! The library is organized into several modules that work together:
//!
//! 1. [`scanner`] - Recursively scans project directories for Rust files
//! 2. [`parser`] - Parses Rust source files into Abstract Syntax Trees (AST)
//! 3. [`detector`] - Automatically detects which web frameworks are used
//! 4. [`extractor`] - Extracts route information from framework-specific code
//! 5. [`type_resolver`] - Resolves Rust types and their definitions
//! 6. [`schema_generator`] - Converts Rust types to OpenAPI schemas
//! 7. [`openapi_builder`] - Constructs the complete OpenAPI document
//! 8. [`serializer`] - Serializes the document to YAML or JSON
//!
//! # Example Usage
//!
//! ```no_run
//! use openapi_from_source::{
//!     scanner::FileScanner,
//!     parser::AstParser,
//!     detector::FrameworkDetector,
//!     extractor::{RouteExtractor, axum::AxumExtractor},
//!     type_resolver::TypeResolver,
//!     schema_generator::SchemaGenerator,
//!     openapi_builder::OpenApiBuilder,
//!     serializer::serialize_yaml,
//! };
//! use std::path::PathBuf;
//!
//! // Scan project directory
//! let scanner = FileScanner::new(PathBuf::from("./my-project"));
//! let scan_result = scanner.scan().unwrap();
//!
//! // Parse files
//! let parse_results = AstParser::parse_files(&scan_result.rust_files);
//! let parsed_files: Vec<_> = parse_results.into_iter().filter_map(Result::ok).collect();
//!
//! // Detect frameworks
//! let detection = FrameworkDetector::detect(&parsed_files);
//!
//! // Extract routes
//! let extractor = AxumExtractor;
//! let routes = extractor.extract_routes(&parsed_files);
//!
//! // Build OpenAPI document
//! let type_resolver = TypeResolver::new(parsed_files);
//! let mut schema_gen = SchemaGenerator::new(type_resolver);
//! let mut builder = OpenApiBuilder::new();
//! for route in &routes {
//!     builder.add_route(route, &mut schema_gen);
//! }
//! let document = builder.build(schema_gen);
//!
//! // Serialize to YAML
//! let yaml = serialize_yaml(&document).unwrap();
//! println!("{}", yaml);
//! ```
//!
//! # Command-Line Interface
//!
//! For command-line usage, see the [`cli`] module which provides a complete CLI application.

pub mod cli;
pub mod scanner;
pub mod parser;
pub mod detector;
pub mod extractor;
pub mod type_resolver;
pub mod schema_generator;
pub mod openapi_builder;
pub mod serializer;
pub mod error;
