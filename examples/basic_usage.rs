//! Basic usage example for the Alpha Vantage Documentation Extractor.
//!
//! This example demonstrates the simplest way to extract documentation
//! from Alpha Vantage and save it to a markdown file.

use alphavantage_doc_extractor::adapters::Config;
use alphavantage_doc_extractor::utils::init_logging;
use std::process;

/// Basic extraction example
///
/// This function shows how to:
/// 1. Set up basic configuration
/// 2. Initialize logging
/// 3. Run the extraction pipeline
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with info level
    let _correlation_id = init_logging("info", "pretty")?;

    println!("🚀 Alpha Vantage Documentation Extractor - Basic Example");
    println!("=====================================================");

    // Create basic configuration
    let config = Config {
        url: "https://www.alphavantage.co/documentation".to_string(),
        output: "alphavantage-api-docs.md".to_string(),
        log_level: "info".to_string(),
        log_format: "pretty".to_string(),
        no_validation: false,
        backup: true,
    };

    // Validate configuration
    config.validate().map_err(|e| {
        eprintln!("❌ Configuration validation failed: {}", e);
        process::exit(1);
    })?;

    println!("✅ Configuration validated");
    println!("📄 Output will be saved to: {}", config.output);
    println!("🔗 Source URL: {}", config.url);
    println!("💾 Backup enabled: {}", config.backup);

    // Note: In a real application, you would call the run() function here
    // For this example, we just demonstrate configuration setup

    println!("\n🎉 Configuration setup complete!");
    println!("💡 To run the actual extraction, use:");
    println!(
        "   cargo run -- --url {} --output {}",
        config.url, config.output
    );

    Ok(())
}
