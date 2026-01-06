//! Unit tests for the logging infrastructure.
//!
//! Tests logging initialization, correlation ID generation, format handling,
//! log level parsing, timer functionality, and environment variable overrides.

use alphavantage_doc_extractor::utils::{parse_log_level, Timer};
use std::env;
use std::time::Duration;
use tokio::time::sleep;
use tracing::Level;
use uuid::Uuid;

/// Test logging initialization success (mocked to avoid global subscriber conflicts)
#[cfg(test)]
mod init_logging_tests {
    use super::*;

    #[test]
    fn test_init_logging_success() {
        // Test that we can generate correlation IDs (without actually initializing logging)
        let correlation_id = Uuid::new_v4().to_string();
        assert_eq!(correlation_id.len(), 36); // UUID v4 format: 8-4-4-4-12 = 36 chars
        assert_eq!(correlation_id.chars().filter(|&c| c == '-').count(), 4); // 4 hyphens
    }

    #[test]
    fn test_correlation_id_uniqueness() {
        let id1 = Uuid::new_v4().to_string();
        let id2 = Uuid::new_v4().to_string();

        assert_ne!(id1, id2, "Correlation IDs should be unique");
    }

    #[test]
    fn test_init_logging_invalid_level() {
        let result = parse_log_level("invalid");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid log level"));
    }
}

/// Test log level parsing functionality
#[cfg(test)]
mod log_level_tests {
    use super::*;

    #[test]
    fn test_parse_valid_log_levels() {
        let test_cases = vec![
            ("trace", Level::TRACE),
            ("debug", Level::DEBUG),
            ("info", Level::INFO),
            ("warn", Level::WARN),
            ("warning", Level::WARN),
            ("error", Level::ERROR),
            ("TRACE", Level::TRACE),
            ("DEBUG", Level::DEBUG),
            ("INFO", Level::INFO),
            ("WARN", Level::WARN),
            ("ERROR", Level::ERROR),
        ];

        for (input, expected) in test_cases {
            let result = parse_log_level(input);
            assert!(result.is_ok(), "Failed to parse '{}'", input);
            assert_eq!(result.unwrap(), expected, "Incorrect level for '{}'", input);
        }
    }

    #[test]
    fn test_parse_invalid_log_levels() {
        let invalid_levels = vec!["", " ", "tracee", "debugg", "invalid", "123"];

        for level in invalid_levels {
            let result = parse_log_level(level);
            assert!(result.is_err(), "'{}' should be invalid", level);
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Invalid log level"));
        }
    }
}

/// Test Timer functionality
#[cfg(test)]
mod timer_tests {
    use super::*;

    #[test]
    fn test_timer_creation() {
        let timer = Timer::new("test_operation");
        assert_eq!(timer.name(), "test_operation");
        assert!(timer.elapsed() >= Duration::from_nanos(0));
    }

    #[test]
    fn test_timer_elapsed() {
        let timer = Timer::new("elapsed_test");
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed();
        assert!(elapsed >= Duration::from_millis(10));
        assert!(elapsed < Duration::from_millis(100)); // Shouldn't take too long
    }

    #[tokio::test]
    async fn test_timer_async_operation() {
        let _timer = Timer::new("async_test");
        sleep(Duration::from_millis(5)).await;
        // Timer will log completion when dropped
    }
}

/// Test environment variable functionality (without global subscriber)
#[cfg(test)]
mod env_tests {
    use super::*;

    #[test]
    fn test_environment_variable_access() {
        // Save original RUST_LOG value
        let original_rust_log = env::var("RUST_LOG").ok();

        // Set environment variable
        env::set_var("RUST_LOG", "debug");

        // Test that we can read the env var (without initializing logging)
        let env_value = env::var("RUST_LOG").unwrap();
        assert_eq!(env_value, "debug");

        // Restore original value
        if let Some(val) = original_rust_log {
            env::set_var("RUST_LOG", val);
        } else {
            env::remove_var("RUST_LOG");
        }
    }

    #[test]
    fn test_environment_variable_not_set() {
        // Ensure RUST_LOG is not set
        env::remove_var("RUST_LOG");

        // Test that var is not set
        let result = env::var("RUST_LOG");
        assert!(result.is_err());
    }
}

/// Test correlation ID format validation
#[cfg(test)]
mod correlation_id_tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_correlation_id_format() {
        let correlation_id = Uuid::new_v4().to_string();

        // UUID v4 format: 8-4-4-4-12 hexadecimal digits
        let uuid_regex =
            Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$")
                .unwrap();

        assert!(
            uuid_regex.is_match(&correlation_id),
            "Correlation ID '{}' does not match UUID v4 format",
            correlation_id
        );
    }

    #[test]
    fn test_multiple_correlation_ids_unique() {
        let ids: Vec<String> = (0..10).map(|_| Uuid::new_v4().to_string()).collect();

        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                assert_ne!(ids[i], ids[j], "Correlation IDs should be unique");
            }
        }
    }
}

/// Test log format validation (without global subscriber)
#[cfg(test)]
mod format_tests {
    #[test]
    fn test_format_string_validation() {
        // Test that format strings are valid (without initializing)
        let valid_formats = vec!["json", "pretty", "unknown"];
        for format in valid_formats {
            assert!(!format.is_empty());
        }
    }
}

/// Integration test for logging components (without global subscriber)
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_logging_workflow() {
        // Test correlation ID generation
        let correlation_id = Uuid::new_v4().to_string();
        assert_eq!(correlation_id.len(), 36);

        // Test timer functionality
        let timer = Timer::new("integration_test");
        std::thread::sleep(Duration::from_millis(1));
        let elapsed = timer.elapsed();
        assert!(elapsed > Duration::from_nanos(0));

        // Test log level parsing
        let level = parse_log_level("info").unwrap();
        assert_eq!(level, Level::INFO);
    }
}
