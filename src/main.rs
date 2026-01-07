//! Alpha Vantage Documentation Extractor
//!
//! A tool to extract and process API documentation from Alpha Vantage
//! and convert it to LLM-optimized Markdown format.

use alphavantage_doc_extractor::adapters::{Config, FileWriter, HttpClient};
use alphavantage_doc_extractor::domain::parser::identify_main_content;
use alphavantage_doc_extractor::domain::renderer::OutputValidator;
use alphavantage_doc_extractor::domain::{
    ContentExtractor, HtmlCleaner, MarkdownRenderer, RawUrl, ValidatedUrl,
};
use alphavantage_doc_extractor::ports::Writer;
use alphavantage_doc_extractor::utils::{init_logging, ExtractionError};
use std::time::Instant;
use tracing::{error, info, warn};

/// Main entry point for the Alpha Vantage Documentation Extractor.
///
/// Parses command-line arguments, validates configuration, initializes logging,
/// and orchestrates the complete document extraction and rendering pipeline.
#[tokio::main]
async fn main() {
    let start_time = Instant::now();

    // Parse CLI arguments
    let config = Config::from_args();

    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("Configuration validation failed: {}", e);
        std::process::exit(1);
    }

    // Initialize logging system
    let correlation_id = match init_logging(&config.log_level, &config.log_format) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Failed to initialize logging: {}", e);
            std::process::exit(1);
        }
    };

    // Log application startup and configuration
    info!(
        correlation_id = %correlation_id,
        url = %config.url,
        output = %config.output,
        log_level = %config.log_level,
        log_format = %config.log_format,
        no_validation = %config.no_validation,
        backup = %config.backup,
        "Alpha Vantage Documentation Extractor starting"
    );

    // Run the extraction pipeline
    let result = run(config, correlation_id.clone()).await;

    // Log completion status and duration
    let duration = start_time.elapsed();
    match result {
        Ok(_) => {
            info!(
                correlation_id = %correlation_id,
                duration_ms = %duration.as_millis(),
                "Extraction completed successfully"
            );
            std::process::exit(0);
        }
        Err(e) => {
            error!(
                correlation_id = %correlation_id,
                duration_ms = %duration.as_millis(),
                error = %e,
                "Extraction failed"
            );
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Run the complete document extraction and rendering pipeline.
///
/// Orchestrates all components: HTTP fetching, HTML parsing, content extraction,
/// markdown rendering, validation, and file writing.
async fn run(config: Config, correlation_id: String) -> Result<(), ExtractionError> {
    info!(correlation_id = %correlation_id, "Starting document extraction pipeline");

    // Step 1: Create HTTP client
    let client = HttpClient::new().map_err(|e| {
        error!(correlation_id = %correlation_id, error = %e, "Failed to create HTTP client");
        e
    })?;

    // Step 2: Validate and prepare URL
    info!(correlation_id = %correlation_id, url = %config.url, "Validating source URL");
    let raw_url = RawUrl::new(config.url.clone());
    let validated_url = ValidatedUrl::validate(raw_url).map_err(|e| {
        error!(correlation_id = %correlation_id, url = %config.url, error = %e, "URL validation failed");
        ExtractionError::UrlParse(e)
    })?;

    // Step 3: Fetch HTML document
    info!(correlation_id = %correlation_id, "Fetching HTML document");
    let raw_html = client.fetch(validated_url.clone()).await.map_err(|e| {
        error!(correlation_id = %correlation_id, url = %validated_url, error = %e, "HTTP fetch failed");
        e
    })?;

    info!(
        correlation_id = %correlation_id,
        content_length = %raw_html.content_length(),
        "Successfully fetched HTML document"
    );

    // Step 4: Parse HTML document
    info!(correlation_id = %correlation_id, "Parsing HTML document");
    let _html_dom = scraper::Html::parse_document(&raw_html.content);

    // Step 5: Clean HTML content
    info!(correlation_id = %correlation_id, "Cleaning HTML content");
    let cleaner = HtmlCleaner::new();
    let cleaned_html_string = cleaner.clean(&raw_html.content).map_err(|e| {
        error!(correlation_id = %correlation_id, error = %e, "HTML cleaning failed");
        e
    })?;

    // Step 6: Parse cleaned HTML back to Html DOM
    let cleaned_html_dom = scraper::Html::parse_document(&cleaned_html_string);

    // Step 7: Identify main content area
    info!(correlation_id = %correlation_id, "Identifying main content area");
    let main_content = identify_main_content(&cleaned_html_dom).map_err(|e| {
        error!(correlation_id = %correlation_id, error = %e, "Content identification failed");
        e
    })?;

    // Step 8: Extract API documentation content
    info!(correlation_id = %correlation_id, "Extracting API documentation content");
    let extractor = ContentExtractor::new(&cleaned_html_dom);
    let document_structure = extractor.extract_categories(main_content).map_err(|e| {
        error!(correlation_id = %correlation_id, error = %e, "Content extraction failed");
        e
    })?;

    let total_endpoints: usize = document_structure
        .iter()
        .map(|cat| cat.endpoints.len())
        .sum();
    info!(
        correlation_id = %correlation_id,
        categories = %document_structure.len(),
        endpoints = %total_endpoints,
        "Successfully extracted API documentation structure"
    );

    // Create DocumentStructure from the categories
    let doc_metadata = alphavantage_doc_extractor::domain::DocumentMetadata::new(
        "Alpha Vantage API Documentation".to_string(),
        validated_url,
    )
    .with_counts(total_endpoints, document_structure.len());

    let document_structure = alphavantage_doc_extractor::domain::DocumentStructure::new(
        doc_metadata,
        document_structure,
    );

    // Step 8: Render to markdown
    info!(correlation_id = %correlation_id, "Rendering to LLM-optimized markdown");
    let renderer = MarkdownRenderer::new();
    let markdown = renderer.render(&document_structure).map_err(|e| {
        error!(correlation_id = %correlation_id, error = %e, "Markdown rendering failed");
        e
    })?;

    // Step 9: Validate output (if enabled)
    if !config.no_validation {
        info!(correlation_id = %correlation_id, "Validating output");
        let validator = OutputValidator::new();
        let validation_report = validator.validate(&markdown).map_err(|e| {
            error!(correlation_id = %correlation_id, error = %e, "Output validation failed");
            e
        })?;

        if !validation_report.is_valid() {
            warn!(
                correlation_id = %correlation_id,
                errors = %validation_report.errors.len(),
                warnings = %validation_report.warnings.len(),
                "Output validation found issues"
            );

            for error in &validation_report.errors {
                error!(correlation_id = %correlation_id, error = %error);
            }

            for warning in &validation_report.warnings {
                warn!(correlation_id = %correlation_id, warning = %warning);
            }

            return Err(ExtractionError::Render(
                alphavantage_doc_extractor::utils::RenderError::ValidationFailed(format!(
                    "Output validation failed with {} errors and {} warnings",
                    validation_report.errors.len(),
                    validation_report.warnings.len()
                )),
            ));
        } else {
            info!(correlation_id = %correlation_id, "Output validation passed");
        }
    } else {
        info!(correlation_id = %correlation_id, "Output validation skipped");
    }

    // Step 10: Write to file
    info!(correlation_id = %correlation_id, output_path = %config.output, backup = %config.backup, "Writing output file");
    let writer = FileWriter::new();

    if config.backup {
        writer.write_with_backup(&config.output, &markdown).map_err(|e| {
            error!(correlation_id = %correlation_id, output_path = %config.output, error = %e, "File writing with backup failed");
            e
        })?;
    } else {
        writer.write(&config.output, &markdown).map_err(|e| {
            error!(correlation_id = %correlation_id, output_path = %config.output, error = %e, "File writing failed");
            e
        })?;
    }

    // Log final statistics
    let output_size = writer.get_file_size(&config.output).unwrap_or(0);
    info!(
        correlation_id = %correlation_id,
        output_path = %config.output,
        output_size_bytes = %output_size,
        categories = %document_structure.categories.len(),
        endpoints = %total_endpoints,
        "Document extraction and rendering completed successfully"
    );

    Ok(())
}
