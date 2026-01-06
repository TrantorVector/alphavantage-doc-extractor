//! Integration tests for content area identification.
//!
//! Tests the main content detection algorithm with real Alpha Vantage URLs.

use alphavantage_doc_extractor::adapters::HttpClient;
use alphavantage_doc_extractor::domain::{identify_main_content, RawUrl};
use alphavantage_doc_extractor::utils::init_logging;

#[tokio::test]
async fn test_content_identification_with_real_url() {
    // Initialize logging for the test
    let _correlation_id = init_logging("info", "pretty").unwrap();

    let client = HttpClient::new().unwrap();

    // Fetch real Alpha Vantage documentation URL
    let url = RawUrl::new("https://www.alphavantage.co/documentation/".to_string());
    let validated_url = url.try_into().unwrap();

    let result = client.fetch(validated_url).await;

    assert!(result.is_ok(), "Failed to fetch Alpha Vantage documentation: {:?}", result.err());

    let raw_html = result.unwrap();

    // Parse the HTML document
    let parsed_doc = raw_html.try_into().unwrap();

    // Identify main content area
    let main_content = identify_main_content(&parsed_doc.dom).unwrap();

    // Extract text from main content
    let content_text = alphavantage_doc_extractor::domain::extract_text(&main_content);

    // Verify content meets requirements
    assert!(content_text.len() > 1000, "Content length {} is not > 1000 characters", content_text.len());

    // Verify content contains expected API information
    assert!(content_text.contains("TIME_SERIES"), "Content does not contain 'TIME_SERIES'");

    println!("✅ Successfully identified main content area with {} characters", content_text.len());
    println!("✅ Content contains TIME_SERIES API information");
}
