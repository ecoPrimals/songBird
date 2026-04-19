// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Network-related constants

use std::time::Duration;

/// Default host constant
pub const DEFAULT_HOST: &str = "localhost";

/// Default host IPv4 constant — re-export from canonical source.
pub const DEFAULT_HOST_V4: &str = songbird_types::constants::LOCALHOST;

/// Default bind address constant.
pub const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1:8080";

/// Default orchestrator port
pub const DEFAULT_ORCHESTRATOR_PORT: u16 = 8080;

/// Default development port
pub const DEFAULT_DEV_PORT: u16 = 8080;

/// Default dashboard port
pub const DEFAULT_DASHBOARD_PORT: u16 = 3000;

// ============================================================================
// ⚠️ DEPRECATED PRIMAL ENDPOINT CONSTANTS - REMOVED FOR SOVEREIGNTY
// ============================================================================
//
// The following hardcoded endpoint constants have been REMOVED for sovereignty
// compliance. They violated the principle that primals should only have self-
// knowledge and discover other primals at runtime.
//
// OLD (REMOVED):
// - DEFAULT_COMPUTE_PROVIDER_ENDPOINT / legacy DEFAULT_TOADSTOOL_ENDPOINT (compute capability)
// - DEFAULT_SQUIRREL_ENDPOINT (AI)
// - DEFAULT_STORAGE_PROVIDER_ENDPOINT / legacy DEFAULT_NESTGATE_ENDPOINT (storage capability)
// - DEFAULT_BEARDOG_ENDPOINT (security)
// - DEFAULT_*_PORT constants
//
// **SOVEREIGNTY VIOLATION**: Hardcoded primal endpoints violate sovereignty principles.
// Each primal should know only itself and discover others at runtime.
//
// NEW (USE INSTEAD):
// ```rust
// use songbird_config::primal_discovery::*;
//
// // Discovers ANY provider with the capability
// let compute = get_compute_endpoint().await?;    // set COMPUTE_ENDPOINT
// let ai = get_ai_endpoint().await?;              // set AI_ENDPOINT
// let storage = get_storage_endpoint().await?;    // set STORAGE_ENDPOINT
// let security = get_security_endpoint().await?;  // set SECURITY_ENDPOINT
//
// // Or use RuntimeDiscoveryEngine for dynamic discovery
// let engine = RuntimeDiscoveryEngine::new();
// let service = engine.discover_by_capability("compute").await?;
// ```
//
// See: crates/songbird-config/src/primal_discovery.rs
// See: crates/songbird-config/src/runtime_discovery.rs
// See: specs/INDIVIDUAL_HUMAN_DIGNITY_SPECIFICATION.md
// ============================================================================

/// Default connection timeout
// MIGRATED: Use songbird_types::unified_constants::timeouts::DEFAULT_CONNECTION_TIMEOUT instead
/// Default retry delay
pub const DEFAULT_RETRY_DELAY: Duration = Duration::from_millis(1000);
/// Default `crate::constants::network::DEFAULT_HOST` address
// MIGRATED: Use songbird_types::unified_constants::network::DEFAULT_LOCALHOST instead
/// Production bind address
pub const PRODUCTION_BIND_ADDRESS: &str = "0.0.0.0";
