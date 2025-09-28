use CanonicalSongbirdConfig;
//! Validation tests module
//!
//! This module consolidates all validation tests that were previously in the oversized
//! validation_tests.rs file (1066+ lines). Each test category is now in its own file
//! for better maintainability and code organization.

pub mod port_validation_tests;

// Additional validation modules can be added here as needed:
// When creating new validation test files, uncomment and implement:
// pub mod url_validation_tests;
// pub mod ip_validation_tests;
// pub mod timeout_validation_tests;
// pub mod retry_validation_tests;
// pub mod thread_pool_validation_tests;
// pub mod memory_validation_tests;
// pub mod buffer_validation_tests;
// pub mod percentage_validation_tests;
// pub mod rate_limit_validation_tests;
// pub mod file_path_validation_tests;
// pub mod integration_tests; 