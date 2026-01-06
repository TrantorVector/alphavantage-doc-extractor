//! Utility modules - Shared infrastructure
//!
//! Common utilities used across the application layers.

pub mod error;
pub mod logging;
pub mod validation;

// Re-export error types for convenience
pub use error::{
    ContentExtractionError, ExtractionError, ExtractionResult, NetworkError, NetworkResult,
    ParseError, ParseResult, RenderError, RenderResult, Result, UrlParseError,
};

// Re-export logging utilities
pub use logging::{init_logging, parse_log_level, Timer};
