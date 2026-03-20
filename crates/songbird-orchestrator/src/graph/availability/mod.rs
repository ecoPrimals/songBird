// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Primal availability checking and alternative suggestions
//!
//! This module integrates with the service registry (v3.20.0) to check if required
//! primals are available for graph execution and suggest alternatives when needed.
//!
//! # Design Principles
//!
//! - **Zero Hardcoding**: Uses service registry for runtime discovery
//! - **Capability-Based**: Discovers by capability, not primal name
//! - **Health-Aware**: Considers primal health status in decisions
//! - **Protocol-Agnostic**: Supports multiple protocols with compatibility scoring

mod checker;
mod types;

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test assertions")]
mod tests;

pub use checker::AvailabilityChecker;
pub use types::{
    AlternativePrimal, AlternativeRecommendation, AlternativeSuggestions, AvailabilityReport,
    AvailabilitySummary, NodeAvailability, NodeAvailabilityStatus,
};
