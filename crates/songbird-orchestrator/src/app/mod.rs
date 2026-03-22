// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![expect(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]

// Module declarations
pub mod command_handler; // v4.0.0 (Feb 6): Deep Debt - command handling extraction
pub mod connection_manager; // Progressive trust connection management (Jan 2026)
pub mod core;
pub mod discovery;
pub mod discovery_bridge; // Discovery→Federation bridge (v3.10.0 refactoring)
pub mod discovery_startup; // v3.10.3 (Jan 6): Smart refactoring - discovery system startup
pub mod federation;
pub mod federation_setup; // v3.10.3 (Jan 6): Smart refactoring - federation coordinator setup
pub mod hardware_detection; // v3.10.3 (Jan 6): Smart refactoring - runtime hardware detection
pub mod health;
pub mod http_server; // Public for E2E tests
pub mod initialization; // v3.10.3 (Jan 6): Smart refactoring - component initialization
pub mod network;
pub mod security_setup; // v3.10.3 (Jan 6): Smart refactoring - capability-based security discovery
pub mod startup;
pub mod startup_orchestration; // v4.0.0 (Feb 6): Deep Debt - 7-stage startup extraction (275→20 lines per stage)

// Test modules (v3.3 - Jan 2026)
#[cfg(test)]
mod tests_birdsong_integration; // BirdSong listener/broadcaster E2E tests

#[cfg(test)]
mod tests_discovery_bridge; // Discovery→Federation bridge unit & E2E tests (v3.10.1)

#[cfg(test)]
mod core_tests;

// Re-exports for backwards compatibility
pub use core::SongbirdOrchestrator;
pub use health::{HealthCheckReport, OrchestratorStatus};
pub use network::{detect_primary_ip, get_local_ip_for_connectivity_test, parse_bind_address};
pub use startup::{Orchestrator, start_orchestrator};

// use songbird_federation::{//     FederationConfig,
//     canonical_federation::CanonicalFederation)
// }; // Temporarily disabled - complex type mismatches need resolution
// use songbird_network::gaming::GamingManager; // Temporarily disabled - gaming module not available
// use songbird_security::UniversalSecurityIntegration; // Temporarily disabled for consolidation

// Import anonymous discovery and trust escalation
