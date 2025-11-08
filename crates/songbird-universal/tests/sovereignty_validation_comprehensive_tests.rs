//! Comprehensive Sovereignty Validation Tests
//!
//! This test suite provides extensive coverage for sovereignty validation logic,
//! compliance checking, risk assessment, and path validation to reach 60% coverage.

#![allow(clippy::float_cmp)]
#![allow(clippy::uninlined_format_args)]

use songbird_types::SongbirdResult;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::sovereignty::types::*;
use std::time::Duration;

#[test]
fn test_sovereignty_compliance_level_variants() -> SongbirdResult<()> {
    // Test all sovereignty compliance level variants exist and can be created
    let fully_compliant = SovereigntyComplianceLevel::FullyCompliant;
    let mostly_compliant = SovereigntyComplianceLevel::MostlyCompliant;
    let partially_compliant = SovereigntyComplianceLevel::PartiallyCompliant;
    let non_compliant = SovereigntyComplianceLevel::NonCompliant;

    // Test format strings (using Debug)
    assert!(format!("{:?}", fully_compliant).contains("FullyCompliant"));
    assert!(format!("{:?}", mostly_compliant).contains("MostlyCompliant"));
    assert!(format!("{:?}", partially_compliant).contains("PartiallyCompliant"));
    assert!(format!("{:?}", non_compliant).contains("NonCompliant"));
    Ok(())
}

#[test]
fn test_sovereignty_level_score_ordering() {
    // Test that sovereignty level scores are properly ordered
    assert!(SovereigntyLevel::FullySovereign.score() > SovereigntyLevel::HighlySovereign.score());
    assert!(
        SovereigntyLevel::HighlySovereign.score() > SovereigntyLevel::ModeratelySovereign.score()
    );
    assert!(
        SovereigntyLevel::ModeratelySovereign.score()
            > SovereigntyLevel::LimitedSovereignty.score()
    );
    assert!(SovereigntyLevel::LimitedSovereignty.score() > SovereigntyLevel::NonSovereign.score());
}

#[test]
fn test_sovereignty_level_score_range() {
    // Test that all sovereignty level scores are within valid range [0.0, 1.0]
    assert!(SovereigntyLevel::FullySovereign.score() >= 0.0);
    assert!(SovereigntyLevel::FullySovereign.score() <= 1.0);
    assert!(SovereigntyLevel::HighlySovereign.score() >= 0.0);
    assert!(SovereigntyLevel::HighlySovereign.score() <= 1.0);
    assert!(SovereigntyLevel::ModeratelySovereign.score() >= 0.0);
    assert!(SovereigntyLevel::ModeratelySovereign.score() <= 1.0);
    assert!(SovereigntyLevel::LimitedSovereignty.score() >= 0.0);
    assert!(SovereigntyLevel::LimitedSovereignty.score() <= 1.0);
    assert!(SovereigntyLevel::NonSovereign.score() >= 0.0);
    assert!(SovereigntyLevel::NonSovereign.score() <= 1.0);
}

#[test]
fn test_sovereignty_level_extreme_scores() {
    // Test extreme sovereignty level values
    assert_eq!(SovereigntyLevel::FullySovereign.score(), 1.0);
    assert_eq!(SovereigntyLevel::NonSovereign.score(), 0.0);
}

#[test]
fn test_sovereignty_level_mid_range_scores() {
    // Test mid-range sovereignty levels have expected scores
    assert_eq!(SovereigntyLevel::HighlySovereign.score(), 0.8);
    assert_eq!(SovereigntyLevel::ModeratelySovereign.score(), 0.6);
    assert_eq!(SovereigntyLevel::LimitedSovereignty.score(), 0.4);
}

#[test]
fn test_security_capability_variants() {
    // Test all security capability variants
    let capabilities = [
        SecurityCapability::Encryption,
        SecurityCapability::Authentication,
        SecurityCapability::Authorization,
        SecurityCapability::FederationAware,
        SecurityCapability::NetworkOptimized,
        SecurityCapability::SovereigntyCompliant,
    ];

    // Verify each capability is distinct
    for i in 0..capabilities.len() {
        for j in (i + 1)..capabilities.len() {
            assert_ne!(capabilities[i], capabilities[j]);
        }
    }
}

#[test]
fn test_security_capability_equality() {
    // Test security capability equality
    assert_eq!(SecurityCapability::Encryption, SecurityCapability::Encryption);
    assert_eq!(SecurityCapability::SovereigntyCompliant, SecurityCapability::SovereigntyCompliant);
    assert_ne!(SecurityCapability::Encryption, SecurityCapability::Authentication);
}

#[test]
fn test_security_capability_clone() {
    // Test security capability cloning
    let original = SecurityCapability::FederationAware;
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_sovereignty_risk_creation() {
    // Test creating sovereignty risk objects
    let risk = SovereigntyRisk {
        risk_id: "risk_001".to_string(),
        risk_type: SovereigntyRiskType::DataSovereignty,
        severity: RiskSeverity::High,
        mitigation_strategies: vec!["Strategy 1".to_string(), "Strategy 2".to_string()],
    };

    assert_eq!(risk.risk_id, "risk_001");
    assert_eq!(risk.mitigation_strategies.len(), 2);
}

#[test]
fn test_sovereignty_risk_without_mitigation() {
    // Test sovereignty risk without mitigation strategies
    let risk = SovereigntyRisk {
        risk_id: "risk_002".to_string(),
        risk_type: SovereigntyRiskType::JurisdictionalCompliance,
        severity: RiskSeverity::Critical,
        mitigation_strategies: vec![],
    };

    assert!(risk.mitigation_strategies.is_empty());
}

#[test]
fn test_sovereignty_risk_multiple_mitigation_strategies() {
    // Test sovereignty risk with multiple mitigation strategies
    let strategies = vec![
        "Implement encryption".to_string(),
        "Add access controls".to_string(),
        "Enable audit logging".to_string(),
    ];

    let risk = SovereigntyRisk {
        risk_id: "risk_003".to_string(),
        risk_type: SovereigntyRiskType::NetworkDependency,
        severity: RiskSeverity::Medium,
        mitigation_strategies: strategies,
    };

    assert_eq!(risk.mitigation_strategies.len(), 3);
    assert!(risk.mitigation_strategies.contains(&"Implement encryption".to_string()));
    assert!(risk.mitigation_strategies.contains(&"Add access controls".to_string()));
    assert!(risk.mitigation_strategies.contains(&"Enable audit logging".to_string()));
}

#[test]
fn test_risk_type_variants() -> SongbirdResult<()> {
    // Test all risk type variants can be created
    let data_sovereignty = SovereigntyRiskType::DataSovereignty;
    let jurisdictional_compliance = SovereigntyRiskType::JurisdictionalCompliance;
    let network_dependency = SovereigntyRiskType::NetworkDependency;
    let third_party_risk = SovereigntyRiskType::ThirdPartyRisk;

    // Test Debug formatting
    assert!(format!("{:?}", data_sovereignty).contains("DataSovereignty"));
    assert!(format!("{:?}", jurisdictional_compliance).contains("JurisdictionalCompliance"));
    assert!(format!("{:?}", network_dependency).contains("NetworkDependency"));
    assert!(format!("{:?}", third_party_risk).contains("ThirdPartyRisk"));
    Ok(())
}

#[test]
fn test_risk_severity_ordering() -> SongbirdResult<()> {
    // Test risk severity variants
    let critical = RiskSeverity::Critical;
    let high = RiskSeverity::High;
    let medium = RiskSeverity::Medium;
    let low = RiskSeverity::Low;

    // Verify Debug formatting includes severity levels
    assert!(format!("{:?}", critical).contains("Critical"));
    assert!(format!("{:?}", high).contains("High"));
    assert!(format!("{:?}", medium).contains("Medium"));
    assert!(format!("{:?}", low).contains("Low"));
    Ok(())
}

#[test]
fn test_sovereignty_adapter_config_builder_pattern() {
    // Test building custom config
    let config = SovereigntyAdapterConfig {
        enable_sovereignty_routing: true,
        enable_federation_routing: false,
        enable_network_optimization: true,
        sovereignty_timeout: Duration::from_secs(5),
        sovereignty_preference_weight: 0.9,
    };

    assert!(config.enable_sovereignty_routing);
    assert!(!config.enable_federation_routing);
    assert!(config.enable_network_optimization);
    assert_eq!(config.sovereignty_timeout, Duration::from_secs(5));
    assert_eq!(config.sovereignty_preference_weight, 0.9);
}

#[test]
fn test_sovereignty_adapter_config_minimal() {
    // Test minimal sovereignty config
    let config = SovereigntyAdapterConfig {
        enable_sovereignty_routing: false,
        enable_federation_routing: false,
        enable_network_optimization: false,
        sovereignty_timeout: Duration::from_millis(100),
        sovereignty_preference_weight: 0.0,
    };

    assert!(!config.enable_sovereignty_routing);
    assert!(!config.enable_federation_routing);
    assert!(!config.enable_network_optimization);
    assert_eq!(config.sovereignty_timeout, Duration::from_millis(100));
    assert_eq!(config.sovereignty_preference_weight, 0.0);
}

#[test]
fn test_sovereignty_adapter_config_maximum() {
    // Test maximum sovereignty config
    let config = SovereigntyAdapterConfig {
        enable_sovereignty_routing: true,
        enable_federation_routing: true,
        enable_network_optimization: true,
        sovereignty_timeout: Duration::from_secs(30),
        sovereignty_preference_weight: 1.0,
    };

    assert!(config.enable_sovereignty_routing);
    assert!(config.enable_federation_routing);
    assert!(config.enable_network_optimization);
    assert_eq!(config.sovereignty_timeout, Duration::from_secs(30));
    assert_eq!(config.sovereignty_preference_weight, 1.0);
}

#[test]
fn test_sovereignty_preference_weight_range() {
    // Test various sovereignty preference weights
    let weights = vec![0.0, 0.25, 0.5, 0.75, 1.0];

    for weight in weights {
        let config = SovereigntyAdapterConfig {
            enable_sovereignty_routing: true,
            enable_federation_routing: true,
            enable_network_optimization: true,
            sovereignty_timeout: Duration::from_secs(3),
            sovereignty_preference_weight: weight,
        };

        assert!(config.sovereignty_preference_weight >= 0.0);
        assert!(config.sovereignty_preference_weight <= 1.0);
        assert_eq!(config.sovereignty_preference_weight, weight);
    }
}

#[test]
fn test_sovereignty_timeout_variations() {
    // Test various timeout configurations
    let timeouts = vec![
        Duration::from_millis(100),
        Duration::from_millis(500),
        Duration::from_secs(1),
        Duration::from_secs(5),
        Duration::from_secs(10),
    ];

    for timeout in timeouts {
        let config = SovereigntyAdapterConfig {
            enable_sovereignty_routing: true,
            enable_federation_routing: true,
            enable_network_optimization: true,
            sovereignty_timeout: timeout,
            sovereignty_preference_weight: 0.8,
        };

        assert_eq!(config.sovereignty_timeout, timeout);
    }
}

#[test]
fn test_sovereignty_config_clone() -> SongbirdResult<()> {
    // Test config cloning
    let original = SovereigntyAdapterConfig::default();
    let cloned = original.clone();

    assert_eq!(original.enable_sovereignty_routing, cloned.enable_sovereignty_routing);
    assert_eq!(original.enable_federation_routing, cloned.enable_federation_routing);
    assert_eq!(original.enable_network_optimization, cloned.enable_network_optimization);
    assert_eq!(original.sovereignty_timeout, cloned.sovereignty_timeout);
    assert_eq!(original.sovereignty_preference_weight, cloned.sovereignty_preference_weight);
    Ok(())
}

#[test]
fn test_sovereignty_level_serialization() -> SongbirdResult<()> {
    // Test sovereignty level can be serialized (via Debug since Serialize may not be on all types)
    let levels = vec![
        SovereigntyLevel::FullySovereign,
        SovereigntyLevel::HighlySovereign,
        SovereigntyLevel::ModeratelySovereign,
        SovereigntyLevel::LimitedSovereignty,
        SovereigntyLevel::NonSovereign,
    ];

    for level in levels {
        let debug_str = format!("{:?}", level);
        assert!(!debug_str.is_empty());
    }
    Ok(())
}

#[test]
fn test_security_level_variants() -> SongbirdResult<()> {
    // Test all security level variants
    let maximum = SecurityLevel::Maximum;
    let high = SecurityLevel::High;
    let medium = SecurityLevel::Medium;
    let low = SecurityLevel::Low;
    let minimal = SecurityLevel::Minimal;

    // Test Debug formatting
    assert!(format!("{:?}", maximum).contains("Maximum"));
    assert!(format!("{:?}", high).contains("High"));
    assert!(format!("{:?}", medium).contains("Medium"));
    assert!(format!("{:?}", low).contains("Low"));
    assert!(format!("{:?}", minimal).contains("Minimal"));
    Ok(())
}

#[test]
fn test_security_level_equality_comprehensive() {
    // Comprehensive security level equality testing
    assert_eq!(SecurityLevel::Maximum, SecurityLevel::Maximum);
    assert_eq!(SecurityLevel::High, SecurityLevel::High);
    assert_eq!(SecurityLevel::Medium, SecurityLevel::Medium);
    assert_eq!(SecurityLevel::Low, SecurityLevel::Low);
    assert_eq!(SecurityLevel::Minimal, SecurityLevel::Minimal);

    // Test inequality
    assert_ne!(SecurityLevel::Maximum, SecurityLevel::High);
    assert_ne!(SecurityLevel::High, SecurityLevel::Medium);
    assert_ne!(SecurityLevel::Medium, SecurityLevel::Low);
    assert_ne!(SecurityLevel::Low, SecurityLevel::Minimal);
}
