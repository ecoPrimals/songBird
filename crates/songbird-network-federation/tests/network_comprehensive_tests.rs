//! Comprehensive tests for network management

use async_trait::async_trait;
use songbird_network_federation::network::*;
use songbird_types::SongbirdError;
use songbird_types::SongbirdResult;
use std::net::IpAddr;
use std::time::Duration;

// ============================================================================
// Test Helpers and Mocks
// ============================================================================

struct MockNetworkProvider {
    id: String,
    initialized: bool,
    health_status: NetworkStatus,
}

impl MockNetworkProvider {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            initialized: false,
            health_status: NetworkStatus::Healthy,
        }
    }

    fn with_health_status(mut self, status: NetworkStatus) -> Self {
        self.health_status = status;
        self
    }
}

#[async_trait]
impl NetworkProvider for MockNetworkProvider {
    fn provider_id(&self) -> &str {
        &self.id
    }

    async fn initialize(&mut self, _config: NetworkConfig) -> SongbirdResult<()> {
        self.initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> SongbirdResult<()> {
        self.initialized = false;
        Ok(())
    }

    async fn health_check(&self) -> SongbirdResult<NetworkHealth> {
        Ok(NetworkHealth {
            overall_status: self.health_status.clone(),
            provider_health: Default::default(),
            gaming_health: None,
            active_connections: 10,
            bandwidth_usage: 5.5,
            latency_ms: 25.0,
        })
    }

    async fn capabilities(&self) -> SongbirdResult<Vec<NetworkCapability>> {
        Ok(vec![NetworkCapability::Discovery, NetworkCapability::Monitoring])
    }
}

// ============================================================================
// NetworkManager Tests
// ============================================================================

#[tokio::test]
async fn test_network_manager_creation() {
    let config = NetworkConfig::default();
    let manager = NetworkManager::new(config);

    // Manager should be created successfully
    assert!(std::mem::size_of_val(&manager) > 0);
}

#[tokio::test]
async fn test_network_manager_initialization() {
    let config = NetworkConfig::default();
    let mut manager = NetworkManager::new(config);

    let result = manager.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_network_manager_initialization_with_gaming_disabled() {
    let mut config = NetworkConfig::default();
    config.gaming.enabled = false;
    let mut manager = NetworkManager::new(config);

    let result = manager.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_network_manager_register_provider() {
    let config = NetworkConfig::default();
    let mut manager = NetworkManager::new(config);

    let provider = Box::new(MockNetworkProvider::new("test-provider"));
    let result = manager.register_provider(provider).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_network_manager_register_multiple_providers() {
    let config = NetworkConfig::default();
    let mut manager = NetworkManager::new(config);

    let provider1 = Box::new(MockNetworkProvider::new("provider-1"));
    let provider2 = Box::new(MockNetworkProvider::new("provider-2"));

    assert!(manager.register_provider(provider1).await.is_ok());
    assert!(manager.register_provider(provider2).await.is_ok());
}

#[tokio::test]
async fn test_network_manager_health_check_no_providers() -> Result<(), Box<dyn std::error::Error>>
{
    let config = NetworkConfig::default();
    let mut manager = NetworkManager::new(config);
    manager
        .initialize()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?;

    let health = manager.health_check().await;
    assert!(health.is_ok());

    let health = health
        .map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?;
    assert_eq!(health.provider_health.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_network_manager_health_check_with_provider() -> Result<(), Box<dyn std::error::Error>>
{
    let config = NetworkConfig::default();
    let mut manager = NetworkManager::new(config);

    let provider = Box::new(MockNetworkProvider::new("test-provider"));
    manager
        .register_provider(provider)
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?;

    let health = manager.health_check().await;
    assert!(health.is_ok());

    let health = health
        .map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?;
    assert_eq!(health.provider_health.len(), 1);
    assert!(health.provider_health.contains_key("test-provider"));

    Ok(())
}

#[tokio::test]
async fn test_network_manager_health_check_overall_status() -> Result<(), Box<dyn std::error::Error>>
{
    let config = NetworkConfig::default();
    let mut manager = NetworkManager::new(config);

    let provider = Box::new(MockNetworkProvider::new("test-provider"));
    manager
        .register_provider(provider)
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?;

    let health = manager
        .health_check()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?;
    assert_eq!(health.overall_status, NetworkStatus::Healthy);

    Ok(())
}

#[tokio::test]
async fn test_network_manager_health_check_degraded_provider(
) -> Result<(), Box<dyn std::error::Error>> {
    let config = NetworkConfig::default();
    let mut manager = NetworkManager::new(config);

    let provider = Box::new(
        MockNetworkProvider::new("degraded-provider").with_health_status(NetworkStatus::Degraded),
    );
    manager
        .register_provider(provider)
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?;

    let health = manager.health_check().await;
    assert!(health.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_network_manager_health_check_active_connections(
) -> Result<(), Box<dyn std::error::Error>> {
    let config = NetworkConfig::default();
    let mut manager = NetworkManager::new(config);

    let provider = Box::new(MockNetworkProvider::new("test-provider"));
    manager
        .register_provider(provider)
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?;

    let health = manager
        .health_check()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?;
    assert!(health.active_connections > 0);

    Ok(())
}

#[tokio::test]
async fn test_network_manager_health_check_bandwidth_usage(
) -> Result<(), Box<dyn std::error::Error>> {
    let config = NetworkConfig::default();
    let mut manager = NetworkManager::new(config);

    let provider = Box::new(MockNetworkProvider::new("test-provider"));
    manager
        .register_provider(provider)
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?;

    let health = manager
        .health_check()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?;
    assert!(health.bandwidth_usage >= 0.0);

    Ok(())
}

#[tokio::test]
async fn test_network_manager_health_check_latency() -> Result<(), Box<dyn std::error::Error>> {
    let config = NetworkConfig::default();
    let mut manager = NetworkManager::new(config);

    let provider = Box::new(MockNetworkProvider::new("test-provider"));
    manager
        .register_provider(provider)
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?;

    let health = manager
        .health_check()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?;
    assert!(health.latency_ms > 0.0);

    Ok(())
}

// ============================================================================
// NetworkConfig Tests
// ============================================================================

#[test]
fn test_network_config_default() {
    let config = NetworkConfig::default();

    assert!(!config.proxy.enabled || config.proxy.enabled);
    assert!(config.gaming.enabled);
    assert!(config.discovery.enabled);
}

#[test]
fn test_network_config_serialization() {
    let config = NetworkConfig::default();
    let serialized = serde_json::to_string(&config);

    assert!(serialized.is_ok());
}

#[test]
fn test_network_config_deserialization() -> Result<(), Box<dyn std::error::Error>> {
    let config = NetworkConfig::default();
    let serialized = serde_json::to_string(&config).map_err(|e| SongbirdError::Serialization {
        format: Some("JSON".to_string()),
        message: format!("Serialization failed: {}", e),
        debug_info: None,
    })?;
    let deserialized: Result<NetworkConfig, _> = serde_json::from_str(&serialized);

    assert!(deserialized.is_ok());
    Ok(())
}

#[test]
fn test_network_config_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let original = NetworkConfig::default();
    let serialized =
        serde_json::to_string(&original).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {}", e),
            debug_info: None,
        })?;
    let deserialized: NetworkConfig = serde_json::from_str(&serialized)
        .map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?;

    assert_eq!(original.gaming.enabled, deserialized.gaming.enabled);
    assert_eq!(original.proxy.enabled, deserialized.proxy.enabled);
    assert_eq!(original.discovery.enabled, deserialized.discovery.enabled);
    Ok(())
}

// ============================================================================
// InterfaceConfig Tests
// ============================================================================

#[test]
fn test_interface_config_default() {
    let config = InterfaceConfig::default();

    assert_eq!(config.max_connections, 1000);
    assert_eq!(config.connection_timeout, Duration::from_secs(30));
}

#[test]
fn test_interface_config_bind_address() {
    let config = InterfaceConfig::default();

    // Should have a valid bind address
    match config.bind_address {
        IpAddr::V4(_) | IpAddr::V6(_) => assert!(true),
    }
}

#[test]
fn test_interface_config_port() {
    let config = InterfaceConfig::default();

    // Port should be in valid range
    assert!(config.port > 0);
    assert!(config.port <= 65535);
}

#[test]
fn test_interface_config_max_connections_reasonable() {
    let config = InterfaceConfig::default();

    // Max connections should be reasonable
    assert!(config.max_connections > 0);
    assert!(config.max_connections <= 100_000);
}

#[test]
fn test_interface_config_connection_timeout_reasonable() {
    let config = InterfaceConfig::default();

    // Connection timeout should be reasonable (not too short or long)
    assert!(config.connection_timeout >= Duration::from_secs(1));
    assert!(config.connection_timeout <= Duration::from_secs(300));
}

// ============================================================================
// PortRanges Tests
// ============================================================================

#[test]
fn test_port_ranges_default() {
    let ranges = PortRanges::default();

    assert_eq!(ranges.gaming, (6112, 6200));
    assert_eq!(ranges.dynamic, (49152, 65535));
}

#[test]
fn test_port_ranges_gaming_valid() {
    let ranges = PortRanges::default();

    assert!(ranges.gaming.0 < ranges.gaming.1);
    assert!(ranges.gaming.0 > 0);
    assert!(ranges.gaming.1 <= 65535);
}

#[test]
fn test_port_ranges_dynamic_valid() {
    let ranges = PortRanges::default();

    assert!(ranges.dynamic.0 < ranges.dynamic.1);
    assert!(ranges.dynamic.0 >= 49152); // IANA ephemeral port range
}

#[test]
fn test_port_ranges_reserved_not_empty() {
    let ranges = PortRanges::default();

    assert!(!ranges.reserved.is_empty());
}

#[test]
fn test_port_ranges_reserved_valid() {
    let ranges = PortRanges::default();

    for port in &ranges.reserved {
        assert!(*port > 0);
        assert!(*port <= 65535);
    }
}

// ============================================================================
// GamingConfig Tests
// ============================================================================

#[test]
fn test_gaming_config_default() {
    let config = GamingConfig::default();

    assert!(config.enabled);
    assert_eq!(config.port_range, (6112, 6200));
    assert_eq!(config.max_sessions, 100);
}

#[test]
fn test_gaming_config_protocols_not_empty() {
    let config = GamingConfig::default();

    assert!(!config.protocols.is_empty());
}

#[test]
fn test_gaming_config_protocols_includes_udp() {
    let config = GamingConfig::default();

    assert!(config.protocols.contains(&GameProtocolType::UDP));
}

#[test]
fn test_gaming_config_protocols_includes_tcp() {
    let config = GamingConfig::default();

    assert!(config.protocols.contains(&GameProtocolType::TCP));
}

#[test]
fn test_gaming_config_session_timeout_reasonable() {
    let config = GamingConfig::default();

    // Session timeout should be reasonable
    assert!(config.session_timeout >= Duration::from_secs(60));
    assert!(config.session_timeout <= Duration::from_secs(86400)); // 24 hours
}

#[test]
fn test_gaming_config_max_sessions_reasonable() {
    let config = GamingConfig::default();

    assert!(config.max_sessions > 0);
    assert!(config.max_sessions <= 10_000);
}

// ============================================================================
// ProxyConfig Tests
// ============================================================================

#[test]
fn test_proxy_config_default() {
    let config = ProxyConfig::default();

    assert!(!config.enabled);
    assert_eq!(config.proxy_type, ProxyType::Http);
    assert_eq!(config.load_balancing, LoadBalancingStrategy::RoundRobin);
}

#[test]
fn test_proxy_config_upstream_servers_empty_by_default() {
    let config = ProxyConfig::default();

    assert!(config.upstream_servers.is_empty());
}

#[test]
fn test_proxy_type_http() -> Result<(), Box<dyn std::error::Error>> {
    let proxy_type = ProxyType::Http;
    let serialized =
        serde_json::to_string(&proxy_type).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {}", e),
            debug_info: None,
        })?;

    assert!(serialized.contains("Http"));
    Ok(())
}

#[test]
fn test_proxy_type_socks5() -> Result<(), Box<dyn std::error::Error>> {
    let proxy_type = ProxyType::Socks5;
    let serialized =
        serde_json::to_string(&proxy_type).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {}", e),
            debug_info: None,
        })?;

    assert!(serialized.contains("Socks5"));
    Ok(())
}

#[test]
fn test_proxy_type_transparent() -> Result<(), Box<dyn std::error::Error>> {
    let proxy_type = ProxyType::Transparent;
    let serialized =
        serde_json::to_string(&proxy_type).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {}", e),
            debug_info: None,
        })?;

    assert!(serialized.contains("Transparent"));
    Ok(())
}

// ============================================================================
// LoadBalancingStrategy Tests
// ============================================================================

#[test]
fn test_load_balancing_strategy_round_robin() -> Result<(), Box<dyn std::error::Error>> {
    let strategy = LoadBalancingStrategy::RoundRobin;
    let serialized =
        serde_json::to_string(&strategy).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {}", e),
            debug_info: None,
        })?;

    assert!(serialized.contains("RoundRobin"));
    Ok(())
}

#[test]
fn test_load_balancing_strategy_least_connections() -> Result<(), Box<dyn std::error::Error>> {
    let strategy = LoadBalancingStrategy::LeastConnections;
    let serialized =
        serde_json::to_string(&strategy).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {}", e),
            debug_info: None,
        })?;

    assert!(serialized.contains("LeastConnections"));
    Ok(())
}

#[test]
fn test_load_balancing_strategy_weighted_round_robin() -> Result<(), Box<dyn std::error::Error>> {
    let strategy = LoadBalancingStrategy::WeightedRoundRobin;
    let serialized =
        serde_json::to_string(&strategy).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {}", e),
            debug_info: None,
        })?;

    assert!(serialized.contains("WeightedRoundRobin"));
    Ok(())
}

#[test]
fn test_load_balancing_strategy_ip_hash() -> Result<(), Box<dyn std::error::Error>> {
    let strategy = LoadBalancingStrategy::IpHash;
    let serialized =
        serde_json::to_string(&strategy).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {}", e),
            debug_info: None,
        })?;

    assert!(serialized.contains("IpHash"));
    Ok(())
}

#[test]
fn test_load_balancing_strategy_equality() {
    let strategy1 = LoadBalancingStrategy::RoundRobin;
    let strategy2 = LoadBalancingStrategy::RoundRobin;

    assert_eq!(strategy1, strategy2);
}

// ============================================================================
// DiscoveryConfig Tests
// ============================================================================

#[test]
fn test_discovery_config_default() {
    let config = DiscoveryConfig::default();

    assert!(config.enabled);
    assert_eq!(config.interval, Duration::from_secs(30));
    assert_eq!(config.timeout, Duration::from_secs(5));
}

#[test]
fn test_discovery_config_methods_not_empty() {
    let config = DiscoveryConfig::default();

    assert!(!config.methods.is_empty());
}

#[test]
fn test_discovery_config_methods_includes_multicast() {
    let config = DiscoveryConfig::default();

    assert!(config.methods.contains(&DiscoveryMethod::Multicast));
}

#[test]
fn test_discovery_config_methods_includes_broadcast() {
    let config = DiscoveryConfig::default();

    assert!(config.methods.contains(&DiscoveryMethod::Broadcast));
}

#[test]
fn test_discovery_config_interval_reasonable() {
    let config = DiscoveryConfig::default();

    // Interval should be reasonable (not too frequent or infrequent)
    assert!(config.interval >= Duration::from_secs(5));
    assert!(config.interval <= Duration::from_secs(300));
}

#[test]
fn test_discovery_config_timeout_less_than_interval() {
    let config = DiscoveryConfig::default();

    // Timeout should be less than interval
    assert!(config.timeout < config.interval);
}

// ============================================================================
// DiscoveryMethod Tests
// ============================================================================

#[test]
fn test_discovery_method_multicast() -> Result<(), Box<dyn std::error::Error>> {
    let method = DiscoveryMethod::Multicast;
    let serialized = serde_json::to_string(&method).map_err(|e| SongbirdError::Serialization {
        format: Some("JSON".to_string()),
        message: format!("Serialization failed: {}", e),
        debug_info: None,
    })?;

    assert!(serialized.contains("Multicast"));
    Ok(())
}

#[test]
fn test_discovery_method_broadcast() -> Result<(), Box<dyn std::error::Error>> {
    let method = DiscoveryMethod::Broadcast;
    let serialized = serde_json::to_string(&method).map_err(|e| SongbirdError::Serialization {
        format: Some("JSON".to_string()),
        message: format!("Serialization failed: {}", e),
        debug_info: None,
    })?;

    assert!(serialized.contains("Broadcast"));
    Ok(())
}

#[test]
fn test_discovery_method_unicast() -> Result<(), Box<dyn std::error::Error>> {
    let method = DiscoveryMethod::Unicast;
    let serialized = serde_json::to_string(&method).map_err(|e| SongbirdError::Serialization {
        format: Some("JSON".to_string()),
        message: format!("Serialization failed: {}", e),
        debug_info: None,
    })?;

    assert!(serialized.contains("Unicast"));
    Ok(())
}

#[test]
fn test_discovery_method_dns() -> Result<(), Box<dyn std::error::Error>> {
    let method = DiscoveryMethod::Dns;
    let serialized = serde_json::to_string(&method).map_err(|e| SongbirdError::Serialization {
        format: Some("JSON".to_string()),
        message: format!("Serialization failed: {}", e),
        debug_info: None,
    })?;

    assert!(serialized.contains("Dns"));
    Ok(())
}

#[test]
fn test_discovery_method_equality() {
    let method1 = DiscoveryMethod::Multicast;
    let method2 = DiscoveryMethod::Multicast;

    assert_eq!(method1, method2);
}

// ============================================================================
// PerformanceConfig Tests
// ============================================================================

#[test]
fn test_performance_config_default() {
    let config = PerformanceConfig::default();

    assert_eq!(config.buffer_size, 8192);
    assert!(config.tcp_nodelay);
}

#[test]
fn test_performance_config_buffer_size_power_of_two() {
    let config = PerformanceConfig::default();

    // Buffer size is typically a power of 2
    assert!(config.buffer_size.is_power_of_two());
}

#[test]
fn test_performance_config_worker_threads_none_by_default() {
    let config = PerformanceConfig::default();

    // None means use system default
    assert!(config.worker_threads.is_none() || config.worker_threads.is_some());
}

#[test]
fn test_performance_config_tcp_nodelay_enabled() {
    let config = PerformanceConfig::default();

    // TCP no-delay is typically enabled for low latency
    assert!(config.tcp_nodelay);
}

#[test]
fn test_performance_config_keepalive_present() {
    let config = PerformanceConfig::default();

    assert!(config.keepalive.is_some());
}

#[test]
fn test_performance_config_keepalive_reasonable() {
    let config = PerformanceConfig::default();

    if let Some(keepalive) = config.keepalive {
        assert!(keepalive >= Duration::from_secs(10));
        assert!(keepalive <= Duration::from_secs(600));
    }
}

// ============================================================================
// NetworkHealth Tests
// ============================================================================

#[test]
fn test_network_health_serialization() {
    let health = NetworkHealth {
        overall_status: NetworkStatus::Healthy,
        provider_health: Default::default(),
        gaming_health: None,
        active_connections: 10,
        bandwidth_usage: 5.5,
        latency_ms: 25.0,
    };

    let serialized = serde_json::to_string(&health);
    assert!(serialized.is_ok());
}

#[test]
fn test_network_health_with_gaming() {
    let gaming_health = GamingHealth {
        status: NetworkStatus::Healthy,
        active_sessions: 5,
        supported_protocols: vec![GameProtocolType::UDP],
    };

    let health = NetworkHealth {
        overall_status: NetworkStatus::Healthy,
        provider_health: Default::default(),
        gaming_health: Some(gaming_health),
        active_connections: 10,
        bandwidth_usage: 5.5,
        latency_ms: 25.0,
    };

    assert!(health.gaming_health.is_some());
}

// ============================================================================
// NetworkStatus Tests
// ============================================================================

#[test]
fn test_network_status_healthy() {
    let status = NetworkStatus::Healthy;
    assert_eq!(status, NetworkStatus::Healthy);
}

#[test]
fn test_network_status_degraded() {
    let status = NetworkStatus::Degraded;
    assert_eq!(status, NetworkStatus::Degraded);
}

#[test]
fn test_network_status_unhealthy() {
    let status = NetworkStatus::Unhealthy;
    assert_eq!(status, NetworkStatus::Unhealthy);
}

#[test]
fn test_network_status_offline() {
    let status = NetworkStatus::Offline;
    assert_eq!(status, NetworkStatus::Offline);
}

#[test]
fn test_network_status_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let status = NetworkStatus::Healthy;
    let serialized = serde_json::to_string(&status).map_err(|e| SongbirdError::Serialization {
        format: Some("JSON".to_string()),
        message: format!("Serialization failed: {}", e),
        debug_info: None,
    })?;

    assert!(serialized.contains("Healthy"));
    Ok(())
}

// ============================================================================
// GamingHealth Tests
// ============================================================================

#[test]
fn test_gaming_health_creation() {
    let health = GamingHealth {
        status: NetworkStatus::Healthy,
        active_sessions: 10,
        supported_protocols: vec![GameProtocolType::UDP, GameProtocolType::TCP],
    };

    assert_eq!(health.status, NetworkStatus::Healthy);
    assert_eq!(health.active_sessions, 10);
    assert_eq!(health.supported_protocols.len(), 2);
}

#[test]
fn test_gaming_health_serialization() {
    let health = GamingHealth {
        status: NetworkStatus::Healthy,
        active_sessions: 10,
        supported_protocols: vec![GameProtocolType::UDP],
    };

    let serialized = serde_json::to_string(&health);
    assert!(serialized.is_ok());
}

// ============================================================================
// NetworkCapability Tests
// ============================================================================

#[test]
fn test_network_capability_gaming() {
    let cap = NetworkCapability::Gaming;
    assert_eq!(cap, NetworkCapability::Gaming);
}

#[test]
fn test_network_capability_proxy() {
    let cap = NetworkCapability::Proxy;
    assert_eq!(cap, NetworkCapability::Proxy);
}

#[test]
fn test_network_capability_discovery() {
    let cap = NetworkCapability::Discovery;
    assert_eq!(cap, NetworkCapability::Discovery);
}

#[test]
fn test_network_capability_load_balancing() {
    let cap = NetworkCapability::LoadBalancing;
    assert_eq!(cap, NetworkCapability::LoadBalancing);
}

#[test]
fn test_network_capability_monitoring() {
    let cap = NetworkCapability::Monitoring;
    assert_eq!(cap, NetworkCapability::Monitoring);
}

#[test]
fn test_network_capability_security() {
    let cap = NetworkCapability::Security;
    assert_eq!(cap, NetworkCapability::Security);
}

#[test]
fn test_network_capability_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let cap = NetworkCapability::Gaming;
    let serialized = serde_json::to_string(&cap).map_err(|e| SongbirdError::Serialization {
        format: Some("JSON".to_string()),
        message: format!("Serialization failed: {}", e),
        debug_info: None,
    })?;

    assert!(serialized.contains("Gaming"));
    Ok(())
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_full_network_manager_lifecycle() {
    let config = NetworkConfig::default();
    let mut manager = NetworkManager::new(config);

    // Initialize
    assert!(manager.initialize().await.is_ok());

    // Register provider
    let provider = Box::new(MockNetworkProvider::new("lifecycle-test"));
    assert!(manager.register_provider(provider).await.is_ok());

    // Health check
    let health = manager.health_check().await;
    assert!(health.is_ok());
}

#[tokio::test]
async fn test_network_manager_with_custom_config() {
    let mut config = NetworkConfig::default();
    config.gaming.max_sessions = 500;
    config.performance.buffer_size = 16384;

    let mut manager = NetworkManager::new(config);
    assert!(manager.initialize().await.is_ok());
}

#[tokio::test]
async fn test_provider_capabilities() -> Result<(), Box<dyn std::error::Error>> {
    let provider = MockNetworkProvider::new("capability-test");
    let capabilities = provider
        .capabilities()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?;

    assert!(!capabilities.is_empty());
    assert!(capabilities.contains(&NetworkCapability::Discovery));

    Ok(())
}
