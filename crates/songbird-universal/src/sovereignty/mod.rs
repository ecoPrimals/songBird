// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # 🏛️ Sovereignty-Aware Universal Adapter System
//!
//! **MODULAR ARCHITECTURE** ✅
//!
//! This module provides sovereignty-aware routing and federation capabilities
//! while maintaining primal self-knowledge principles. The system is split
//! into focused, maintainable modules under 500 lines each.

pub mod adapter;
pub mod federation;
pub mod network_optimizer;
pub mod router;
pub mod types;

#[cfg(test)]
mod router_comprehensive_tests;

// Re-export main interfaces
pub use adapter::SovereigntyAwareAdapter;
pub use types::{
    ExpectedNetworkEffect, FederationCapability, PathSegment, PathSovereigntyAssessment,
    RoutingDecisionMetadata, RoutingPath, SovereigntyAdapterConfig,
    SovereigntyAwareRoutingDecision,
};
