//! Adapters layer - External interface implementations
//!
//! This module contains concrete implementations of ports that interface
//! with external systems (CLI, HTTP, filesystem, etc.).

pub mod cli;
pub mod file_writer;
pub mod http_client;

// Re-export commonly used adapters
pub use cli::Config;
pub use file_writer::FileWriter;
pub use http_client::HttpClient;
