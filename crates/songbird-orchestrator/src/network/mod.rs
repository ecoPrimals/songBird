// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Network Management
//!
//! Intelligent network interface management, binding strategies, and endpoint abstraction.

pub mod binding;
pub mod connectivity_test;
pub mod sovereign_socket;

// Re-export commonly used types
pub use connectivity_test::{ConnectivityRemediator, ConnectivityTestResult, ConnectivityTester};
pub use sovereign_socket::{SovereignBinder, SovereignSocket};

pub use binding::NetworkBindingStrategy;
