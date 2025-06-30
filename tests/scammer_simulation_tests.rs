use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
//!
 //! Scammer Simulation Tests - Modular Entry Point
 *
 //! This file now serves as a clean entry point to the modular scammer simulation test system.
 //! The original 1027-line test suite has been refactored into focused, maintainable modules.
 //! 
 //! The scammer simulation tests are organized into:
 //! - Real-world scammer tactic simulations
 //! - Social engineering attack tests
 //! - Family-specific protection tests
 //! - Suspicious behavior detection tests
//!

// Import the modular scammer simulation test system
mod scammer_simulation;

// Re-export for convenience
pub use scammer_simulation::*;
