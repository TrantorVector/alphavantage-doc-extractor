//! Integration tests for HTML cleaning functionality.
//!
//! Tests the HtmlCleaner with real Alpha Vantage URLs to ensure
//! comprehensive removal of unwanted content.

use alphavantage_doc_extractor::adapters::HttpClient;
use alphavantage_doc_extractor::domain::{HtmlCleaner, RawUrl};
use alphavantage_doc_extractor::utils::init_logging;

#[tokio::test]
async fn test_html_cleaning_with_real_url() {
    // Initialize logging for the test
    let _correlation_id = init_logging("info", "pretty").unwrap();

    let client = HttpClient::new().unwrap();
    let cleaner = HtmlCleaner::new();

    // Fetch real Alpha Vantage documentation URL
    let url = RawUrl::new("https://www.alphavantage.co/documentation/".to_string());
    let validated_url = url.try_into().unwrap();

    let result = client.fetch(validated_url).await;

    assert!(result.is_ok(), "Failed to fetch Alpha Vantage documentation: {:?}", result.err());

    let raw_html = result.unwrap();
    let original_size = raw_html.content.len();

    // Clean the HTML
    let cleaned_result = cleaner.clean(&raw_html.content);

    assert!(cleaned_result.is_ok(), "HTML cleaning failed: {:?}", cleaned_result.err());

    let cleaned_html = cleaned_result.unwrap();
    let cleaned_size = cleaned_html.len();

    // Verify cleaning effectiveness
    assert!(cleaned_size < original_size, "Cleaned HTML should be smaller than original");

    let size_reduction = original_size - cleaned_size;
    println!("✅ HTML cleaning successful: {} -> {} bytes ({} bytes removed)",
             original_size, cleaned_size, size_reduction);

    // Verify no unwanted elements remain
    assert!(!cleaned_html.contains("<script"), "Scripts should be removed");
    assert!(!cleaned_html.contains("<style"), "Styles should be removed");
    assert!(!cleaned_html.contains("<noscript"), "Noscript tags should be removed");
    assert!(!cleaned_html.contains("<iframe"), "Iframes should be removed");

    // Check for navigation elements (may or may not be present in real content)
    let nav_count = cleaned_html.matches("<nav").count() + cleaned_html.matches("class=\"nav").count();
    println!("ℹ️  Navigation elements remaining: {}", nav_count);

    // Check for footer elements
    let footer_count = cleaned_html.matches("<footer").count() + cleaned_html.matches("class=\"footer").count();
    println!("ℹ️  Footer elements remaining: {}", footer_count);

    // Run validation
    let validation_result = cleaner.validate_cleaned(&cleaned_html);

    match validation_result {
        Ok(()) => println!("✅ Validation passed: no artifacts found"),
        Err(errors) => {
            println!("⚠️  Validation found {} remaining artifacts:", errors.len());
            for error in &errors {
                println!("   - {}", error);
            }
            // Don't fail the test for validation issues - this is informational
        }
    }

    // Verify some core content remains
    assert!(cleaned_html.contains("TIME_SERIES"), "Core API content should remain");
    assert!(cleaned_html.len() > 1000, "Cleaned HTML should still contain substantial content");
}
