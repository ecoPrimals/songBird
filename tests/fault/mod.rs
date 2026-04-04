// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals
//! Fault Injection Tests for Songbird
//!
//! These tests use deterministic fault injection to verify
//! specific error handling paths and recovery scenarios.
//!
//! Unlike chaos tests which are random, fault tests are:
//! - Deterministic and repeatable
//! - Target specific failure modes
//! - Verify specific error handling code paths

#![cfg(test)]

pub mod component_failures;
pub mod integration_failures;
pub mod recovery_scenarios;

/// Common fault injection utilities
pub mod common {
    /// Fault injection point
    pub enum FaultType {
        NetworkError,
        TimeoutError,
        ResourceError,
        ValidationError,
        PanicError,
    }
    
    /// Fault injection configuration
    pub struct FaultInjection {
        pub fault_type: FaultType,
        pub component: String,
        pub trigger_count: usize, // Trigger after N operations
    }
}

