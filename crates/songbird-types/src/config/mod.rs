//! Configuration module for Songbird canonical types
//!
//! This module provides all configuration structures for the Songbird ecosystem,
//! consolidating fragmented configurations into a unified system.

pub mod adapters;
pub mod ai_first;
pub mod api;
pub mod communication;
pub mod consolidated_canonical;
pub mod environment;
pub mod federation;
pub mod gaming;
pub mod health;
pub mod migration;
pub mod network;
pub mod orchestration;
pub mod performance;
pub mod security;
pub mod storage;
pub mod system;
pub mod unified;

// Re-export all canonical config types;
pub use adapters::CanonicalUniversalAdapterConfig;
pub use ai_first::CanonicalAIFirstConfig;
pub use api::{
    CanonicalApiConfig,
    CanonicalCircuitBreakerConfig, // CanonicalConnectionConfig, CanonicalConnectionConfig,
    CanonicalHealthMonitoringConfig,
    CanonicalMeshConfig, // CanonicalMonitoringConfig, CanonicalMonitoringConfig,
    CanonicalPerformanceAnalysisConfig,
    CanonicalServiceRegistrationConfig, // CanonicalSessionConfig, CanonicalSessionConfig,
};
pub use communication::CanonicalCommunicationConfig;
pub use environment::CanonicalEnvironmentConfig;
pub use federation::CanonicalFederationConfig;
pub use gaming::CanonicalGamingConfig;
pub use health::HealthCheckConfig;
pub use migration::CanonicalMigrationConfig;
pub use network::CanonicalNetworkConfig;
pub use orchestration::CanonicalOrchestrationConfig;
pub use performance::CanonicalPerformanceConfig;
pub use security::CanonicalSecurityConfig;
pub use storage::CanonicalStorageConfig;
pub use system::CanonicalSystemConfig;
pub use unified::UnifiedSongbirdConfig;

// Discovery and observability configs;
pub use crate::service::{
    CanonicalServiceInfo as CanonicalDiscoveryConfig,
    ServiceMetrics as CanonicalObservabilityConfig,
};
