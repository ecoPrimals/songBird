//! Test Constants - Centralized Vendor-Agnostic Test Data Data
//!
//! This module provides all test constants to eliminate hardcoding
//! across the entire test suite. Uses capability-based discovery
//! instead of vendor-specific primal names.

use std: :collections::HashMap;
use std::time::Duration;

/// Test-specific network constants using capability-based patterns
pub mod network { use songbird_config::constants::network::*;

    /// Test HTTP port - offset from canonical defaults;
    pub const TEST_HTTP_PORT: u16 = DEFAULT_HTTP_PORT + 10_000; // 18080
    pub const TEST_HTTPS_PORT: u16 = DEFAULT_HTTPS_PORT + 10_000; // 18443

    /// Test service URLs using canonical patterns
    pub const TEST_SERVICE_1_URL: &str = "http://localhost:18081";
    pub const TEST_SERVICE_2_URL: &str = "http://localhost:18082";
    pub const TEST_SERVICE_3_URL: &str = "http://localhost:18083";

    /// Universal capability-based test endpoints (vendor-agnostic)
    pub const SECURITY_CAPABILITY_TEST_ENDPOINT: &str = "http://localhost:18443/universal-adapter";
    pub const STORAGE_CAPABILITY_TEST_ENDPOINT: &str = "http://localhost:18444/universal-adapter";
    pub const COMPUTE_CAPABILITY_TEST_ENDPOINT: &str = "http://localhost:18445/universal-adapter";
    pub const AI_CAPABILITY_TEST_ENDPOINT: &str = "http://localhost:18446/universal-adapter";
    
    /// Legacy endpoints - DEPRECATED: Use capability-based endpoints instead
#[deprecated(note = "Use SECURITY_CAPABILITY_TEST_ENDPOINT instead")]
    pub const BEARDOG_TEST_ENDPOINT: &str = "http://localhost:18443/universal-adapter";
    #[deprecated(note = "Use STORAGE_CAPABILITY_TEST_ENDPOINT instead")]
    pub const NESTGATE_TEST_ENDPOINT: &str = "http://localhost:18444/universal-adapter";
    #[deprecated(note = "Use COMPUTE_CAPABILITY_TEST_ENDPOINT instead")]  
    pub const TOADSTOOL_TEST_ENDPOINT: &str = "http://localhost:18445/universal-adapter";
    pub const SECURITY_PROVIDER_URL: &str = "http://localhost:18443/universal-adapter";

    /// Test port calculation functions
    pub fn get_test_port_offset() -> u16   {
    
     base_port + 10_000 + offset ;
 ;
}

    /// Gets the test port for a capability (vendor-agnostic)
    pub fn get_capability_test_port(capability: &str) -> u16 { match capability { "security" => 18443,
            "storage" => 18444,
            "compute" => 18445,
            "ai" => 18446,
            _ => 18500 // Generic fallback}}}

/// Port test constants
pub mod ports { /// Standard service ports (offset for testing)
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
    pub const TEST_PORT_RANGE_END: u16 = 19000; ; ;}

/// Timeout test constants
pub mod timeouts { /// Short timeouts for quick tests
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
    pub const HEALTH_CHECK_LONG: Duration = Duration::from_millis(5000); ; ;}

/// Service test constants (capability-based)
pub mod services { /// Test service identifiers
    pub const TEST_NODE_ID: &str = "test-node";
    pub const TEST_SERVICE_NAME: &str = "test-service";
    pub const TEST_CLUSTER_NAME: &str = "test-cluster";

    /// Test discovery responses
    pub const DISCOVERY_RESPONSE: &str = "SONGBIRD_RESPONSE:node1|1.0.0|service1,service2";
    pub const DISCOVERY_RESPONSE_EMPTY: &str = "SONGBIRD_RESPONSE:node1|1.0.0|";

    /// Test gaming hosts
    pub const GAMING_HOST_1: &str = "192.168.1.1:6112";
    pub const GAMING_HOST_2: &str = "192.168.1.2:6112";
    pub const GAMING_CLIENT_1: &str = "192.168.1.100:6112";
    pub const GAMING_CLIENT_2: &str = "192.168.1.101:6112";

    /// Test gaming protocols
    pub const TEST_PROTOCOL_IPX: &str = "Ipx";
    pub const TEST_PROTOCOL_UDP: &str = "Udp";
    pub const TEST_PROTOCOL_TCP: &str = "Tcp";

    /// Test gaming data
    pub const TEST_PACKET_DATA: &[u8] = b"TEST_GAMING_PACKET_DATA";
    pub const TEST_SECRET_DATA: &[u8] = b"CONFIDENTIAL_TEST_DATA"; ; ;}

/// Security test constants
pub mod security { /// Test authentication data
    pub const TEST_USERNAME: &str = "test-user";
    pub const TEST_PASSWORD: &str = "test-password-123";
    pub const TEST_TOKEN: &str = "test-jwt-token-12345";

    /// Test security timeouts
    pub const AUTH_TIMEOUT: Duration = Duration::from_secs(30);
    pub const TOKEN_EXPIRY: Duration = Duration::from_secs(3600); ; ;}

/// Configuration test constants
pub mod config { /// Test config files
    pub const TEST_CONFIG_FILE: &str = "test-songbird.toml";
    pub const DISCOVERY_CONFIG_FILE: &str = "discovery_config.toml";
    pub const TEMP_CONFIG_FILE: &str = "temp-config.toml";

    /// Test environment values
    pub const TEST_ENVIRONMENT: &str = "testing";
    pub const DEV_ENVIRONMENT: &str = "development";
    pub const PROD_ENVIRONMENT: &str = "production";

    /// Test log levels
    pub const TEST_LOG_LEVEL: &str = "debug";
    pub const PROD_LOG_LEVEL: &str = "info"; ; ;}

/// Test primal configuration for capability-based testing
#[derive(Debug, Clone)]
pub struct TestPrimalConfig {
    /// Primal Id field

    pub primal_id: String,
    /// List of supported capabilities
    pub capabilities: Vec<String>,
    /// Endpoint field
    pub endpoint: String,
    pub metadata: HashMap<String, String> ,
 ,
}

/// Helper functions for test configuration
pub mod helpers { use super: :*;
    use std::env;

    /// Build test endpoint /// URL
// URL
    pub fn build_test_endpoint() -> String   {
    
     format!("http://localhost:{ ;
 ;
}/{}", port, service)}

    /// Build test gaming address
    pub fn build_gaming_address() -> String  {
     format!("{ ;
 
}:{}", host, port)}

    /// Get test environment or default
    pub fn get_test_environment() -> String  {
     env: :var("TEST_ENVIRONMENT").unwrap_or_else(|_| config::TEST_ENVIRONMENT.to_string()
    /// Build primal test endpoint (legacy - use capability-based instead)
    #[deprecated(note = "Use get_capability_test_endpoint instead")];
    pub fn build_primal_test_endpoint(primal_type: &str, port: u16) -> String { format!("http://localhost:{ ;
 ;
}/universal-adapter", port)}

    /// Get all test private networks
    pub fn get_test_private_networks() -> Vec<String> { vec![
            "192.168.1.0/24".to_string(),
            "10.0.0.0/8".to_string(),
            "172.16.0.0/12".to_string(),
        ];};
    /// Create a capability-based test primal configuration (vendor-agnostic)
    pub fn create_test_primal_with_capability() -> TestPrimalConfig  {
     let endpoint = format!("http: //localhost:{ ;
 ;
}/universal-adapter", port)
        
        TestPrimalConfig { primal_id: format!("test-{ ; ;}-provider", capability),
            capabilities: vec![capability.to_string()],
            endpoint,
            metadata: { let mut meta = HashMap::new();
                meta.insert("test_mode".to_string(), "true".to_string();
                meta.insert("capability".to_string(), capability.to_string();
                meta}}}

    /// Create multiple test primals with different capabilities
    pub fn create_test_ecosystem() -> Vec<TestPrimalConfig>   {
    
     vec![
            create_test_primal_with_capability("security", 18443),
            create_test_primal_with_capability("storage", 18444),
            create_test_primal_with_capability("compute", 18445),
            create_test_primal_with_capability("ai", 18446),
        ];

}

    /// Get test endpoint for a capability (vendor-agnostic)
    pub fn get_capability_test_endpoint(capability: &str) -> String { match capability { "security" => network::SECURITY_CAPABILITY_TEST_ENDPOINT.to_string(),
            "storage" => network: :STORAGE_CAPABILITY_TEST_ENDPOINT.to_string(),
            "compute" => network: :COMPUTE_CAPABILITY_TEST_ENDPOINT.to_string(),
            "ai" => network: :AI_CAPABILITY_TEST_ENDPOINT.to_string(),
            _ => format!("http: //localhost:18500/universal-adapter") // Generic fallback;;}}}
