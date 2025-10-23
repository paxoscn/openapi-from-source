use anyhow::Result;
use clap::{Parser, ValueEnum};
use log::{debug, info};
use std::path::PathBuf;

/// Rust OpenAPI Generator - Automatically generate OpenAPI documentation from Rust web projects
#[derive(Parser, Debug)]
#[command(name = "openapi-from-source")]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Path to the Rust project directory
    #[arg(value_name = "PROJECT_PATH")]
    pub project_path: PathBuf,

    /// Output format (yaml or json)
    #[arg(short = 'f', long = "format", value_enum, default_value = "yaml")]
    pub output_format: OutputFormat,

    /// Output file path (if not specified, outputs to stdout)
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    pub output_path: Option<PathBuf>,

    /// Specify the web framework to parse (if not specified, auto-detect)
    #[arg(short = 'w', long = "framework", value_enum)]
    pub framework: Option<Framework>,

    /// Enable verbose output
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,
}

/// Output format options
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    /// YAML format
    Yaml,
    /// JSON format
    Json,
}

/// Supported web frameworks
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Hash)]
pub enum Framework {
    /// Axum framework
    Axum,
    /// Actix-Web framework
    #[value(name = "actix-web")]
    ActixWeb,
}

/// Parse command line arguments
pub fn parse_args() -> Result<CliArgs> {
    let args = CliArgs::parse();
    parse_args_from_parsed(args)
}

/// Validate and log already-parsed arguments
pub fn parse_args_from_parsed(args: CliArgs) -> Result<CliArgs> {
    debug!("Parsed arguments: {:?}", args);

    // Validate project path exists
    if !args.project_path.exists() {
        anyhow::bail!(
            "Project path does not exist: {}",
            args.project_path.display()
        );
    }

    // Validate project path is a directory
    if !args.project_path.is_dir() {
        anyhow::bail!(
            "Project path is not a directory: {}",
            args.project_path.display()
        );
    }

    info!("Project path: {}", args.project_path.display());
    info!("Output format: {:?}", args.output_format);
    if let Some(ref output) = args.output_path {
        info!("Output file: {}", output.display());
    } else {
        info!("Output: stdout");
    }
    if let Some(ref framework) = args.framework {
        info!("Framework: {:?}", framework);
    } else {
        info!("Framework: auto-detect");
    }

    Ok(args)
}

/// Run the main workflow
pub fn run(args: CliArgs) -> Result<()> {
    use crate::detector::{DetectionResult, FrameworkDetector};
    use crate::extractor::actix::ActixExtractor;
    use crate::extractor::axum::AxumExtractor;
    use crate::extractor::{HttpMethod, RouteExtractor, RouteInfo};
    use crate::openapi_builder::OpenApiBuilder;
    use crate::parser::{AstParser, ParsedFile};
    use crate::scanner::FileScanner;
    use crate::schema_generator::SchemaGenerator;
    use crate::serializer::{serialize_json, serialize_yaml, write_to_file};
    use crate::type_resolver::TypeResolver;
    
    // Helper function to convert HTTP method to string
    let method_str = |method: &HttpMethod| -> &str {
        match method {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Head => "HEAD",
        }
    };
    
    info!("Starting OpenAPI document generation...");
    info!("Project path: {}", args.project_path.display());
    
    // Step 1: Scan directory for Rust files
    info!("Scanning project directory...");
    let scanner = FileScanner::new(args.project_path.clone());
    let scan_result = scanner.scan()?;
    
    info!("Found {} Rust files", scan_result.rust_files.len());
    if !scan_result.warnings.is_empty() {
        for warning in &scan_result.warnings {
            log::warn!("{}", warning);
        }
    }
    
    if scan_result.rust_files.is_empty() {
        anyhow::bail!("No Rust files found in the project directory");
    }
    
    // Step 2: Parse files into AST
    info!("Parsing Rust files...");
    let parse_results = AstParser::parse_files(&scan_result.rust_files);
    
    let parsed_files: Vec<ParsedFile> = parse_results
        .into_iter()
        .filter_map(|r| {
            match r {
                Ok(parsed) => Some(parsed),
                Err(e) => {
                    debug!("Skipping file due to parse error: {}", e);
                    None
                }
            }
        })
        .collect();
    
    info!("Successfully parsed {} files", parsed_files.len());
    
    if parsed_files.is_empty() {
        anyhow::bail!("No files could be parsed successfully");
    }
    
    // Step 3: Detect framework (or use user-specified framework)
    let frameworks = if let Some(framework) = args.framework {
        info!("Using user-specified framework: {:?}", framework);
        vec![framework]
    } else {
        info!("Detecting web frameworks...");
        let detection_result: DetectionResult = FrameworkDetector::detect(&parsed_files);
        
        if detection_result.frameworks.is_empty() {
            anyhow::bail!(
                "No supported web framework detected. Please specify a framework using --framework option.\n\
                 Supported frameworks: axum, actix-web"
            );
        }
        
        info!("Detected frameworks: {:?}", detection_result.frameworks);
        detection_result.frameworks
    };
    
    // Step 4: Extract routes using appropriate extractors
    info!("Extracting routes...");
    let mut all_routes: Vec<RouteInfo> = Vec::new();
    
    for framework in &frameworks {
        debug!("Extracting routes for framework: {:?}", framework);
        
        let extractor: Box<dyn RouteExtractor> = match framework {
            Framework::Axum => Box::new(AxumExtractor),
            Framework::ActixWeb => Box::new(ActixExtractor),
        };
        
        // Extract routes from all files at once (extractor needs access to all functions)
        let routes = extractor.extract_routes(&parsed_files);
        debug!("Extracted {} routes for {:?}", routes.len(), framework);
        all_routes.extend(routes);
    }
    
    info!("Extracted {} total routes", all_routes.len());
    
    if all_routes.is_empty() {
        log::warn!("No routes found in the project");
    }
    
    // Step 5: Initialize type resolver and schema generator
    info!("Initializing type resolver...");
    let type_resolver = TypeResolver::new(parsed_files);
    let mut schema_gen = SchemaGenerator::new(type_resolver);
    
    // Step 6: Build OpenAPI document
    info!("Building OpenAPI document...");
    let mut builder = OpenApiBuilder::new();
    
    for route in &all_routes {
        debug!("Adding route: {} {}", method_str(&route.method), route.path);
        builder.add_route(route, &mut schema_gen);
    }
    
    let document = builder.build(schema_gen);
    info!("OpenAPI document built successfully");
    
    // Step 7: Serialize to requested format
    info!("Serializing to {:?} format...", args.output_format);
    let content = match args.output_format {
        OutputFormat::Yaml => serialize_yaml(&document)?,
        OutputFormat::Json => serialize_json(&document)?,
    };
    
    // Step 8: Output to file or stdout
    if let Some(output_path) = &args.output_path {
        info!("Writing output to: {}", output_path.display());
        write_to_file(&content, output_path)?;
        info!("Successfully wrote OpenAPI document to {}", output_path.display());
    } else {
        println!("{}", content);
    }
    
    // Step 9: Display summary
    info!("Generation complete!");
    info!("Summary:");
    info!("  - Files scanned: {}", scan_result.rust_files.len());
    info!("  - Files parsed: {}", all_routes.len());
    info!("  - Routes found: {}", all_routes.len());
    info!("  - Frameworks: {:?}", frameworks);
    
    Ok(())
}


