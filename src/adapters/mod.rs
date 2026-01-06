//! Adapters layer - External interface implementations
//!
//! This module contains concrete implementations of ports that interface
//! with external systems (CLI, HTTP, filesystem, etc.).

pub mod cli;
pub mod file_writer;
pub mod http_client;
