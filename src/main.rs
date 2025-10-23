//! Rust OpenAPI Generator - Command-line tool for generating OpenAPI documentation.
//!
//! This binary provides a command-line interface for automatically generating OpenAPI 3.0
//! documentation from Rust web projects. It analyzes your source code to extract route
//! definitions and type information, then generates a complete OpenAPI specification.
//!
//! # Usage
//!
//! ```bash
//! openapi-generator [OPTIONS] <PROJECT_PATH>
//! ```
//!
//! # Examples
//!
//! Generate YAML documentation:
//! ```bash
//! openapi-generator ./my-api-project -o openapi.yaml
//! ```
//!
//! Generate JSON documentation:
//! ```bash
//! openapi-generator ./my-api-project -f json -o openapi.json
//! ```
//!
//! Enable verbose logging:
//! ```bash
//! openapi-generator ./my-api-project -v
//! ```

mod cli;
mod scanner;
mod parser;
mod detector;
mod extractor;
mod type_resolver;
mod schema_generator;
mod openapi_builder;
mod serializer;
mod error;

use anyhow::Result;
use clap::Parser;
use log::info;

fn main() -> Result<()> {
    // We need to parse args twice: once to get verbose flag, then again after logger init
    // First, do a quick parse just to check for verbose flag
    let args_for_verbose = cli::CliArgs::parse();
    
    // Initialize logger based on verbose flag
    let log_level = if args_for_verbose.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    
    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .init();

    info!("Rust OpenAPI Generator starting...");

    // Now do the full parse with validation
    let args = cli::parse_args_from_parsed(args_for_verbose)?;

    // Run the main workflow
    cli::run(args)?;

    info!("OpenAPI document generation completed successfully");

    Ok(())
}
