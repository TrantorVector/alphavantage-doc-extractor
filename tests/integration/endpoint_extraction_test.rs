//! Integration tests for endpoint extraction functionality.
//!
//! These tests verify that the ContentExtractor can successfully extract
//! endpoints from real Alpha Vantage documentation pages.

use alphavantage_doc_extractor::adapters::HttpClient;
use alphavantage_doc_extractor::domain::{ContentExtractor, RawHtml, ValidatedUrl};
use alphavantage_doc_extractor::utils::UrlParseError;

use std::time::Duration;
use tokio_test::block_on;

#[test]
fn test_endpoint_extraction_integration() {
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

        // Extract categories with endpoints
        let extractor = ContentExtractor::new(&parsed_doc.document);
        let categories = extractor.extract_categories(main_content)
            .expect("Failed to extract categories");

        // Count total endpoints across all categories
        let total_endpoints: usize = categories.iter()
            .map(|cat| cat.endpoints.len())
            .sum();

        // Verify we have a reasonable number of endpoints
        assert!(total_endpoints >= 30, "Expected at least 30 endpoints, found {}", total_endpoints);
        assert!(total_endpoints <= 100, "Expected at most 100 endpoints, found {}", total_endpoints);

        // Print summary
        println!("Extracted {} categories with {} total endpoints:", categories.len(), total_endpoints);
        for category in &categories {
            println!("  {}: {} endpoints", category.name, category.endpoints.len());
        }

        // Verify known endpoints exist
        let all_endpoints: Vec<&alphavantage_doc_extractor::domain::ApiEndpoint> = categories.iter()
            .flat_map(|cat| &cat.endpoints)
            .collect();

        let function_names: Vec<&str> = all_endpoints.iter()
            .map(|endpoint| endpoint.function_name.as_str())
            .collect();

        // Check for expected endpoints
        let has_time_series_intraday = function_names.contains(&"TIME_SERIES_INTRADAY");
        let has_time_series_daily = function_names.contains(&"TIME_SERIES_DAILY");
        let has_overview = function_names.contains(&"OVERVIEW");

        assert!(has_time_series_intraday || has_time_series_daily,
                "Expected TIME_SERIES_INTRADAY or TIME_SERIES_DAILY not found. Available: {:?}", function_names);
        assert!(has_overview, "Expected OVERVIEW endpoint not found. Available: {:?}", function_names);

        // Verify endpoints have reasonable properties
        for endpoint in all_endpoints {
            // Function names should be uppercase with underscores
            assert!(endpoint.function_name.chars().all(|c| c.is_uppercase() || c == '_'),
                   "Function name '{}' contains invalid characters", endpoint.function_name);

            // Descriptions should not be empty and not too long
            assert!(!endpoint.description.is_empty(),
                   "Endpoint '{}' has empty description", endpoint.function_name);
            assert!(endpoint.description.len() <= 500,
                   "Endpoint '{}' description is too long: {} chars", endpoint.function_name, endpoint.description.len());

            // Should not have promotional content in descriptions
            let lower_desc = endpoint.description.to_lowercase();
            assert!(!lower_desc.contains("claim your") && !lower_desc.contains("subscribe"),
                   "Endpoint '{}' description contains promotional content: {}", endpoint.function_name, endpoint.description);
        }

        // Check that some categories have multiple endpoints
        let categories_with_endpoints: Vec<&alphavantage_doc_extractor::domain::ApiCategory> = categories.iter()
            .filter(|cat| !cat.endpoints.is_empty())
            .collect();

        assert!(!categories_with_endpoints.is_empty(), "No categories have endpoints");
        assert!(categories_with_endpoints.len() >= 3, "Expected at least 3 categories with endpoints, found {}", categories_with_endpoints.len());

        println!("Successfully extracted {} endpoints from Alpha Vantage documentation across {} categories",
                total_endpoints, categories.len());
    });
}
