//! Serialization module for converting OpenAPI documents to YAML or JSON format.
//!
//! This module provides functions to serialize OpenAPI documents into standard formats
//! and write them to files or return them as strings.

use crate::openapi_builder::OpenApiDocument;
use anyhow::{Context, Result};
use log::debug;
use std::fs;
use std::path::Path;

/// Serializes an OpenAPI document to YAML format.
///
/// The output is formatted as standard YAML, suitable for use with OpenAPI tools
/// and documentation generators.
///
/// # Arguments
///
/// * `doc` - The OpenAPI document to serialize
///
/// # Returns
///
/// Returns the YAML string representation of the document.
///
/// # Errors
///
/// Returns an error if serialization fails.
///
/// # Example
///
/// ```ignore
/// use openapi_generator::openapi_builder::OpenApiBuilder;
/// use openapi_generator::serializer::serialize_yaml;
/// use openapi_generator::schema_generator::SchemaGenerator;
/// use openapi_generator::type_resolver::TypeResolver;
///
/// let builder = OpenApiBuilder::new();
/// let type_resolver = TypeResolver::new(vec![]);
/// let schema_gen = SchemaGenerator::new(type_resolver);
/// let doc = builder.build(schema_gen);
/// let yaml = serialize_yaml(&doc).unwrap();
/// println!("{}", yaml);
/// ```
pub fn serialize_yaml(doc: &OpenApiDocument) -> Result<String> {
    debug!("Serializing OpenAPI document to YAML");
    serde_yaml::to_string(doc)
        .context("Failed to serialize OpenAPI document to YAML")
}

/// Serializes an OpenAPI document to JSON format with pretty printing.
///
/// The output is formatted with indentation for readability, making it suitable
/// for human review and version control.
///
/// # Arguments
///
/// * `doc` - The OpenAPI document to serialize
///
/// # Returns
///
/// Returns the JSON string representation of the document.
///
/// # Errors
///
/// Returns an error if serialization fails.
///
/// # Example
///
/// ```ignore
/// use openapi_generator::openapi_builder::OpenApiBuilder;
/// use openapi_generator::serializer::serialize_json;
/// use openapi_generator::schema_generator::SchemaGenerator;
/// use openapi_generator::type_resolver::TypeResolver;
///
/// let builder = OpenApiBuilder::new();
/// let type_resolver = TypeResolver::new(vec![]);
/// let schema_gen = SchemaGenerator::new(type_resolver);
/// let doc = builder.build(schema_gen);
/// let json = serialize_json(&doc).unwrap();
/// println!("{}", json);
/// ```
pub fn serialize_json(doc: &OpenApiDocument) -> Result<String> {
    debug!("Serializing OpenAPI document to JSON");
    serde_json::to_string_pretty(doc)
        .context("Failed to serialize OpenAPI document to JSON")
}

/// Writes string content to a file.
///
/// Creates the file if it doesn't exist, or overwrites it if it does.
/// Parent directories are not created automatically.
///
/// # Arguments
///
/// * `content` - The string content to write
/// * `path` - The file path to write to
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// Returns an error if the file cannot be created or written to.
pub fn write_to_file(content: &str, path: &Path) -> Result<()> {
    debug!("Writing content to file: {}", path.display());
    
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    
    fs::write(path, content)
        .with_context(|| format!("Failed to write to file: {}", path.display()))?;
    
    debug!("Successfully wrote {} bytes to {}", content.len(), path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openapi_builder::{Info, OpenApiBuilder, OpenApiDocument};
    use std::collections::HashMap;
    use tempfile::TempDir;

    /// Helper function to create a minimal OpenAPI document for testing
    fn create_test_document() -> OpenApiDocument {
        OpenApiDocument {
            openapi: "3.0.0".to_string(),
            info: Info {
                title: "Test API".to_string(),
                version: "1.0.0".to_string(),
                description: Some("A test API".to_string()),
            },
            paths: HashMap::new(),
            components: None,
        }
    }

    #[test]
    fn test_serialize_yaml() {
        let doc = create_test_document();
        let result = serialize_yaml(&doc);
        
        assert!(result.is_ok());
        let yaml = result.unwrap();
        
        // Check that YAML contains expected fields
        assert!(yaml.contains("openapi:"));
        assert!(yaml.contains("3.0.0"));
        assert!(yaml.contains("info:"));
        assert!(yaml.contains("title:"));
        assert!(yaml.contains("Test API"));
        assert!(yaml.contains("version:"));
        assert!(yaml.contains("1.0.0"));
        assert!(yaml.contains("description:"));
        assert!(yaml.contains("A test API"));
        assert!(yaml.contains("paths:"));
    }

    #[test]
    fn test_serialize_json() {
        let doc = create_test_document();
        let result = serialize_json(&doc);
        
        assert!(result.is_ok());
        let json = result.unwrap();
        
        // Check that JSON contains expected fields
        assert!(json.contains("\"openapi\""));
        assert!(json.contains("\"3.0.0\""));
        assert!(json.contains("\"info\""));
        assert!(json.contains("\"title\""));
        assert!(json.contains("\"Test API\""));
        assert!(json.contains("\"version\""));
        assert!(json.contains("\"1.0.0\""));
        assert!(json.contains("\"description\""));
        assert!(json.contains("\"A test API\""));
        assert!(json.contains("\"paths\""));
        
        // Verify it's valid JSON by parsing it back
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["openapi"], "3.0.0");
        assert_eq!(parsed["info"]["title"], "Test API");
    }

    #[test]
    fn test_serialize_json_pretty_format() {
        let doc = create_test_document();
        let json = serialize_json(&doc).unwrap();
        
        // Check that JSON is pretty-printed (contains newlines and indentation)
        assert!(json.contains('\n'));
        assert!(json.contains("  ")); // Should have indentation
        
        // Count lines - pretty printed JSON should have multiple lines
        let line_count = json.lines().count();
        assert!(line_count > 5, "Pretty printed JSON should have multiple lines");
    }

    #[test]
    fn test_write_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.yaml");
        let content = "test content";
        
        let result = write_to_file(content, &file_path);
        
        assert!(result.is_ok());
        assert!(file_path.exists());
        
        let read_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_write_to_file_creates_directories() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("subdir").join("nested").join("test.yaml");
        let content = "test content";
        
        let result = write_to_file(content, &file_path);
        
        assert!(result.is_ok());
        assert!(file_path.exists());
        
        let read_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_write_to_file_overwrites_existing() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.yaml");
        
        // Write initial content
        write_to_file("initial content", &file_path).unwrap();
        
        // Overwrite with new content
        let new_content = "new content";
        let result = write_to_file(new_content, &file_path);
        
        assert!(result.is_ok());
        
        let read_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(read_content, new_content);
    }

    #[test]
    fn test_serialize_yaml_with_complex_document() {
        use crate::extractor::{HttpMethod, RouteInfo};
        use crate::parser::AstParser;
        use crate::schema_generator::SchemaGenerator;
        use crate::type_resolver::TypeResolver;
        use std::io::Write;
        
        // Create a test file with a struct
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"pub struct User { pub id: u32, pub name: String }").unwrap();
        
        // Parse and create document
        let parsed = AstParser::parse_file(&file_path).unwrap();
        let type_resolver = TypeResolver::new(vec![parsed]);
        let mut schema_gen = SchemaGenerator::new(type_resolver);
        
        let mut builder = OpenApiBuilder::new();
        let route = RouteInfo::new(
            "/users".to_string(),
            HttpMethod::Get,
            "get_users".to_string(),
        );
        builder.add_route(&route, &mut schema_gen);
        
        let doc = builder.build(schema_gen);
        let yaml = serialize_yaml(&doc).unwrap();
        
        // Verify YAML structure
        assert!(yaml.contains("openapi:"));
        assert!(yaml.contains("paths:"));
        assert!(yaml.contains("/users:"));
        assert!(yaml.contains("get:"));
    }

    #[test]
    fn test_serialize_json_with_complex_document() {
        use crate::extractor::{HttpMethod, Parameter, ParameterLocation, RouteInfo, TypeInfo};
        use crate::parser::AstParser;
        use crate::schema_generator::SchemaGenerator;
        use crate::type_resolver::TypeResolver;
        use std::io::Write;
        
        // Create a test file with a struct
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"pub struct User { pub id: u32, pub name: String }").unwrap();
        
        // Parse and create document
        let parsed = AstParser::parse_file(&file_path).unwrap();
        let type_resolver = TypeResolver::new(vec![parsed]);
        let mut schema_gen = SchemaGenerator::new(type_resolver);
        
        let mut builder = OpenApiBuilder::new();
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
        
        let doc = builder.build(schema_gen);
        let json = serialize_json(&doc).unwrap();
        
        // Verify JSON structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["openapi"], "3.0.0");
        assert!(parsed["paths"].is_object());
        assert!(parsed["paths"]["/users/{id}"].is_object());
        assert!(parsed["paths"]["/users/{id}"]["get"].is_object());
    }

    #[test]
    fn test_roundtrip_yaml_serialization() {
        let doc = create_test_document();
        let yaml = serialize_yaml(&doc).unwrap();
        
        // Deserialize back
        let deserialized: OpenApiDocument = serde_yaml::from_str(&yaml).unwrap();
        
        assert_eq!(deserialized.openapi, doc.openapi);
        assert_eq!(deserialized.info.title, doc.info.title);
        assert_eq!(deserialized.info.version, doc.info.version);
        assert_eq!(deserialized.info.description, doc.info.description);
    }

    #[test]
    fn test_roundtrip_json_serialization() {
        let doc = create_test_document();
        let json = serialize_json(&doc).unwrap();
        
        // Deserialize back
        let deserialized: OpenApiDocument = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.openapi, doc.openapi);
        assert_eq!(deserialized.info.title, doc.info.title);
        assert_eq!(deserialized.info.version, doc.info.version);
        assert_eq!(deserialized.info.description, doc.info.description);
    }

    #[test]
    fn test_write_yaml_file_end_to_end() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("openapi.yaml");
        
        let doc = create_test_document();
        let yaml = serialize_yaml(&doc).unwrap();
        
        write_to_file(&yaml, &file_path).unwrap();
        
        // Read back and verify
        let content = fs::read_to_string(&file_path).unwrap();
        let deserialized: OpenApiDocument = serde_yaml::from_str(&content).unwrap();
        
        assert_eq!(deserialized.info.title, "Test API");
    }

    #[test]
    fn test_write_json_file_end_to_end() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("openapi.json");
        
        let doc = create_test_document();
        let json = serialize_json(&doc).unwrap();
        
        write_to_file(&json, &file_path).unwrap();
        
        // Read back and verify
        let content = fs::read_to_string(&file_path).unwrap();
        let deserialized: OpenApiDocument = serde_json::from_str(&content).unwrap();
        
        assert_eq!(deserialized.info.title, "Test API");
    }
}
