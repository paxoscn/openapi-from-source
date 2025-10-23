use anyhow::Result;
use log::warn;
use std::path::PathBuf;
use walkdir::WalkDir;

/// File scanner for traversing project directories.
///
/// The `FileScanner` recursively walks through a project directory to find all Rust source files.
/// It automatically skips common directories that should be ignored, such as `target` and hidden
/// directories (those starting with `.`).
///
/// # Example
///
/// ```no_run
/// use rust_openapi_generator::scanner::FileScanner;
/// use std::path::PathBuf;
///
/// let scanner = FileScanner::new(PathBuf::from("./my-project"));
/// let result = scanner.scan().unwrap();
/// println!("Found {} Rust files", result.rust_files.len());
/// ```
pub struct FileScanner {
    root_path: PathBuf,
}

/// Result of directory scanning operation.
///
/// Contains the list of discovered Rust files and any warnings encountered during scanning.
pub struct ScanResult {
    /// List of paths to all discovered `.rs` files
    pub rust_files: Vec<PathBuf>,
    /// Warning messages for any issues encountered (e.g., inaccessible directories)
    pub warnings: Vec<String>,
}

impl FileScanner {
    /// Creates a new `FileScanner` for the specified root directory.
    ///
    /// # Arguments
    ///
    /// * `root_path` - The root directory to scan for Rust files
    pub fn new(root_path: PathBuf) -> Self {
        Self { root_path }
    }

    /// Scans the directory tree and collects all `.rs` files.
    ///
    /// This method recursively traverses the directory tree starting from the root path,
    /// collecting all files with the `.rs` extension. It automatically skips:
    /// - The `target` directory (build artifacts)
    /// - Hidden directories (starting with `.`)
    ///
    /// If any directories or files cannot be accessed, warnings are logged and added to
    /// the result, but scanning continues.
    ///
    /// # Returns
    ///
    /// Returns a `ScanResult` containing the list of discovered files and any warnings.
    ///
    /// # Errors
    ///
    /// Returns an error if the root directory cannot be accessed.
    pub fn scan(&self) -> Result<ScanResult> {
        let mut rust_files = Vec::new();
        let mut warnings = Vec::new();

        for entry in WalkDir::new(&self.root_path)
            .into_iter()
            .filter_entry(|e| {
                // Don't filter the root directory itself
                if e.path() == self.root_path {
                    return true;
                }
                
                // Skip target directory and hidden directories
                let file_name = e.file_name().to_string_lossy();
                let is_hidden = file_name.starts_with('.');
                let is_target = file_name == "target";
                
                !is_hidden && !is_target
            })
        {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    
                    // Check if it's a .rs file
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
                        rust_files.push(path.to_path_buf());
                    }
                }
                Err(e) => {
                    // Record warning for inaccessible directories/files
                    let warning = format!("Failed to access path: {}", e);
                    warn!("{}", warning);
                    warnings.push(warning);
                }
            }
        }

        Ok(ScanResult {
            rust_files,
            warnings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scan_normal_directory() {
        // Create temporary test directory structure
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create test files
        fs::write(root.join("main.rs"), "fn main() {}").unwrap();
        fs::write(root.join("lib.rs"), "pub fn test() {}").unwrap();
        fs::write(root.join("readme.md"), "# README").unwrap();

        // Scan directory
        let scanner = FileScanner::new(root.to_path_buf());
        let result = scanner.scan().unwrap();

        // Verify results
        assert_eq!(result.rust_files.len(), 2);
        assert!(result.warnings.is_empty());
        
        let file_names: Vec<String> = result
            .rust_files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        
        assert!(file_names.contains(&"main.rs".to_string()));
        assert!(file_names.contains(&"lib.rs".to_string()));
    }

    #[test]
    fn test_scan_empty_directory() {
        // Create empty temporary directory
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Scan directory
        let scanner = FileScanner::new(root.to_path_buf());
        let result = scanner.scan().unwrap();

        // Verify results
        assert_eq!(result.rust_files.len(), 0);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_scan_nested_directories() {
        // Create temporary test directory structure with nested directories
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create nested structure
        fs::create_dir(root.join("src")).unwrap();
        fs::create_dir(root.join("src/models")).unwrap();
        fs::create_dir(root.join("tests")).unwrap();

        // Create test files
        fs::write(root.join("main.rs"), "fn main() {}").unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn test() {}").unwrap();
        fs::write(root.join("src/models/user.rs"), "struct User {}").unwrap();
        fs::write(root.join("tests/integration.rs"), "#[test] fn test() {}").unwrap();

        // Scan directory
        let scanner = FileScanner::new(root.to_path_buf());
        let result = scanner.scan().unwrap();

        // Verify results
        assert_eq!(result.rust_files.len(), 4);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_scan_skips_target_directory() {
        // Create temporary test directory structure
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create target directory with files
        fs::create_dir(root.join("target")).unwrap();
        fs::write(root.join("target/build.rs"), "fn main() {}").unwrap();
        
        // Create normal file
        fs::write(root.join("main.rs"), "fn main() {}").unwrap();

        // Scan directory
        let scanner = FileScanner::new(root.to_path_buf());
        let result = scanner.scan().unwrap();

        // Verify results - should only find main.rs, not target/build.rs
        assert_eq!(result.rust_files.len(), 1);
        assert!(result.warnings.is_empty());
        assert_eq!(
            result.rust_files[0].file_name().unwrap().to_string_lossy(),
            "main.rs"
        );
    }

    #[test]
    fn test_scan_skips_hidden_directories() {
        // Create temporary test directory structure
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create hidden directory with files
        fs::create_dir(root.join(".git")).unwrap();
        fs::write(root.join(".git/config.rs"), "// config").unwrap();
        
        // Create normal file
        fs::write(root.join("main.rs"), "fn main() {}").unwrap();

        // Scan directory
        let scanner = FileScanner::new(root.to_path_buf());
        let result = scanner.scan().unwrap();

        // Verify results - should only find main.rs, not .git/config.rs
        assert_eq!(result.rust_files.len(), 1);
        assert!(result.warnings.is_empty());
        assert_eq!(
            result.rust_files[0].file_name().unwrap().to_string_lossy(),
            "main.rs"
        );
    }

    #[test]
    fn test_scan_filters_non_rust_files() {
        // Create temporary test directory structure
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create various file types
        fs::write(root.join("main.rs"), "fn main() {}").unwrap();
        fs::write(root.join("readme.md"), "# README").unwrap();
        fs::write(root.join("config.toml"), "[package]").unwrap();
        fs::write(root.join("script.sh"), "#!/bin/bash").unwrap();

        // Scan directory
        let scanner = FileScanner::new(root.to_path_buf());
        let result = scanner.scan().unwrap();

        // Verify results - should only find .rs files
        assert_eq!(result.rust_files.len(), 1);
        assert!(result.warnings.is_empty());
        assert_eq!(
            result.rust_files[0].file_name().unwrap().to_string_lossy(),
            "main.rs"
        );
    }
}
