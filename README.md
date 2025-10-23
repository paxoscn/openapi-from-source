# Rust OpenAPI Generator

A command-line tool that automatically generates OpenAPI 3.0 documentation from Rust web projects. The tool uses static code analysis to extract route information and data structures from your Rust source code without requiring compilation or runtime execution.

## Features

- 🚀 **Zero Runtime Dependencies**: Pure static analysis - no need to compile or run your project
- 🎯 **Multi-Framework Support**: Works with Axum and Actix-Web frameworks
- 📝 **OpenAPI 3.0 Compliant**: Generates standard-compliant documentation
- 🔄 **Multiple Output Formats**: Supports both YAML and JSON output
- 🧩 **Type Resolution**: Automatically resolves Rust types and generates schemas
- 🎨 **Serde Integration**: Respects Serde attributes like `rename`, `skip`, and `flatten`
- 📊 **Progress Logging**: Detailed progress information and error reporting

## Supported Frameworks

- **Axum**: Extracts routes from `Router::new()`, `.route()`, `.get()`, `.post()`, etc.
- **Actix-Web**: Extracts routes from `#[get]`, `#[post]`, and other route macros

## Installation

### From Source

```bash
git clone https://github.com/paxoscn/openapi-generator.git
cd openapi-generator
cargo build --release
```

The binary will be available at `target/release/openapi-generator`.

### Using Cargo Install

```bash
cargo install openapi-generator
```

## Usage

### Basic Usage

Generate OpenAPI documentation for your Rust project:

```bash
openapi-generator /path/to/your/project
```

This will output the OpenAPI document in YAML format to stdout.

### Command-Line Options

```
Usage: openapi-generator [OPTIONS] <PROJECT_PATH>

Arguments:
  <PROJECT_PATH>  Path to the Rust project directory

Options:
  -f, --format <FORMAT>      Output format (yaml or json) [default: yaml]
  -o, --output <FILE>        Output file path (if not specified, outputs to stdout)
  -w, --framework <FRAMEWORK> Specify the web framework to parse (if not specified, auto-detect)
                             [possible values: axum, actix-web]
  -v, --verbose              Enable verbose output
  -h, --help                 Print help
  -V, --version              Print version
```

### Examples

#### Generate YAML output to a file

```bash
openapi-generator ./my-api-project -o openapi.yaml
```

#### Generate JSON output

```bash
openapi-generator ./my-api-project -f json -o openapi.json
```

#### Specify framework explicitly

```bash
openapi-generator ./my-api-project -w axum -o openapi.yaml
```

#### Enable verbose logging

```bash
openapi-generator ./my-api-project -v
```

#### Output to stdout and pipe to another tool

```bash
openapi-generator ./my-api-project | yq eval '.'
```

## How It Works

1. **File Scanning**: Recursively scans the project directory for `.rs` files
2. **AST Parsing**: Parses Rust source files into Abstract Syntax Trees using the `syn` crate
3. **Framework Detection**: Automatically detects which web framework(s) are used
4. **Route Extraction**: Extracts route definitions, HTTP methods, and handler functions
5. **Type Resolution**: Analyzes data structures used in request/response types
6. **Schema Generation**: Converts Rust types to OpenAPI schemas
7. **Document Building**: Constructs a complete OpenAPI 3.0 document
8. **Serialization**: Outputs the document in YAML or JSON format

## Supported Route Patterns

### Axum

```rust
use axum::{Router, routing::{get, post}, Json, extract::Path};

// Simple routes
let app = Router::new()
    .route("/users", get(list_users))
    .route("/users", post(create_user));

// Path parameters
let app = Router::new()
    .route("/users/:id", get(get_user));

// Nested routes
let app = Router::new()
    .nest("/api", api_routes());

// Extractors
async fn create_user(Json(payload): Json<CreateUserRequest>) -> Json<User> {
    // ...
}
```

### Actix-Web

```rust
use actix_web::{get, post, web, HttpResponse};

// Route macros
#[get("/users")]
async fn list_users() -> HttpResponse {
    // ...
}

#[post("/users")]
async fn create_user(user: web::Json<CreateUserRequest>) -> HttpResponse {
    // ...
}

// Path parameters
#[get("/users/{id}")]
async fn get_user(path: web::Path<i32>) -> HttpResponse {
    // ...
}

// Scopes
web::scope("/api")
    .service(list_users)
    .service(create_user)
```

## Type Resolution

The tool automatically resolves Rust types and generates appropriate OpenAPI schemas:

- **Primitive types**: `String`, `i32`, `bool`, etc. → OpenAPI primitive types
- **Collections**: `Vec<T>` → array schemas
- **Options**: `Option<T>` → marks fields as non-required
- **Custom structs**: Generates schema definitions with references
- **Serde attributes**: Respects `#[serde(rename)]`, `#[serde(skip)]`, `#[serde(flatten)]`

### Example

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    #[serde(rename = "email_address")]
    pub email: String,
    pub age: Option<i32>,
    #[serde(skip)]
    pub password_hash: String,
}
```

This generates an OpenAPI schema with:
- `id` as integer
- `name` as string
- `email_address` as string (renamed)
- `age` as optional integer
- `password_hash` excluded from schema

## Limitations

- **Static Analysis Only**: Cannot handle dynamically generated routes
- **No Type Inference**: Requires explicit type annotations on handler functions
- **Macro Expansion**: Does not expand complex custom macros
- **Response Types**: May not accurately infer all response types

## Troubleshooting

### No routes found

If the tool reports no routes found:
- Ensure your project uses supported frameworks (Axum or Actix-Web)
- Check that route definitions follow standard patterns
- Try specifying the framework explicitly with `-w`
- Enable verbose mode with `-v` to see detailed parsing information

### Parse errors

If files fail to parse:
- Ensure your code compiles successfully with `cargo check`
- The tool will skip unparseable files and continue with others
- Check verbose output to see which files are being skipped

### Missing type definitions

If schemas are incomplete:
- Ensure types used in handlers are defined in the same project
- The tool may use placeholder schemas for unresolvable types
- Check that Serde derives are present on data structures

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [syn](https://github.com/dtolnay/syn) for Rust parsing
- Uses [clap](https://github.com/clap-rs/clap) for CLI
- Serialization powered by [serde](https://github.com/serde-rs/serde)
