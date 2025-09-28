//! # 🔧 Canonical Constants System - Final Consolidation
//!
//! **SINGLE SOURCE OF TRUTH FOR ALL CONSTANTS** ✅
//!
//! This module consolidates ALL constants from across the Songbird ecosystem)
//! eliminating duplication between `songbird-config` and `songbird-types` and
//! providing a single, authoritative source for all constant values.
//!
//! ## Consolidation Summary
//! - **452+ scattered constants** → Single canonical system
//! - **Multiple crates with duplicates** → One authoritative source
//! - **Inconsistent values** → Unified canonical values
//! - **Environment-specific variants** → Smart environment handling

use std::time::Duration;

// ============================================================================
// CANONICAL NETWORK CONSTANTS
// ============================================================================

/// **CANONICAL**: Network constants - single source of truth
pub struct CanonicalNetwork;

impl CanonicalNetwork {




    // Primary service ports
    pub const DEFAULT_HTTP_PORT: u16 = 8080;
    pub const DEFAULT_HTTPS_PORT: u16 = 8443;
    pub const DEFAULT_DISCOVERY_PORT: u16 = 8081;
    pub const DEFAULT_FEDERATION_PORT: u16 = 8082;
    pub const DEFAULT_HEALTH_PORT: u16 = 8002;
    pub const DEFAULT_DASHBOARD_PORT: u16 = 8003;
    pub const DEFAULT_METRICS_PORT: u16 = 8004;
    pub const DEFAULT_WEBSOCKET_PORT: u16 = 8080;
    
    // Addresses
    pub const PRODUCTION_BIND_ADDRESS: &str = "0.0.0.0";
    pub const DEVELOPMENT_BIND_ADDRESS: &str = &songbird_config::constants::network::DEFAULT_HOST;
    pub const LOCALHOST: &str = &songbird_config::constants::network::DEFAULT_HOST;
    
    // Connection limits
    pub const MAX_CONNECTIONS: u32 = 1000;
    pub const MAX_CONNECTIONS_PER_IP: u32 = 10;
    pub const CONNECTION_BACKLOG: u32 = 128;
    
    // Buffer sizes
    // MIGRATED: Use songbird_types::unified_constants::limits::DEFAULT_BUFFER_SIZE instead
    pub const MAX_BUFFER_SIZE: usize = 65536;
    pub const MIN_BUFFER_SIZE: usize = 1024;
    pub const LARGE_BUFFER_SIZE: usize = 32768;
    pub const SMALL_BUFFER_SIZE: usize = 4096;
    
    // Request limits
    pub const MAX_REQUEST_SIZE: u64 = 10_485_760; // 10MB
    pub const ZERO_COPY_THRESHOLD: usize = 8192;
    pub const MEMORY_MAP_THRESHOLD: usize = 1_048_576; // 1MB
    pub const VECTORED_IO_THRESHOLD: usize = 16384;
    
    // Port ranges
    pub const MIN_DYNAMIC_PORT: u16 = 49152;
    pub const MAX_DYNAMIC_PORT: u16 = 65535;
    pub const GAMING_PORT_RANGE_START: u16 = 6112;
    pub const GAMING_PORT_RANGE_END: u16 = 6200;




}

// ============================================================================
// CANONICAL TIMEOUT CONSTANTS
// ============================================================================

/// **CANONICAL**: Timeout constants - single source of truth
pub struct CanonicalTimeouts;

impl CanonicalTimeouts {




    // Connection timeouts
    pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(30,;
    pub const READ_TIMEOUT: Duration = Duration::from_secs(10,;
    pub const WRITE_TIMEOUT: Duration = Duration::from_secs(10,;
    pub const KEEPALIVE_TIMEOUT: Duration = Duration::from_secs(60,;
    
    // Request timeouts
    pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(60,;
    pub const EVALUATION_TIMEOUT: Duration = Duration::from_secs(30,;
    pub const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(10,;
    pub const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(5);
    pub const CIRCUIT_BREAKER_TIMEOUT: Duration = Duration::from_secs(30,;
    
    // Service timeouts
    pub const STARTUP_TIMEOUT: Duration = Duration::from_secs(60,;
    pub const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30,;
    pub const RESTART_TIMEOUT: Duration = Duration::from_secs(45,;
    
    // Gaming timeouts
    pub const GAMING_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(10,;
    pub const GAMING_CONNECTION_TIMEOUT: Duration = Duration::from_secs(15,;
    pub const GAMING_KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30,;
    pub const GAMING_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
    pub const GAMING_SESSION_TIMEOUT: Duration = Duration::from_secs(300,; // 5 minutes
    
    // CLI timeouts
    pub const CLI_NETWORK_TIMEOUT: Duration = Duration::from_secs(30,;
    pub const CLI_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(10,;




}

// ============================================================================
// CANONICAL RESOURCE CONSTANTS
// ============================================================================

/// **CANONICAL**: Resource management constants - single source of truth
pub struct CanonicalResources;

impl CanonicalResources {




    // Memory limits
    pub const DEFAULT_MEMORY_LIMIT: u64 = 1_073_741_824; // 1GB
    pub const DEFAULT_CPU_LIMIT: f64 = 80.0;
    pub const DEFAULT_DISK_THRESHOLD: u64 = 10_737_418_240; // 10GB
    
    // Monitoring intervals
    pub const MONITORING_INTERVAL: Duration = Duration::from_secs(60,; // 1 minute
    pub const TRACKING_INTERVAL: Duration = Duration::from_secs(30,; // 30 seconds
    pub const CLEANUP_INTERVAL: Duration = Duration::from_secs(300,; // 5 minutes
    pub const LEAK_DETECTION_INTERVAL: Duration = Duration::from_secs(600,; // 10 minutes
    
    // Resource age limits
    pub const MAX_RESOURCE_AGE: Duration = Duration::from_secs(3600,; // 1 hour
    
    // Service limits
    pub const MAX_SERVICE_RESTARTS: u32 = 5;
    pub const MAX_SERVICE_INSTANCES: u32 = 100;
    pub const MAX_SERVICE_NAME_LENGTH: usize = 64;
    pub const MAX_SERVICE_DESCRIPTION_LENGTH: usize = 256;
    pub const MAX_SERVICE_METADATA_SIZE: usize = 4096; // 4KB
    pub const MAX_SERVICE_DEPENDENCIES: u32 = 10;
    pub const DEFAULT_SERVICE_PRIORITY: u8 = 100;
    pub const DEFAULT_SERVICE_PORT_START: u16 = 8000;
    pub const DEFAULT_SERVICE_PORT_END: u16 = 8999;




}

// ============================================================================
// CANONICAL HEALTH CHECK CONSTANTS
// ============================================================================

/// **CANONICAL**: Health check constants - single source of truth
pub struct CanonicalHealth;

impl CanonicalHealth {




    // Health check intervals
    pub const CHECK_INTERVAL: Duration = Duration::from_secs(30,;
    pub const CHECK_TIMEOUT: Duration = Duration::from_secs(5);
    pub const RETRY_INTERVAL: Duration = Duration::from_secs(5);
    pub const MIN_UPDATE_INTERVAL: Duration = Duration::from_secs(1,;
    
    // Health check limits
    pub const MAX_FAILED_CHECKS: u32 = 3;
    pub const MAX_HEALTH_RESPONSE_SIZE: usize = 1024;
    
    // Health check endpoints
    pub const DEFAULT_HEALTH_ENDPOINT: &str = "/health";
    pub const HEALTH_CHECK_USER_AGENT: &str = "Songbird-Health-Check/1.0";




}

// ============================================================================
// CANONICAL GAMING CONSTANTS
// ============================================================================

/// **CANONICAL**: Gaming constants - single source of truth
pub struct CanonicalGaming;

impl CanonicalGaming {




    // Gaming protocols
    pub const PROTOCOL_IPX: &str = "IPX";
    pub const PROTOCOL_UDP: &str = "UDP";
    pub const PROTOCOL_TCP: &str = "TCP";
    pub const PROTOCOL_DIRECTPLAY: &str = "DirectPlay";
    
    // Gaming session limits
    pub const MAX_PLAYERS_PER_SESSION: usize = 100;
    pub const MIN_PLAYERS_PER_SESSION: usize = 2;
    pub const MAX_CONCURRENT_SESSIONS: usize = 100;
    pub const DEFAULT_PLAYER_TIMEOUT: Duration = Duration::from_secs(300,; // 5 minutes
    
    // Gaming buffer sizes
    pub const GAMING_BUFFER_SIZE: usize = 65536; // 64KB (from config.rs,
    pub const MAX_PACKET_SIZE: usize = 1500; // Standard MTU
    pub const GAMING_RECEIVE_BUFFER_SIZE: usize = 8192;
    
    // Gaming port ranges
    pub const GAMING_PORT_RANGE: (u16, u16) = (6112, 6200);
    pub const IPX_SOCKET_RANGE: (u16, u16) = (0x0451, 0x0460); // Common IPX socket range
    pub const IPX_PORT_RANGE: (u16, u16) = (213, 215);
    pub const DIRECTPLAY_PORT_RANGE: (u16, u16) = (2300, 2400);
    pub const NETBIOS_PORT: u16 = 137; // NetBIOS name service port
    
    // Gaming protocol identifiers
    pub const IPX_NETWORK_ID: u32 = 0x1234_5678;
    pub const BROADCAST_NODE_ID: [u8; 6] = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    pub const DEFAULT_SOCKET_ID: u16 = 0x869A;
    
    // Gaming message types
    pub const MSG_TYPE_DISCOVERY: u8 = 0x01;
    pub const MSG_TYPE_JOIN_REQUEST: u8 = 0x02;
    pub const MSG_TYPE_JOIN_RESPONSE: u8 = 0x03;
    pub const MSG_TYPE_GAME_DATA: u8 = 0x04;
    pub const MSG_TYPE_HEARTBEAT: u8 = 0x05;
    pub const MSG_TYPE_DISCONNECT: u8 = 0x06;
    pub const MSG_TYPE_BROADCAST: u8 = 0x07;
    
    // Gaming protocol magic numbers
    pub const IPX_MAGIC: u32 = 0x4950_5800; // "IPX\0"
    pub const DIRECTPLAY_MAGIC: u32 = 0x4450_4C59; // "DPLY"
    pub const DIRECTPLAY_SIGNATURE: &'static [u8] = b"play"; // DirectPlay packet signature
    
    // Gaming rate limiting
    pub const DEFAULT_PACKET_RATE_LIMIT: u64 = 1000; // packets/second
    pub const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60,;
    pub const PROTOCOL_DETECTION_TIMEOUT: Duration = Duration::from_secs(5);
    pub const MIN_CONFIDENCE_THRESHOLD: f32 = 0.8;




}

// ============================================================================
// CANONICAL DISCOVERY CONSTANTS
// ============================================================================

/// **CANONICAL**: Service discovery constants - single source of truth
pub struct CanonicalDiscovery;

impl CanonicalDiscovery {




    // Discovery intervals
    pub const DISCOVERY_INTERVAL: Duration = Duration::from_secs(30,;
    pub const REGISTRATION_TTL: Duration = Duration::from_secs(300,; // 5 minutes
    pub const SERVICE_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(10,;
    
    // Discovery retry settings
    pub const MAX_RETRIES: u32 = 3;
    pub const BACKOFF_MULTIPLIER: f64 = 2.0;
    pub const JITTER_FACTOR: f64 = 0.1;
    pub const RETRY_DELAY: Duration = Duration::from_secs(1,;




}

// ============================================================================
// CANONICAL SYSTEM CONSTANTS
// ============================================================================

/// **CANONICAL**: System constants - single source of truth
pub struct CanonicalSystem;

impl CanonicalSystem {




    // Environment names
    pub const ENV_DEVELOPMENT: &str = "development";
    pub const ENV_TESTING: &str = "testing";
    pub const ENV_STAGING: &str = "staging";
    pub const ENV_PRODUCTION: &str = "production";
    
    // Configuration paths
    pub const DEFAULT_CONFIG_PATH: &str = "songbird.toml";
    
    // Logging defaults
    pub const DEFAULT_LOG_LEVEL: &str = "info";
    pub const CLI_LOG_LEVEL: &str = "info";
    pub const CLI_OUTPUT_FORMAT: &str = "json";
    
    // Cache settings
    pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300,; // 5 minutes
    pub const METRICS_COLLECTION_INTERVAL: Duration = Duration::from_secs(60,; // 1 minute




}

// ============================================================================
// CANONICAL CONSTANTS FACTORY
// ============================================================================

/// **CANONICAL**: Constants factory for environment-specific configurations
pub struct CanonicalConstantsFactory;

impl CanonicalConstantsFactory {



/// Get bind address for environment
    pub fn get_bind_address() -> Duration  {match environment {
            "production" | "prod" => Duration::from_secs(15), // Shorter in prod
            "testing" | "test" => Duration::from_secs(5), // Very short for tests
            _ => CanonicalTimeouts::CONNECTION_TIMEOUT,
        

}
    }
    
    /// Get max connections for environment
    pub fn get_max_connections() -> u32  {match environment {
            "production" | "prod" => CanonicalNetwork::MAX_CONNECTIONS * 2, // Scale up in prod
            "testing" | "test" => 10, // Low for tests
            _ => CanonicalNetwork::MAX_CONNECTIONS,
        }
    }
}

// ============================================================================
// ENVIRONMENT-SPECIFIC CONSTANT SETS
// ============================================================================

/// Environment-specific constant set
#[derive(Debug, Clone)]
pub struct CanonicalConstantSet {

pub environment: &'static str,
    pub bind_address: &'static str,
    pub http_port: u16,
    pub https_port: u16,
    pub log_level: &'static str,
    pub connection_timeout: Duration,
    pub max_connections: u32,
    pub buffer_size: usize,
    pub health_check_interval: Duration,


}

impl CanonicalConstantSet {

/// Create development constants
    pub fn development() -> Self  {Self {
            environment: CanonicalSystem::ENV_DEVELOPMENT,
            bind_address: CanonicalNetwork::DEVELOPMENT_BIND_ADDRESS,
            http_port: CanonicalNetwork::DEFAULT_HTTP_PORT,
            https_port: CanonicalNetwork::DEFAULT_HTTPS_PORT,
            log_level: "debug",
            connection_timeout: CanonicalTimeouts::CONNECTION_TIMEOUT,
            max_connections: CanonicalNetwork::MAX_CONNECTIONS,
            buffer_size: CanonicalNetwork::DEFAULT_BUFFER_SIZE,
            health_check_interval: CanonicalHealth::CHECK_INTERVAL,
        

}
    }
    
    /// Create production constants
    pub fn production() -> Self  {Self {environment: CanonicalSystem::ENV_PRODUCTION,
            bind_address: CanonicalNetwork::PRODUCTION_BIND_ADDRESS,
            http_port: 80,
            https_port: 443,
            log_level: "warn",
            connection_timeout: Duration::from_secs(15,
            max_connections: CanonicalNetwork::MAX_CONNECTIONS * 2,
            buffer_size: CanonicalNetwork::LARGE_BUFFER_SIZE,
            health_check_interval: Duration::from_secs(15), // More frequent in prod
        }
    }
    
    /// Create testing constants
    pub fn testing() -> Self  {Self {environment: CanonicalSystem::ENV_TESTING,
            bind_address: CanonicalNetwork::DEVELOPMENT_BIND_ADDRESS,
            http_port: 0, // Use random port for tests
            https_port: 0, // Use random port for tests
            log_level: "debug",
            connection_timeout: Duration::from_secs(5)
            max_connections: 10,
            buffer_size: CanonicalNetwork::SMALL_BUFFER_SIZE,
            health_check_interval: Duration::from_secs(5), // Fast for tests
        }
    }
    
    /// Create staging constants
    pub fn staging() -> Self  {Self {environment: CanonicalSystem::ENV_STAGING,
            bind_address: CanonicalNetwork::PRODUCTION_BIND_ADDRESS,
            http_port: CanonicalNetwork::DEFAULT_HTTP_PORT,
            https_port: CanonicalNetwork::DEFAULT_HTTPS_PORT,
            log_level: "info",
            connection_timeout: CanonicalTimeouts::CONNECTION_TIMEOUT,
            max_connections: CanonicalNetwork::MAX_CONNECTIONS,
            buffer_size: CanonicalNetwork::DEFAULT_BUFFER_SIZE,
            health_check_interval: CanonicalHealth::CHECK_INTERVAL,
        }
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Get environment-appropriate constants
pub fn get_constants_for_environment() -> CanonicalConstantSet  {match env  {"development" | "dev" => CanonicalConstantSet::development(),
        "production" | "prod" => CanonicalConstantSet::production(),
        "testing" | "test" => CanonicalConstantSet::testing(),
        "staging" | "stage" => CanonicalConstantSet::staging(),
        _ => CanonicalConstantSet::development(), // Default to development
    }
}

/// Get default bind address (backward compatibility function)
pub fn get_default_bind_address() -> &'static str {
    CanonicalNetwork::DEVELOPMENT_BIND_ADDRESS
}

// ============================================================================
// LEGACY COMPATIBILITY EXPORTS
// ============================================================================

/// Legacy compatibility module for gradual migration
pub mod legacy {
    use super::*;
    
    pub use super::CanonicalNetwork as Network;
    pub use super::CanonicalTimeouts as Timeouts;
    pub use super::CanonicalResources as Resources;
    pub use super::CanonicalHealth as Health;
    pub use super::CanonicalGaming as Gaming;
    pub use super::CanonicalDiscovery as Discovery;
    pub use super::CanonicalSystem as System;
use songbird_types::unified_constants::*;
use songbird_config;
    
    // Re-export individual constants for backward compatibility
    pub const DEFAULT_PORT: u16 = CanonicalNetwork::DEFAULT_HTTP_PORT;
    pub const DEFAULT_DISCOVERY_PORT: u16 = CanonicalNetwork::DEFAULT_DISCOVERY_PORT;
    pub const DEFAULT_FEDERATION_PORT: u16 = CanonicalNetwork::DEFAULT_FEDERATION_PORT;
    pub const DEFAULT_CONFIG_PATH: &str = CanonicalSystem::DEFAULT_CONFIG_PATH;
    pub const DEFAULT_LOG_LEVEL: &str = CanonicalSystem::DEFAULT_LOG_LEVEL;
    pub const DEFAULT_CACHE_TTL: Duration = CanonicalSystem::DEFAULT_CACHE_TTL;
    pub const DEFAULT_EVALUATION_TIMEOUT: Duration = CanonicalTimeouts::EVALUATION_TIMEOUT;
    pub const DEFAULT_METRICS_INTERVAL: Duration = CanonicalSystem::METRICS_COLLECTION_INTERVAL;
} 