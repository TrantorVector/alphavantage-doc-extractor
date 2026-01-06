//! Integration tests for HTTP client functionality.
//!
//! These tests verify the HTTP client works with real-world URLs and services.

use alphavantage_doc_extractor::adapters::HttpClient;
use alphavantage_doc_extractor::domain::RawUrl;
use alphavantage_doc_extractor::utils::init_logging;

#[tokio::test]
async fn test_fetch_alpha_vantage_documentation() {
    // Initialize logging for the test
    let _correlation_id = init_logging("info", "pretty").unwrap();

    let client = HttpClient::new().unwrap();

    // Fetch real Alpha Vantage documentation URL
    let url = RawUrl::new("https://www.alphavantage.co/documentation/".to_string());
    let validated_url = url.try_into().unwrap();

    let result = client.fetch(validated_url).await;

    assert!(result.is_ok(), "Failed to fetch Alpha Vantage documentation: {:?}", result.err());

    let html = result.unwrap();

    // Verify content size > 10000 bytes
    assert!(html.content.len() > 10000, "Content size {} is not > 10000 bytes", html.content.len());

    // Verify content contains "Alpha Vantage" and "API"
    assert!(html.content.contains("Alpha Vantage"), "Content does not contain 'Alpha Vantage'");
    assert!(html.content.contains("API"), "Content does not contain 'API'");
}
