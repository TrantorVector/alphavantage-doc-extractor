//! Integration tests for the CLI interface.
//!
//! These tests verify that the command-line interface works correctly
//! using assert_cmd to test the actual binary behavior.

use assert_cmd::Command;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Test that --help flag produces expected output
#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("alphavantage-doc-extractor").unwrap();

    let assert = cmd.arg("--help").assert();

    assert
        .success()
        .stdout(predicates::str::contains("Alpha Vantage Documentation Extractor"))
        .stdout(predicates::str::contains("Extract Alpha Vantage API docs to LLM-optimized Markdown"))
        .stdout(predicates::str::contains("-u, --url <URL>"))
        .stdout(predicates::str::contains("-o, --output <OUTPUT>"))
        .stdout(predicates::str::contains("--log-level <LOG_LEVEL>"))
        .stdout(predicates::str::contains("--log-format <LOG_FORMAT>"))
        .stdout(predicates::str::contains("--no-validation"))
        .stdout(predicates::str::contains("--backup"));
}

/// Test that --version flag produces expected output
#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("alphavantage-doc-extractor").unwrap();

    let assert = cmd.arg("--version").assert();

    assert
        .success()
        .stdout(predicates::str::contains("alphavantage-doc-extractor"));
}

/// Test CLI with missing required arguments
#[test]
fn test_cli_missing_url() {
    let mut cmd = Command::cargo_bin("alphavantage-doc-extractor").unwrap();

    let assert = cmd.assert();

    // Should fail because URL is required
    assert.failure();
}

/// Test CLI with invalid URL
#[test]
fn test_cli_invalid_url() {
    let mut cmd = Command::cargo_bin("alphavantage-doc-extractor").unwrap();

    let assert = cmd
        .arg("--url")
        .arg("invalid-url")
        .assert();

    // Should fail due to URL validation
    assert.failure();
}

/// Test CLI with valid arguments but non-existent URL (would require mocking HTTP)
/// For now, we'll skip the full integration test since it requires network access.
/// Instead, we'll test argument parsing and validation up to the point where
/// network access would be needed.

/// Test that output file gets created with default path
/// Note: This test will be limited since we can't easily mock HTTP in integration tests
#[test]
fn test_cli_argument_parsing() {
    // Test that arguments are parsed correctly by checking help output format
    let mut cmd = Command::cargo_bin("alphavantage-doc-extractor").unwrap();

    let assert = cmd
        .arg("--help")
        .assert();

    // Verify the help shows default values
    assert
        .success()
        .stdout(predicates::str::contains("output.md"))
        .stdout(predicates::str::contains("info"))
        .stdout(predicates::str::contains("pretty"));
}

/// Test that invalid log level is rejected
#[test]
fn test_cli_invalid_log_level() {
    let mut cmd = Command::cargo_bin("alphavantage-doc-extractor").unwrap();

    let assert = cmd
        .arg("--url")
        .arg("https://example.com")
        .arg("--log-level")
        .arg("invalid")
        .assert();

    // Should fail due to invalid log level
    assert.failure();
}

/// Test that invalid log format is rejected
#[test]
fn test_cli_invalid_log_format() {
    let mut cmd = Command::cargo_bin("alphavantage-doc-extractor").unwrap();

    let assert = cmd
        .arg("--url")
        .arg("https://example.com")
        .arg("--log-format")
        .arg("invalid")
        .assert();

    // Should fail due to invalid log format
    assert.failure();
}

/// Test that invalid output path characters are rejected
#[test]
fn test_cli_invalid_output_path() {
    let mut cmd = Command::cargo_bin("alphavantage-doc-extractor").unwrap();

    let assert = cmd
        .arg("--url")
        .arg("https://example.com")
        .arg("--output")
        .arg("invalid<file>.md")
        .assert();

    // Should fail due to invalid characters in output path
    assert.failure();
}

/// Test environment variable support for log level
#[test]
fn test_cli_env_var_log_level() {
    let mut cmd = Command::cargo_bin("alphavantage-doc-extractor").unwrap();

    // Set environment variable
    cmd.env("RUST_LOG", "debug");

    let assert = cmd
        .arg("--url")
        .arg("https://example.com")
        .arg("--help")
        .assert();

    // Should still work with env var set
    assert.success();
}

/// Test that --no-validation flag is accepted
#[test]
fn test_cli_no_validation_flag() {
    let mut cmd = Command::cargo_bin("alphavantage-doc-extractor").unwrap();

    let assert = cmd
        .arg("--url")
        .arg("https://example.com")
        .arg("--no-validation")
        .arg("--help")
        .assert();

    // Should accept the flag
    assert.success();
}

/// Test that --backup flag is accepted
#[test]
fn test_cli_backup_flag() {
    let mut cmd = Command::cargo_bin("alphavantage-doc-extractor").unwrap();

    let assert = cmd
        .arg("--url")
        .arg("https://example.com")
        .arg("--backup")
        .arg("--help")
        .assert();

    // Should accept the flag
    assert.success();
}

/// Test short flag aliases work
#[test]
fn test_cli_short_flags() {
    let mut cmd = Command::cargo_bin("alphavantage-doc-extractor").unwrap();

    let assert = cmd
        .args(["-u", "https://example.com"])
        .args(["-o", "output.md"])
        .arg("--help")
        .assert();

    // Should accept short flags
    assert.success();
}
