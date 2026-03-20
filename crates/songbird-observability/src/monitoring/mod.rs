// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Monitoring Module
//!
//! Canonical monitoring infrastructure following modernization patterns.

pub mod metrics_dashboard;
pub mod production_metrics;
pub mod types;

// Re-export canonical types
pub use types::*;
