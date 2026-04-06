// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # 🎯 Consolidated Core Orchestration
//!
//! **ORCHESTRATION CORE CONSOLIDATION** ✅
//!
//! This module consolidates the core orchestration functionality previously
//! scattered across songbird-core and songbird-orchestrator.

// Core orchestration modules
pub mod api;
pub mod benchmarks;
pub mod biome;
pub mod caching;
pub mod execution;
pub mod load_balancer;
/// Capability-based metrics snapshots and primal discovery for telemetry.
pub mod metrics;
pub mod orchestrator;
pub mod performance;
pub mod registry;
pub mod robustness;
pub mod routing; // ✅ NEW: Intelligent capability routing (Nov 9, 2025)
pub mod scaling;
pub mod zero_touch;

mod consolidated_config;
mod consolidated_engine;
mod orchestrator_health;

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test assertions")]
mod consolidated_tests;

// Re-export key functionality for convenience
pub use api::{ApiConfig, ApiHandler, CoreApi};
// Legacy ServiceRegistry from biome - keeping for backward compatibility
pub use biome::ServiceRegistry;
pub use consolidated_config::{
    ConsolidatedOrchestratorConfig, DeploymentStrategy, PerformanceConfig, RegistryConfig,
    ScalingConfig, ZeroTouchConfig,
};
pub use consolidated_engine::ConsolidatedOrchestrator;
pub use load_balancer::{LoadBalancer, LoadBalancingStrategy};
pub use orchestrator::{CoreOrchestrator, OrchestratorConfig};
pub use orchestrator_health::{ComponentHealth, HealthStatus, OrchestratorHealth};
pub use performance::{PerformanceMetrics, PerformanceMonitor};
pub use registry::{CapabilityRegistry, HeartbeatConfig};
pub use robustness::{CircuitBreaker, RetryPolicy};
pub use scaling::{AutoScaler, ScalingPolicy};

pub use songbird_config::canonical::resilience::LoadBalancerConfig as CanonicalLoadBalancerConfig;
