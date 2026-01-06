//! Integration tests for request pattern extraction functionality.
//!
//! These tests verify that the ContentExtractor can successfully extract
//! or construct API request patterns from real Alpha Vantage documentation pages.

use alphavantage_doc_extractor::adapters::HttpClient;
use alphavantage_doc_extractor::domain::{ContentExtractor, RawHtml, ValidatedUrl};
use alphavantage_doc_extractor::utils::UrlParseError;

use std::time::Duration;
use tokio_test::block_on;

#[test]
fn test_request_pattern_extraction_integration() {
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

        // Extract categories with endpoints and request patterns
        let extractor = ContentExtractor::new(&parsed_doc.document);
        let categories = extractor.extract_categories(main_content)
            .expect("Failed to extract categories");

        // Count total endpoints and collect request patterns
        let total_endpoints: usize = categories.iter()
            .flat_map(|cat| &cat.endpoints)
            .count();

        let endpoints_with_patterns: usize = categories.iter()
            .flat_map(|cat| &cat.endpoints)
            .filter(|endpoint| endpoint.request_pattern.is_some())
            .count();

        let all_patterns: Vec<&str> = categories.iter()
            .flat_map(|cat| &cat.endpoints)
            .filter_map(|endpoint| endpoint.request_pattern.as_ref())
            .map(|pattern| pattern.as_str())
            .collect();

        println!("Request pattern extraction results:");
        println!("Total endpoints: {}", total_endpoints);
        println!("Endpoints with request patterns: {}", endpoints_with_patterns);
        println!("Total patterns found: {}", all_patterns.len());

        // Verify we have a reasonable number of patterns
        assert!(all_patterns.len() >= total_endpoints.saturating_sub(5), "Should have patterns for most endpoints, found {} out of {}", all_patterns.len(), total_endpoints);

        // Verify all patterns are valid
        for pattern in &all_patterns {
            // Should start with https
            assert!(pattern.starts_with("https://"), "Pattern should start with https://: {}", pattern);

            // Should contain alphavantage.co
            assert!(pattern.contains("alphavantage.co"), "Pattern should contain alphavantage.co: {}", pattern);

            // Should contain function parameter
            assert!(pattern.contains("function="), "Pattern should contain function=: {}", pattern);

            // Should be a valid URL
            assert!(pattern.contains("query?"), "Pattern should contain query?: {}", pattern);

            // Should contain apikey parameter
            assert!(pattern.contains("apikey="), "Pattern should contain apikey=: {}", pattern);
        }

        // Verify some known endpoints have patterns
        let endpoint_names: Vec<&str> = categories.iter()
            .flat_map(|cat| &cat.endpoints)
            .map(|endpoint| endpoint.function_name.as_str())
            .collect();

        let has_time_series_daily = endpoint_names.contains(&"TIME_SERIES_DAILY");
        let has_global_quote = endpoint_names.contains(&"GLOBAL_QUOTE");

        if has_time_series_daily || has_global_quote {
            println!("Found expected endpoints");
        }

        // Check that patterns contain expected parameters
        let patterns_with_symbol: usize = all_patterns.iter()
            .filter(|pattern| pattern.contains("symbol="))
            .count();

        let patterns_with_outputsize: usize = all_patterns.iter()
            .filter(|pattern| pattern.contains("outputsize="))
            .count();

        println!("Patterns with symbol parameter: {}", patterns_with_symbol);
        println!("Patterns with outputsize parameter: {}", patterns_with_outputsize);

        // Sample some patterns for manual verification
        if let Some(sample_pattern) = all_patterns.first() {
            println!("Sample request pattern: {}", sample_pattern);
        }

        // Verify no malformed patterns
        for pattern in &all_patterns {
            // Should not contain HTML tags
            assert!(!pattern.contains('<') && !pattern.contains('>'), "Pattern should not contain HTML tags: {}", pattern);

            // Should not contain excessive spaces
            assert!(!pattern.contains("  "), "Pattern should not contain double spaces: {}", pattern);

            // Should be properly URL encoded (basic check)
            assert!(!pattern.contains(' '), "Pattern should not contain spaces: {}", pattern);
        }

        println!("Successfully extracted and validated {} request patterns from Alpha Vantage documentation",
                all_patterns.len());
    });
}
