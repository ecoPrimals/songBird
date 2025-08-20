//! Songbird Unwrap Migrator Library - 2025 Edition
//!
//! This library provides tools for systematically migrating unwrap/expect/panic
//! patterns to use Songbird's unified error handling system.

// Temporarily disable problematic modules
// pub mod systematic_migrator;
// pub mod enhanced_migrator;
// pub mod comprehensive_migrator;
pub mod modernized_migrator;

// Re-export main types for convenience
// pub use systematic_migrator::SystematicUnwrapMigrator;
// pub use enhanced_migrator::EnhancedUnwrapMigrator;
// pub use comprehensive_migrator::ComprehensiveMigrator;
pub use modernized_migrator::ModernizedMigrator;
