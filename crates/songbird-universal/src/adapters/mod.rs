// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Capability-Based Adapters for Universal Orchestration
//!
//! **SOVEREIGNTY PRINCIPLE**: These adapters work with capabilities, NOT primal names.
//! Each primal only knows itself. Songbird discovers capabilities dynamically.
//!
//! ## Design Philosophy
//!
//! Like in ecology, each organism exists independently:
//! - Songbird doesn't "know" specific primals exist (`security provider`, `storage provider`, etc.)
//! - Songbird only knows "something provides security capability"
//! - Discovery is dynamic through `ZeroKnowledgeBootstrap`
//! - No 2^n hardcoded connections - only universal adapter for network effects
//!
//! ## Capability-Based Adapters
//!
//! - `ComputeAdapter` - Any compute capability provider
//! - `SecurityAdapter` - Any security capability provider
//! - `StorageAdapter` - Any storage capability provider  
//! - `AIAdapter` - Any AI capability provider
//!
//! ## Example Implementations
//!
//! See `examples/integration/ecosystem-primals/` for how specific primals
//! in our ecosystem happen to implement these capabilities. But the production
//! code here doesn't know about them!

pub mod ai;
#[cfg(test)]
#[path = "capability_adapter_coverage_tests.rs"]
mod capability_adapter_coverage_tests;
#[cfg(test)]
#[path = "capability_transport_tests.rs"]
mod capability_transport_tests;
pub mod compute;
#[cfg(test)]
mod discovery_test_sync;
pub(crate) mod transport;

pub mod security;
#[cfg(test)]
mod security_btsp_tests;
pub mod storage;

// Re-export adapters
pub use ai::{AIAdapter, AIHealth, AIMetrics, ModelType};
pub use compute::{ComputeAdapter, ComputeMetrics, HealthStatus as ComputeHealth};

pub use security::{SecurityAdapter, SecurityHealth, SecurityMetrics};
pub use storage::{StorageAdapter, StorageHealth, StorageMetrics};
