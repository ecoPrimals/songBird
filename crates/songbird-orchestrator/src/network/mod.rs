//! Network Management
//!
//! Intelligent network interface management, binding strategies, and endpoint abstraction.

pub mod binding;
pub mod connectivity_test;

// Re-export commonly used types
pub use connectivity_test::{ConnectivityTester, ConnectivityTestResult, ConnectivityRemediator};

pub use binding::NetworkBindingStrategy;

