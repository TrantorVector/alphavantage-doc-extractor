//! Performance benchmarks for the Alpha Vantage Documentation Extractor.
//!
//! This module uses criterion.rs to benchmark various components of the
//! extraction pipeline and ensure performance targets are met.

use alphavantage_doc_extractor::adapters::HttpClient;
use alphavantage_doc_extractor::domain::parser::identify_main_content;
use alphavantage_doc_extractor::domain::renderer::OutputValidator;
use alphavantage_doc_extractor::domain::{
    ContentExtractor, HtmlCleaner, MarkdownRenderer, RawUrl, ValidatedUrl,
};
use alphavantage_doc_extractor::utils::init_logging;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

/// Benchmark the complete extraction pipeline.
///
/// Target: Complete extraction in <30 seconds
/// Target: Peak memory usage <100MB
fn bench_full_extraction_pipeline(c: &mut Criterion) {
    // Initialize logging (suppress output for benchmarks)
    let _correlation_id = init_logging("error", "pretty").unwrap();

    c.bench_function("full_extraction_pipeline", |b| {
        b.iter(|| {
            // Setup: Create HTTP client and fetch documentation
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let client = HttpClient::new().unwrap();
                let raw_url = RawUrl::new("https://www.alphavantage.co/documentation".to_string());
                let validated_url = ValidatedUrl::validate(raw_url).unwrap();

                let raw_html = client.fetch(validated_url.clone()).await.unwrap();

                // Parse HTML document
                let html_dom = scraper::Html::parse_document(&raw_html.content);

                // Clean HTML content
                let cleaner = HtmlCleaner::new();
                let cleaned_html_string = cleaner.clean(&raw_html.content).unwrap();
                let cleaned_html_dom = scraper::Html::parse_document(&cleaned_html_string);

                // Identify main content area
                let main_content = identify_main_content(&cleaned_html_dom).unwrap();

                // Extract API documentation content
                let extractor = ContentExtractor::new(&cleaned_html_dom);
                let document_structure = extractor.extract_categories(main_content).unwrap();

                // Create DocumentStructure
                let total_endpoints: usize = document_structure
                    .iter()
                    .map(|cat| cat.endpoints.len())
                    .sum();
                let doc_metadata = alphavantage_doc_extractor::domain::DocumentMetadata::new(
                    "Alpha Vantage API Documentation".to_string(),
                    validated_url,
                )
                .with_counts(total_endpoints, document_structure.len());

                let document_structure = alphavantage_doc_extractor::domain::DocumentStructure::new(
                    doc_metadata,
                    document_structure,
                );

                // Render to markdown
                let renderer = MarkdownRenderer::new();
                let markdown = renderer.render(&document_structure).unwrap();

                // Validate output
                let validator = OutputValidator::new();
                let validation_report = validator.validate(&markdown).unwrap();
                assert!(validation_report.is_valid());

                // Return the result to prevent optimization
                black_box(markdown)
            });
        });
    });
}

/// Benchmark HTML parsing performance.
fn bench_html_parsing(c: &mut Criterion) {
    // Initialize logging
    let _correlation_id = init_logging("error", "pretty").unwrap();

    c.bench_function("html_parsing", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let client = HttpClient::new().unwrap();
                let raw_url = RawUrl::new("https://www.alphavantage.co/documentation".to_string());
                let validated_url = ValidatedUrl::validate(raw_url).unwrap();

                let raw_html = client.fetch(validated_url).await.unwrap();
                let html_dom = scraper::Html::parse_document(&raw_html.content);

                black_box(html_dom)
            });
        });
    });
}

/// Benchmark HTML cleaning performance.
fn bench_html_cleaning(c: &mut Criterion) {
    // Initialize logging
    let _correlation_id = init_logging("error", "pretty").unwrap();

    c.bench_function("html_cleaning", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let client = HttpClient::new().unwrap();
                let raw_url = RawUrl::new("https://www.alphavantage.co/documentation".to_string());
                let validated_url = ValidatedUrl::validate(raw_url).unwrap();

                let raw_html = client.fetch(validated_url).await.unwrap();

                let cleaner = HtmlCleaner::new();
                let cleaned_html = cleaner.clean(&raw_html.content).unwrap();

                black_box(cleaned_html)
            });
        });
    });
}

/// Benchmark content extraction performance.
fn bench_content_extraction(c: &mut Criterion) {
    // Initialize logging
    let _correlation_id = init_logging("error", "pretty").unwrap();

    c.bench_function("content_extraction", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let client = HttpClient::new().unwrap();
                let raw_url = RawUrl::new("https://www.alphavantage.co/documentation".to_string());
                let validated_url = ValidatedUrl::validate(raw_url).unwrap();

                let raw_html = client.fetch(validated_url).await.unwrap();

                let cleaner = HtmlCleaner::new();
                let cleaned_html_string = cleaner.clean(&raw_html.content).unwrap();
                let cleaned_html_dom = scraper::Html::parse_document(&cleaned_html_string);

                let main_content = identify_main_content(&cleaned_html_dom).unwrap();

                let extractor = ContentExtractor::new(&cleaned_html_dom);
                let document_structure = extractor.extract_categories(main_content).unwrap();

                black_box(document_structure)
            });
        });
    });
}

/// Benchmark markdown rendering performance.
fn bench_markdown_rendering(c: &mut Criterion) {
    // Initialize logging
    let _correlation_id = init_logging("error", "pretty").unwrap();

    c.bench_function("markdown_rendering", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let client = HttpClient::new().unwrap();
                let raw_url = RawUrl::new("https://www.alphavantage.co/documentation".to_string());
                let validated_url = ValidatedUrl::validate(raw_url).unwrap();

                let raw_html = client.fetch(validated_url.clone()).await.unwrap();

                let cleaner = HtmlCleaner::new();
                let cleaned_html_string = cleaner.clean(&raw_html.content).unwrap();
                let cleaned_html_dom = scraper::Html::parse_document(&cleaned_html_string);

                let main_content = identify_main_content(&cleaned_html_dom).unwrap();

                let extractor = ContentExtractor::new(&cleaned_html_dom);
                let document_structure = extractor.extract_categories(main_content).unwrap();

                let total_endpoints: usize = document_structure
                    .iter()
                    .map(|cat| cat.endpoints.len())
                    .sum();
                let doc_metadata = alphavantage_doc_extractor::domain::DocumentMetadata::new(
                    "Alpha Vantage API Documentation".to_string(),
                    validated_url,
                )
                .with_counts(total_endpoints, document_structure.len());

                let document_structure = alphavantage_doc_extractor::domain::DocumentStructure::new(
                    doc_metadata,
                    document_structure,
                );

                let renderer = MarkdownRenderer::new();
                let markdown = renderer.render(&document_structure).unwrap();

                black_box(markdown)
            });
        });
    });
}

/// Benchmark output validation performance.
fn bench_output_validation(c: &mut Criterion) {
    // Initialize logging
    let _correlation_id = init_logging("error", "pretty").unwrap();

    c.bench_function("output_validation", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let client = HttpClient::new().unwrap();
                let raw_url = RawUrl::new("https://www.alphavantage.co/documentation".to_string());
                let validated_url = ValidatedUrl::validate(raw_url).unwrap();

                let raw_html = client.fetch(validated_url.clone()).await.unwrap();

                let cleaner = HtmlCleaner::new();
                let cleaned_html_string = cleaner.clean(&raw_html.content).unwrap();
                let cleaned_html_dom = scraper::Html::parse_document(&cleaned_html_string);

                let main_content = identify_main_content(&cleaned_html_dom).unwrap();

                let extractor = ContentExtractor::new(&cleaned_html_dom);
                let document_structure = extractor.extract_categories(main_content).unwrap();

                let total_endpoints: usize = document_structure
                    .iter()
                    .map(|cat| cat.endpoints.len())
                    .sum();
                let doc_metadata = alphavantage_doc_extractor::domain::DocumentMetadata::new(
                    "Alpha Vantage API Documentation".to_string(),
                    validated_url,
                )
                .with_counts(total_endpoints, document_structure.len());

                let document_structure = alphavantage_doc_extractor::domain::DocumentStructure::new(
                    doc_metadata,
                    document_structure,
                );

                let renderer = MarkdownRenderer::new();
                let markdown = renderer.render(&document_structure).unwrap();

                let validator = OutputValidator::new();
                let validation_report = validator.validate(&markdown).unwrap();

                black_box(validation_report)
            });
        });
    });
}

criterion_group! {
    name = extraction_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(10)
        .warm_up_time(Duration::from_secs(1));
    targets =
        bench_full_extraction_pipeline,
        bench_html_parsing,
        bench_html_cleaning,
        bench_content_extraction,
        bench_markdown_rendering,
        bench_output_validation
}

criterion_main!(extraction_benches);
