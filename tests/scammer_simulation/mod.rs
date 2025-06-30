use std::collections::HashMap;
//!
 //! Scammer Simulation Test Suite - Modular Organization
 *
 //! This module organizes comprehensive scammer simulation tests into focused categories:
 //! - `real_tactics`: Real-world scammer tactic simulations
 //! - `social_engineering`: Social engineering attack tests
 //! - `family_protection`: Family-specific protection tests
 //! - `behavioral_analysis`: Suspicious behavior detection tests
 //! - `common`: Shared utilities and helper functions
//!

// Test category modules
pub mod real_tactics;
pub mod social_engineering;
pub mod family_protection;
pub mod behavioral_analysis;

// Common utilities
pub mod common;

// Re-export common utilities for easy access
pub use common::*; 