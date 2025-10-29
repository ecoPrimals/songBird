// Tests for network configuration module
// Included via mod.rs with #[cfg(test)]

use super::*;

#[test]
fn test_network_config_defaults() {
    let config = NetworkConfig::default();
    assert_eq!(config.orchestrator_port, 8080);
    assert_eq!(config.gaming.starcraft_port, 6112);
    assert_eq!(config.gaming.aoe2_port, 2300);
}

#[test]
fn test_endpoint_generation() {
    let config = NetworkConfig::default();
    let endpoint = config.orchestrator_endpoint();
    assert_eq!(endpoint.port(), 8080);
}

#[test]
fn test_gaming_port_lookup() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = NetworkConfig::default();
    assert_eq!(
        config.gaming_port("starcraft").map_err(|e| SongbirdError::configuration(format!(
            "Test: starcraft port should be found: {e}"
        )))?,
        6112
    );
    assert_eq!(
        config.gaming_port("aoe2").map_err(|e| SongbirdError::configuration(format!(
            "Test: aoe2 port should be found: {e}"
        )))?,
        2300
    );
    assert!(config.gaming_port("unknown").is_err());
    Ok(())
}

#[test]
fn test_timeout_lookup() {
    let config = NetworkConfig::default();
    assert_eq!(config.timeout("connection"), Duration::from_secs(10));
    assert_eq!(config.timeout("health_check"), Duration::from_secs(5));
    assert_eq!(config.timeout("unknown"), Duration::from_secs(30));
}

#[test]
fn test_port_range() {
    let range = PortRange {
        start: 100,
        end: 200,
    };
    assert!(range.contains(150));
    assert!(!range.contains(50));
    assert!(!range.contains(250));
}

#[test]
fn test_config_validation() {
    let mut config = NetworkConfig::default();
    assert!(config.validate().is_ok());

    // Test invalid port range
    config.gaming_port_range.start = 200;
    config.gaming_port_range.end = 100;
    assert!(config.validate().is_err());
}

#[test]
fn test_gaming_scale_configs() {
    use GamingScale;
    // use songbird_config; // FIXED: Circular import removed

    let home_config = NetworkConfig::for_gaming_scale(&GamingScale::Home);
    let lan_config = NetworkConfig::for_gaming_scale(&GamingScale::LanParty);

    assert!(
        home_config.connection_limits.max_total_connections
            < lan_config.connection_limits.max_total_connections
    );
}

/// Example configurations for different gaming scales
#[allow(dead_code)]
pub fn example_configurations() -> Vec<(GamingScale, NetworkConfig)> {
    let home_config = NetworkConfig::for_gaming_scale(&GamingScale::Home);
    let lan_config = NetworkConfig::for_gaming_scale(&GamingScale::LanParty);

    vec![(GamingScale::Home, home_config), (GamingScale::LanParty, lan_config)]
}

#[test]
fn test_port_range_edge_cases() {
    let range = PortRange {
        start: 100,
        end: 100,
    };
    assert!(range.contains(100), "Port range should contain the single port when start == end");

    let range = PortRange {
        start: 1,
        end: 65535,
    };
    assert!(range.contains(1), "Should contain minimum port");
    assert!(range.contains(65535), "Should contain maximum port");
    assert!(!range.contains(0), "Should not contain port 0");
}

#[test]
fn test_network_timeouts_validation() {
    let timeouts = NetworkTimeouts {
        connection: Duration::from_secs(10),
        request: Duration::from_secs(30),
        health_check: Duration::from_secs(5),
        default: Duration::from_secs(20),
    };

    assert_eq!(timeouts.connection, Duration::from_secs(10));
    assert_eq!(timeouts.request, Duration::from_secs(30));
    assert_eq!(timeouts.health_check, Duration::from_secs(5));
    assert_eq!(timeouts.default, Duration::from_secs(20));
}

#[test]
fn test_connection_limits_validation() {
    let limits = ConnectionLimits {
        max_total_connections: 1000,
        max_connections_per_host: 10,
        max_retries: 3,
        pool_idle_timeout_secs: 300,
    };

    assert_eq!(limits.max_total_connections, 1000);
    assert_eq!(limits.max_connections_per_host, 10);
    assert_eq!(limits.max_retries, 3);
    assert_eq!(limits.pool_idle_timeout_secs, 300);
    assert!(
        limits.max_connections_per_host <= limits.max_total_connections,
        "Per-host limit should not exceed total limit"
    );
}

#[test]
fn test_gaming_network_config_defaults() {
    let gaming_config = GamingNetworkConfig {
        starcraft_port: 6112,
        aoe2_port: 2300,
        cnc_port_range: PortRange {
            start: 1234,
            end: 1240,
        },
        detection_interface: Some("eth0".to_string()),
        bridge_buffer_size: 65536,
    };

    assert_eq!(gaming_config.starcraft_port, 6112);
    assert_eq!(gaming_config.aoe2_port, 2300);
    assert_eq!(gaming_config.cnc_port_range.start, 1234);
    assert_eq!(gaming_config.cnc_port_range.end, 1240);
    assert_eq!(gaming_config.bridge_buffer_size, 65536);
}

#[test]
fn test_network_config_from_environment_fallback() {
    // Test that from_environment falls back to defaults when env vars are missing
    let config = NetworkConfig::default();

    // Should have valid default ports even without env vars
    assert!(config.orchestrator_port > 0, "Should have valid orchestrator port");
    assert!(config.health_port > 0, "Should have valid health port");
    assert!(config.dashboard_port > 0, "Should have valid dashboard port");
}

#[test]
fn test_gaming_port_error_handling() {
    let config = NetworkConfig::default();

    // Test unknown game returns error
    let result = config.gaming_port("nonexistent_game");
    assert!(result.is_err(), "Should return error for unknown game name");

    // Test empty string returns error
    let result = config.gaming_port("");
    assert!(result.is_err(), "Should return error for empty game name");
}

#[test]
fn test_timeout_lookup_with_defaults() {
    let config = NetworkConfig::default();

    // Test known timeouts
    let conn_timeout = config.timeout("connection");
    assert!(conn_timeout.as_secs() > 0, "Connection timeout should be positive");

    // Test unknown timeout returns default
    let unknown_timeout = config.timeout("unknown_timeout_type");
    assert_eq!(
        unknown_timeout,
        Duration::from_secs(30),
        "Unknown timeout should return 30s default"
    );
}

#[test]
fn test_network_config_validation_error_cases() {
    let mut config = NetworkConfig::default();

    // Test invalid port range (start > end)
    config.gaming_port_range.start = 9000;
    config.gaming_port_range.end = 8000;
    let result = config.validate();
    assert!(result.is_err(), "Validation should fail when start port > end port");

    // Test valid range after fix
    config.gaming_port_range.start = 8000;
    config.gaming_port_range.end = 9000;
    let result = config.validate();
    assert!(result.is_ok(), "Validation should pass with valid port range");
}

#[test]
fn test_gaming_scale_configurations_ordering() {
    let home = NetworkConfig::for_gaming_scale(&GamingScale::Home);
    let lan = NetworkConfig::for_gaming_scale(&GamingScale::LanParty);
    let tournament = NetworkConfig::for_gaming_scale(&GamingScale::Tournament);
    let professional = NetworkConfig::for_gaming_scale(&GamingScale::Professional);

    // Verify scaling of connection limits
    assert!(
        home.connection_limits.max_total_connections < lan.connection_limits.max_total_connections,
        "LAN should support more connections than Home"
    );
    assert!(
        lan.connection_limits.max_total_connections
            < tournament.connection_limits.max_total_connections,
        "Tournament should support more connections than LAN"
    );
    assert!(
        tournament.connection_limits.max_total_connections
            < professional.connection_limits.max_total_connections,
        "Professional should support most connections"
    );
}

#[test]
fn test_bind_address_configurations() {
    let config = NetworkConfig::default();

    // Test bind address is valid IP
    assert!(
        config.bind_address.is_ipv4() || config.bind_address.is_ipv6(),
        "Bind address should be valid IPv4 or IPv6"
    );

    // Test production bind address
    assert!(
        config.production_bind_address.is_ipv4() || config.production_bind_address.is_ipv6(),
        "Production bind address should be valid IP"
    );
}

#[test]
fn test_local_bind_address() {
    let config = NetworkConfig::default();
    let result = config.local_bind_address();

    assert!(result.is_ok(), "Should be able to create local bind address");

    if let Ok(addr) = result {
        assert!(addr.port() > 0, "Port should be valid");
    }
}

#[test]
fn test_port_range_all_ports() {
    let range = PortRange {
        start: 8000,
        end: 8005,
    };
    let ports = range.all_ports();

    assert!(!ports.is_empty(), "Should return collection of all ports in range");
    assert_eq!(ports.len(), 6, "Should include all 6 ports in range (8000-8005)");
    assert!(ports.contains(&8000), "Should include start port");
    assert!(ports.contains(&8005), "Should include end port");
}

#[test]
fn test_random_port_generation() {
    let range = PortRange {
        start: 8000,
        end: 9000,
    };
    let port = range.random_port();

    assert!(
        port >= range.start && port <= range.end,
        "Random port should be within configured range"
    );
}

#[test]
fn test_next_gaming_port_allocation() {
    let config = NetworkConfig::default();
    let exclude = vec![6112, 6113, 6114];
    let result = config.next_gaming_port(&exclude);

    assert!(result.is_ok(), "Should be able to allocate next available gaming port");

    if let Ok(port) = result {
        assert!(!exclude.contains(&port), "Allocated port should not be in exclusion list");
        assert!(
            port >= config.gaming_port_range.start && port <= config.gaming_port_range.end,
            "Allocated port should be within gaming port range"
        );
    }
}

#[test]
fn test_federation_endpoints_validation() {
    let mut config = NetworkConfig::default();
    config.federation_endpoints =
        vec!["http://node1:8080".to_string(), "http://node2:8080".to_string()];

    assert_eq!(config.federation_endpoints.len(), 2, "Should have configured federation endpoints");
    assert!(
        !config.federation_endpoints.is_empty(),
        "Federation endpoints should not be empty after configuration"
    );
}

#[test]
fn test_stun_servers_configuration() {
    let mut config = NetworkConfig::default();
    config.stun_servers = vec![
        "stun:stun.l.google.com:19302".to_string(),
        "stun:stun1.l.google.com:19302".to_string(),
    ];

    assert_eq!(config.stun_servers.len(), 2, "Should have configured STUN servers");
}

#[test]
fn test_allowed_networks_configuration() {
    let mut config = NetworkConfig::default();
    config.allowed_networks = vec!["192.168.1.0/24".to_string(), "10.0.0.0/8".to_string()];

    assert_eq!(config.allowed_networks.len(), 2, "Should have configured allowed networks");
    assert!(
        !config.allowed_networks.is_empty(),
        "Allowed networks should not be empty after configuration"
    );
}

#[test]
fn test_connection_timeout_configuration() {
    let config = NetworkConfig::default();

    assert!(config.connection_timeout.as_secs() > 0, "Connection timeout should be positive");
    assert!(
        config.connection_timeout.as_secs() < 300,
        "Connection timeout should be reasonable (< 5 minutes)"
    );
}

#[test]
fn test_request_timeout_configuration() {
    let config = NetworkConfig::default();

    assert!(config.request_timeout.as_secs() > 0, "Request timeout should be positive");
    assert!(
        config.request_timeout >= config.timeouts.connection,
        "Request timeout should be >= connection timeout"
    );
}

#[test]
fn test_worker_threads_configuration() {
    let config = NetworkConfig::default();

    assert!(config.worker_threads > 0, "Should have at least one worker thread");
    assert!(config.worker_threads <= 1024, "Worker threads should be reasonable (<= 1024)");
}

#[test]
fn test_max_bandwidth_configuration() {
    let config = NetworkConfig::default();

    assert!(config.max_bandwidth_mbps > 0, "Max bandwidth should be positive");
}

#[test]
fn test_discovery_ports_configuration() {
    let mut config = NetworkConfig::default();
    config.discovery_ports = vec![8081, 8082, 8083];

    assert_eq!(config.discovery_ports.len(), 3, "Should have configured discovery ports");
    assert!(config.discovery_ports.iter().all(|&p| p > 0), "All discovery ports should be valid");
}
