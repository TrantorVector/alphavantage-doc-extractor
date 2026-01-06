//! Error handling utilities for the Alpha Vantage Documentation Extractor.
//!
//! This module provides a comprehensive hierarchical error system using thiserror.
//! All error types implement Send + Sync for async compatibility.

use thiserror::Error;

/// Top-level error type for the extraction system.
/// This encompasses all possible failures that can occur during documentation extraction.
#[derive(Error, Debug)]
pub enum ExtractionError {
    /// Network-related failures (HTTP requests, timeouts, etc.)
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    /// HTML parsing and structure analysis failures
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    /// Content extraction failures (categories, endpoints, parameters)
    #[error("Content extraction error: {0}")]
    ContentExtraction(#[from] ContentExtractionError),

    /// Markdown rendering and validation failures
    #[error("Render error: {0}")]
    Render(#[from] RenderError),

    /// URL parsing and validation failures
    #[error("URL parse error: {0}")]
    UrlParse(#[from] UrlParseError),
}

/// Network-related errors during HTTP operations.
/// Handles connection issues, timeouts, and HTTP response problems.
#[derive(Error, Debug)]
pub enum NetworkError {
    /// Underlying HTTP request failed (connection, DNS, etc.)
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    /// HTTP error response with status code and message
    #[error("HTTP {0} error: {1}")]
    HttpError(u16, String),

    /// Failed to read response body
    #[error("Failed to read response body: {0}")]
    BodyReadFailed(String),

    /// Request timed out after {0} seconds
    #[error("Request timed out after {0} seconds")]
    Timeout(u64),

    /// Maximum retry attempts ({0}) exceeded
    #[error("Maximum retry attempts ({0}) exceeded")]
    MaxRetriesExceeded(u32),
}

/// HTML parsing and DOM analysis errors.
/// Handles malformed HTML and missing expected elements.
#[derive(Error, Debug)]
pub enum ParseError {
    /// Invalid or malformed HTML content
    #[error("Invalid HTML: {0}")]
    InvalidHtml(String),

    /// Required HTML element not found using selector
    #[error("Missing HTML element: {0}")]
    MissingElement(String),

    /// Page title element not found or empty
    #[error("Missing page title")]
    MissingTitle,

    /// Invalid CSS selector syntax
    #[error("Invalid CSS selector: {0}")]
    InvalidSelector(String),
}

/// Content extraction errors for API documentation.
/// Handles failures when extracting categories, endpoints, and parameters.
#[derive(Error, Debug)]
pub enum ContentExtractionError {
    /// No main content area found in the page
    #[error("No main content area found in the page")]
    NoMainContentFound,

    /// Failed to extract category information
    #[error("Category extraction failed for '{0}': {1}")]
    CategoryExtractionFailed(String, String),

    /// Failed to extract endpoint information
    #[error("Endpoint extraction failed for '{0}': {1}")]
    EndpointExtractionFailed(String, String),

    /// Parameter table structure is invalid or malformed
    #[error("Invalid parameter table structure: {0}")]
    InvalidParameterTable(String),

    /// Extracted content is empty or contains no useful information
    #[error("Extracted content is empty")]
    EmptyContent,

    /// Failed to parse code block content
    #[error("Code block parsing failed: {0}")]
    CodeBlockParseFailed(String),
}

/// Markdown rendering and output validation errors.
/// Handles issues with generating and validating final output.
#[derive(Error, Debug)]
pub enum RenderError {
    /// Markdown generation process failed
    #[error("Markdown generation failed: {0}")]
    MarkdownGenerationFailed(String),

    /// Invalid output file path or permissions issue
    #[error("Invalid output path: {0}")]
    InvalidOutputPath(String),

    /// Generated output failed validation checks
    #[error("Output validation failed: {0}")]
    ValidationFailed(String),
}

/// URL parsing and validation errors.
/// Handles malformed URLs and protocol issues.
#[derive(Error, Debug)]
pub enum UrlParseError {
    /// URL parsing failed with underlying error
    #[error("URL parsing failed: {0}")]
    ParseFailed(#[from] url::ParseError),

    /// URL scheme is not supported (must be http/https)
    #[error("Unsupported URL scheme '{0}' - only http/https supported")]
    UnsupportedScheme(String),

    /// URL is missing required host component
    #[error("URL missing host component")]
    MissingHost,

    /// URL contains invalid characters or format
    #[error("Invalid URL format: {0}")]
    InvalidFormat(String),
}

// Type aliases for cleaner Result types
/// Standard Result type using ExtractionError
pub type Result<T> = std::result::Result<T, ExtractionError>;

/// Result type for network operations
pub type NetworkResult<T> = std::result::Result<T, NetworkError>;

/// Result type for parsing operations
pub type ParseResult<T> = std::result::Result<T, ParseError>;

/// Result type for content extraction operations
pub type ExtractionResult<T> = std::result::Result<T, ContentExtractionError>;

/// Result type for rendering operations
pub type RenderResult<T> = std::result::Result<T, RenderError>;
