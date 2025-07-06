//! Basic Tests for Firewall Module

use songbird_lib::firewall::*;
use tokio::test;

/// Test FirewallConfig creation and defaults
#[test]
async fn test_firewall_config_creation() {
    let config = FirewallConfig {
        enabled: true,
        allowed_ports: vec![8080, 8081, 9090],
        backend: FirewallBackend {
            backend_type: "iptables".to_string(),
        },
        security: SecurityConfig {
            security_level: "high".to_string(),
        },
        songbird_rules: SongbirdRules {
            lan_only: true,
            federation_port: 8081,
            metrics_port: 9090,
            discovery_enabled: true,
        },
        optional_rules: OptionalRules {
            ssh_enabled: false,
            ssh_port: 22,
            web_ui_enabled: true,
        },
        logging: LoggingConfig {
            enabled: true,
        },
        orchestrator_port: 8080,
        web_ui_port: 8888,
        allow_local_access: true,
        block_external_access: false,
    };
    
    assert!(config.enabled);
    assert_eq!(config.allowed_ports.len(), 3);
    assert_eq!(config.backend.backend_type, "iptables");
    assert_eq!(config.security.security_level, "high");
    assert!(config.songbird_rules.lan_only);
    assert!(!config.optional_rules.ssh_enabled);
    assert!(config.logging.enabled);
}

/// Test FirewallBackend creation
#[test]
async fn test_firewall_backend_creation() {
    let backend = FirewallBackend {
        backend_type: "ufw".to_string(),
    };
    
    assert_eq!(backend.backend_type, "ufw");
}

/// Test SecurityConfig creation
#[test]
async fn test_security_config_creation() {
    let security = SecurityConfig {
        security_level: "medium".to_string(),
    };
    
    assert_eq!(security.security_level, "medium");
}

/// Test SongbirdRules creation
#[test]
async fn test_songbird_rules_creation() {
    let rules = SongbirdRules {
        lan_only: false,
        federation_port: 8081,
        metrics_port: 9090,
        discovery_enabled: true,
    };
    
    assert!(!rules.lan_only);
    assert_eq!(rules.federation_port, 8081);
    assert_eq!(rules.metrics_port, 9090);
    assert!(rules.discovery_enabled);
}

/// Test OptionalRules creation
#[test]
async fn test_optional_rules_creation() {
    let rules = OptionalRules {
        ssh_enabled: true,
        ssh_port: 2222,
        web_ui_enabled: false,
    };
    
    assert!(rules.ssh_enabled);
    assert_eq!(rules.ssh_port, 2222);
    assert!(!rules.web_ui_enabled);
}

/// Test LoggingConfig creation
#[test]
async fn test_logging_config_creation() {
    let logging = LoggingConfig {
        enabled: false,
    };
    
    assert!(!logging.enabled);
}

/// Test FirewallConfig cloning
#[test]
async fn test_firewall_config_cloning() {
    let config = FirewallConfig {
        enabled: true,
        allowed_ports: vec![8080, 8081],
        backend: FirewallBackend {
            backend_type: "iptables".to_string(),
        },
        security: SecurityConfig {
            security_level: "high".to_string(),
        },
        songbird_rules: SongbirdRules {
            lan_only: true,
            federation_port: 8081,
            metrics_port: 9090,
            discovery_enabled: true,
        },
        optional_rules: OptionalRules {
            ssh_enabled: false,
            ssh_port: 22,
            web_ui_enabled: true,
        },
        logging: LoggingConfig {
            enabled: true,
        },
        orchestrator_port: 8080,
        web_ui_port: 8888,
        allow_local_access: true,
        block_external_access: false,
    };
    
    let cloned_config = config.clone();
    
    assert_eq!(config.enabled, cloned_config.enabled);
    assert_eq!(config.allowed_ports.len(), cloned_config.allowed_ports.len());
    assert_eq!(config.backend.backend_type, cloned_config.backend.backend_type);
    assert_eq!(config.security.security_level, cloned_config.security.security_level);
}

/// Test FirewallConfig serialization
#[test]
async fn test_firewall_config_serialization() {
    let config = FirewallConfig {
        enabled: true,
        allowed_ports: vec![8080, 8081],
        backend: FirewallBackend {
            backend_type: "iptables".to_string(),
        },
        security: SecurityConfig {
            security_level: "high".to_string(),
        },
        songbird_rules: SongbirdRules {
            lan_only: true,
            federation_port: 8081,
            metrics_port: 9090,
            discovery_enabled: true,
        },
        optional_rules: OptionalRules {
            ssh_enabled: false,
            ssh_port: 22,
            web_ui_enabled: true,
        },
        logging: LoggingConfig {
            enabled: true,
        },
        orchestrator_port: 8080,
        web_ui_port: 8888,
        allow_local_access: true,
        block_external_access: false,
    };
    
    // Test serialization to JSON
    let json = serde_json::to_string(&config).expect("Should serialize to JSON");
    assert!(json.contains("enabled"));
    assert!(json.contains("allowed_ports"));
    assert!(json.contains("backend"));
    
    // Test deserialization from JSON
    let deserialized: FirewallConfig = serde_json::from_str(&json)
        .expect("Should deserialize from JSON");
    
    assert_eq!(deserialized.enabled, config.enabled);
    assert_eq!(deserialized.allowed_ports.len(), config.allowed_ports.len());
    assert_eq!(deserialized.backend.backend_type, config.backend.backend_type);
}

/// Test FirewallConfig with different backends
#[test]
async fn test_firewall_config_backends() {
    let backends = vec![
        "iptables",
        "ufw",
        "firewalld",
        "pf",
        "netfilter",
    ];
    
    for backend_type in backends {
        let config = FirewallConfig {
            enabled: true,
            allowed_ports: vec![8080],
            backend: FirewallBackend {
                backend_type: backend_type.to_string(),
            },
            security: SecurityConfig {
                security_level: "medium".to_string(),
            },
            songbird_rules: SongbirdRules {
                lan_only: true,
                federation_port: 8081,
                metrics_port: 9090,
                discovery_enabled: true,
            },
            optional_rules: OptionalRules {
                ssh_enabled: false,
                ssh_port: 22,
                web_ui_enabled: true,
            },
            logging: LoggingConfig {
                enabled: true,
            },
            orchestrator_port: 8080,
            web_ui_port: 8888,
            allow_local_access: true,
            block_external_access: false,
        };
        
        assert_eq!(config.backend.backend_type, backend_type);
        assert!(config.enabled);
    }
}

/// Test FirewallConfig with different security levels
#[test]
async fn test_firewall_config_security_levels() {
    let security_levels = vec![
        "low",
        "medium",
        "high",
        "maximum",
    ];
    
    for security_level in security_levels {
        let config = FirewallConfig {
            enabled: true,
            allowed_ports: vec![8080],
            backend: FirewallBackend {
                backend_type: "iptables".to_string(),
            },
            security: SecurityConfig {
                security_level: security_level.to_string(),
            },
            songbird_rules: SongbirdRules {
                lan_only: true,
                federation_port: 8081,
                metrics_port: 9090,
                discovery_enabled: true,
            },
            optional_rules: OptionalRules {
                ssh_enabled: false,
                ssh_port: 22,
                web_ui_enabled: true,
            },
            logging: LoggingConfig {
                enabled: true,
            },
            orchestrator_port: 8080,
            web_ui_port: 8888,
            allow_local_access: true,
            block_external_access: false,
        };
        
        assert_eq!(config.security.security_level, security_level);
        assert!(config.enabled);
    }
} 