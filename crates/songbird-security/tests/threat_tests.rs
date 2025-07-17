#[macro_use]
extern crate songbird_security;

// Threat Detection Tests for Songbird Security
//
// This module contains comprehensive tests for threat detection capabilities
// including malware detection, phishing protection, and behavioral analysis.

use std::time::SystemTime;

mod test_framework;
use test_framework::*;

/// Test basic threat detection capabilities
#[tokio::test]
async fn test_basic_threat_detection() {
    let framework = init_security_test!("Basic Threat Detection");

    // Test various threat types
    let threat_tests = vec![
        (ThreatType::Malware, 0.9, true),
        (ThreatType::Phishing, 0.8, true),
        (ThreatType::UnauthorizedAccess, 0.7, true),
        (ThreatType::NetworkIntrusion, 0.6, true),
        (ThreatType::DataExfiltration, 0.5, true),
    ];

    for (threat_type, confidence, should_detect) in threat_tests {
        let threat_scenario = TestThreatScenario {
            scenario_id: format!("basic_threat_{:?}", threat_type),
            threat_type: threat_type.clone(),
            severity: ThreatSeverity::High,
            source: ThreatSource::External,
            target: "test_system".to_string(),
            description: format!("Basic threat detection test for {:?}", threat_type),
            indicators: vec![ThreatIndicator {
                indicator_type: "signature".to_string(),
                value: "test_signature".to_string(),
                confidence,
                timestamp: SystemTime::now(),
            }],
            expected_response: ThreatResponse::Block,
            confidence,
        };

        let detected = framework.run_threat_detection_test(threat_scenario).await;
        if should_detect {
            assert!(detected, "Should detect {:?} threat", threat_type);
        } else {
            assert!(!detected, "Should not detect {:?} threat", threat_type);
        }
    }

    println!("✅ All basic threat detection tests passed");
}

/// Test threat severity levels
#[tokio::test]
async fn test_threat_severity_levels() {
    let framework = init_security_test!("Threat Severity Levels");

    // Test different severity levels
    let severity_tests = vec![
        (ThreatSeverity::Critical, 0.9, true),
        (ThreatSeverity::High, 0.8, true),
        (ThreatSeverity::Medium, 0.6, true),
        (ThreatSeverity::Low, 0.4, false),
        (ThreatSeverity::Informational, 0.2, false),
    ];

    for (severity, confidence, should_detect) in severity_tests {
        let threat_scenario = TestThreatScenario {
            scenario_id: format!("severity_test_{:?}", severity),
            threat_type: ThreatType::Malware,
            severity: severity.clone(),
            source: ThreatSource::External,
            target: "test_system".to_string(),
            description: format!("Severity level test for {:?}", severity),
            indicators: vec![ThreatIndicator {
                indicator_type: "severity_test".to_string(),
                value: "test_indicator".to_string(),
                confidence,
                timestamp: SystemTime::now(),
            }],
            expected_response: ThreatResponse::Block,
            confidence,
        };

        let detected = framework.run_threat_detection_test(threat_scenario).await;
        if should_detect {
            assert!(detected, "Should detect {:?} severity threat", severity);
        } else {
            assert!(
                !detected,
                "Should not detect {:?} severity threat",
                severity
            );
        }
    }

    println!("✅ All threat severity level tests passed");
}

/// Test threat source analysis
#[tokio::test]
async fn test_threat_source_analysis() {
    let framework = init_security_test!("Threat Source Analysis");

    // Test different threat sources
    let source_tests = vec![
        (ThreatSource::External, 0.8, true),
        (ThreatSource::Internal, 0.7, true),
        (ThreatSource::Unknown, 0.6, true),
    ];

    for (source, confidence, should_detect) in source_tests {
        let threat_scenario = TestThreatScenario {
            scenario_id: format!("source_test_{:?}", source),
            threat_type: ThreatType::UnauthorizedAccess,
            severity: ThreatSeverity::High,
            source: source.clone(),
            target: "test_system".to_string(),
            description: format!("Source analysis test for {:?}", source),
            indicators: vec![ThreatIndicator {
                indicator_type: "source_test".to_string(),
                value: "test_indicator".to_string(),
                confidence,
                timestamp: SystemTime::now(),
            }],
            expected_response: ThreatResponse::Block,
            confidence,
        };

        let detected = framework.run_threat_detection_test(threat_scenario).await;
        if should_detect {
            assert!(detected, "Should detect {:?} source threat", source);
        } else {
            assert!(!detected, "Should not detect {:?} source threat", source);
        }
    }

    println!("✅ All threat source analysis tests passed");
}

/// Test threat confidence thresholds
#[tokio::test]
async fn test_threat_confidence_thresholds() {
    let framework = init_security_test!("Threat Confidence Thresholds");

    // Test different confidence levels
    let confidence_tests = vec![
        (0.9, true),  // High confidence
        (0.8, true),  // Medium-high confidence
        (0.6, true),  // Medium confidence
        (0.4, false), // Low confidence
        (0.2, false), // Very low confidence
    ];

    for (confidence, should_detect) in confidence_tests {
        let threat_scenario = TestThreatScenario {
            scenario_id: format!("confidence_test_{}", confidence),
            threat_type: ThreatType::Malware,
            severity: ThreatSeverity::High,
            source: ThreatSource::External,
            target: "test_system".to_string(),
            description: format!("Confidence threshold test for {}", confidence),
            indicators: vec![ThreatIndicator {
                indicator_type: "confidence_test".to_string(),
                value: "test_indicator".to_string(),
                confidence,
                timestamp: SystemTime::now(),
            }],
            expected_response: ThreatResponse::Block,
            confidence,
        };

        let detected = framework.run_threat_detection_test(threat_scenario).await;
        if should_detect {
            assert!(
                detected,
                "Should detect threat with confidence {}",
                confidence
            );
        } else {
            assert!(
                !detected,
                "Should not detect threat with confidence {}",
                confidence
            );
        }
    }

    println!("✅ All threat confidence threshold tests passed");
}

/// Test comprehensive threat detection workflow
#[tokio::test]
async fn test_comprehensive_threat_detection() {
    let framework = init_security_test!("Comprehensive Threat Detection");

    // Test a complex threat scenario
    let complex_threat = TestThreatScenario {
        scenario_id: "complex_threat_001".to_string(),
        threat_type: ThreatType::NetworkIntrusion,
        severity: ThreatSeverity::Critical,
        source: ThreatSource::External,
        target: "production_system".to_string(),
        description: "Complex network intrusion with multiple indicators".to_string(),
        indicators: vec![
            ThreatIndicator {
                indicator_type: "network_signature".to_string(),
                value: "malicious_pattern_123".to_string(),
                confidence: 0.95,
                timestamp: SystemTime::now(),
            },
            ThreatIndicator {
                indicator_type: "behavioral_anomaly".to_string(),
                value: "unusual_access_pattern".to_string(),
                confidence: 0.85,
                timestamp: SystemTime::now(),
            },
            ThreatIndicator {
                indicator_type: "geolocation".to_string(),
                value: "suspicious_location".to_string(),
                confidence: 0.75,
                timestamp: SystemTime::now(),
            },
        ],
        expected_response: ThreatResponse::Block,
        confidence: 0.95,
    };

    let detected = framework.run_threat_detection_test(complex_threat).await;
    assert!(
        detected,
        "Should detect complex threat with multiple indicators"
    );

    println!("✅ Comprehensive threat detection test passed");
}
