//! Integration tests for category extraction functionality.
//!
//! These tests verify that the ContentExtractor can successfully extract
//! categories from real Alpha Vantage documentation pages.

use alphavantage_doc_extractor::adapters::HttpClient;
use alphavantage_doc_extractor::domain::{ContentExtractor, RawHtml, ValidatedUrl};
use alphavantage_doc_extractor::utils::UrlParseError;

use std::time::Duration;
use tokio_test::block_on;

#[test]
fn test_category_extraction_integration() {
    block_on(async {
        // Create HTTP client
        let client = HttpClient::new(
            Duration::from_secs(30), // timeout
            3,                       // max retries
            Duration::from_millis(100), // initial backoff
        ).expect("Failed to create HTTP client");

        // Alpha Vantage documentation URL
        let url = "https://www.alphavantage.co/documentation/";

        // Fetch the page
        let raw_url = alphavantage_doc_extractor::domain::RawUrl::new(url.to_string());
        let validated_url = ValidatedUrl::try_from(raw_url)
            .expect("Failed to validate URL");

        let response = client.fetch(validated_url)
            .await
            .expect("Failed to fetch Alpha Vantage documentation");

        // Parse the HTML
        let raw_html = RawHtml::new(response, validated_url);
        let parsed_doc = alphavantage_doc_extractor::domain::ParsedDocument::try_from(raw_html)
            .expect("Failed to parse HTML document");

        // Identify main content area
        let main_content = alphavantage_doc_extractor::domain::parser::identify_main_content(parsed_doc.document.clone())
            .expect("Failed to identify main content area");

        // Extract categories
        let extractor = ContentExtractor::new(&parsed_doc.document);
        let categories = extractor.extract_categories(main_content)
            .expect("Failed to extract categories");

        // Verify we have a reasonable number of categories
        assert!(categories.len() >= 6, "Expected at least 6 categories, found {}", categories.len());
        assert!(categories.len() <= 15, "Expected at most 15 categories, found {}", categories.len());

        // Verify known categories are present
        let category_names: Vec<&str> = categories.iter().map(|c| c.name.as_str()).collect();
        println!("Extracted categories:");
        for name in &category_names {
            println!("  - {}", name);
        }

        // Check for expected categories (case-insensitive)
        let has_time_series = category_names.iter()
            .any(|name| name.to_lowercase().contains("time series"));
        let has_fundamental = category_names.iter()
            .any(|name| name.to_lowercase().contains("fundamental"));

        assert!(has_time_series, "Expected 'Time Series' category not found in: {:?}", category_names);
        assert!(has_fundamental, "Expected 'Fundamental' category not found in: {:?}", category_names);

        // Verify categories have reasonable names (not empty, not too long)
        for category in &categories {
            assert!(!category.name.trim().is_empty(), "Category name should not be empty");
            assert!(category.name.len() <= 100, "Category name '{}' is too long", category.name);
        }

        println!("Successfully extracted {} categories from Alpha Vantage documentation", categories.len());
    });
}
