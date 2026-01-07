//! Advanced usage example for the Alpha Vantage Documentation Extractor.
//!
//! This example demonstrates advanced configuration options and
//! programmatic usage of the extraction library.

use alphavantage_doc_extractor::adapters::{Config, FileWriter, HttpClient};
use alphavantage_doc_extractor::domain::parser::identify_main_content;
use alphavantage_doc_extractor::domain::renderer::OutputValidator;
use alphavantage_doc_extractor::domain::{
    ContentExtractor, HtmlCleaner, MarkdownRenderer, RawUrl, ValidatedUrl,
};
use alphavantage_doc_extractor::ports::Writer;
use alphavantage_doc_extractor::utils::init_logging;
use std::collections::HashMap;

/// Advanced extraction with custom configuration
///
/// This example shows:
/// - Custom logging configuration
/// - Step-by-step pipeline execution
/// - Error handling and recovery
/// - Performance monitoring
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize detailed logging
    let correlation_id = init_logging("debug", "pretty")?;

    println!("🚀 Alpha Vantage Documentation Extractor - Advanced Example");
    println!("=========================================================");
    println!("Correlation ID: {}", correlation_id);

    // Advanced configuration with all options
    let config = Config {
        url: "https://www.alphavantage.co/documentation".to_string(),
        output: "advanced-extraction-result.md".to_string(),
        log_level: "debug".to_string(),
        log_format: "pretty".to_string(),
        no_validation: false, // Enable validation
        backup: true,         // Create backups
    };

    // Validate configuration
    config.validate().map_err(|e| {
        eprintln!("❌ Configuration validation failed: {}", e);
        std::process::exit(1);
    })?;

    println!("✅ Configuration validated");
    println!("🔧 Advanced settings:");
    println!("   • Debug logging enabled");
    println!("   • Output validation: enabled");
    println!("   • Backup creation: enabled");

    // Execute the pipeline manually for demonstration
    let result = run_advanced_extraction(config, correlation_id.clone()).await;

    match result {
        Ok(stats) => {
            println!("✅ Extraction completed successfully!");
            println!("📊 Statistics:");
            println!("   • Categories extracted: {}", stats.categories);
            println!("   • Endpoints found: {}", stats.endpoints);
            println!("   • Output size: {} KB", stats.output_size_kb);
            println!("   • Processing time: {:.2}s", stats.processing_time);
        }
        Err(e) => {
            eprintln!("❌ Extraction failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Statistics from the extraction process
#[derive(Debug)]
struct ExtractionStats {
    categories: usize,
    endpoints: usize,
    output_size_kb: f64,
    processing_time: f64,
}

/// Run the complete extraction pipeline with detailed monitoring
async fn run_advanced_extraction(
    config: Config,
    correlation_id: String,
) -> Result<ExtractionStats, Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();

    println!("🔄 Starting advanced extraction pipeline...");

    // Step 1: HTTP Client with custom configuration
    println!("🌐 Initializing HTTP client...");
    let client = HttpClient::new().map_err(|e| {
        println!("❌ Failed to create HTTP client: {}", e);
        e
    })?;

    // Step 2: URL validation with detailed logging
    println!("🔗 Validating source URL: {}", config.url);
    let raw_url = RawUrl::new(config.url.clone());
    let validated_url = ValidatedUrl::validate(raw_url).map_err(|e| {
        println!("❌ URL validation failed: {}", e);
        e
    })?;

    // Step 3: Fetch HTML with progress indication
    println!("📡 Fetching HTML content...");
    let raw_html = client.fetch(validated_url.clone()).await.map_err(|e| {
        println!("❌ HTTP fetch failed: {}", e);
        e
    })?;

    println!("📦 Downloaded {} bytes", raw_html.content_length());

    // Step 4: HTML cleaning with statistics
    println!("🧹 Cleaning HTML content...");
    let cleaner = HtmlCleaner::new();
    let cleaned_html_string = cleaner.clean(&raw_html.content).map_err(|e| {
        println!("❌ HTML cleaning failed: {}", e);
        e
    })?;

    println!(
        "🧽 HTML cleaned, reduced from {} to {} bytes",
        raw_html.content.len(),
        cleaned_html_string.len()
    );

    // Step 5: Parse and identify main content
    println!("🔍 Parsing HTML and identifying main content...");
    let cleaned_html_dom = scraper::Html::parse_document(&cleaned_html_string);
    let main_content = identify_main_content(&cleaned_html_dom).map_err(|e| {
        println!("❌ Content identification failed: {}", e);
        e
    })?;

    println!("🎯 Main content area identified");

    // Step 6: Extract documentation structure
    println!("📋 Extracting API documentation structure...");
    let extractor = ContentExtractor::new(&cleaned_html_dom);
    let categories = extractor.extract_categories(main_content).map_err(|e| {
        println!("❌ Content extraction failed: {}", e);
        e
    })?;

    let total_endpoints: usize = categories.iter().map(|cat| cat.endpoints.len()).sum();
    println!(
        "📊 Found {} categories with {} total endpoints",
        categories.len(),
        total_endpoints
    );

    // Step 7: Build document structure
    println!("🏗️ Building document structure...");
    let doc_metadata = alphavantage_doc_extractor::domain::DocumentMetadata::new(
        "Alpha Vantage API Documentation".to_string(),
        validated_url,
    )
    .with_counts(total_endpoints, categories.len());

    let document_structure =
        alphavantage_doc_extractor::domain::DocumentStructure::new(doc_metadata, categories);

    // Step 8: Render with LLM optimization
    println!("📝 Rendering to LLM-optimized Markdown...");
    let renderer = MarkdownRenderer::new();
    let markdown = renderer
        .render(&document_structure, !config.no_validation)
        .map_err(|e| {
            println!("❌ Markdown rendering failed: {}", e);
            e
        })?;

    println!("✨ Generated {} characters of markdown", markdown.len());

    // Step 9: Validate output (if enabled)
    if !config.no_validation {
        println!("✅ Validating output...");
        let validator = OutputValidator::new();
        let validation_report = validator.validate(&markdown).map_err(|e| {
            println!("❌ Output validation failed: {}", e);
            e
        })?;

        if validation_report.is_valid() {
            println!("🎉 Output validation passed!");
            println!(
                "   📏 Errors: {}, Warnings: {}",
                validation_report.errors.len(),
                validation_report.warnings.len()
            );
        } else {
            println!("⚠️ Output validation found issues:");
            for error in &validation_report.errors {
                println!("   ❌ {}", error);
            }
            for warning in &validation_report.warnings {
                println!("   ⚠️ {}", warning);
            }
        }
    }

    // Step 10: Write to file with backup
    println!("💾 Writing to file: {}", config.output);
    let writer = FileWriter::new();

    if config.backup {
        writer
            .write_with_backup(&config.output, &markdown)
            .await
            .map_err(|e| {
                println!("❌ File writing with backup failed: {}", e);
                e
            })?;
        println!("💼 Backup created successfully");
    } else {
        writer.write(&config.output, &markdown).await.map_err(|e| {
            println!("❌ File writing failed: {}", e);
            e
        })?;
    }

    // Calculate statistics
    let processing_time = start_time.elapsed().as_secs_f64();
    let output_size_kb = writer.get_file_size(&config.output).unwrap_or(0) as f64 / 1024.0;

    println!(
        "🎯 File written successfully: {:.1} KB in {:.2}s",
        output_size_kb, processing_time
    );

    Ok(ExtractionStats {
        categories: document_structure.categories.len(),
        endpoints: total_endpoints,
        output_size_kb,
        processing_time,
    })
}

/// Custom configuration builder for advanced use cases
#[derive(Debug)]
pub struct AdvancedConfigBuilder {
    config: Config,
    custom_headers: HashMap<String, String>,
    timeout_seconds: u64,
}

impl AdvancedConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: Config::from_args(),
            custom_headers: HashMap::new(),
            timeout_seconds: 30,
        }
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.config.url = url.into();
        self
    }

    pub fn with_output(mut self, output: impl Into<String>) -> Self {
        self.config.output = output.into();
        self
    }

    pub fn with_debug_logging(mut self) -> Self {
        self.config.log_level = "debug".to_string();
        self
    }

    pub fn without_validation(mut self) -> Self {
        self.config.no_validation = true;
        self
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    pub fn add_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom_headers.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> Result<Config, Box<dyn std::error::Error>> {
        self.config.validate()?;
        Ok(self.config)
    }
}

// Example of using the builder pattern
#[allow(dead_code)]
fn example_builder_usage() -> Result<(), Box<dyn std::error::Error>> {
    let config = AdvancedConfigBuilder::new()
        .with_url("https://www.alphavantage.co/documentation")
        .with_output("custom-output.md")
        .with_debug_logging()
        .with_timeout(60)
        .add_header("User-Agent", "AlphaVantage-Extractor/1.0")
        .build()?;

    println!("Built configuration: {:?}", config);
    Ok(())
}
