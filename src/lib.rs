//! # Alpha Vantage Documentation Extractor
//!
//! A tool to extract and process documentation from Alpha Vantage API.
//!
//! This library follows hexagonal architecture with strict separation between
//! domain logic and external dependencies.

pub mod adapters;
pub mod domain;
pub mod ports;
pub mod utils;

// Re-export main domain types
// TODO: Uncomment when domain modules are implemented
// pub use domain::{
//     models::*,
//     extractor::*,
//     transformer::*,
//     renderer::*,
// };

// Re-export utility types
// TODO: Uncomment when utils modules are implemented
// pub use utils::{error::*, logging::*, validation::*};
