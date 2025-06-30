use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
#[allow(dead_code, unused_imports, unused_variables)]
// Firewall Configuration Wizard Tests
//
// Comprehensive test suite for the Firewall Configuration Wizard functionality.
use std::path::PathBuf;

use songbird_gaming_bridge::config::environment::{EnvironmentAware, EnvironmentConfig};
use songbird_gaming_bridge::firewall::{
    AddressSpec, BackendConfig, BackendType, ConnectivityReport, ConnectivityTester, DefaultPolicy,
    Direction, FirewallConfig, FirewallRule, FirewallWizard, LogLevel, LoggingConfig,
    OptionalRuleConfig, PortRange, Protocol, RuleAction, SecurityConfig, SecurityLevel,
    SecurityValidator, SongbirdRuleConfig, SystemInfo, ValidationResult,
};

/// Test Firewall Config creation and defaults
#[tokio::test]
async fn test_firewall_config_default() {
    let config = FirewallConfig::default();

    // Verify default values
    assert_eq!(config.backend.backend_type, BackendType::AutoDetect);
    assert!(config.backend.auto_detect);
    assert_eq!(config.default_policy.inbound, RuleAction::Deny);
    assert_eq!(config.default_policy.outbound, RuleAction::Allow);
    assert_eq!(config.songbird_rules.orchestrator_port, 8080);
    assert_eq!(config.songbird_rules.federation_port, 8765);
    assert_eq!(config.songbird_rules.metrics_port, 9090);
    assert!(config.songbird_rules.discovery_enabled);
    assert!(config.songbird_rules.lan_only);
    assert!(!config.optional_rules.ssh_enabled);
    assert!(!config.optional_rules.web_ui_enabled);
    assert_eq!(config.security.security_level, SecurityLevel::High);
    assert_eq!(config.security.min_security_score, 80);
    assert!(config.logging.enabled);
    assert_eq!(config.logging.level, LogLevel::Info);
}

/// Test environment-aware configuration - home-hpc
#[tokio::test]
async fn test_firewall_config_home_hpc_environment() {
    let mut env_config = EnvironmentConfig::default();
    env_config.custom_mappings.insert(
        "SONGBIRD_ENVIRONMENT_TYPE".to_string(),
        "home-hpc".to_string(),
    );

    let config = FirewallConfig::from_env_with_config(&env_config);

    assert_eq!(config.default_policy.inbound, RuleAction::Deny);
    assert_eq!(config.default_policy.outbound, RuleAction::Allow);
    assert_eq!(config.security.security_level, SecurityLevel::High);
    assert!(config.songbird_rules.lan_only);
    assert!(!config.logging.log_denied); // Reduce noise in home environment
}

/// Test development environment configuration
#[tokio::test]
async fn test_firewall_config_development_environment() {
    let mut env_config = EnvironmentConfig::default();
    env_config.custom_mappings.insert(
        "SONGBIRD_ENVIRONMENT_TYPE".to_string(),
        "development".to_string(),
    );

    let config = FirewallConfig::from_env_with_config(&env_config);

    assert_eq!(config.security.security_level, SecurityLevel::Medium);
    assert!(config.songbird_rules.lan_only);
    assert!(config.logging.log_denied); // More verbose for development
    assert_eq!(config.logging.level, LogLevel::Debug);
}

/// Test production environment configuration
#[tokio::test]
async fn test_firewall_config_production_environment() {
    let mut env_config = EnvironmentConfig::default();
    env_config.custom_mappings.insert(
        "SONGBIRD_ENVIRONMENT_TYPE".to_string(),
        "production".to_string(),
    );

    let config = FirewallConfig::from_env_with_config(&env_config);

    assert_eq!(config.security.security_level, SecurityLevel::Maximum);
    assert!(config.songbird_rules.lan_only);
    assert!(config.security.intrusion_detection);
}

/// Test environment variable overrides
#[tokio::test]
async fn test_firewall_config_env_var_overrides() {
    let mut env_config = EnvironmentConfig::default();
    env_config
        .custom_mappings
        .insert("SONGBIRD_FIREWALL_BACKEND".to_string(), "ufw".to_string());
    env_config
        .custom_mappings
        .insert("SONGBIRD_SECURITY_LEVEL".to_string(), "maximum".to_string());
    env_config
        .custom_mappings
        .insert("SONGBIRD_ORCHESTRATOR_PORT".to_string(), "8888".to_string());
    env_config
        .custom_mappings
        .insert("SONGBIRD_FEDERATION_PORT".to_string(), "9999".to_string());

    let config = FirewallConfig::from_env_with_config(&env_config);

    assert_eq!(config.backend.backend_type, BackendType::Ufw);
    assert_eq!(config.security.security_level, SecurityLevel::Maximum);
    assert_eq!(config.songbird_rules.orchestrator_port, 8888);
    assert_eq!(config.songbird_rules.federation_port, 9999);
}

/// Test Firewall Wizard creation
#[tokio::test]
async fn test_firewall_wizard_creation() {
    let config = FirewallConfig::default();
    let wizard = FirewallWizard::new(config.clone());

    // Wizard should be created successfully
    // Note: We can't directly access private fields, but we can test the public interface
    let _ = wizard;
}

/// Test Firewall Wizard rule generation
#[tokio::test]
async fn test_firewall_wizard_rule_generation() {
    let config = FirewallConfig::default();
    let wizard = FirewallWizard::new(config);

    let rules = wizard
        .generate_songbird_rules()
        .expect("Test assertion failed");

    // Should have at least 4 core rules
    assert!(rules.len() >= 4);

    // Check for core Songbird rules
    let orchestrator_rule = rules.iter().find(|r| r.name == "Songbird-Orchestrator-API");
    assert!(orchestrator_rule.is_some());
    let orchestrator_rule = orchestrator_rule.expect("Test assertion failed");
    assert_eq!(orchestrator_rule.action, RuleAction::Allow);
    assert_eq!(orchestrator_rule.direction, Direction::Inbound);
    assert_eq!(orchestrator_rule.protocol, Protocol::Tcp);
    assert_eq!(orchestrator_rule.port_range.start, 8080);
    assert_eq!(orchestrator_rule.source, AddressSpec::PrivateNetworks);

    let federation_rule = rules.iter().find(|r| r.name == "Songbird-Federation");
    assert!(federation_rule.is_some());
    let federation_rule = federation_rule.expect("Test assertion failed");
    assert_eq!(federation_rule.action, RuleAction::Allow);
    assert_eq!(federation_rule.direction, Direction::Both);
    assert_eq!(federation_rule.protocol, Protocol::Tcp);
    assert_eq!(federation_rule.port_range.start, 8765);

    let metrics_rule = rules.iter().find(|r| r.name == "Songbird-Metrics");
    assert!(metrics_rule.is_some());
    let metrics_rule = metrics_rule.expect("Test assertion failed");
    assert_eq!(metrics_rule.action, RuleAction::Allow);
    assert_eq!(metrics_rule.direction, Direction::Inbound);
    assert_eq!(metrics_rule.protocol, Protocol::Tcp);
    assert_eq!(metrics_rule.port_range.start, 9090);
    assert_eq!(metrics_rule.source, AddressSpec::Localhost);
}

/// Test optional rules generation
#[tokio::test]
async fn test_firewall_wizard_optional_rules() {
    let mut config = FirewallConfig::default();
    config.optional_rules.ssh_enabled = true;
    config.optional_rules.web_ui_enabled = true;

    let wizard = FirewallWizard::new(config);
    let rules = wizard
        .generate_songbird_rules()
        .expect("Test assertion failed");

    // Should include SSH and Web UI rules
    let ssh_rule = rules.iter().find(|r| r.name == "SSH-Access");
    assert!(ssh_rule.is_some());
    let ssh_rule = ssh_rule.expect("Test assertion failed");
    assert_eq!(ssh_rule.port_range.start, 22);

    let web_ui_rule = rules.iter().find(|r| r.name == "Web-UI");
    assert!(web_ui_rule.is_some());
    let web_ui_rule = web_ui_rule.expect("Test assertion failed");
    assert_eq!(web_ui_rule.port_range.start, 3000);
}

/// Test System Information detection
#[tokio::test]
async fn test_system_info_detection() {
    let system_info = SystemInfo::detect().await.expect("Test assertion failed");

    // Should detect the current OS
    assert!(!system_info.os_type.is_empty());
    assert!(!system_info.available_backends.is_empty());

    // Should always have Manual backend available
    assert!(system_info
        .available_backends
        .contains(&BackendType::Manual));

    // OS-specific backend checks
    match system_info.os_type.as_str() {
        "linux" => {
            // Linux should have at least Manual backend
            assert!(system_info
                .available_backends
                .contains(&BackendType::Manual));
        }
        "windows" => {
            assert!(system_info
                .available_backends
                .contains(&BackendType::WindowsDefender));
        }
        "macos" => {
            assert!(system_info.available_backends.contains(&BackendType::Pfctl));
        }
        _ => {
            // Unknown OS should at least have Manual
            assert!(system_info
                .available_backends
                .contains(&BackendType::Manual));
        }
    }
}

/// Test Security Validator
#[tokio::test]
async fn test_security_validator() {
    let validator = SecurityValidator::new();

    // Test secure rules (LAN-only)
    let secure_rules = vec![FirewallRule {
        name: "Songbird-Orchestrator-API".to_string(),
        action: RuleAction::Allow,
        direction: Direction::Inbound,
        protocol: Protocol::Tcp,
        port_range: PortRange::single(8080),
        source: AddressSpec::PrivateNetworks,
        destination: AddressSpec::Any,
        priority: 100,
        enabled: true,
    }];

    let result = validator.validate_rules(&secure_rules);
    assert!(result.passed);
    assert!(result.critical_issues.is_empty());
    assert_eq!(result.score, 100); // Perfect score for secure rules

    // Test insecure rules (internet-exposed)
    let insecure_rules = vec![FirewallRule {
        name: "Songbird-Orchestrator-API".to_string(),
        action: RuleAction::Allow,
        direction: Direction::Inbound,
        protocol: Protocol::Tcp,
        port_range: PortRange::single(8080),
        source: AddressSpec::Any, // This should trigger a critical issue
        destination: AddressSpec::Any,
        priority: 100,
        enabled: true,
    }];

    let result = validator.validate_rules(&insecure_rules);
    assert!(!result.passed);
    assert!(!result.critical_issues.is_empty());
    assert!(result.score < 100);

    // Check that the critical issue is about internet exposure
    assert!(result.critical_issues[0].contains("exposes Songbird port"));
    assert!(result.critical_issues[0].contains("internet"));
}

/// Test Connectivity Tester
#[tokio::test]
async fn test_connectivity_tester() {
    let tester = ConnectivityTester::new();
    let config = FirewallConfig::default();

    // Test connectivity (will likely fail since no services are running)
    let result = tester
        .test_songbird_connectivity(&config)
        .await
        .expect("Test assertion failed");

    // Should have attempted to test both orchestrator and metrics ports
    assert!(result.failures.len() >= 2 || result.successful_tests.len() >= 2);

    // Check that it tested the right ports
    let has_orchestrator_test = result.failures.iter().any(|f| f.contains("8080"))
        || result.successful_tests.iter().any(|s| s.contains("8080"));
    let has_metrics_test = result.failures.iter().any(|f| f.contains("9090"))
        || result.successful_tests.iter().any(|s| s.contains("9090"));

    assert!(has_orchestrator_test);
    assert!(has_metrics_test);
}

/// Test Backend Type enumeration
#[tokio::test]
async fn test_backend_types() {
    let backends = vec![
        BackendType::AutoDetect,
        BackendType::Ufw,
        BackendType::Iptables,
        BackendType::WindowsDefender,
        BackendType::Pfctl,
        BackendType::FreeBsdPf,
        BackendType::Manual,
    ];

    // Test serialization/deserialization
    for backend in backends {
        let serialized = serde_json::to_string(&backend).expect("Test assertion failed");
        let deserialized: BackendType =
            serde_json::from_str(&serialized).expect("Test assertion failed");
        assert_eq!(backend, deserialized);
    }
}

/// Test Security Levels
#[tokio::test]
async fn test_security_levels() {
    let levels = vec![
        SecurityLevel::Low,
        SecurityLevel::Medium,
        SecurityLevel::High,
        SecurityLevel::Maximum,
    ];

    // Test serialization/deserialization
    for level in levels {
        let serialized = serde_json::to_string(&level).expect("Test assertion failed");
        let deserialized: SecurityLevel =
            serde_json::from_str(&serialized).expect("Test assertion failed");
        assert_eq!(level, deserialized);
    }
}

/// Test Rule Actions
#[tokio::test]
async fn test_rule_actions() {
    let actions = vec![
        RuleAction::Allow,
        RuleAction::Deny,
        RuleAction::Reject,
        RuleAction::Log,
    ];

    // Test serialization/deserialization
    for action in actions {
        let serialized = serde_json::to_string(&action).expect("Test assertion failed");
        let deserialized: RuleAction =
            serde_json::from_str(&serialized).expect("Test assertion failed");
        assert_eq!(action, deserialized);
    }
}

/// Test Traffic Direction
#[tokio::test]
async fn test_directions() {
    let directions = vec![Direction::Inbound, Direction::Outbound, Direction::Both];

    // Test serialization/deserialization
    for direction in directions {
        let serialized = serde_json::to_string(&direction).expect("Test assertion failed");
        let deserialized: Direction =
            serde_json::from_str(&serialized).expect("Test assertion failed");
        assert_eq!(direction, deserialized);
    }
}

/// Test Protocols
#[tokio::test]
async fn test_protocols() {
    let protocols = vec![Protocol::Tcp, Protocol::Udp, Protocol::Icmp, Protocol::Any];

    // Test serialization/deserialization
    for protocol in protocols {
        let serialized = serde_json::to_string(&protocol).expect("Test assertion failed");
        let deserialized: Protocol =
            serde_json::from_str(&serialized).expect("Test assertion failed");
        assert_eq!(protocol, deserialized);
    }
}

/// Test Port Range
#[tokio::test]
async fn test_port_range() {
    let single_port = PortRange::single(8080);
    assert_eq!(single_port.start, 8080);
    assert_eq!(single_port.end, 8080);

    let port_range = PortRange::range(8000, 8999);
    assert_eq!(port_range.start, 8000);
    assert_eq!(port_range.end, 8999);

    // Test serialization/deserialization
    let serialized = serde_json::to_string(&single_port).expect("Test assertion failed");
    let deserialized: PortRange = serde_json::from_str(&serialized).expect("Test assertion failed");
    assert_eq!(single_port, deserialized);
}

/// Test Address Specifications
#[tokio::test]
async fn test_address_specs() {
    let specs = vec![
        AddressSpec::Any,
        AddressSpec::Localhost,
        AddressSpec::PrivateNetworks,
        AddressSpec::Specific("192.168.1.100".to_string()),
        AddressSpec::Multicast("239.1.1.1".to_string()),
    ];

    // Test serialization/deserialization
    for spec in specs {
        let serialized = serde_json::to_string(&spec).expect("Test assertion failed");
        let deserialized: AddressSpec =
            serde_json::from_str(&serialized).expect("Test assertion failed");
        assert_eq!(spec, deserialized);
    }
}

/// Test Log Levels
#[tokio::test]
async fn test_log_levels() {
    let levels = vec![
        LogLevel::Error,
        LogLevel::Warn,
        LogLevel::Info,
        LogLevel::Debug,
        LogLevel::Trace,
    ];

    // Test serialization/deserialization
    for level in levels {
        let serialized = serde_json::to_string(&level).expect("Test assertion failed");
        let deserialized: LogLevel =
            serde_json::from_str(&serialized).expect("Test assertion failed");
        assert_eq!(level, deserialized);
    }
}

/// Test configuration serialization and deserialization
#[tokio::test]
async fn test_config_serialization() {
    let config = FirewallConfig::default();

    // Test TOML serialization
    let toml_str = toml::to_string(&config).expect("Test assertion failed");
    let deserialized_config: FirewallConfig =
        toml::from_str(&toml_str).expect("Test assertion failed");

    // Verify key fields are preserved
    assert_eq!(
        config.backend.backend_type,
        deserialized_config.backend.backend_type
    );
    assert_eq!(
        config.security.security_level,
        deserialized_config.security.security_level
    );
    assert_eq!(
        config.songbird_rules.orchestrator_port,
        deserialized_config.songbird_rules.orchestrator_port
    );
    assert_eq!(
        config.songbird_rules.lan_only,
        deserialized_config.songbird_rules.lan_only
    );
}

/// Test complex firewall configuration scenario
#[tokio::test]
async fn test_complex_firewall_scenario() {
    let mut config = FirewallConfig::default();

    // Customize configuration for a complex enterprise scenario
    config.backend.backend_type = BackendType::Ufw;
    config.security.security_level = SecurityLevel::Maximum;
    config.security.intrusion_detection = true;
    config.songbird_rules.orchestrator_port = 8888;
    config.songbird_rules.federation_port = 9999;
    config.optional_rules.ssh_enabled = true;
    config.optional_rules.ssh_port = 2222;
    config.logging.level = LogLevel::Debug;
    config.logging.log_denied = true;

    let wizard = FirewallWizard::new(config.clone());
    let rules = wizard
        .generate_songbird_rules()
        .expect("Test assertion failed");

    // Verify configuration
    assert_eq!(config.backend.backend_type, BackendType::Ufw);
    assert_eq!(config.security.security_level, SecurityLevel::Maximum);
    assert!(config.security.intrusion_detection);
    assert_eq!(config.songbird_rules.orchestrator_port, 8888);
    assert_eq!(config.songbird_rules.federation_port, 9999);
    assert!(config.optional_rules.ssh_enabled);
    assert_eq!(config.optional_rules.ssh_port, 2222);

    // Verify rules include custom ports and SSH
    let orchestrator_rule = rules
        .iter()
        .find(|r| r.name == "Songbird-Orchestrator-API")
        .expect("Test assertion failed");
    assert_eq!(orchestrator_rule.port_range.start, 8888);

    let federation_rule = rules
        .iter()
        .find(|r| r.name == "Songbird-Federation")
        .expect("Test assertion failed");
    assert_eq!(federation_rule.port_range.start, 9999);

    let ssh_rule = rules
        .iter()
        .find(|r| r.name == "SSH-Access")
        .expect("Test assertion failed");
    assert_eq!(ssh_rule.port_range.start, 2222);
}

/// Test firewall rule validation edge cases
#[tokio::test]
async fn test_firewall_validation_edge_cases() {
    let validator = SecurityValidator::new();

    // Test empty rules
    let empty_rules = vec![];
    let result = validator.validate_rules(&empty_rules);
    assert!(result.passed);
    assert_eq!(result.score, 100);

    // Test non-Songbird port with Any source (should only warn, not fail)
    let non_songbird_rules = vec![FirewallRule {
        name: "Custom-Service".to_string(),
        action: RuleAction::Allow,
        direction: Direction::Inbound,
        protocol: Protocol::Tcp,
        port_range: PortRange::single(3000),
        source: AddressSpec::Any,
        destination: AddressSpec::Any,
        priority: 100,
        enabled: true,
    }];

    let result = validator.validate_rules(&non_songbird_rules);
    assert!(result.passed); // Should pass since it's not a Songbird port
    assert!(!result.warnings.is_empty()); // But should have warnings
    assert!(result.score < 100); // Score should be reduced
}

/// Test backend auto-detection logic
#[tokio::test]
async fn test_backend_auto_detection() {
    let system_info = SystemInfo::detect().await.expect("Test assertion failed");

    // Test that auto-detection would work based on current OS
    match system_info.os_type.as_str() {
        "linux" => {
            // Linux should prefer UFW if available, then iptables, then manual
            if system_info.available_backends.contains(&BackendType::Ufw) {
                // UFW is preferred
                assert!(true);
            } else if system_info
                .available_backends
                .contains(&BackendType::Iptables)
            {
                // iptables is fallback
                assert!(true);
            } else {
                // Manual is final fallback
                assert!(system_info
                    .available_backends
                    .contains(&BackendType::Manual));
            }
        }
        "windows" => {
            assert!(system_info
                .available_backends
                .contains(&BackendType::WindowsDefender));
        }
        "macos" => {
            assert!(system_info.available_backends.contains(&BackendType::Pfctl));
        }
        _ => {
            // Unknown OS should at least have Manual
            assert!(system_info
                .available_backends
                .contains(&BackendType::Manual));
        }
    }
}

/// Test firewall configuration with disabled discovery
#[tokio::test]
async fn test_firewall_config_disabled_discovery() {
    let mut config = FirewallConfig::default();
    config.songbird_rules.discovery_enabled = false;

    let wizard = FirewallWizard::new(config);
    let rules = wizard
        .generate_songbird_rules()
        .expect("Test assertion failed");

    // Should not have discovery rule when disabled
    let discovery_rule = rules.iter().find(|r| r.name == "Songbird-Discovery");
    assert!(discovery_rule.is_none());
}

/// Test firewall configuration with non-LAN mode
#[tokio::test]
async fn test_firewall_config_non_lan_mode() {
    let mut config = FirewallConfig::default();
    config.songbird_rules.lan_only = false;

    let wizard = FirewallWizard::new(config);
    let rules = wizard
        .generate_songbird_rules()
        .expect("Test assertion failed");

    // Should allow from Any when not LAN-only (but this should trigger security warnings)
    let orchestrator_rule = rules
        .iter()
        .find(|r| r.name == "Songbird-Orchestrator-API")
        .expect("Test assertion failed");
    assert_eq!(orchestrator_rule.source, AddressSpec::Any);

    // Validate that this triggers security issues
    let validator = SecurityValidator::new();
    let result = validator.validate_rules(&rules);
    assert!(!result.passed); // Should fail security validation
    assert!(!result.critical_issues.is_empty());
}

/// Test environment configuration inheritance
#[tokio::test]
async fn test_environment_configuration_inheritance() {
    // Test that unknown environment types fall back to defaults
    let mut env_config = EnvironmentConfig::default();
    env_config.custom_mappings.insert(
        "SONGBIRD_ENVIRONMENT_TYPE".to_string(),
        "unknown-environment".to_string(),
    );

    let config = FirewallConfig::from_env_with_config(&env_config);

    // Should use default values for unknown environment
    assert_eq!(config.default_policy.inbound, RuleAction::Deny);
    assert_eq!(config.default_policy.outbound, RuleAction::Allow);
    assert_eq!(config.security.security_level, SecurityLevel::High);
    assert!(config.songbird_rules.lan_only);
}
