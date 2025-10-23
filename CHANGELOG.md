# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2024-10-23

### Added
- Initial release of Rust OpenAPI Generator
- Support for Axum framework route extraction
  - Router definitions with `.route()`, `.get()`, `.post()`, etc.
  - Nested routes with `.nest()`
  - Path parameters (e.g., `/users/:id`)
  - Request extractors: `Json<T>`, `Path<T>`, `Query<T>`
- Support for Actix-Web framework route extraction
  - Route macros: `#[get]`, `#[post]`, `#[put]`, `#[delete]`, `#[patch]`
  - Scoped routes with `.scope()`
  - Path parameters (e.g., `/users/{id}`)
  - Request extractors: `web::Json<T>`, `web::Path<T>`, `web::Query<T>`
- Automatic framework detection
- Type resolution for Rust data structures
  - Primitive types (String, integers, booleans, etc.)
  - Collections (Vec<T>)
  - Optional types (Option<T>)
  - Custom structs with field analysis
  - Recursive type resolution
- Serde attribute support
  - `#[serde(rename = "...")]`
  - `#[serde(skip)]`
  - `#[serde(flatten)]`
- OpenAPI 3.0 document generation
  - Path definitions with operations
  - Parameter schemas (path, query)
  - Request body schemas
  - Response definitions
  - Component schemas with references
- Multiple output formats
  - YAML (default)
  - JSON
- Command-line interface with clap
  - Project path argument
  - Format selection (`-f`, `--format`)
  - Output file specification (`-o`, `--output`)
  - Framework override (`-w`, `--framework`)
  - Verbose logging (`-v`, `--verbose`)
  - Help and version information
- Comprehensive error handling
  - Clear error messages
  - Graceful handling of parse failures
  - Warning logs for skipped files
- Progress logging
  - File scanning progress
  - Parse results
  - Route extraction summary
  - Document generation status
- Integration tests with fixture projects
- Unit tests for all major components

### Known Limitations
- Static analysis only - cannot handle dynamically generated routes
- No complete type inference - requires explicit type annotations
- Does not expand complex custom macros
- May not accurately infer all response types
- Circular type references use placeholder schemas

[Unreleased]: https://github.com/yourusername/rust-openapi-generator/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/rust-openapi-generator/releases/tag/v0.1.0
