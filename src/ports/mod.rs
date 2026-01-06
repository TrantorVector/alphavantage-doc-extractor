//! Ports layer - Abstract interfaces for external dependencies
//!
//! This module defines traits that abstract external systems.
//! Domain code depends on these traits, adapters implement them.

pub mod fetcher;
pub mod writer;
