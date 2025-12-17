//! Canonical Network Configuration - Modular Implementation
//!
//! **REFACTORED**: November 10, 2025 - Split from monolithic 1,261-line file
//! into focused domain modules for improved maintainability.
//!
//! ## Module Organization
//!
//! - `core` - Core network configuration and `PeerType`
//! - `gaming` - Gaming-specific configuration and scales
//! - `timeouts` - Timeout configurations
//! - `cors` - CORS configuration
//! - `limits` - Connection limits and rate limiting
//! - `ports` - Port range configuration
//! - `advanced` - Advanced features (SSL, proxy, discovery, measurements)
//!
//! ## Backward Compatibility
//!
//! All types are re-exported at the module level for full backward compatibility.
//! Existing code using `use songbird_config::canonical::network::*` will continue to work.

// Module declarations
pub mod advanced;
pub mod core;
pub mod cors;
pub mod gaming;
pub mod limits;
pub mod ports;
pub mod timeouts;

// ============================================================================
// RE-EXPORTS FOR BACKWARD COMPATIBILITY
// ============================================================================

// Core types
pub use core::{
    CanonicalNetworkConfig,
    NetworkConfig, // Type alias
    PeerType,
};

// Gaming types
pub use gaming::{GamingNetworkConfig, GamingScale};

// Timeout types
pub use timeouts::{NetworkTimeouts, TimeoutConfig};

// CORS types
pub use cors::CorsConfig;

// Limits and pooling
pub use limits::{ConnectionLimits, ConnectionPoolConfig, LoadBalancingConfig, RateLimitingConfig};

// Port types
pub use ports::PortRange;

// Advanced features
pub use advanced::{
    // Topology and measurements
    DiscoveryNetworkTopology,
    DomainConfig,

    NetworkConnection,
    NetworkInterfaceConfig,
    NetworkMeasurement,

    ProxyConfig,
    // Proxy and SSL
    ReverseProxyConfig,
    SelfAwareConfig,
    ServiceDiscoveryEndpoints,

    // Service endpoints and discovery
    ServiceEndpoint,
    SocketBufferConfig,
    SslConfig,
    // NAT traversal and local discovery
    TURNRelay,
    // TCP/UDP configuration
    TcpConfig,
    TcpKeepAliveConfig,
    UPnPDevice,

    UdpConfig,
    UniversalDiscoveryConfig,
};

// Circuit breaker is defined in resilience module
pub use super::resilience::CircuitBreakerConfig;

// ============================================================================
// MODULE DOCUMENTATION AND MIGRATION NOTES
// ============================================================================

/// # Migration from Monolithic File
///
/// **Date**: November 10, 2025\
/// **Previous**: Single 1,261-line `network.rs` file\
/// **Current**: 6 focused modules totaling same functionality
///
/// ## Changes
///
/// - **NONE** to public API - full backward compatibility maintained
/// - **Internal**: Code organized into logical domain modules
/// - **Benefits**: Easier navigation, better IDE support, clearer ownership
///
/// ## File Size Breakdown
///
/// ```
/// core.rs     ~450 lines  - Core config and PeerType
/// gaming.rs   ~100 lines  - Gaming configuration
/// timeouts.rs ~80 lines   - Timeout configs
/// cors.rs     ~30 lines   - CORS configuration
/// limits.rs   ~100 lines  - Connection limits
/// ports.rs    ~20 lines   - Port ranges
/// advanced.rs ~450 lines  - Advanced features
/// mod.rs      ~100 lines  - This file (re-exports)
/// ──────────────────────
/// Total:      ~1,330 lines (includes new documentation)
/// ```
///
/// ## Verification
///
/// All existing imports continue to work:
///
/// ```rust
/// // All of these still work exactly as before:
/// use songbird_config::canonical::network::CanonicalNetworkConfig;
/// use songbird_config::canonical::network::NetworkConfig;
/// use songbird_config::canonical::network::*;
/// ```

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backward_compatibility_imports() {
        // Verify all key types are accessible via re-exports
        let _config: CanonicalNetworkConfig = CanonicalNetworkConfig::default();
        let _peer: PeerType = PeerType::default();
        let _gaming: GamingNetworkConfig = GamingNetworkConfig::default();
        let _timeouts: NetworkTimeouts = NetworkTimeouts::default();
        let _cors: CorsConfig = CorsConfig::default();
        let _limits: ConnectionLimits = ConnectionLimits::default();
        let _ports: PortRange = PortRange::default();
    }

    #[test]
    fn test_config_creation() {
        let config = CanonicalNetworkConfig::default();
        assert_eq!(config.orchestrator_port, 8080);
        assert_eq!(config.gaming.starcraft_port, 6112);
    }
}
