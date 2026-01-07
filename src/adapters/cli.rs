//! Command-line interface for the Alpha Vantage Documentation Extractor.
//!
//! This module provides the CLI interface using clap for parsing command-line
//! arguments and configuration. It defines the main configuration structure
//! and validation logic for the application.
//!
//! The CLI supports extracting documentation from Alpha Vantage URLs,
//! with options for output customization, logging, and validation.

use crate::utils::{ExtractionError, RenderError};
use clap::Parser;

/// Configuration for the Alpha Vantage Documentation Extractor.
///
/// This struct defines all command-line arguments and their validation.
/// It uses clap's derive API for automatic argument parsing and help generation.
#[derive(Parser, Debug, Clone)]
#[command(name = "alphavantage-doc-extractor")]
#[command(
    version,
    author,
    about = "Extract Alpha Vantage API docs to LLM-optimized Markdown"
)]
pub struct Config {
    /// Source URL to extract documentation from
    #[arg(short, long, help = "Source URL to extract")]
    pub url: String,

    /// Output file path for the generated markdown
    #[arg(short, long, default_value = "output.md", help = "Output file path")]
    pub output: String,

    /// Logging level (error, warn, info, debug, trace)
    #[arg(long, default_value = "info", env = "RUST_LOG", help = "Logging level")]
    pub log_level: String,

    /// Log output format (pretty or json)
    #[arg(long, default_value = "pretty", value_parser = ["pretty", "json"], help = "Log output format")]
    pub log_format: String,

    /// Skip output validation (faster but less safe)
    #[arg(long, help = "Skip output validation")]
    pub no_validation: bool,

    /// Create backup of existing output file before writing
    #[arg(long, help = "Create backup of existing output file")]
    pub backup: bool,
}

impl Config {
    /// Create configuration from command-line arguments.
    ///
    /// Parses command-line arguments using clap's derive API.
    pub fn from_args() -> Self {
        Self::parse()
    }

    /// Validate the configuration values.
    ///
    /// Performs validation on URL format, output path, and log level.
    /// Returns an error if any validation fails.
    pub fn validate(&self) -> Result<(), ExtractionError> {
        // Validate URL format
        self.validate_url()?;

        // Validate output path
        self.validate_output_path()?;

        // Validate log level
        self.validate_log_level()?;

        // Validate log format
        self.validate_log_format()?;

        Ok(())
    }

    /// Validate the source URL format.
    fn validate_url(&self) -> Result<(), ExtractionError> {
        if self.url.trim().is_empty() {
            return Err(ExtractionError::Render(RenderError::InvalidOutputPath(
                "URL cannot be empty".to_string(),
            )));
        }

        // Basic URL validation - must start with http:// or https://
        if !self.url.starts_with("http://") && !self.url.starts_with("https://") {
            return Err(ExtractionError::Render(RenderError::InvalidOutputPath(
                format!("URL must start with http:// or https://: {}", self.url),
            )));
        }

        // Check for basic URL structure (has at least one dot)
        if !self.url.contains('.') {
            return Err(ExtractionError::Render(RenderError::InvalidOutputPath(
                format!("URL appears to be malformed: {}", self.url),
            )));
        }

        Ok(())
    }

    /// Validate the output file path.
    fn validate_output_path(&self) -> Result<(), ExtractionError> {
        if self.output.trim().is_empty() {
            return Err(ExtractionError::Render(RenderError::InvalidOutputPath(
                "Output path cannot be empty".to_string(),
            )));
        }

        // Check for obviously invalid characters
        let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
        for &ch in &invalid_chars {
            if self.output.contains(ch) {
                return Err(ExtractionError::Render(RenderError::InvalidOutputPath(
                    format!(
                        "Output path contains invalid character '{}': {}",
                        ch, self.output
                    ),
                )));
            }
        }

        Ok(())
    }

    /// Validate the log level.
    fn validate_log_level(&self) -> Result<(), ExtractionError> {
        let valid_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_levels.contains(&self.log_level.as_str()) {
            return Err(ExtractionError::Render(RenderError::InvalidOutputPath(
                format!(
                    "Invalid log level '{}'. Must be one of: {}",
                    self.log_level,
                    valid_levels.join(", ")
                ),
            )));
        }

        Ok(())
    }

    /// Validate the log format.
    fn validate_log_format(&self) -> Result<(), ExtractionError> {
        let valid_formats = ["pretty", "json"];
        if !valid_formats.contains(&self.log_format.as_str()) {
            return Err(ExtractionError::Render(RenderError::InvalidOutputPath(
                format!(
                    "Invalid log format '{}'. Must be one of: {}",
                    self.log_format,
                    valid_formats.join(", ")
                ),
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test URL validation
    #[test]
    fn test_validate_url_valid() {
        let config = Config {
            url: "https://www.alphavantage.co/documentation".to_string(),
            output: "test.md".to_string(),
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
            no_validation: false,
            backup: false,
        };

        assert!(config.validate_url().is_ok());
    }

    #[test]
    fn test_validate_url_invalid_scheme() {
        let config = Config {
            url: "ftp://example.com".to_string(),
            output: "test.md".to_string(),
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
            no_validation: false,
            backup: false,
        };

        assert!(config.validate_url().is_err());
    }

    #[test]
    fn test_validate_url_empty() {
        let config = Config {
            url: "".to_string(),
            output: "test.md".to_string(),
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
            no_validation: false,
            backup: false,
        };

        assert!(config.validate_url().is_err());
    }

    /// Test output path validation
    #[test]
    fn test_validate_output_path_valid() {
        let config = Config {
            url: "https://example.com".to_string(),
            output: "docs/api.md".to_string(),
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
            no_validation: false,
            backup: false,
        };

        assert!(config.validate_output_path().is_ok());
    }

    #[test]
    fn test_validate_output_path_invalid_chars() {
        let config = Config {
            url: "https://example.com".to_string(),
            output: "docs<invalid>.md".to_string(),
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
            no_validation: false,
            backup: false,
        };

        assert!(config.validate_output_path().is_err());
    }

    /// Test log level validation
    #[test]
    fn test_validate_log_level_valid() {
        let config = Config {
            url: "https://example.com".to_string(),
            output: "test.md".to_string(),
            log_level: "debug".to_string(),
            log_format: "pretty".to_string(),
            no_validation: false,
            backup: false,
        };

        assert!(config.validate_log_level().is_ok());
    }

    #[test]
    fn test_validate_log_level_invalid() {
        let config = Config {
            url: "https://example.com".to_string(),
            output: "test.md".to_string(),
            log_level: "invalid".to_string(),
            log_format: "pretty".to_string(),
            no_validation: false,
            backup: false,
        };

        assert!(config.validate_log_level().is_err());
    }

    /// Test full config validation
    #[test]
    fn test_validate_config_valid() {
        let config = Config {
            url: "https://www.alphavantage.co/documentation".to_string(),
            output: "output.md".to_string(),
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
            no_validation: false,
            backup: true,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_config_invalid() {
        let config = Config {
            url: "invalid-url".to_string(),
            output: "output.md".to_string(),
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
            no_validation: false,
            backup: false,
        };

        assert!(config.validate().is_err());
    }
}
