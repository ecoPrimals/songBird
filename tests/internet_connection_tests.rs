use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
#[allow(dead_code, unused_imports, unused_variables)]
// Internet Connection Wizard Tests
//
// Comprehensive test suite for the Internet Connection Wizard functionality.
use std::time::Duration;

use songbird_gaming_bridge::config::environment::{EnvironmentAware, EnvironmentConfig};
use songbird_gaming_bridge::config::OrchestratorConfig;
use songbird_gaming_bridge::errors::SongbirdError;
use songbird_gaming_bridge::internet_connection::{
    AuthenticationMethod, ConfigurationDiscoveryConfig, ConnectionType, ContactInfo, ContactMethod,
    DiscoveryMethod, EncryptionProtocol, FamilyNetworkConfig, InternetConnectionConfig,
    InternetConnectionWizard, InternetMonitoringConfig, InternetSecurityConfig, NetworkConstraints,
    PortDiscoveryMode, SongbirdPortConfiguration, TechnicalLevel, TunnelConfig, TunnelTechnology,
};

/// Test Internet Connection Config creation and defaults
#[tokio::test]
async fn test_internet_connection_config_default() {
    let config = InternetConnectionConfig::default();

    // Verify default values
    assert!(config.discovery.enabled);
    assert_eq!(
        config.discovery.discovery_method,
        DiscoveryMethod::SongbirdApi
    );
    assert_eq!(config.tunnel.technology, TunnelTechnology::WireGuard);
    assert_eq!(config.tunnel.interface_name, "songbird0");
    assert_eq!(
        config.security.encryption.protocol,
        EncryptionProtocol::WireGuard
    );
    assert_eq!(
        config.security.authentication.method,
        AuthenticationMethod::Certificate
    );
    assert_eq!(config.family.network_name, "songbird-family-hpc");
    assert_eq!(config.family.connection_type, ConnectionType::Permanent);
    assert!(config.monitoring.tunnel_health_monitoring);
}

/// Test environment-aware configuration
#[tokio::test]
async fn test_internet_connection_config_environment_aware() {
    // Test family-network environment
    let mut env_config = EnvironmentConfig::default();
    env_config.custom_mappings.insert(
        "SONGBIRD_ENVIRONMENT_TYPE".to_string(),
        "family-network".to_string(),
    );

    let config = InternetConnectionConfig::from_env_with_config(&env_config);

    assert!(config.family.simplified_setup);
    assert!(!config.family.auto_discovery);
    assert!(config.security.authentication.mutual_auth);
    assert_eq!(config.tunnel.technology, TunnelTechnology::WireGuard);
}

/// Test student-mobile environment configuration
#[tokio::test]
async fn test_internet_connection_config_student_mobile_environment() {
    let mut env_config = EnvironmentConfig::default();
    env_config.custom_mappings.insert(
        "SONGBIRD_ENVIRONMENT_TYPE".to_string(),
        "student-mobile".to_string(),
    );

    let config = InternetConnectionConfig::from_env_with_config(&env_config);

    assert_eq!(config.tunnel.technology, TunnelTechnology::Tailscale);
    assert!(config.tunnel.bandwidth_optimization);
    assert_eq!(config.family.connection_type, ConnectionType::Temporary);
    assert!(config.monitoring.bandwidth_monitoring);
}

/// Test remote-office environment configuration
#[tokio::test]
async fn test_internet_connection_config_remote_office_environment() {
    let mut env_config = EnvironmentConfig::default();
    env_config.custom_mappings.insert(
        "SONGBIRD_ENVIRONMENT_TYPE".to_string(),
        "remote-office".to_string(),
    );

    let config = InternetConnectionConfig::from_env_with_config(&env_config);

    assert_eq!(
        config.security.encryption.key_rotation,
        Duration::from_secs(12 * 3600)
    );
    assert_eq!(config.tunnel.technology, TunnelTechnology::WireGuard);
    assert!(config.monitoring.security_monitoring);
}

/// Test specific environment variable overrides
#[tokio::test]
async fn test_internet_connection_config_env_var_overrides() {
    let mut env_config = EnvironmentConfig::default();
    env_config.custom_mappings.insert(
        "SONGBIRD_TUNNEL_TECHNOLOGY".to_string(),
        "tailscale".to_string(),
    );
    env_config.custom_mappings.insert(
        "SONGBIRD_FAMILY_NETWORK_NAME".to_string(),
        "custom-family-network".to_string(),
    );
    env_config.custom_mappings.insert(
        "SONGBIRD_TUNNEL_INTERFACE".to_string(),
        "custom-interface".to_string(),
    );

    let config = InternetConnectionConfig::from_env_with_config(&env_config);

    assert_eq!(config.tunnel.technology, TunnelTechnology::Tailscale);
    assert_eq!(config.family.network_name, "custom-family-network");
    assert_eq!(config.tunnel.interface_name, "custom-interface");
}

/// Test SongbirdPortConfiguration functionality
#[tokio::test]
async fn test_songbird_port_configuration() {
    let config = SongbirdPortConfiguration::default();

    // Test default ports
    assert_eq!(config.orchestrator_port, 8080);
    assert_eq!(config.federation_port, 8081);
    assert_eq!(config.metrics_port, 9090);
    assert_eq!(config.discovery_port, 5353);

    // Test get_all_required_ports
    let ports = config.get_all_required_ports();
    assert!(ports.contains(&8080));
    assert!(ports.contains(&8081));
    assert!(ports.contains(&9090));
    assert!(ports.contains(&5353));

    // Test is_songbird_port
    assert!(config.is_songbird_port(8080));
    assert!(config.is_songbird_port(8081));
    assert!(!config.is_songbird_port(3000));
}

/// Test SongbirdPortConfiguration with additional ports
#[tokio::test]
async fn test_songbird_port_configuration_with_additional_ports() {
    let mut config = SongbirdPortConfiguration::default();
    config.additional_service_ports = vec![8082, 8083];

    let ports = config.get_all_required_ports();
    assert!(ports.contains(&8082));
    assert!(ports.contains(&8083));
    assert!(config.is_songbird_port(8082));
    assert!(config.is_songbird_port(8083));
}

/// Test Internet Connection Wizard creation
#[tokio::test]
async fn test_internet_connection_wizard_creation() {
    let config = InternetConnectionConfig::default();
    let wizard = InternetConnectionWizard::new(config.clone());

    // Wizard should be created successfully
    // Note: We can't directly access private fields, but we can test the public interface
    let _ = wizard;
}

/// Test Internet Connection Wizard port discovery
#[tokio::test]
async fn test_internet_connection_wizard_port_discovery() {
    let config = InternetConnectionConfig::default();
    let wizard = InternetConnectionWizard::new(config);

    // Test port discovery - should fall back to defaults since no real services are running
    let discovered_ports = wizard.discover_songbird_ports().await;
    assert!(discovered_ports.is_ok());

    let ports = discovered_ports.expect("Test assertion failed");
    assert_eq!(ports.orchestrator_port, 8080); // Default fallback
    assert_eq!(ports.federation_port, 8081);
    assert_eq!(ports.metrics_port, 9090);
    assert_eq!(ports.discovery_port, 5353);
}

/// Test Internet Connection Wizard start wizard functionality
#[tokio::test]
async fn test_internet_connection_wizard_start_wizard() {
    let config = InternetConnectionConfig::default();
    let mut wizard = InternetConnectionWizard::new(config);

    // Test that start_wizard runs without error
    // This is primarily a simulation since we don't have real services
    let result = wizard.start_wizard().await;
    assert!(result.is_ok());
}

/// Test discovery method functionality
#[tokio::test]
async fn test_discovery_methods() {
    let methods = vec![
        DiscoveryMethod::SongbirdApi,
        DiscoveryMethod::ConfigFile,
        DiscoveryMethod::Environment,
        DiscoveryMethod::NetworkScan,
        DiscoveryMethod::Multicast,
        DiscoveryMethod::DnsServiceDiscovery,
    ];

    // Test serialization/deserialization
    for method in methods {
        let serialized = serde_json::to_string(&method).expect("Test assertion failed");
        let deserialized: DiscoveryMethod =
            serde_json::from_str(&serialized).expect("Test assertion failed");
        assert_eq!(method, deserialized);
    }
}

/// Test tunnel technology enum
#[tokio::test]
async fn test_tunnel_technology() {
    let technologies = vec![
        TunnelTechnology::WireGuard,
        TunnelTechnology::Tailscale,
        TunnelTechnology::OpenVPN,
        TunnelTechnology::ZeroTier,
    ];

    // Test serialization/deserialization
    for tech in technologies {
        let serialized = serde_json::to_string(&tech).expect("Test assertion failed");
        let deserialized: TunnelTechnology =
            serde_json::from_str(&serialized).expect("Test assertion failed");
        assert_eq!(tech, deserialized);
    }
}

/// Test configuration discovery config
#[tokio::test]
async fn test_configuration_discovery_config() {
    let config = ConfigurationDiscoveryConfig::default();

    assert!(config.mode == crate::federation::FederationMode::Peer);
    assert_eq!(config.discovery_method, DiscoveryMethod::SongbirdApi);
    assert!(config
        .fallback_methods
        .contains(&DiscoveryMethod::ConfigFile));
    assert!(config
        .fallback_methods
        .contains(&DiscoveryMethod::Environment));
    assert!(config
        .fallback_methods
        .contains(&DiscoveryMethod::NetworkScan));
    assert_eq!(config.refresh_interval, Duration::from_secs(3600));
}

/// Test tunnel configuration
#[tokio::test]
async fn test_tunnel_config() {
    let config = TunnelConfig::default();

    assert_eq!(config.technology, TunnelTechnology::WireGuard);
    assert_eq!(config.interface_name, "songbird0");
    assert!(config.bandwidth_optimization);
    assert!(config.auto_reconnect);
    assert!(config.kill_switch);
    assert_eq!(config.mtu, 1420);
    assert_eq!(config.keepalive_interval, Some(Duration::from_secs(25)));
}

/// Test security configuration
#[tokio::test]
async fn test_security_config() {
    let config = InternetSecurityConfig::default();

    assert!(config.encryption.enabled);
    assert_eq!(config.encryption.protocol, EncryptionProtocol::WireGuard);
    assert_eq!(
        config.encryption.key_rotation,
        Duration::from_secs(24 * 3600)
    );
    assert!(config.encryption.perfect_forward_secrecy);
    assert_eq!(config.encryption.cipher_suite, "ChaCha20Poly1305");

    assert_eq!(
        config.authentication.method,
        AuthenticationMethod::Certificate
    );
    assert!(config.authentication.mutual_auth);
    assert!(config.authentication.certificate_pinning);
    assert!(!config.authentication.auto_accept);

    assert!(config.network_policy.songbird_only);
    assert_eq!(
        config.network_policy.port_discovery,
        PortDiscoveryMode::Dynamic
    );
    assert!(config.network_policy.traffic_analysis_protection);
    assert!(config.network_policy.kill_switch);
}

/// Test family network configuration
#[tokio::test]
async fn test_family_network_config() {
    let config = FamilyNetworkConfig::default();

    assert_eq!(config.network_name, "songbird-family-hpc");
    assert!(config.simplified_setup);
    assert!(!config.auto_discovery);
    assert_eq!(config.connection_type, ConnectionType::Permanent);
    assert!(!config.trust_on_first_use);
}

/// Test network constraints
#[tokio::test]
async fn test_network_constraints() {
    let constraints = NetworkConstraints::default();

    assert!(!constraints.bandwidth_limited);
    assert!(constraints.behind_nat);
    assert!(constraints.dynamic_ip);
    assert!(constraints.isp_restrictions.is_empty());
}

/// Test contact info
#[tokio::test]
async fn test_contact_info() {
    let contact = ContactInfo::default();

    assert_eq!(contact.preferred_method, ContactMethod::Email);
    assert!(contact.email.is_none());
    assert!(contact.phone.is_none());
}

/// Test monitoring configuration
#[tokio::test]
async fn test_monitoring_config() {
    let config = InternetMonitoringConfig::default();

    assert!(config.tunnel_health_monitoring);
    assert!(config.bandwidth_monitoring);
    assert!(config.security_monitoring);
    assert!(config.family_status_reporting);
    assert_eq!(config.monitoring_interval, Duration::from_secs(60));
}

/// Test configuration serialization and deserialization
#[tokio::test]
async fn test_config_serialization() {
    let config = InternetConnectionConfig::default();

    // Test TOML serialization
    let toml_str = toml::to_string(&config).expect("Test assertion failed");
    let deserialized_config: InternetConnectionConfig =
        toml::from_str(&toml_str).expect("Test assertion failed");

    // Verify key fields are preserved
    assert_eq!(
        config.tunnel.technology,
        deserialized_config.tunnel.technology
    );
    assert_eq!(
        config.family.network_name,
        deserialized_config.family.network_name
    );
    assert_eq!(
        config.security.encryption.enabled,
        deserialized_config.security.encryption.enabled
    );
}

/// Test error handling in wizard
#[tokio::test]
async fn test_wizard_error_handling() {
    let config = InternetConnectionConfig::default();
    let wizard = InternetConnectionWizard::new(config);

    // Test that error conditions are handled gracefully
    // This tests the fallback behavior when no real services are available
    let result = wizard.discover_songbird_ports().await;
    assert!(
        result.is_ok(),
        "Port discovery should fall back to defaults"
    );
}

/// Test configuration validation
#[tokio::test]
async fn test_configuration_validation() {
    let mut config = InternetConnectionConfig::default();

    // Test invalid tunnel interface name
    config.tunnel.interface_name = "".to_string();

    // Test that empty interface name is still valid (would be handled by implementation)
    let serialized = serde_json::to_string(&config).expect("Test assertion failed");
    assert!(serialized.contains("\"interface_name\":\"\""));
}

/// Test port configuration edge cases
#[tokio::test]
async fn test_port_configuration_edge_cases() {
    let mut config = SongbirdPortConfiguration::default();

    // Test with duplicate ports in additional_service_ports
    config.additional_service_ports = vec![8080, 8080, 8081];

    let ports = config.get_all_required_ports();

    // Should deduplicate ports
    let port_count_8080 = ports.iter().filter(|&&p| p == 8080).count();
    let port_count_8081 = ports.iter().filter(|&&p| p == 8081).count();

    assert_eq!(port_count_8080, 1);
    assert_eq!(port_count_8081, 1);
}

/// Test family member configuration
#[tokio::test]
async fn test_family_member_configuration() {
    use songbird_gaming_bridge::internet_connection::FamilyMember;

    let member = FamilyMember {
        name: "Alice".to_string(),
        location: "Home Office".to_string(),
        technical_level: TechnicalLevel::Intermediate,
        available_hardware: vec!["Laptop".to_string(), "Raspberry Pi".to_string()],
        network_constraints: NetworkConstraints::default(),
        contact_info: ContactInfo::default(),
    };

    assert_eq!(member.name, "Alice");
    assert_eq!(member.location, "Home Office");
    assert_eq!(member.technical_level, TechnicalLevel::Intermediate);
    assert_eq!(member.available_hardware.len(), 2);
}

/// Test connection type variations
#[tokio::test]
async fn test_connection_types() {
    let types = vec![
        ConnectionType::Permanent,
        ConnectionType::Temporary,
        ConnectionType::Scheduled,
    ];

    for conn_type in types {
        let serialized = serde_json::to_string(&conn_type).expect("Test assertion failed");
        let deserialized: ConnectionType =
            serde_json::from_str(&serialized).expect("Test assertion failed");
        assert_eq!(conn_type, deserialized);
    }
}

/// Test technical level enum
#[tokio::test]
async fn test_technical_levels() {
    let levels = vec![
        TechnicalLevel::Beginner,
        TechnicalLevel::Intermediate,
        TechnicalLevel::Advanced,
    ];

    for level in levels {
        let serialized = serde_json::to_string(&level).expect("Test assertion failed");
        let deserialized: TechnicalLevel =
            serde_json::from_str(&serialized).expect("Test assertion failed");
        assert_eq!(level, deserialized);
    }
}

/// Test authentication method enum
#[tokio::test]
async fn test_authentication_methods() {
    let methods = vec![
        AuthenticationMethod::Certificate,
        AuthenticationMethod::PreSharedKey,
        AuthenticationMethod::OAuth,
    ];

    for method in methods {
        let serialized = serde_json::to_string(&method).expect("Test assertion failed");
        let deserialized: AuthenticationMethod =
            serde_json::from_str(&serialized).expect("Test assertion failed");
        assert_eq!(method, deserialized);
    }
}

/// Test port discovery mode enum
#[tokio::test]
async fn test_port_discovery_modes() {
    let modes = vec![
        PortDiscoveryMode::Dynamic,
        PortDiscoveryMode::Static,
        PortDiscoveryMode::Hybrid,
    ];

    for mode in modes {
        let serialized = serde_json::to_string(&mode).expect("Test assertion failed");
        let deserialized: PortDiscoveryMode =
            serde_json::from_str(&serialized).expect("Test assertion failed");
        assert_eq!(mode, deserialized);
    }
}

/// Test contact method enum
#[tokio::test]
async fn test_contact_methods() {
    let methods = vec![
        ContactMethod::Email,
        ContactMethod::Phone,
        ContactMethod::Chat,
    ];

    for method in methods {
        let serialized = serde_json::to_string(&method).expect("Test assertion failed");
        let deserialized: ContactMethod =
            serde_json::from_str(&serialized).expect("Test assertion failed");
        assert_eq!(method, deserialized);
    }
}

/// Test complex configuration scenario
#[tokio::test]
async fn test_complex_configuration_scenario() {
    let mut config = InternetConnectionConfig::default();

    // Customize configuration for a complex family network scenario
    config.family.network_name = "smith-family-research-network".to_string();
    config.family.simplified_setup = false;
    config.tunnel.technology = TunnelTechnology::WireGuard;
    config.security.encryption.key_rotation = Duration::from_secs(12 * 3600); // 12 hours
    config.monitoring.monitoring_interval = Duration::from_secs(30); // 30 seconds

    // Add custom discovery endpoints
    config.discovery.discovery_endpoints = vec![
        "http://192.168.1.100:8080".to_string(),
        "http://10.0.0.1:8080".to_string(),
    ];

    // Verify configuration
    assert_eq!(config.family.network_name, "smith-family-research-network");
    assert!(!config.family.simplified_setup);
    assert_eq!(config.tunnel.technology, TunnelTechnology::WireGuard);
    assert_eq!(
        config.security.encryption.key_rotation,
        Duration::from_secs(12 * 3600)
    );
    assert_eq!(
        config.monitoring.monitoring_interval,
        Duration::from_secs(30)
    );
    assert_eq!(config.discovery.discovery_endpoints.len(), 2);
}

/// Test wizard integration with environment variables
#[tokio::test]
async fn test_wizard_environment_integration() {
    // Set up environment configuration
    let mut env_config = EnvironmentConfig::default();
    env_config.custom_mappings.insert(
        "SONGBIRD_ENVIRONMENT_TYPE".to_string(),
        "family-network".to_string(),
    );
    env_config.custom_mappings.insert(
        "SONGBIRD_TUNNEL_TECHNOLOGY".to_string(),
        "wireguard".to_string(),
    );

    let config = InternetConnectionConfig::from_env_with_config(&env_config);
    let wizard = InternetConnectionWizard::new(config);

    // Test that wizard can be created with environment configuration
    let result = wizard.discover_songbird_ports().await;
    assert!(result.is_ok());
}
