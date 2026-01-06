//! Logging system demonstration.
//!
//! This example shows how to use the Alpha Vantage Documentation Extractor logging system,
//! including initialization with different formats, correlation IDs, timers, and different log levels.

use alphavantage_doc_extractor::utils::{init_logging, Timer};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, trace, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Alpha Vantage Logging Demo ===\n");

    // Demo 1: Initialize with pretty format
    println!("1. Initializing logging with pretty format...");
    let correlation_id_pretty = init_logging("info", "pretty")?;
    println!("   Correlation ID: {}\n", correlation_id_pretty);

    demonstrate_log_levels().await;
    demonstrate_timer().await;

    // Clean shutdown
    println!("\n2. Switching to JSON format...");
    let correlation_id_json = init_logging("debug", "json")?;
    println!("   Correlation ID: {}\n", correlation_id_json);

    demonstrate_json_logging().await;

    // Demo 3: Environment variable override
    println!("\n3. Environment variable override demo...");
    println!("   Set RUST_LOG=warn and run again to see environment override in action\n");

    Ok(())
}

/// Demonstrate different log levels with pretty formatting
async fn demonstrate_log_levels() {
    println!("   Demonstrating different log levels:");

    trace!("This is a TRACE message (usually not shown)");
    debug!("This is a DEBUG message");
    info!("This is an INFO message");
    warn!("This is a WARN message");
    error!("This is an ERROR message");

    println!();
}

/// Demonstrate timer functionality
async fn demonstrate_timer() {
    println!("   Demonstrating timer functionality:");

    {
        let _timer = Timer::new("demo_operation");
        info!("Starting timed operation...");

        // Simulate some work
        sleep(Duration::from_millis(100)).await;

        info!("Operation completed (timer will log duration on drop)");
    } // Timer logs here

    println!();
}

/// Demonstrate JSON logging format
async fn demonstrate_json_logging() {
    println!("   Demonstrating JSON format logging:");

    info!(
        correlation_id = "demo-12345",
        operation = "json_demo",
        user_id = 42,
        "JSON formatted log with structured data"
    );

    {
        let _timer = Timer::new("json_operation");
        debug!("This debug message will appear in JSON format");
        sleep(Duration::from_millis(50)).await;
        warn!("Warning in JSON format");
    }

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_demo_functions() {
        // Initialize logging for tests
        let _correlation_id = init_logging("warn", "pretty").unwrap();

        // Test that demo functions don't panic
        demonstrate_log_levels().await;
        demonstrate_timer().await;
        demonstrate_json_logging().await;
    }
}
