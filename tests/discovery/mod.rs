use std::collections::HashMap;
//!
 //! Discovery Service Test Suite
 //! 
 //! This module organizes comprehensive tests for the Songbird Discovery Service
 //! into focused, maintainable test modules:
 //! 
 //! - `basic_tests`: Basic service registration and discovery
 //! - `health_tests`: Service health management tests  
 //! - `federation_tests`: Federation capabilities tests
 //! - `performance_tests`: Performance and scalability tests
 //! - `resource_tests`: Resource-aware selection tests
 //! - `trust_tests`: Trust verification system tests
 //! - `config_tests`: Configuration system tests
//!

// Test module declarations
pub mod basic_tests;
pub mod health_tests;  
pub mod federation_tests;
pub mod performance_tests;
pub mod resource_tests;
pub mod trust_tests;
pub mod config_tests;

// Common test utilities and helpers
pub mod common;

// Re-export common test functions for easy access
pub use common::*; 