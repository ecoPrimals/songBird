use std::collections::HashMap;
//!
 //! Comprehensive Test Suite - Modular Organization
 *
 //! This module organizes the comprehensive test suite into focused, maintainable phases:
 //! - `runner`: Master test suite orchestration and reporting
 //! - `unit_tests`: Unit test phase implementation
 //! - `integration_tests`: Integration test phase implementation  
 //! - `penetration_tests`: Security penetration test phase
 //! - `scammer_protection`: Scammer protection test phase
 //! - `family_safety`: Family safety test phase
 //! - `performance_tests`: Performance and stress test phase
 //! - `e2e_tests`: End-to-end workflow test phase
//!

// Core test runner infrastructure
pub mod runner;

// Test phase modules
pub mod unit_tests;
pub mod integration_tests;
pub mod penetration_tests;
pub mod scammer_protection;
pub mod family_safety;
pub mod performance_tests;
pub mod e2e_tests;

// Common utilities
pub mod common;

// Re-export main runner for easy access
pub use runner::*; 