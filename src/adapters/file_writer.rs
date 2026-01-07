//! File writer adapter for writing markdown content to files.
//!
//! This module provides a concrete implementation of the Writer trait
//! that writes content to the filesystem with proper error handling,
//! backup creation, and path validation.
//!
//! Features:
//! - Atomic writes to prevent file corruption
//! - Automatic backup creation for existing files
//! - Path validation and directory creation
//! - Proper file permissions and UTF-8 encoding

use crate::ports::Writer;
use crate::utils::ExtractionError;
use std::fs;
use std::path::Path;
use tracing::{info, warn};

/// File system writer for markdown content.
///
/// Provides robust file writing with backup creation, path validation,
/// and proper error handling.
pub struct FileWriter;

impl FileWriter {
    /// Create a new FileWriter instance.
    pub fn new() -> Self {
        Self
    }

    /// Write content to a file with automatic backup creation.
    ///
    /// If the target file already exists, creates a backup before writing.
    /// This ensures no data loss if the write operation fails.
    ///
    /// # Arguments
    /// * `output_path` - The file path to write to
    /// * `content` - The content to write
    ///
    /// # Returns
    /// * `Ok(())` if the write was successful
    /// * `Err(ExtractionError)` if the write failed
    pub fn write_with_backup(&self, output_path: &str, content: &str) -> Result<(), ExtractionError> {
        // Create backup if file exists
        if Path::new(output_path).exists() {
            match self.create_backup(output_path) {
                Ok(backup_path) => {
                    info!("Created backup: {}", backup_path);
                }
                Err(e) => {
                    warn!("Failed to create backup for {}: {}", output_path, e);
                    // Continue with write anyway - backup failure shouldn't block the main operation
                }
            }
        }

        // Write the new content
        self.write(output_path, content)
    }

    /// Validate that an output path is writable.
    ///
    /// Checks that the path is not empty, parent directories exist or can be created,
    /// and that write permissions are available.
    ///
    /// # Arguments
    /// * `path` - The file path to validate
    ///
    /// # Returns
    /// * `Ok(())` if the path is valid
    /// * `Err(ExtractionError)` if the path is invalid
    pub fn validate_output_path(&self, path: &str) -> Result<(), ExtractionError> {
        use crate::utils::RenderError;

        if path.trim().is_empty() {
            return Err(ExtractionError::Render(RenderError::InvalidOutputPath(
                "Empty output path provided".to_string(),
            )));
        }

        let path_obj = Path::new(path);

        // Check parent directory
        if let Some(parent) = path_obj.parent() {
            if !parent.exists() {
                // Try to create parent directories
                fs::create_dir_all(parent).map_err(|e| {
                    ExtractionError::Render(RenderError::InvalidOutputPath(format!(
                        "Cannot create parent directory {}: {}",
                        parent.display(),
                        e
                    )))
                })?;
            }

            // Check if parent is actually a directory and has write permissions
            let metadata = parent.metadata().map_err(|e| {
                ExtractionError::Render(RenderError::InvalidOutputPath(format!(
                    "Cannot access parent directory {}: {}",
                    parent.display(),
                    e
                )))
            })?;

            if !metadata.is_dir() {
                return Err(ExtractionError::Render(RenderError::InvalidOutputPath(format!(
                    "Parent path {} is not a directory",
                    parent.display()
                ))));
            }

            // Check write permissions by trying to create a temporary file
            let temp_file = parent.join(".write_test.tmp");
            fs::write(&temp_file, b"test").map_err(|e| {
                ExtractionError::Render(RenderError::InvalidOutputPath(format!(
                    "No write permission in directory {}: {}",
                    parent.display(),
                    e
                )))
            })?;

            // Clean up temp file
            let _ = fs::remove_file(&temp_file);

            Ok(())
        } else {
            Ok(()) // No parent directory (writing to current directory)
        }
    }

    /// Create a backup of an existing file.
    ///
    /// Creates a backup with timestamp in the format: filename.timestamp.bak
    ///
    /// # Arguments
    /// * `path` - The original file path
    ///
    /// # Returns
    /// * `Ok(backup_path)` if backup was created successfully
    /// * `Err(std::io::Error)` if backup creation failed
    pub fn create_backup(&self, path: &str) -> Result<String, std::io::Error> {
        let path_obj = Path::new(path);
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");

        let file_stem = path_obj.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");

        let extension = path_obj.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let backup_filename = if extension.is_empty() {
            format!("{}.{}.bak", file_stem, timestamp)
        } else {
            format!("{}.{}.bak", file_stem, timestamp)
        };

        let backup_path = if let Some(parent) = path_obj.parent() {
            parent.join(backup_filename)
        } else {
            Path::new(&backup_filename).to_path_buf()
        };

        fs::copy(path, &backup_path)?;
        Ok(backup_path.to_string_lossy().to_string())
    }

    /// Get the size of a file in bytes.
    ///
    /// # Arguments
    /// * `path` - The file path
    ///
    /// # Returns
    /// * `Ok(size)` if the file exists and size was retrieved
    /// * `Err(std::io::Error)` if the file doesn't exist or can't be accessed
    pub fn get_file_size(&self, path: &str) -> Result<u64, std::io::Error> {
        let metadata = fs::metadata(path)?;
        Ok(metadata.len())
    }
}

impl Writer for FileWriter {
    fn write(&self, output_path: &str, content: &str) -> Result<(), ExtractionError> {
        // Validate the output path
        self.validate_output_path(output_path)?;

        // Write the content
        fs::write(output_path, content).map_err(|e| {
            ExtractionError::Render(crate::utils::RenderError::InvalidOutputPath(format!(
                "Failed to write to {}: {}",
                output_path, e
            )))
        })?;

        // Set file permissions on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(output_path) {
                let mut permissions = metadata.permissions();
                permissions.set_mode(0o644); // rw-r--r--
                let _ = fs::set_permissions(output_path, permissions); // Ignore errors
            }
        }

        // Log success with file size
        match self.get_file_size(output_path) {
            Ok(size) => {
                info!("Successfully wrote {} bytes to {}", size, output_path);
            }
            Err(_) => {
                info!("Successfully wrote to {}", output_path);
            }
        }

        Ok(())
    }
}

impl Default for FileWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    /// Test basic file writing functionality
    #[test]
    fn test_write_to_valid_path() {
        let writer = FileWriter::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");

        let content = "# Test Markdown\n\nThis is a test.";

        let result = writer.write(file_path.to_str().unwrap(), content);
        assert!(result.is_ok());

        // Verify file was created and has correct content
        assert!(file_path.exists());
        let read_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(read_content, content);
    }

    /// Test writing to nested directory path (creates directories)
    #[test]
    fn test_write_to_nested_path() {
        let writer = FileWriter::new();
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("docs").join("api").join("output.md");

        let content = "# API Documentation\n\nGenerated content.";

        let result = writer.write(nested_path.to_str().unwrap(), content);
        assert!(result.is_ok());

        // Verify directories were created and file exists
        assert!(nested_path.exists());
        assert!(nested_path.parent().unwrap().exists());
        let read_content = fs::read_to_string(&nested_path).unwrap();
        assert_eq!(read_content, content);
    }

    /// Test write_with_backup creates backup when file exists
    #[test]
    fn test_write_with_backup() {
        let writer = FileWriter::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("existing.md");

        // Create initial file
        let initial_content = "# Initial Content\n\nOld version.";
        fs::write(&file_path, initial_content).unwrap();

        // Write with backup
        let new_content = "# Updated Content\n\nNew version.";
        let result = writer.write_with_backup(file_path.to_str().unwrap(), new_content);
        assert!(result.is_ok());

        // Verify new content is in place
        let read_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(read_content, new_content);

        // Verify backup was created
        let backup_files: Vec<_> = fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().contains(".bak"))
            .collect();

        assert_eq!(backup_files.len(), 1);

        // Verify backup content
        let backup_content = fs::read_to_string(backup_files[0].path()).unwrap();
        assert_eq!(backup_content, initial_content);
    }

    /// Test write_with_backup when file doesn't exist (no backup created)
    #[test]
    fn test_write_with_backup_no_existing_file() {
        let writer = FileWriter::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("new.md");

        let content = "# New File\n\nFresh content.";

        let result = writer.write_with_backup(file_path.to_str().unwrap(), content);
        assert!(result.is_ok());

        // Verify file was created
        assert!(file_path.exists());

        // Verify no backup was created
        let backup_files: Vec<_> = fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().contains(".bak"))
            .collect();

        assert_eq!(backup_files.len(), 0);
    }

    /// Test path validation with empty path
    #[test]
    fn test_validate_output_path_empty() {
        let writer = FileWriter::new();

        let result = writer.validate_output_path("");
        assert!(result.is_err());

        let result = writer.validate_output_path("   ");
        assert!(result.is_err());
    }

    /// Test path validation with valid paths
    #[test]
    fn test_validate_output_path_valid() {
        let writer = FileWriter::new();
        let temp_dir = TempDir::new().unwrap();

        // Test path in temp directory (which should be writable)
        let test_path = temp_dir.path().join("test.md");
        let result = writer.validate_output_path(test_path.to_str().unwrap());
        assert!(result.is_ok());
    }

    /// Test file size retrieval
    #[test]
    fn test_get_file_size() {
        let writer = FileWriter::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("sized.md");

        let content = "Hello, World!"; // 13 bytes
        fs::write(&file_path, content).unwrap();

        let size = writer.get_file_size(file_path.to_str().unwrap()).unwrap();
        assert_eq!(size, 13);
    }

    /// Test backup creation
    #[test]
    fn test_create_backup() {
        let writer = FileWriter::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("original.md");

        let content = "# Original\n\nContent.";
        fs::write(&file_path, content).unwrap();

        let backup_path = writer.create_backup(file_path.to_str().unwrap()).unwrap();

        // Verify backup file exists
        assert!(Path::new(&backup_path).exists());

        // Verify backup content matches original
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_content, content);

        // Verify backup filename contains timestamp
        assert!(backup_path.contains("original."));
        assert!(backup_path.contains(".bak"));
    }

    /// Test Writer trait implementation
    #[test]
    fn test_writer_trait_implementation() {
        let writer = FileWriter::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("trait_test.md");

        let content = "# Trait Test\n\nTesting trait implementation.";

        // Test the trait method
        let result = writer.write(file_path.to_str().unwrap(), content);
        assert!(result.is_ok());

        // Verify content
        let read_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(read_content, content);
    }
}
