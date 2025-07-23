//! Test implementations broken down into focused modules
//!
//! This module was refactored from a single 1074-line file into smaller,
//! more maintainable modules for better organization and readability.

pub mod audit_logger;
pub mod compliance_checker;
pub mod encryption_tester;
pub mod security_framework;
pub mod threat_detector;
pub mod universal_setup;
pub mod zero_trust;

// Re-export main setup functions
pub use universal_setup::{example_security_primal_capabilities, setup_universal_security_example};

// Re-export all test implementations
pub use audit_logger::AuditLogger;
pub use compliance_checker::ComplianceChecker;
pub use encryption_tester::EncryptionTester;
pub use security_framework::SecurityTestingFramework;
pub use threat_detector::ThreatDetector;
pub use zero_trust::ZeroTrustEngine;
