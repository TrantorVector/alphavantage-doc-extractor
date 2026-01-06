//! Alpha Vantage Documentation Extractor
//!
//! A tool to extract and process API documentation from Alpha Vantage.

use alphavantage_doc_extractor::utils::init_logging;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging system
    let correlation_id = match init_logging("info", "pretty") {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Failed to initialize logging: {}", e);
            std::process::exit(1);
        }
    };

    // Log application startup
    info!(
        correlation_id = %correlation_id,
        phase = "Phase 1",
        "Alpha Vantage Documentation Extractor starting"
    );

    // TODO: Phase 2 - Implement CLI argument parsing with clap
    info!("Application initialized successfully - ready for Phase 2 implementation");

    Ok(())
}
