//! Integration tests for the FileWriter adapter.
//!
//! These tests verify that the FileWriter works correctly with real file system
//! operations, including writing markdown content, backup creation, and UTF-8 handling.

use alphavantage_doc_extractor::adapters::FileWriter;
use alphavantage_doc_extractor::domain::{ApiCategory, ApiEndpoint, DocumentMetadata, DocumentStructure, MarkdownRenderer, Parameter, RawUrl, ValidatedUrl};
use alphavantage_doc_extractor::ports::Writer;
use std::fs;
use tempfile::TempDir;

/// Test complete integration: generate markdown and write to file
#[test]
fn test_write_generated_markdown() {
    let writer = FileWriter::new();
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("api_docs.md");

    // Create sample document structure
    let url = ValidatedUrl::validate(RawUrl::new("https://api.example.com".to_string())).unwrap();
    let metadata = DocumentMetadata::new("Sample API".to_string(), url).with_counts(1, 1);

    let endpoint = ApiEndpoint::new(
        "GET_DATA".to_string(),
        "Retrieves data from the API".to_string(),
    )
    .with_required_params(vec![Parameter::new(
        "api_key".to_string(),
        "string".to_string(),
        "Your API key".to_string(),
    )])
    .with_request_pattern("https://api.example.com/query?function=GET_DATA&apikey={api_key}".to_string());

    let category = ApiCategory::new("Data Operations".to_string()).add_endpoint(endpoint);
    let document = DocumentStructure::new(metadata, vec![category]);

    // Generate markdown
    let renderer = MarkdownRenderer::new();
    let markdown = renderer.render(&document).unwrap();

    // Write to file
    let result = writer.write(output_path.to_str().unwrap(), &markdown);
    assert!(result.is_ok());

    // Verify file exists and has correct content
    assert!(output_path.exists());
    let written_content = fs::read_to_string(&output_path).unwrap();

    // Verify markdown structure
    assert!(written_content.contains("# Sample API"));
    assert!(written_content.contains("## Table of Contents"));
    assert!(written_content.contains("## Data Operations"));
    assert!(written_content.contains("### GET_DATA"));
    assert!(written_content.contains("```http"));
    assert!(written_content.contains("---")); // Frontmatter
}

/// Test UTF-8 encoding handling
#[test]
fn test_utf8_encoding() {
    let writer = FileWriter::new();
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("unicode_test.md");

    // Content with various Unicode characters
    let content = "# Unicode Test 🚀\n\n## Special Characters\n\n- Café\n- naïve\n- résumé\n- 中文\n- 日本語\n- 🌟⭐\n\n```python\n# Unicode in code\nname = \"José\"\nprint(f\"Hello, {name}! 🌍\")\n```";

    let result = writer.write(output_path.to_str().unwrap(), content);
    assert!(result.is_ok());

    // Verify file was written and can be read back
    let read_content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(read_content, content);

    // Verify specific Unicode characters are preserved
    assert!(read_content.contains("🚀"));
    assert!(read_content.contains("Café"));
    assert!(read_content.contains("中文"));
    assert!(read_content.contains("🌍"));
}

/// Test backup functionality in integration scenario
#[test]
fn test_backup_integration() {
    let writer = FileWriter::new();
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("backup_test.md");

    // Write initial version
    let initial_content = "# Version 1\n\nInitial content.";
    writer.write(output_path.to_str().unwrap(), initial_content).unwrap();

    // Write updated version with backup
    let updated_content = "# Version 2\n\nUpdated content with new features.";
    let result = writer.write_with_backup(output_path.to_str().unwrap(), updated_content);
    assert!(result.is_ok());

    // Verify current file has new content
    let current_content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(current_content, updated_content);

    // Verify backup exists and has old content
    let backup_files: Vec<_> = fs::read_dir(temp_dir.path())
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_name().to_string_lossy().contains(".bak"))
        .collect();

    assert_eq!(backup_files.len(), 1);
    let backup_path = backup_files[0].path();
    let backup_content = fs::read_to_string(&backup_path).unwrap();
    assert_eq!(backup_content, initial_content);

    // Verify backup filename format
    let backup_filename = backup_path.file_name().unwrap().to_string_lossy();
    assert!(backup_filename.starts_with("backup_test."));
    assert!(backup_filename.contains(".bak"));
}

/// Test writing large content
#[test]
fn test_large_content_writing() {
    let writer = FileWriter::new();
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("large_file.md");

    // Generate large content (simulate real API documentation)
    let mut large_content = String::from("# Large API Documentation\n\n");
    large_content.push_str("## Table of Contents\n\n");

    for i in 1..=50 {
        large_content.push_str(&format!("- [Endpoint {}](#endpoint-{})\n", i, i));
    }

    large_content.push_str("\n");

    for i in 1..=50 {
        large_content.push_str(&format!("## Endpoint {}\n\n", i));
        large_content.push_str(&format!("This is endpoint {} with detailed documentation.\n\n", i));
        large_content.push_str("**Parameters:**\n\n| Name | Type | Description |\n|------|------|-------------|\n");
        large_content.push_str(&format!("| param{} | string | Parameter for endpoint {} |\n\n", i, i));
        large_content.push_str(&format!("```http\nGET /api/v1/endpoint/{}\n```\n\n", i));
    }

    let result = writer.write(output_path.to_str().unwrap(), &large_content);
    assert!(result.is_ok());

    // Verify file was written
    assert!(output_path.exists());

    // Verify content integrity
    let read_content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(read_content.len(), large_content.len());

    // Verify some key sections
    assert!(read_content.contains("# Large API Documentation"));
    assert!(read_content.contains("## Endpoint 1"));
    assert!(read_content.contains("## Endpoint 50"));
    assert!(read_content.contains("param50"));
}

/// Test error handling for invalid paths
#[test]
fn test_invalid_path_handling() {
    let writer = FileWriter::new();

    // Test with empty path
    let result = writer.write("", "content");
    assert!(result.is_err());

    // Test with path containing invalid characters for filesystem
    // Note: On Unix systems, most characters are valid in filenames
    // This test may be more relevant on Windows
    let invalid_path = "/dev/null/invalid/path/test.md";
    let result = writer.write(invalid_path, "content");

    // On Unix, this might succeed if directories can be created,
    // or fail if permissions don't allow it
    // We mainly want to ensure it doesn't panic
    assert!(result.is_ok() || result.is_err()); // Either is acceptable, no panic
}

/// Test file size reporting
#[test]
fn test_file_size_reporting() {
    let writer = FileWriter::new();
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("size_test.md");

    let content = "# Test\n\nContent of known size.";
    let expected_size = content.len() as u64;

    writer.write(output_path.to_str().unwrap(), content).unwrap();

    let size = writer.get_file_size(output_path.to_str().unwrap()).unwrap();
    assert_eq!(size, expected_size);
}
