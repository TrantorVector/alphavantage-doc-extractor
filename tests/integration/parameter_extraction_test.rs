//! Integration tests for parameter extraction functionality.
//!
//! These tests verify that the ContentExtractor can successfully extract
//! parameters from real Alpha Vantage documentation pages.

use alphavantage_doc_extractor::adapters::HttpClient;
use alphavantage_doc_extractor::domain::{ContentExtractor, RawHtml, ValidatedUrl};
use alphavantage_doc_extractor::utils::UrlParseError;

use std::time::Duration;
use tokio_test::block_on;

#[test]
fn test_parameter_extraction_integration() {
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

        // Extract categories with endpoints and parameters
        let extractor = ContentExtractor::new(&parsed_doc.document);
        let categories = extractor.extract_categories(main_content)
            .expect("Failed to extract categories");

        // Find TIME_SERIES_DAILY endpoint
        let time_series_daily = categories.iter()
            .flat_map(|cat| &cat.endpoints)
            .find(|endpoint| endpoint.function_name == "TIME_SERIES_DAILY");

        assert!(time_series_daily.is_some(), "TIME_SERIES_DAILY endpoint not found");
        let endpoint = time_series_daily.unwrap();

        // Verify required parameters
        let required_names: Vec<&str> = endpoint.required_params.iter()
            .map(|p| p.name.as_str())
            .collect();

        assert!(required_names.contains(&"function"), "function parameter missing from TIME_SERIES_DAILY");
        assert!(required_names.contains(&"symbol"), "symbol parameter missing from TIME_SERIES_DAILY");
        assert!(required_names.contains(&"apikey"), "apikey parameter missing from TIME_SERIES_DAILY");

        // Check for optional parameters (may be present)
        let optional_names: Vec<&str> = endpoint.optional_params.iter()
            .map(|p| p.name.as_str())
            .collect();

        let has_outputsize = required_names.contains(&"outputsize") || optional_names.contains(&"outputsize");
        let has_datatype = required_names.contains(&"datatype") || optional_names.contains(&"datatype");

        // These are commonly optional parameters
        println!("TIME_SERIES_DAILY has outputsize parameter: {}", has_outputsize);
        println!("TIME_SERIES_DAILY has datatype parameter: {}", has_datatype);

        // Verify parameter properties
        for param in &endpoint.required_params {
            assert!(!param.name.is_empty(), "Parameter name should not be empty");
            assert!(!param.value_type.is_empty(), "Parameter type should not be empty");
            assert!(!param.description.is_empty(), "Parameter description should not be empty");

            // Check for reasonable defaults
            if let Some(default) = &param.default {
                assert!(!default.is_empty(), "Default value should not be empty if present");
            }

            // Check for reasonable options
            if !param.options.is_empty() {
                assert!(param.options.len() >= 2, "Options should have at least 2 values if present");
                for option in &param.options {
                    assert!(!option.is_empty(), "Option value should not be empty");
                }
            }
        }

        // Print detailed parameter information
        println!("TIME_SERIES_DAILY endpoint parameters:");
        println!("Required parameters ({}):", endpoint.required_params.len());
        for param in &endpoint.required_params {
            println!("  - {} ({}): {}", param.name, param.value_type, param.description);
            if let Some(default) = &param.default {
                println!("    Default: {}", default);
            }
            if !param.options.is_empty() {
                println!("    Options: {:?}", param.options);
            }
        }

        if !endpoint.optional_params.is_empty() {
            println!("Optional parameters ({}):", endpoint.optional_params.len());
            for param in &endpoint.optional_params {
                println!("  - {} ({}): {}", param.name, param.value_type, param.description);
            }
        }

        // Count total parameters across all endpoints
        let total_required: usize = categories.iter()
            .flat_map(|cat| &cat.endpoints)
            .map(|endpoint| endpoint.required_params.len())
            .sum();

        let total_optional: usize = categories.iter()
            .flat_map(|cat| &cat.endpoints)
            .map(|endpoint| endpoint.optional_params.len())
            .sum();

        println!("Total parameters extracted: {} required, {} optional",
                total_required, total_optional);

        // Basic sanity checks
        assert!(total_required > 0, "Should have at least some required parameters");
        assert!(total_required >= categories.iter().flat_map(|cat| &cat.endpoints).count(),
               "Should have at least one required parameter per endpoint");

        println!("Successfully extracted parameters from Alpha Vantage documentation");
    });
}
