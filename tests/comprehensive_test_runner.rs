use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
//!
 //! Comprehensive Test Runner - Modular Entry Point
 *
 //! This file now serves as a clean entry point to the modular comprehensive test system.
 //! The original 1071-line test suite has been refactored into focused, maintainable modules.
 //! 
 //! The comprehensive test suite is organized into focused phases:
 //! - Unit Tests
 //! - Integration Tests  
 //! - Security Penetration Tests
 //! - Scammer Protection Tests
 //! - Family Safety Tests
 //! - Performance and Stress Tests
 //! - End-to-End Workflow Tests
//!

// Import the modular comprehensive test system
mod comprehensive;

// Re-export the main test runner
pub use comprehensive::*;
