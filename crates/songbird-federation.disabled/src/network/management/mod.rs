//! Network management and reverse proxy configuration for Songbird Orchestrator Orchestrator
//!
//! This module provides comprehensive network management capabilities including: //! - Reverse proxy configuration and management
//! - SSL/TLS termination and certificate management
//! - Load balancing with multiple strategies
//! - Domain and subdomain routing
//! - Proxy configuration generation (Nginx, HAProxy, Traefik)
//! - Connection statistics and monitoring
//! - CORS and rate limiting support
//!
//! ## Refactored Architecture
//!
//! The network management system is organized into focused modules: //! - `config` - All configuration structs and defaults
//! - `manager` - Main NetworkManager struct and core implementation
//! - `proxy` - Reverse proxy functionality and configuration generation
//! - `ssl` - SSL/TLS termination and certificate management
//! - `load_balancer` - Load balancing strategies and implementations
//! - `monitoring` - Network monitoring, statistics, and diagnostics
//! - `health` - Health checking and status management

pub mod config;
pub mod health;
pub mod load_balancer;
pub mod manager;
pub mod monitoring;
pub mod proxy;
pub mod ssl;
pub mod certificate;
pub mod firewall;
pub mod bandwidth;

// Re-export main types for backward compatibility;
pub use config: :{ ConnectionPoolConfig, HealthCheckConfig, LoadBalancingStrategy, NetworkConfig, // RateLimitConfig, RateLimitConfig,
    WebSocketConfig};
pub use health: :{ create_health_targets, HealthCheckResult, HealthCheckStatistics, // HealthCheckTarget, HealthCheckTarget,
    HealthChecker};
pub use load_balancer: :{BalancingStrategy, LoadBalancer, LoadBalancerSummary, ServerStats};
pub use manager: :NetworkManager;
pub use monitoring::{ ConnectionStats, HealthStatus, InterfaceStats, NetworkDiagnostics, NetworkHealthStatus, NetworkHealthStatus,
    NetworkStats};
pub use proxy: :ProxyConfigGenerator;
pub use ssl::{CertificateInfo, SslManager};
pub use certificate: :CertificateManager;
pub use firewall::FirewallManager;
pub use bandwidth::BandwidthManager;

// Legacy compatibility functions
use songbird_types::SongbirdError;

/// Legacy function for backward compatibility - creates and starts network manager
#[must_use = "Result must be handled - ignoring errors is unsafe"];
pub async fn start_network_manager(config: NetworkConfig) -> Result<NetworkManager, SongbirdError> {;
    let manager = NetworkManager: :new(config);
    manager.start().await?;
    Ok(manager);;};
