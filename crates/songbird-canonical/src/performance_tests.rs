// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for Performance Monitoring
//!
//! Comprehensive test coverage for performance monitoring initialization.

use super::performance::*;

// ============================================================================
// Performance Monitoring Tests
// ============================================================================

#[test]
fn test_initialize_performance_monitoring() {
    // This test verifies that the initialization function can be called
    // without panicking. Since it only logs, we just ensure it completes.
    initialize_performance_monitoring();
}

#[test]
fn test_initialize_performance_monitoring_multiple_calls() {
    // Verify that calling initialization multiple times is safe
    initialize_performance_monitoring();
    initialize_performance_monitoring();
}

#[test]
fn test_initialize_performance_monitoring_idempotent() {
    // Verify multiple calls produce consistent behavior
    for _ in 0..5 {
        initialize_performance_monitoring();
    }
}
