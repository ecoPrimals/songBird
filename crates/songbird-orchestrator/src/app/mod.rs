#![allow(dead_code)]

// Module declarations
pub mod core;
pub mod discovery;
pub mod federation;
pub mod health;
pub mod http_server; // Public for E2E tests
pub mod network;
pub mod startup;

// Re-exports for backwards compatibility
pub use core::SongbirdOrchestrator;
pub use health::{run_health_check, HealthCheckReport, OrchestratorStatus};
pub use network::{detect_primary_ip, get_local_ip_for_connectivity_test, parse_bind_address};
pub use startup::{start_orchestrator, Orchestrator};

// use songbird_federation::{//     FederationConfig,
//     canonical_federation::CanonicalFederation)
// }; // Temporarily disabled - complex type mismatches need resolution
// use songbird_network::gaming::GamingManager; // Temporarily disabled - gaming module not available
// use songbird_security::UniversalSecurityIntegration; // Temporarily disabled for consolidation

// Import anonymous discovery and trust escalation

