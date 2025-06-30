use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
//!
 //! Songbird Discovery Service Tests - Modular Entry Point
 *
 //! This file now serves as a clean entry point to the modular discovery test system.
 //! The original 1457-line test suite has been refactored into focused, maintainable modules.
 //! 
 //! The discovery tests are organized into:
 //! - Basic registration and discovery tests
 //! - Health management tests  
 //! - Federation capabilities tests
 //! - Performance and scalability tests
 //! - Resource-aware selection tests
 //! - Trust verification tests
 //! - Configuration system tests
//!

// Import all test modules
mod discovery;

// Re-export for convenience
pub use discovery::*;
