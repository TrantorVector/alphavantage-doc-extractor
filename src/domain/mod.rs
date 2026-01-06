//! Domain layer - Pure business logic
//!
//! This module contains the core business logic that is independent of
//! external systems. Domain models, parsing, extraction, transformation,
//! and rendering logic live here.
//!
//! Domain code MUST NOT import from adapters, ports, or external crates
//! (except serde/thiserror for serialization and error definitions).

pub mod extractor;
pub mod models;
pub mod parser;
pub mod renderer;
pub mod transformer;

// Re-export commonly used domain types
pub use models::{
    ApiCategory, ApiEndpoint, CodeExample, DocumentMetadata, DocumentStructure, OutputMetadata,
    Parameter, ParsedDocument, RawHtml, RawUrl, ValidatedUrl,
};
