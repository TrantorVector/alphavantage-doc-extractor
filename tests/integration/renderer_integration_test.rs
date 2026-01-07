//! Integration tests for the MarkdownRenderer.
//!
//! These tests verify that the renderer works correctly with real-world
//! document structures and produces properly formatted markdown output.

use alphavantage_doc_extractor::domain::MarkdownRenderer;
use std::fs;

/// Test rendering a complete document structure from fixture
#[test]
fn test_render_complete_document() {
    let renderer = MarkdownRenderer::new();

    // Load test fixture
    let fixture_path = "tests/fixtures/sample_structure.json";
    let fixture_content = fs::read_to_string(fixture_path)
        .expect("Failed to read sample_structure.json fixture");

    // Parse the JSON fixture
    let document: alphavantage_doc_extractor::domain::DocumentStructure =
        serde_json::from_str(&fixture_content)
            .expect("Failed to parse sample_structure.json");

    // Render to markdown
    let result = renderer.render(&document, true);
    assert!(result.is_ok(), "Rendering should succeed");

    let markdown = result.unwrap();

    // Verify basic structure
    assert!(markdown.contains("# Alpha Vantage API Documentation"));
    assert!(markdown.contains("**Source:** https://www.alphavantage.co/documentation"));
    assert!(markdown.contains("**Endpoints:** 5"));
    assert!(markdown.contains("**Categories:** 2"));

    // Verify categories
    assert!(markdown.contains("## Stock Time Series Data"));
    assert!(markdown.contains("## Technical Indicators"));

    // Verify endpoints
    assert!(markdown.contains("### TIME_SERIES_INTRADAY"));
    assert!(markdown.contains("### TIME_SERIES_DAILY"));
    assert!(markdown.contains("### SMA"));
    assert!(markdown.contains("### RSI"));
    assert!(markdown.contains("### STOCHRSI `[PREMIUM]`"));

    // Verify premium badge appears only on STOCHRSI
    assert!(markdown.contains("STOCHRSI `[PREMIUM]`"));
    assert!(!markdown.contains("TIME_SERIES_INTRADAY `[PREMIUM]`"));
    assert!(!markdown.contains("SMA `[PREMIUM]`"));

    // Verify parameter tables
    assert!(markdown.contains("**Required Parameters:**"));
    assert!(markdown.contains("**Optional Parameters:**"));
    assert!(markdown.contains("| Parameter | Type | Description | Default | Options |"));
    assert!(markdown.contains("| function | string | The API function to call | - | - |"));
    assert!(markdown.contains("| interval | string | Time interval between data points | 5min | 1min, 5min, 15min, 30min, 60min |"));

    // Verify code examples
    assert!(markdown.contains("```python"));
    assert!(markdown.contains("import requests"));
    assert!(markdown.contains("r = requests.get(url)"));

    // Verify request patterns
    assert!(markdown.contains("**API Request Pattern:**"));
    assert!(markdown.contains("```http"));
    assert!(markdown.contains("https://www.alphavantage.co/query?function="));

    // Verify proper spacing (categories separated by 3 newlines)
    assert!(markdown.contains("---\n\n\n## Stock Time Series Data"));

    // Verify output has reasonable length
    assert!(markdown.len() > 5000, "Output should be substantial");
    assert!(markdown.len() < 20000, "Output should not be excessively long");

    println!("Generated markdown length: {} characters", markdown.len());
}

/// Test that rendered output can be written to file
#[test]
fn test_render_output_file_format() {
    let renderer = MarkdownRenderer::new();

    // Create a minimal document for testing
    let metadata = alphavantage_doc_extractor::domain::DocumentMetadata::new(
        "Test API".to_string(),
        "https://test.com".to_string().into(),
    )
    .with_counts(1, 1);

    let endpoint = alphavantage_doc_extractor::domain::ApiEndpoint::new(
        "TEST_ENDPOINT".to_string(),
        "A test endpoint".to_string(),
    )
    .with_required_params(vec![
        alphavantage_doc_extractor::domain::Parameter::new(
            "api_key".to_string(),
            "string".to_string(),
            "Your API key".to_string(),
        )
    ]);

    let category = alphavantage_doc_extractor::domain::ApiCategory::new("Test Category".to_string())
        .add_endpoint(endpoint);

    let document = alphavantage_doc_extractor::domain::DocumentStructure::new(metadata, vec![category]);

    let result = renderer.render(&document, true);
    assert!(result.is_ok());

    let markdown = result.unwrap();

    // Verify it looks like valid markdown
    assert!(markdown.starts_with("# "));
    assert!(markdown.contains("\n\n"));
    assert!(markdown.contains("## "));
    assert!(markdown.contains("### "));
    assert!(markdown.contains("| ")); // Tables
    assert!(markdown.contains("```")); // Code blocks

    // Could be written to a .md file
    assert!(markdown.lines().count() > 10);
}
