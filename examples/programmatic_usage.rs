//! Programmatic usage example for the Alpha Vantage Documentation Extractor.
//!
//! This example demonstrates how to use the library programmatically
//! as a dependency in your own Rust applications.

use alphavantage_doc_extractor::adapters::{FileWriter, HttpClient};
use alphavantage_doc_extractor::domain::parser::identify_main_content;
use alphavantage_doc_extractor::domain::{
    ContentExtractor, HtmlCleaner, MarkdownRenderer, RawUrl, ValidatedUrl,
};
use alphavantage_doc_extractor::ports::Writer;

/// Programmatic extraction example
///
/// This example shows how to use the Alpha Vantage Documentation Extractor
/// as a library in your own Rust applications.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Alpha Vantage Documentation Extractor - Programmatic Usage");
    println!("============================================================");

    // Example 1: Basic extraction
    println!("\n📖 Example 1: Basic extraction");
    let result = extract_documentation_basic(
        "https://www.alphavantage.co/documentation",
        "basic-extraction.md",
    )
    .await?;

    println!(
        "✅ Basic extraction completed: {} categories, {} endpoints",
        result.categories, result.endpoints
    );

    // Example 2: Custom processing pipeline
    println!("\n🔄 Example 2: Custom processing pipeline");
    let custom_result = extract_with_custom_processing(
        "https://www.alphavantage.co/documentation",
        "custom-processing.md",
    )
    .await?;

    println!(
        "✅ Custom processing completed: {} bytes processed",
        custom_result.bytes_processed
    );

    // Example 3: Filtered extraction
    println!("\n🎯 Example 3: Filtered extraction (Stock APIs only)");
    let filtered_result = extract_filtered_categories(
        "https://www.alphavantage.co/documentation",
        "stock-apis-only.md",
        |category_name| category_name.contains("Stock"),
    )
    .await?;

    println!(
        "✅ Filtered extraction completed: {} stock-related categories",
        filtered_result.categories
    );

    println!("\n🎉 All programmatic examples completed successfully!");
    Ok(())
}

/// Basic extraction result
#[derive(Debug)]
struct ExtractionResult {
    categories: usize,
    endpoints: usize,
    file_size: usize,
}

/// Perform basic documentation extraction
async fn extract_documentation_basic(
    url: &str,
    output_path: &str,
) -> Result<ExtractionResult, Box<dyn std::error::Error>> {
    // Validate URL
    let raw_url = RawUrl::new(url.to_string());
    let validated_url = ValidatedUrl::validate(raw_url)?;

    // Create HTTP client
    let client = HttpClient::new()?;

    // Fetch HTML
    let raw_html = client.fetch(validated_url.clone()).await?;

    // Clean HTML
    let cleaner = HtmlCleaner::new();
    let cleaned_html = cleaner.clean(&raw_html.content)?;

    // Parse and identify main content
    let html_dom = scraper::Html::parse_document(&cleaned_html);
    let main_content = identify_main_content(&html_dom)?;

    // Extract documentation
    let extractor = ContentExtractor::new(&html_dom);
    let categories = extractor.extract_categories(main_content)?;

    // Create document structure
    let total_endpoints: usize = categories.iter().map(|cat| cat.endpoints.len()).sum();
    let doc_metadata = alphavantage_doc_extractor::domain::DocumentMetadata::new(
        "Alpha Vantage API Documentation".to_string(),
        validated_url,
    )
    .with_counts(total_endpoints, categories.len());

    let document_structure =
        alphavantage_doc_extractor::domain::DocumentStructure::new(doc_metadata, categories);

    // Render to markdown
    let renderer = MarkdownRenderer::new();
    let markdown = renderer.render(&document_structure, true)?;

    // Write to file
    let writer = FileWriter::new();
    writer.write(output_path, &markdown)?;

    let file_size = writer.get_file_size(output_path)?;

    Ok(ExtractionResult {
        categories: document_structure.categories.len(),
        endpoints: total_endpoints,
        file_size: file_size as usize,
    })
}

/// Custom processing result
#[derive(Debug)]
struct CustomProcessingResult {
    bytes_processed: usize,
    processing_time_ms: u128,
    validation_passed: bool,
}

/// Extract with custom processing and detailed metrics
async fn extract_with_custom_processing(
    url: &str,
    output_path: &str,
) -> Result<CustomProcessingResult, Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();

    // Standard extraction pipeline
    let raw_url = RawUrl::new(url.to_string());
    let validated_url = ValidatedUrl::validate(raw_url)?;
    let client = HttpClient::new()?;
    let raw_html = client.fetch(validated_url.clone()).await?;

    // Custom processing: Count bytes at each stage
    let original_bytes = raw_html.content.len();

    let cleaner = HtmlCleaner::new();
    let cleaned_html = cleaner.clean(&raw_html.content)?;
    let cleaned_bytes = cleaned_html.len();

    let html_dom = scraper::Html::parse_document(&cleaned_html);
    let main_content = identify_main_content(&html_dom)?;

    let extractor = ContentExtractor::new(&html_dom);
    let categories = extractor.extract_categories(main_content)?;
    let total_endpoints: usize = categories.iter().map(|cat| cat.endpoints.len()).sum();

    // Create document structure with custom metadata
    let doc_metadata = alphavantage_doc_extractor::domain::DocumentMetadata::new(
        "Custom Processed Alpha Vantage API Documentation".to_string(),
        validated_url,
    )
    .with_counts(total_endpoints, categories.len());

    let document_structure =
        alphavantage_doc_extractor::domain::DocumentStructure::new(doc_metadata, categories);

    // Custom rendering with validation
    let renderer = MarkdownRenderer::new();
    let markdown = renderer.render(&document_structure, true)?;
    let markdown_bytes = markdown.len();

    // Validate the output
    let validator = alphavantage_doc_extractor::domain::renderer::OutputValidator::new();
    let validation_report = validator.validate(&markdown)?;
    let validation_passed = validation_report.is_valid();

    // Write to file
    let writer = FileWriter::new();
    writer.write(output_path, &markdown)?;

    let processing_time = start_time.elapsed().as_millis();
    let total_bytes_processed = original_bytes + cleaned_bytes + markdown_bytes;

    Ok(CustomProcessingResult {
        bytes_processed: total_bytes_processed,
        processing_time_ms: processing_time,
        validation_passed,
    })
}

/// Filtered extraction result
#[derive(Debug)]
struct FilteredExtractionResult {
    categories: usize,
    total_endpoints: usize,
    filtered_endpoints: usize,
}

/// Extract only specific categories based on a filter function
async fn extract_filtered_categories<F>(
    url: &str,
    output_path: &str,
    category_filter: F,
) -> Result<FilteredExtractionResult, Box<dyn std::error::Error>>
where
    F: Fn(&str) -> bool,
{
    // Standard extraction
    let raw_url = RawUrl::new(url.to_string());
    let validated_url = ValidatedUrl::validate(raw_url)?;
    let client = HttpClient::new()?;
    let raw_html = client.fetch(validated_url.clone()).await?;

    let cleaner = HtmlCleaner::new();
    let cleaned_html = cleaner.clean(&raw_html.content)?;
    let html_dom = scraper::Html::parse_document(&cleaned_html);
    let main_content = identify_main_content(&html_dom)?;

    let extractor = ContentExtractor::new(&html_dom);
    let all_categories = extractor.extract_categories(main_content)?;

    // Filter categories
    let filtered_categories: Vec<_> = all_categories
        .into_iter()
        .filter(|cat| category_filter(&cat.name))
        .collect();

    let total_endpoints: usize = filtered_categories
        .iter()
        .map(|cat| cat.endpoints.len())
        .sum();
    let filtered_endpoints = total_endpoints;

    // Create filtered document structure
    let doc_metadata = alphavantage_doc_extractor::domain::DocumentMetadata::new(
        "Filtered Alpha Vantage API Documentation".to_string(),
        validated_url,
    )
    .with_counts(filtered_endpoints, filtered_categories.len());

    let document_structure = alphavantage_doc_extractor::domain::DocumentStructure::new(
        doc_metadata,
        filtered_categories,
    );

    // Render and save
    let renderer = MarkdownRenderer::new();
    let markdown = renderer.render(&document_structure, true)?;

    let writer = FileWriter::new();
    writer.write(output_path, &markdown)?;

    Ok(FilteredExtractionResult {
        categories: document_structure.categories.len(),
        total_endpoints: filtered_endpoints,
        filtered_endpoints,
    })
}

/// Advanced programmatic API usage patterns
mod advanced_patterns {
    use super::*;

    /// Extract documentation with custom retry logic
    pub async fn extract_with_retry(
        url: &str,
        output_path: &str,
        max_retries: usize,
    ) -> Result<ExtractionResult, Box<dyn std::error::Error>> {
        let mut last_error = None;

        for attempt in 1..=max_retries {
            println!("Attempt {} of {}", attempt, max_retries);

            match extract_documentation_basic(url, output_path).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    println!("Attempt {} failed: {}", attempt, e);
                    last_error = Some(e);

                    if attempt < max_retries {
                        // Exponential backoff
                        let delay_ms = 1000 * (2_u64.pow(attempt as u32 - 1));
                        println!("Retrying in {}ms...", delay_ms);
                        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| "All retry attempts failed".into()))
    }

    /// Extract to multiple formats simultaneously
    pub async fn extract_to_multiple_formats(
        url: &str,
        base_output_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Extract once
        let raw_url = RawUrl::new(url.to_string());
        let validated_url = ValidatedUrl::validate(raw_url)?;
        let client = HttpClient::new()?;
        let raw_html = client.fetch(validated_url.clone()).await?;

        let cleaner = HtmlCleaner::new();
        let cleaned_html = cleaner.clean(&raw_html.content)?;
        let html_dom = scraper::Html::parse_document(&cleaned_html);
        let main_content = identify_main_content(&html_dom)?;

        let extractor = ContentExtractor::new(&html_dom);
        let categories = extractor.extract_categories(main_content)?;
        let total_endpoints: usize = categories.iter().map(|cat| cat.endpoints.len()).sum();

        let doc_metadata = alphavantage_doc_extractor::domain::DocumentMetadata::new(
            "Alpha Vantage API Documentation".to_string(),
            validated_url,
        )
        .with_counts(total_endpoints, categories.len());

        let document_structure =
            alphavantage_doc_extractor::domain::DocumentStructure::new(doc_metadata, categories);

        let renderer = MarkdownRenderer::new();
        let writer = FileWriter::new();

        // Generate multiple formats
        let formats = vec![
            ("markdown", format!("{}.md", base_output_path)),
            ("json", format!("{}.json", base_output_path)),
        ];

        for (format_name, output_path) in formats {
            match format_name {
                "markdown" => {
                    let markdown = renderer.render(&document_structure, true)?;
                    writer.write(&output_path, &markdown)?;
                    println!("✅ Generated Markdown: {}", output_path);
                }
                "json" => {
                    // For JSON, we'd serialize the DocumentStructure
                    // This is a placeholder for future JSON export functionality
                    let json_content = format!(
                        "{{\"categories\": {}, \"endpoints\": {}}}",
                        document_structure.categories.len(),
                        total_endpoints
                    );
                    writer.write(&output_path, &json_content)?;
                    println!("✅ Generated JSON: {}", output_path);
                }
                _ => continue,
            }
        }

        Ok(())
    }
}

// Example of using the advanced patterns
#[allow(dead_code)]
async fn example_advanced_usage() -> Result<(), Box<dyn std::error::Error>> {
    // Using retry logic
    let result = advanced_patterns::extract_with_retry(
        "https://www.alphavantage.co/documentation",
        "retry-extraction.md",
        3,
    )
    .await?;

    println!(
        "Retry extraction succeeded: {} categories",
        result.categories
    );

    // Using multiple formats
    advanced_patterns::extract_to_multiple_formats(
        "https://www.alphavantage.co/documentation",
        "multi-format-output",
    )
    .await?;

    println!("Multi-format extraction completed");
    Ok(())
}
