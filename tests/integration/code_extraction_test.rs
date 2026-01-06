//! Integration tests for code example extraction functionality.
//!
//! These tests verify that the ContentExtractor can successfully extract
//! Python code examples from real Alpha Vantage documentation pages
//! while filtering out other languages and duplicates.

use alphavantage_doc_extractor::adapters::HttpClient;
use alphavantage_doc_extractor::domain::{ContentExtractor, RawHtml, ValidatedUrl};
use alphavantage_doc_extractor::utils::UrlParseError;

use std::time::Duration;
use tokio_test::block_on;

#[test]
fn test_code_extraction_integration() {
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

        // Extract categories with endpoints and code examples
        let extractor = ContentExtractor::new(&parsed_doc.document);
        let categories = extractor.extract_categories(main_content)
            .expect("Failed to extract categories");

        // Count total endpoints and code examples
        let total_endpoints: usize = categories.iter()
            .flat_map(|cat| &cat.endpoints)
            .count();

        let endpoints_with_code: usize = categories.iter()
            .flat_map(|cat| &cat.endpoints)
            .filter(|endpoint| endpoint.python_example.is_some())
            .count();

        let all_code_examples: Vec<&alphavantage_doc_extractor::domain::CodeExample> = categories.iter()
            .flat_map(|cat| &cat.endpoints)
            .filter_map(|endpoint| endpoint.python_example.as_ref())
            .collect();

        println!("Code extraction results:");
        println!("Total endpoints: {}", total_endpoints);
        println!("Endpoints with Python code: {}", endpoints_with_code);
        println!("Total Python code examples: {}", all_code_examples.len());

        // Verify we have a reasonable number of code examples
        assert!(all_code_examples.len() >= 20, "Expected at least 20 Python code examples, found {}", all_code_examples.len());
        assert!(all_code_examples.len() <= 100, "Expected at most 100 Python code examples, found {}", all_code_examples.len());

        // Verify all extracted code is actually Python
        for example in &all_code_examples {
            assert_eq!(example.language, "python", "Non-Python code example found");

            // Check for Python keywords
            let has_python_keywords = example.code.contains("import") ||
                example.code.contains("def ") ||
                example.code.contains("requests.get") ||
                example.code.contains("response.json()");

            assert!(has_python_keywords, "Code example doesn't appear to be Python: {}", &example.code[..100]);

            // Verify no other languages leaked through
            assert!(!example.code.contains("<-"), "R code found in Python example");
            assert!(!example.code.contains("library("), "R code found in Python example");
            assert!(!example.code.contains("Sub "), "VBA code found in Python example");
            assert!(!example.code.contains("End Sub"), "VBA code found in Python example");
            assert!(!example.code.contains("function ["), "MATLAB code found in Python example");

            // Verify no JSON responses (pure JSON without code context)
            let trimmed = example.code.trim();
            assert!(!trimmed.starts_with("{") || !trimmed.ends_with("}"),
                   "JSON response found in Python example: {}", &trimmed[..50]);
        }

        // Test deduplication
        let deduplicated = extractor.deduplicate_code_examples(
            all_code_examples.iter().cloned().cloned().collect()
        );

        println!("Deduplication: {} -> {} examples", all_code_examples.len(), deduplicated.len());

        // Deduplication should reduce duplicates (though may not always happen with real data)
        assert!(deduplicated.len() <= all_code_examples.len(),
               "Deduplication should not increase the number of examples");

        // Verify some common endpoints have code examples
        let endpoint_names_with_code: Vec<&str> = categories.iter()
            .flat_map(|cat| &cat.endpoints)
            .filter(|endpoint| endpoint.python_example.is_some())
            .map(|endpoint| endpoint.function_name.as_str())
            .collect();

        println!("Endpoints with code examples: {:?}", endpoint_names_with_code);

        // Check for some expected endpoints
        let has_time_series_daily = endpoint_names_with_code.contains(&"TIME_SERIES_DAILY");
        let has_time_series_intraday = endpoint_names_with_code.contains(&"TIME_SERIES_INTRADAY");
        let has_global_quote = endpoint_names_with_code.contains(&"GLOBAL_QUOTE");

        if has_time_series_daily || has_time_series_intraday || has_global_quote {
            println!("Found expected endpoints with code examples");
        }

        // Print a sample code example
        if let Some(sample_example) = all_code_examples.first() {
            println!("Sample Python code example (first 200 chars):");
            println!("{}", &sample_example.code[..sample_example.code.len().min(200)]);
        }

        println!("Successfully extracted and filtered {} Python code examples from Alpha Vantage documentation",
                all_code_examples.len());
    });
}
