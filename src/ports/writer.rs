//! Port for file writing operations.
//!
//! This module defines the Writer trait that abstracts file system operations,
//! allowing the domain layer to write content to files without knowing the
//! specific implementation details.
//!
//! The trait supports atomic writes, backup creation, and path validation.

use crate::utils::ExtractionError;

/// Port trait for writing content to files.
///
/// Abstracts file system operations to enable different implementations
/// (filesystem, in-memory, cloud storage, etc.) while maintaining
/// the same interface for the domain layer.
pub trait Writer {
    /// Write content to a file at the specified path.
    ///
    /// # Arguments
    /// * `path` - The file path to write to
    /// * `content` - The content to write
    ///
    /// # Returns
    /// * `Ok(())` if the write was successful
    /// * `Err(ExtractionError)` if the write failed
    fn write(&self, path: &str, content: &str) -> Result<(), ExtractionError>;
}
