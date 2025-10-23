use anyhow::{Context, Result};
use log::{debug, warn};
use std::fs;
use std::path::{Path, PathBuf};

/// AST (Abstract Syntax Tree) parser for Rust source files.
///
/// The `AstParser` uses the `syn` crate to parse Rust source code into an abstract syntax tree,
/// which can then be analyzed to extract route definitions, type information, and other metadata.
///
/// # Example
///
/// ```no_run
/// use rust_openapi_generator::parser::AstParser;
/// use std::path::Path;
///
/// let parsed = AstParser::parse_file(Path::new("src/main.rs")).unwrap();
/// println!("Parsed {} items", parsed.syntax_tree.items.len());
/// ```
pub struct AstParser;

/// A successfully parsed Rust file with its abstract syntax tree.
///
/// Contains both the original file path and the parsed syntax tree structure.
#[derive(Debug)]
pub struct ParsedFile {
    /// Path to the source file
    pub path: PathBuf,
    /// The parsed abstract syntax tree
    pub syntax_tree: syn::File,
}

impl AstParser {
    /// Parses a single Rust source file into an AST.
    ///
    /// This method reads the file content and uses `syn::parse_file` to parse it into
    /// a syntax tree. If parsing fails (e.g., due to syntax errors), an error is returned.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Rust source file to parse
    ///
    /// # Returns
    ///
    /// Returns a `ParsedFile` containing the file path and syntax tree on success.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read
    /// - The file contains invalid Rust syntax
    pub fn parse_file(path: &Path) -> Result<ParsedFile> {
        debug!("Parsing file: {}", path.display());
        
        // Read file content
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        
        // Parse the file using syn
        let syntax_tree = syn::parse_file(&content)
            .with_context(|| format!("Failed to parse Rust syntax in file: {}", path.display()))?;
        
        debug!("Successfully parsed file: {}", path.display());
        
        Ok(ParsedFile {
            path: path.to_path_buf(),
            syntax_tree,
        })
    }

    /// Parses multiple Rust source files, continuing even if some fail.
    ///
    /// This method attempts to parse all provided files, collecting both successes and failures.
    /// Files that fail to parse are logged as warnings, but parsing continues for remaining files.
    /// This allows the tool to generate partial documentation even when some files have syntax errors.
    ///
    /// # Arguments
    ///
    /// * `paths` - Slice of file paths to parse
    ///
    /// # Returns
    ///
    /// Returns a vector of `Result<ParsedFile>`, one for each input path. Successful parses
    /// contain `Ok(ParsedFile)`, while failures contain `Err` with error details.
    pub fn parse_files(paths: &[PathBuf]) -> Vec<Result<ParsedFile>> {
        debug!("Parsing {} files", paths.len());
        
        let results: Vec<Result<ParsedFile>> = paths
            .iter()
            .map(|path| {
                match Self::parse_file(path) {
                    Ok(parsed) => Ok(parsed),
                    Err(e) => {
                        warn!("Failed to parse {}: {}", path.display(), e);
                        Err(e)
                    }
                }
            })
            .collect();
        
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        let failure_count = results.len() - success_count;
        
        debug!(
            "Parsing complete: {} succeeded, {} failed",
            success_count, failure_count
        );
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper function to create a temporary file with content
    fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
        let file_path = dir.path().join(name);
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file_path
    }

    #[test]
    fn test_parse_valid_rust_file() {
        let temp_dir = TempDir::new().unwrap();
        let valid_code = r#"
            use std::collections::HashMap;

            pub struct User {
                pub id: u32,
                pub name: String,
            }

            pub fn get_user(id: u32) -> Option<User> {
                None
            }
        "#;

        let file_path = create_temp_file(&temp_dir, "valid.rs", valid_code);
        let result = AstParser::parse_file(&file_path);

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.path, file_path);
        assert!(!parsed.syntax_tree.items.is_empty());
    }

    #[test]
    fn test_parse_invalid_rust_file() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_code = r#"
            pub struct User {
                pub id: u32
                pub name: String  // Missing comma
            }
            
            fn broken( {  // Invalid syntax
                let x = ;
            }
        "#;

        let file_path = create_temp_file(&temp_dir, "invalid.rs", invalid_code);
        let result = AstParser::parse_file(&file_path);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to parse Rust syntax"));
    }

    #[test]
    fn test_parse_nonexistent_file() {
        let result = AstParser::parse_file(Path::new("/nonexistent/file.rs"));

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to read file"));
    }

    #[test]
    fn test_parse_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file(&temp_dir, "empty.rs", "");
        let result = AstParser::parse_file(&file_path);

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(parsed.syntax_tree.items.is_empty());
    }

    #[test]
    fn test_parse_files_batch() {
        let temp_dir = TempDir::new().unwrap();

        let valid_code1 = "pub fn hello() {}";
        let valid_code2 = "pub struct World;";
        let invalid_code = "pub fn broken( {";

        let file1 = create_temp_file(&temp_dir, "file1.rs", valid_code1);
        let file2 = create_temp_file(&temp_dir, "file2.rs", valid_code2);
        let file3 = create_temp_file(&temp_dir, "file3.rs", invalid_code);

        let paths = vec![file1.clone(), file2.clone(), file3.clone()];
        let results = AstParser::parse_files(&paths);

        assert_eq!(results.len(), 3);

        // First two should succeed
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());

        // Third should fail
        assert!(results[2].is_err());

        // Verify the successful parses
        assert_eq!(results[0].as_ref().unwrap().path, file1);
        assert_eq!(results[1].as_ref().unwrap().path, file2);
    }

    #[test]
    fn test_parse_files_all_valid() {
        let temp_dir = TempDir::new().unwrap();

        let code1 = "pub fn func1() {}";
        let code2 = "pub fn func2() {}";
        let code3 = "pub fn func3() {}";

        let file1 = create_temp_file(&temp_dir, "a.rs", code1);
        let file2 = create_temp_file(&temp_dir, "b.rs", code2);
        let file3 = create_temp_file(&temp_dir, "c.rs", code3);

        let paths = vec![file1, file2, file3];
        let results = AstParser::parse_files(&paths);

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn test_parse_files_all_invalid() {
        let temp_dir = TempDir::new().unwrap();

        let invalid1 = "pub fn broken( {";
        let invalid2 = "struct Missing }";
        let invalid3 = "let x = ;";

        let file1 = create_temp_file(&temp_dir, "bad1.rs", invalid1);
        let file2 = create_temp_file(&temp_dir, "bad2.rs", invalid2);
        let file3 = create_temp_file(&temp_dir, "bad3.rs", invalid3);

        let paths = vec![file1, file2, file3];
        let results = AstParser::parse_files(&paths);

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_err()));
    }

    #[test]
    fn test_parse_files_empty_list() {
        let paths: Vec<PathBuf> = vec![];
        let results = AstParser::parse_files(&paths);

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_parse_file_with_complex_syntax() {
        let temp_dir = TempDir::new().unwrap();
        let complex_code = r#"
            use serde::{Deserialize, Serialize};
            use std::collections::HashMap;

            #[derive(Debug, Serialize, Deserialize)]
            pub struct User {
                pub id: u32,
                #[serde(rename = "userName")]
                pub name: String,
                pub email: Option<String>,
            }

            impl User {
                pub fn new(id: u32, name: String) -> Self {
                    Self {
                        id,
                        name,
                        email: None,
                    }
                }
            }

            pub async fn get_users() -> Vec<User> {
                vec![]
            }
        "#;

        let file_path = create_temp_file(&temp_dir, "complex.rs", complex_code);
        let result = AstParser::parse_file(&file_path);

        assert!(result.is_ok());
        let parsed = result.unwrap();
        
        // Should have multiple items (use statements, struct, impl, function)
        assert!(parsed.syntax_tree.items.len() >= 4);
    }
}
