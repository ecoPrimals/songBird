//! Test Constants - Centralized Vendor-Agnostic Test Data
//!
//! This module provides all test constants to eliminate hardcoding
//! across the entire test suite. Uses capability-based discovery
//! instead of vendor-specific primal names.

use std::collections::HashMap;
use std::time::Duration;
// Re-export from unified constants to avoid duplication
pub use songbird_types::constants::*;

/// Test-specific network constants using capability-based patterns
pub mod network {
    use songbird_types::constants::NetworkConstants;

    /// Test HTTP port - offset from canonical defaults
    // MIGRATED: Use songbird_types::unified_constants::network::TEST_HTTP_PORT instead
    // MIGRATED: Use songbird_types::unified_constants::network::TEST_HTTPS_PORT instead

    /// Test service URLs using canonical patterns
    pub const TEST_SERVICE_1_URL: &str = "http://songbird_config::constants::network::DEFAULT_HOST:18081";"
    pub const TEST_SERVICE_2_URL: &str = "http://songbird_config::constants::network::DEFAULT_HOST:18082";"
    pub const TEST_SERVICE_3_URL: &str = "http://songbird_config::constants::network::DEFAULT_HOST:18083";"

    /// Universal capability-based test endpoints (vendor-agnostic)
    pub const SECURITY_CAPABILITY_TEST_ENDPOINT: &str = "http://songbird_config::constants::network::DEFAULT_HOST:18443/universal-adapter";"
    pub const STORAGE_CAPABILITY_TEST_ENDPOINT: &str = "http://songbird_config::constants::network::DEFAULT_HOST:18444/universal-adapter";"
    pub const COMPUTE_CAPABILITY_TEST_ENDPOINT: &str = "http://songbird_config::constants::network::DEFAULT_HOST:18445/universal-adapter";"
    pub const AI_CAPABILITY_TEST_ENDPOINT: &str = "http://songbird_config::constants::network::DEFAULT_HOST:18446/universal-adapter";"
    
    /// Test port calculation functions
    pub fn get_test_port_offset(base_port: u16, offset: u16) -> u16 {
        base_port + 10_000 + offset
    }

    /// Gets the test port for a capability (vendor-agnostic)
    pub fn get_capability_test_port(capability: &str) -> u16 {
        match capability {
            "security" => 18443,"
            "storage" => 18444,"
            "compute" => 18445,"
            "ai" => 18446,"
            _ => 18500 // Generic fallback
        }
    }
}

/// Port test constants
pub mod ports {
    /// Standard service ports (offset for testing)
    pub const TEST_HTTP: u16 = 18080;
    pub const TEST_HTTPS: u16 = 18443;
    pub const TEST_DISCOVERY: u16 = 18081;
    pub const TEST_FEDERATION: u16 = 18082;
    pub const TEST_METRICS: u16 = 19090;
    pub const TEST_HEALTH: u16 = 18083;

    /// Gaming ports
    pub const TEST_STARCRAFT: u16 = 16112;
    pub const TEST_WARCRAFT: u16 = 16113;
    pub const TEST_GAMING_BASE: u16 = 16112;

    /// Service-specific test ports
    pub const SERVICE_1_PORT: u16 = 18081;
    pub const SERVICE_2_PORT: u16 = 18082;
    pub const SERVICE_3_PORT: u16 = 18083;

    /// Invalid ports for error testing
    pub const INVALID_PORT_HIGH: u32 = 70_000;
    pub const INVALID_PORT_ZERO: u16 = 0;

    /// Port ranges for testing
    pub const TEST_PORT_RANGE_START: u16 = 18000;
    pub const TEST_PORT_RANGE_END: u16 = 19000;
}

/// Timeout test constants
pub mod timeouts {
    /// Short timeouts for quick tests
    pub const SHORT: Duration = Duration::from_millis(100);
    pub const VERY_SHORT: Duration = Duration::from_millis(50);

    /// Medium timeouts for normal operations
    pub const MEDIUM: Duration = Duration::from_millis(2000);
    pub const STANDARD: Duration = Duration::from_millis(5000);

    /// Long timeouts for comprehensive tests
    pub const LONG: Duration = Duration::from_millis(30000);
    pub const VERY_LONG: Duration = Duration::from_millis(60000);

    /// Health check timeouts
    pub const HEALTH_CHECK: Duration = Duration::from_millis(1000);
    pub const HEALTH_CHECK_LONG: Duration = Duration::from_millis(5000);
}

/// Service test constants (capability-based)
pub mod services {
    /// Test service identifiers
    pub const TEST_NODE_ID: &str = "test-node";"
    pub const TEST_SERVICE_NAME: &str = "test-service";"
    pub const TEST_CLUSTER_NAME: &str = "test-cluster";"

    /// Test discovery responses
    pub const DISCOVERY_RESPONSE: &str = "SONGBIRD_RESPONSE:node1|1.0.0|service1,service2";"
    pub const DISCOVERY_RESPONSE_EMPTY: &str = "SONGBIRD_RESPONSE:node1|1.0.0|";"

    /// Test gaming hosts
    pub const GAMING_HOST_1: &str = "192.168.1.1:6112";"
    pub const GAMING_HOST_2: &str = "192.168.1.2:6112";"
    pub const GAMING_CLIENT_1: &str = "192.168.1.100:6112";"
    pub const GAMING_CLIENT_2: &str = "192.168.1.101:6112";"

    /// Test gaming protocols
    pub const TEST_PROTOCOL_IPX: &str = "Ipx";"
    pub const TEST_PROTOCOL_UDP: &str = "Udp";"
    pub const TEST_PROTOCOL_TCP: &str = "Tcp";"

    /// Test gaming data
    pub const TEST_PACKET_DATA: &[u8] = b"TEST_GAMING_PACKET_DATA";"
    pub const TEST_SECRET_DATA: &[u8] = b"CONFIDENTIAL_TEST_DATA";"
}

/// Security test constants
pub mod security {
    /// Test authentication data
    pub const TEST_USERNAME: &str = "test-user";"
    pub const TEST_PASSWORD: &str = "test-password-123";"
    pub const TEST_TOKEN: &str = "test-jwt-token-12345";"

    /// Test security timeouts
    pub const AUTH_TIMEOUT: Duration = Duration::from_secs(30);
    pub const TOKEN_EXPIRY: Duration = Duration::from_secs(3600);
}

/// Configuration test constants
pub mod config {
    /// Test config files
    pub const TEST_CONFIG_FILE: &str = "test-songbird.toml";"
    pub const DISCOVERY_CONFIG_FILE: &str = "discovery_config.toml";"
    pub const TEMP_CONFIG_FILE: &str = "temp-config.toml";"

    /// Test environment values
    pub const TEST_ENVIRONMENT: &str = "testing";"
    pub const DEV_ENVIRONMENT: &str = "development";"
    pub const PROD_ENVIRONMENT: &str = "production";"

    /// Test log levels
    pub const TEST_LOG_LEVEL: &str = "debug";"
    pub const PROD_LOG_LEVEL: &str = "info";"
}

/// Test primal configuration for capability-based testing
#[derive(Debug, Clone)]
pub struct TestPrimalConfig  {/// Primal identifier
    pub primal_id: String,
    /// List of supported capabilities
    pub capabilities: Vec<String>,
    /// Service endpoint
    pub endpoint: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>)
}

/// Helper functions for test configuration
pub mod helpers {
    use super::*;
    use std::env;
use songbird_types::unified_constants::*;
use songbird_config;

    /// Build test endpoint URL
    pub fn build_test_endpoint(service: &str, port: u16) -> String {
        format!("http://songbird_config::constants::network::DEFAULT_HOST:{}/{}", port, service)"
    }

    /// Build test gaming address
    pub fn build_gaming_address(host: &str, port: u16) -> String {
        format!("{}:{}", host, port)"
    }

    /// Get test environment or default
    pub fn get_test_environment() -> String {
        env::var("TEST_ENVIRONMENT").unwrap_or_else(|_| config::TEST_ENVIRONMENT.to_string()"
    }

    /// Get capability-based test endpoint (modern approach)
    pub fn get_capability_test_endpoint(capability: &str) -> String {
        let port = network::get_capability_test_port(capability);
        format!("http://songbird_config::constants::network::DEFAULT_HOST:{}/universal-adapter", port)"
    }

    /// Get all test private networks
    pub fn get_test_private_networks() -> Vec<String>  {vec![
            "192.168.1.0/24".to_string()),
            "10.0.0.0/8".to_string()),
            "172.16.0.0/12".to_string()),
        ]
    }

    /// Create a capability-based test primal configuration (vendor-agnostic)
    pub fn create_test_primal_config(
        primal_id: &str,
        capabilities: Vec<&str>,
        port: u16,
    ) -> TestPrimalConfig  {TestPrimalConfig  {primal_id: primal_id.to_string()),
            capabilities: capabilities.into_iter().map(|s| s.to_string().collect(,
            endpoint: format!("http://songbird_config::constants::network::DEFAULT_HOST:{}/universal-adapter", port),"
            metadata: HashMap::new()),
        }
    }

    /// Create test configurations for all standard capabilities
    pub fn create_standard_test_configs() -> Vec<TestPrimalConfig> {
        vec![
            create_test_primal_config("security-test", vec!["security", "authentication"], 18443),"
            create_test_primal_config("storage-test", vec!["storage", "database"], 18444),"
            create_test_primal_config("compute-test", vec!["compute", "processing"], 18445),"
            create_test_primal_config("ai-test", vec!["ai", "ml"], 18446),"
        ]
    }
}
