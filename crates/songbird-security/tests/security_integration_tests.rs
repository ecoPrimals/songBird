// Security Integration Tests for Songbird Security
//
// This module contains comprehensive tests for security integration
// including security workflows, performance benchmarks, and error handling.

use std::time::SystemTime;

mod test_framework;
use test_framework::*;

/// Test basic security integration
#[tokio::test]
async fn test_security_integration() {
    let framework = init_security_test!("Security Integration");

    // Test threat detection integration
    let threat_scenario = TestThreatScenario {
        scenario_id: "security_threat_001".to_string(),
        threat_type: ThreatType::Malware,
        severity: ThreatSeverity::High,
        source: ThreatSource::External,
        target: "test_system".to_string(),
        description: "Security threat detection test".to_string(),
        indicators: vec![ThreatIndicator {
            indicator_type: "file_signature".to_string(),
            value: "malware_signature_xyz".to_string(),
            confidence: 0.9,
            timestamp: SystemTime::now(),
        }],
        expected_response: ThreatResponse::Block,
        confidence: 0.8,
    };

    let threat_detected = framework.run_threat_detection_test(threat_scenario).await;
    assert!(threat_detected, "Security system should detect threats");

    // Test zero trust integration
    let zero_trust_test = test_framework::test_utils::create_test_zero_trust_case(0.8, true);
    let zero_trust_result = framework.run_zero_trust_test(zero_trust_test).await;
    assert!(zero_trust_result, "Zero trust evaluation should pass");

    // Test security audit
    let audit_result = framework.run_security_audit_test("compliance").await;
    assert!(audit_result, "Security audit should pass");

    println!("✅ All security integration tests passed");
}

/// Test threat detection with various scenarios
#[tokio::test]
async fn test_threat_detection_scenarios() {
    let framework = init_security_test!("Threat Detection Scenarios");

    // Test malware detection
    let malware_scenario =
        test_framework::test_utils::create_test_threat_scenario(ThreatType::Malware);
    let malware_detected = framework.run_threat_detection_test(malware_scenario).await;
    assert!(malware_detected, "Malware should be detected");

    // Test phishing detection
    let phishing_scenario =
        test_framework::test_utils::create_test_threat_scenario(ThreatType::Phishing);
    let phishing_detected = framework.run_threat_detection_test(phishing_scenario).await;
    assert!(phishing_detected, "Phishing should be detected");

    // Test unauthorized access detection
    let unauthorized_scenario =
        test_framework::test_utils::create_test_threat_scenario(ThreatType::UnauthorizedAccess);
    let unauthorized_detected = framework
        .run_threat_detection_test(unauthorized_scenario)
        .await;
    assert!(
        unauthorized_detected,
        "Unauthorized access should be detected"
    );

    println!("✅ All threat detection scenarios passed");
}

/// Test zero trust scenarios
#[tokio::test]
async fn test_zero_trust_scenarios() {
    let framework = init_security_test!("Zero Trust Scenarios");

    // Test high trust device - should allow access
    let high_trust_test = test_framework::test_utils::create_test_zero_trust_case(0.9, true);
    let high_trust_result = framework.run_zero_trust_test(high_trust_test).await;
    assert!(
        high_trust_result,
        "High trust device should be allowed access"
    );

    // Test low trust device - should deny access
    let low_trust_test = test_framework::test_utils::create_test_zero_trust_case(0.3, false);
    let low_trust_result = framework.run_zero_trust_test(low_trust_test).await;
    assert!(low_trust_result, "Low trust device should be denied access");

    // Test medium trust device - should allow access
    let medium_trust_test = test_framework::test_utils::create_test_zero_trust_case(0.7, true);
    let medium_trust_result = framework.run_zero_trust_test(medium_trust_test).await;
    assert!(
        medium_trust_result,
        "Medium trust device should be allowed access"
    );

    println!("✅ All zero trust scenarios passed");
}

/// Test security audit functionality
#[tokio::test]
async fn test_security_audits() {
    let framework = init_security_test!("Security Audits");

    // Test compliance audit
    let compliance_result = framework.run_security_audit_test("compliance").await;
    assert!(compliance_result, "Compliance audit should pass");

    // Test vulnerability scan
    let vulnerability_result = framework.run_security_audit_test("vulnerability").await;
    assert!(vulnerability_result, "Vulnerability scan should pass");

    println!("✅ All security audit tests passed");
}

/// Test comprehensive security workflow
#[tokio::test]
async fn test_comprehensive_security_workflow() {
    let framework = init_security_test!("Comprehensive Security Workflow");

    // Step 1: Threat detection
    let threat_scenario = TestThreatScenario {
        scenario_id: "workflow_threat_001".to_string(),
        threat_type: ThreatType::NetworkIntrusion,
        severity: ThreatSeverity::Critical,
        source: ThreatSource::External,
        target: "production_system".to_string(),
        description: "Network intrusion attempt".to_string(),
        indicators: vec![ThreatIndicator {
            indicator_type: "network_signature".to_string(),
            value: "intrusion_pattern_123".to_string(),
            confidence: 0.95,
            timestamp: SystemTime::now(),
        }],
        expected_response: ThreatResponse::Block,
        confidence: 0.95,
    };

    let threat_detected = framework.run_threat_detection_test(threat_scenario).await;
    assert!(threat_detected, "Critical threat should be detected");

    // Step 2: Zero trust evaluation
    let zero_trust_test = ZeroTrustTestCase {
        test_id: "workflow_zt_001".to_string(),
        device_id: "suspicious_device".to_string(),
        user_id: "unknown_user".to_string(),
        resource: "sensitive_data".to_string(),
        action: "read".to_string(),
        context: ZeroTrustContext {
            device_trust_level: 0.2, // Low trust
            network_location: NetworkLocation::External,
            time_of_access: SystemTime::now(),
            behavioral_anomaly_score: 0.8, // High anomaly
            session_state: SessionState::Active,
        },
        expected_access: false, // Should deny access
        verification_steps: vec![
            VerificationStep {
                step_name: "device_verification".to_string(),
                verification_type: VerificationType::DeviceIdentity,
                required: true,
                timeout: std::time::Duration::from_secs(30),
            },
            VerificationStep {
                step_name: "behavioral_analysis".to_string(),
                verification_type: VerificationType::BehavioralAnalysis,
                required: true,
                timeout: std::time::Duration::from_secs(60),
            },
        ],
    };

    let zero_trust_result = framework.run_zero_trust_test(zero_trust_test).await;
    assert!(zero_trust_result, "Suspicious access should be denied");

    // Step 3: Security audit
    let audit_result = framework.run_security_audit_test("compliance").await;
    assert!(audit_result, "Security audit should pass");

    println!("✅ Comprehensive security workflow test passed");
}

/// Performance test for security operations
#[tokio::test]
async fn test_security_performance() {
    let framework = init_security_test!("Security Performance");

    let start_time = std::time::Instant::now();

    // Run multiple threat detection tests
    for i in 0..10 {
        let threat_scenario = TestThreatScenario {
            scenario_id: format!("perf_test_{}", i),
            threat_type: ThreatType::Malware,
            severity: ThreatSeverity::Medium,
            source: ThreatSource::External,
            target: "test_system".to_string(),
            description: format!("Performance test scenario {}", i),
            indicators: vec![ThreatIndicator {
                indicator_type: "test_indicator".to_string(),
                value: format!("test_value_{}", i),
                confidence: 0.7,
                timestamp: SystemTime::now(),
            }],
            expected_response: ThreatResponse::Monitor,
            confidence: 0.7,
        };

        let _result = framework.run_threat_detection_test(threat_scenario).await;
    }

    let elapsed = start_time.elapsed();
    println!("Security performance test completed in {:?}", elapsed);

    // Assert reasonable performance (should complete in under 1 second)
    assert!(elapsed.as_secs() < 1, "Security operations should be fast");

    println!("✅ Security performance test passed");
}
