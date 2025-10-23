use crate::cli::Framework;
use crate::parser::ParsedFile;
use log::debug;
use std::collections::HashSet;
use syn::{Item, UseTree};

/// Framework detector for identifying web frameworks used in a Rust project.
///
/// The `FrameworkDetector` analyzes parsed Rust files to automatically detect which
/// web frameworks are being used. It does this by examining `use` statements for
/// framework-specific imports.
///
/// Currently supports detection of:
/// - Axum (via `use axum::...`)
/// - Actix-Web (via `use actix_web::...`)
pub struct FrameworkDetector;

/// Result of framework detection.
///
/// Contains the list of all detected web frameworks in the project.
pub struct DetectionResult {
    /// List of detected frameworks
    pub frameworks: Vec<Framework>,
}

impl FrameworkDetector {
    /// Detects web frameworks used in the provided parsed files.
    ///
    /// This method scans all `use` statements in the parsed files to identify
    /// framework imports. Multiple frameworks can be detected if the project
    /// uses more than one.
    ///
    /// # Arguments
    ///
    /// * `parsed_files` - Slice of successfully parsed Rust files to analyze
    ///
    /// # Returns
    ///
    /// Returns a `DetectionResult` containing all detected frameworks.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use openapi_from_source::detector::FrameworkDetector;
    /// use openapi_from_source::parser::AstParser;
    /// use std::path::Path;
    ///
    /// let parsed = AstParser::parse_file(Path::new("src/main.rs")).unwrap();
    /// let result = FrameworkDetector::detect(&[parsed]);
    /// println!("Detected {} framework(s)", result.frameworks.len());
    /// ```
    pub fn detect(parsed_files: &[ParsedFile]) -> DetectionResult {
        debug!("Detecting frameworks in {} files", parsed_files.len());
        
        let mut detected_frameworks = HashSet::new();
        
        for parsed_file in parsed_files {
            // Check each item in the syntax tree
            for item in &parsed_file.syntax_tree.items {
                if let Item::Use(use_item) = item {
                    Self::check_use_tree(&use_item.tree, &mut detected_frameworks);
                }
            }
        }
        
        let frameworks: Vec<Framework> = detected_frameworks.into_iter().collect();
        debug!("Detected frameworks: {:?}", frameworks);
        
        DetectionResult { frameworks }
    }
    
    /// Recursively check use tree for framework imports
    fn check_use_tree(tree: &UseTree, detected: &mut HashSet<Framework>) {
        match tree {
            UseTree::Path(path) => {
                let ident = path.ident.to_string();
                
                // Check for axum
                if ident == "axum" {
                    detected.insert(Framework::Axum);
                }
                
                // Check for actix_web
                if ident == "actix_web" {
                    detected.insert(Framework::ActixWeb);
                }
                
                // Recursively check the rest of the path
                Self::check_use_tree(&path.tree, detected);
            }
            UseTree::Group(group) => {
                // Check all items in the group
                for item in &group.items {
                    Self::check_use_tree(item, detected);
                }
            }
            UseTree::Rename(rename) => {
                // Check the original name
                let ident = rename.ident.to_string();
                if ident == "axum" {
                    detected.insert(Framework::Axum);
                }
                if ident == "actix_web" {
                    detected.insert(Framework::ActixWeb);
                }
            }
            UseTree::Name(name) => {
                // Check the name
                let ident = name.ident.to_string();
                if ident == "axum" {
                    detected.insert(Framework::Axum);
                }
                if ident == "actix_web" {
                    detected.insert(Framework::ActixWeb);
                }
            }
            UseTree::Glob(_) => {
                // Glob imports don't help us identify the framework
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::AstParser;
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

    /// Helper function to parse a file and return ParsedFile
    fn parse_test_file(dir: &TempDir, name: &str, content: &str) -> ParsedFile {
        let file_path = create_temp_file(dir, name, content);
        AstParser::parse_file(&file_path).unwrap()
    }

    #[test]
    fn test_detect_axum_framework() {
        let temp_dir = TempDir::new().unwrap();
        
        let axum_code = r#"
            use axum::{Router, routing::get};
            use axum::extract::Path;
            
            pub async fn hello() -> &'static str {
                "Hello, World!"
            }
            
            pub fn app() -> Router {
                Router::new().route("/", get(hello))
            }
        "#;
        
        let parsed = parse_test_file(&temp_dir, "axum.rs", axum_code);
        let result = FrameworkDetector::detect(&[parsed]);
        
        assert_eq!(result.frameworks.len(), 1);
        assert!(result.frameworks.contains(&Framework::Axum));
    }

    #[test]
    fn test_detect_actix_web_framework() {
        let temp_dir = TempDir::new().unwrap();
        
        let actix_code = r#"
            use actix_web::{web, App, HttpResponse, HttpServer};
            use actix_web::middleware::Logger;
            
            #[actix_web::get("/")]
            async fn hello() -> HttpResponse {
                HttpResponse::Ok().body("Hello, World!")
            }
            
            pub fn main() {
                HttpServer::new(|| {
                    App::new().service(hello)
                })
            }
        "#;
        
        let parsed = parse_test_file(&temp_dir, "actix.rs", actix_code);
        let result = FrameworkDetector::detect(&[parsed]);
        
        assert_eq!(result.frameworks.len(), 1);
        assert!(result.frameworks.contains(&Framework::ActixWeb));
    }

    #[test]
    fn test_detect_mixed_frameworks() {
        let temp_dir = TempDir::new().unwrap();
        
        let axum_code = r#"
            use axum::Router;
            
            pub fn axum_app() -> Router {
                Router::new()
            }
        "#;
        
        let actix_code = r#"
            use actix_web::{web, App};
            
            pub fn actix_app() -> App {
                App::new()
            }
        "#;
        
        let parsed_axum = parse_test_file(&temp_dir, "axum.rs", axum_code);
        let parsed_actix = parse_test_file(&temp_dir, "actix.rs", actix_code);
        
        let result = FrameworkDetector::detect(&[parsed_axum, parsed_actix]);
        
        assert_eq!(result.frameworks.len(), 2);
        assert!(result.frameworks.contains(&Framework::Axum));
        assert!(result.frameworks.contains(&Framework::ActixWeb));
    }

    #[test]
    fn test_detect_no_framework() {
        let temp_dir = TempDir::new().unwrap();
        
        let plain_code = r#"
            use std::collections::HashMap;
            use serde::{Serialize, Deserialize};
            
            #[derive(Serialize, Deserialize)]
            pub struct User {
                pub id: u32,
                pub name: String,
            }
            
            pub fn process_user(user: User) -> User {
                user
            }
        "#;
        
        let parsed = parse_test_file(&temp_dir, "plain.rs", plain_code);
        let result = FrameworkDetector::detect(&[parsed]);
        
        assert_eq!(result.frameworks.len(), 0);
    }

    #[test]
    fn test_detect_with_grouped_imports() {
        let temp_dir = TempDir::new().unwrap();
        
        let grouped_code = r#"
            use axum::{
                Router,
                routing::{get, post},
                extract::{Path, Query},
            };
            
            pub fn app() -> Router {
                Router::new()
            }
        "#;
        
        let parsed = parse_test_file(&temp_dir, "grouped.rs", grouped_code);
        let result = FrameworkDetector::detect(&[parsed]);
        
        assert_eq!(result.frameworks.len(), 1);
        assert!(result.frameworks.contains(&Framework::Axum));
    }

    #[test]
    fn test_detect_with_renamed_import() {
        let temp_dir = TempDir::new().unwrap();
        
        let renamed_code = r#"
            use actix_web as actix;
            use actix::web;
            
            pub fn handler() {}
        "#;
        
        let parsed = parse_test_file(&temp_dir, "renamed.rs", renamed_code);
        let result = FrameworkDetector::detect(&[parsed]);
        
        assert_eq!(result.frameworks.len(), 1);
        assert!(result.frameworks.contains(&Framework::ActixWeb));
    }

    #[test]
    fn test_detect_multiple_files_same_framework() {
        let temp_dir = TempDir::new().unwrap();
        
        let code1 = r#"
            use axum::Router;
        "#;
        
        let code2 = r#"
            use axum::routing::get;
        "#;
        
        let code3 = r#"
            use axum::extract::Path;
        "#;
        
        let parsed1 = parse_test_file(&temp_dir, "file1.rs", code1);
        let parsed2 = parse_test_file(&temp_dir, "file2.rs", code2);
        let parsed3 = parse_test_file(&temp_dir, "file3.rs", code3);
        
        let result = FrameworkDetector::detect(&[parsed1, parsed2, parsed3]);
        
        // Should only detect Axum once, not three times
        assert_eq!(result.frameworks.len(), 1);
        assert!(result.frameworks.contains(&Framework::Axum));
    }

    #[test]
    fn test_detect_empty_file_list() {
        let result = FrameworkDetector::detect(&[]);
        
        assert_eq!(result.frameworks.len(), 0);
    }

    #[test]
    fn test_detect_with_nested_use_paths() {
        let temp_dir = TempDir::new().unwrap();
        
        let nested_code = r#"
            use actix_web::web::Json;
            use actix_web::middleware::Logger;
            
            pub fn handler() {}
        "#;
        
        let parsed = parse_test_file(&temp_dir, "nested.rs", nested_code);
        let result = FrameworkDetector::detect(&[parsed]);
        
        assert_eq!(result.frameworks.len(), 1);
        assert!(result.frameworks.contains(&Framework::ActixWeb));
    }

    #[test]
    fn test_detect_with_glob_imports() {
        let temp_dir = TempDir::new().unwrap();
        
        let glob_code = r#"
            use axum::*;
            use axum::routing::*;
            
            pub fn app() {}
        "#;
        
        let parsed = parse_test_file(&temp_dir, "glob.rs", glob_code);
        let result = FrameworkDetector::detect(&[parsed]);
        
        assert_eq!(result.frameworks.len(), 1);
        assert!(result.frameworks.contains(&Framework::Axum));
    }
}
