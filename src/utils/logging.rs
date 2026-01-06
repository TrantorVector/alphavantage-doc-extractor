//! Logging infrastructure for the Alpha Vantage Documentation Extractor.
//!
//! This module provides structured logging with tracing, supporting both JSON and pretty
//! output formats, configurable log levels, and automatic correlation ID generation.

use std::time::{Duration, Instant};
use tracing::{info, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use uuid::Uuid;

/// Initialize the logging system with the specified level and format.
///
/// Returns the generated correlation ID for the application session.
///
/// # Arguments
/// * `log_level` - Log level string (trace, debug, info, warn, error)
/// * `log_format` - Output format ("json" or "pretty")
///
/// # Returns
/// The correlation ID as a string, or an error if initialization fails.
pub fn init_logging(log_level: &str, log_format: &str) -> Result<String, anyhow::Error> {
    // Generate a unique correlation ID for this application session
    let correlation_id = Uuid::new_v4().to_string();

    // Parse the log level (used implicitly through the filter)
    let _level = parse_log_level(log_level)?;

    // Create environment filter with fallback to the specified level
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    // Initialize the tracing subscriber registry based on format
    match log_format {
        "json" => {
            Registry::default()
                .with(filter)
                .with(
                    fmt::layer()
                        .json()
                        .flatten_event(true)
                        .with_current_span(false)
                        .with_span_list(false),
                )
                .init();
        }
        _ => {
            Registry::default()
                .with(filter)
                .with(fmt::layer().pretty())
                .init();
        }
    }

    // Log the initialization with correlation ID and configuration
    info!(
        correlation_id = %correlation_id,
        log_level = %log_level,
        log_format = %log_format,
        "Logging system initialized"
    );

    Ok(correlation_id)
}

/// Parse a log level string into a tracing Level.
///
/// Supports case-insensitive parsing of: trace, debug, info, warn, error.
///
/// # Arguments
/// * `level_str` - The log level string to parse
///
/// # Returns
/// The parsed Level, or an error for invalid level strings.
pub fn parse_log_level(level_str: &str) -> Result<Level, anyhow::Error> {
    match level_str.to_lowercase().as_str() {
        "trace" => Ok(Level::TRACE),
        "debug" => Ok(Level::DEBUG),
        "info" => Ok(Level::INFO),
        "warn" | "warning" => Ok(Level::WARN),
        "error" => Ok(Level::ERROR),
        _ => Err(anyhow::anyhow!(
            "Invalid log level '{}'. Valid levels are: trace, debug, info, warn, error",
            level_str
        )),
    }
}

/// Timer helper for measuring operation durations.
///
/// Automatically logs completion with duration when dropped.
#[derive(Debug)]
pub struct Timer {
    name: String,
    start: Instant,
}

impl Timer {
    /// Create a new timer with the given operation name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
        }
    }

    /// Get the elapsed duration since the timer was created.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get the operation name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let duration = self.elapsed();
        info!(
            operation = %self.name,
            duration_ms = duration.as_millis(),
            "Operation completed"
        );
    }
}

/// Macro for instrumenting functions with automatic timing and logging.
///
/// This macro creates a Timer at the start of the function and logs completion
/// with duration when the function returns.
///
/// # Example
/// ```ignore
/// fn my_function() {
///     let _timer = timer!("my_operation");
///     // ... function body ...
/// } // Timer logs completion here
/// ```
#[macro_export]
macro_rules! timer {
    ($name:expr) => {
        $crate::utils::logging::Timer::new($name)
    };
}
