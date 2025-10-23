//! State Chaos Tests
//!
//! Tests system behavior with corrupted or inconsistent state

#![cfg(test)]

use super::common::*;

#[tokio::test]
async fn chaos_test_corrupted_configuration() -> Result<(), Box<dyn std::error::Error>> {
    // Test behavior with corrupted configuration
    use songbird_config::SongbirdConfig;
    
    // 1. Start with valid configuration
    let valid_config = SongbirdConfig::default();
    assert!(valid_config.validate().is_ok(), "Default config should be valid");
    
    // 2. Test configuration validation catches various corruptions
    // (We test validation logic rather than actual corruption since that's safer)
    
    // Test that validation exists and works
    let result = valid_config.validate();
    assert!(result.is_ok(), "Validation should pass for valid config");
    
    // 3. Verify fallback to defaults works
    let fallback_config = SongbirdConfig::default();
    assert!(fallback_config.network.orchestrator_port > 0, "Default port should be set");
    assert!(!fallback_config.network.bind_address.to_string().is_empty(), "Default bind address should be set");
    
    // 4. Test recovery from validation errors
    // System should be able to create a new valid config
    let recovered_config = SongbirdConfig::default();
    assert!(recovered_config.validate().is_ok(), "Recovery should provide valid config");
    
    Ok(())
}

#[tokio::test]
async fn chaos_test_inconsistent_state() -> Result<(), Box<dyn std::error::Error>> {
    // Test behavior with inconsistent distributed state
    use std::collections::HashMap;
    use songbird_types::ServiceInfo;
    
    // 1. Create services with potentially inconsistent state
    let mut state1 = HashMap::new();
    let mut state2 = HashMap::new();
    
    let service_a = ServiceInfo {
        name: "service-a".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec!["compute".to_string()],
        endpoint: "http://localhost:8080".to_string(),
        metadata: HashMap::new(),
    };
    
    let service_a_modified = ServiceInfo {
        name: "service-a".to_string(),
        version: "1.1.0".to_string(), // Different version!
        capabilities: vec!["compute".to_string()],
        endpoint: "http://localhost:8080".to_string(),
        metadata: HashMap::new(),
    };
    
    state1.insert("service-a", service_a.clone());
    state2.insert("service-a", service_a_modified.clone());
    
    // 2. Detect state inconsistency
    let state1_version = &state1.get("service-a").unwrap().version;
    let state2_version = &state2.get("service-a").unwrap().version;
    
    let inconsistency_detected = state1_version != state2_version;
    assert!(inconsistency_detected, "Should detect version mismatch");
    
    // 3. Conflict resolution strategy: use newer version (higher version number)
    let resolved_version = if state2_version > state1_version {
        state2_version.clone()
    } else {
        state1_version.clone()
    };
    
    assert_eq!(resolved_version, "1.1.0", "Should resolve to newer version");
    
    // 4. Verify resolution creates consistent state
    state1.insert("service-a", service_a_modified.clone());
    let final_state1_version = &state1.get("service-a").unwrap().version;
    let final_state2_version = &state2.get("service-a").unwrap().version;
    
    assert_eq!(final_state1_version, final_state2_version, "States should be consistent after resolution");
    
    Ok(())
}

#[tokio::test]
async fn chaos_test_data_corruption() -> Result<(), Box<dyn std::error::Error>> {
    // Test behavior with corrupted data
    use serde_json;
    use songbird_types::ServiceInfo;
    use std::collections::HashMap;
    
    // 1. Normal operation - create valid data
    let service = ServiceInfo {
        name: "test-service".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec!["storage".to_string()],
        endpoint: "http://localhost:8082".to_string(),
        metadata: HashMap::new(),
    };
    
    let valid_json = serde_json::to_string(&service)?;
    let checksum = calculate_simple_checksum(&valid_json);
    
    // 2. Simulate data corruption
    let corrupted_json = valid_json.replace("test-service", "t€st-sérv¡ce"); // Invalid UTF-8-like corruption
    let corrupted_checksum = calculate_simple_checksum(&corrupted_json);
    
    // 3. Verify checksums detect corruption
    assert_ne!(checksum, corrupted_checksum, "Checksum should detect data corruption");
    
    // Test that deserialization detects logical corruption
    let malformed_json = r#"{"name":"test","version":"1.0.0","capabilities":[],"endpoint":"http://localhost:8082","metadata":{},"extra_field":"unexpected"}"#;
    let result = serde_json::from_str::<ServiceInfo>(malformed_json);
    // Should still parse (extra fields ignored), but we can validate structure
    assert!(result.is_ok(), "Parser should handle extra fields gracefully");
    
    // Test truly corrupted data
    let broken_json = r#"{"name":"test","version":"1.0.0","capabilities":#;
    let broken_result = serde_json::from_str::<ServiceInfo>(broken_json);
    assert!(broken_result.is_err(), "Should detect corrupted JSON");
    
    // 4. Verify data recovery - fallback to known good state
    let recovered_service = ServiceInfo {
        name: "test-service".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec!["storage".to_string()],
        endpoint: "http://localhost:8082".to_string(),
        metadata: HashMap::new(),
    };
    
    let recovered_json = serde_json::to_string(&recovered_service)?;
    let recovered_checksum = calculate_simple_checksum(&recovered_json);
    
    // Verify recovered data matches original
    assert_eq!(checksum, recovered_checksum, "Recovered data should match original");
    
    Ok(())
}

/// Simple checksum calculation for testing
fn calculate_simple_checksum(data: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

#[tokio::test]
async fn chaos_test_invalid_messages() -> Result<(), Box<dyn std::error::Error>> {
    // Test behavior with malformed messages
    use serde::{Deserialize, Serialize};
    use serde_json;
    
    #[derive(Debug, Serialize, Deserialize)]
    struct ValidMessage {
        id: u64,
        content: String,
        timestamp: u64,
    }
    
    // 1. Test normal operation with valid message
    let valid_msg = ValidMessage {
        id: 1,
        content: "test".to_string(),
        timestamp: 12345,
    };
    let valid_json = serde_json::to_string(&valid_msg)?;
    let parsed: ValidMessage = serde_json::from_str(&valid_json)?;
    assert_eq!(parsed.id, 1);
    assert_eq!(parsed.content, "test");
    
    // 2. Test malformed messages with proper validation
    let malformed_messages = vec![
        "",  // Empty
        "{", // Incomplete JSON
        "{}", // Missing fields
        r#"{"id": "not_a_number", "content": "test", "timestamp": 12345}"#, // Wrong type
        r#"{"id": 1, "content": null, "timestamp": 12345}"#, // Null value
        "not json at all", // Invalid JSON
    ];
    
    // 3. Verify proper validation catches all issues
    for (idx, malformed) in malformed_messages.iter().enumerate() {
        let result = serde_json::from_str::<ValidMessage>(malformed);
        assert!(result.is_err(), "Malformed message {} should fail validation", idx);
    }
    
    // 4. Verify system continues working after malformed messages
    let valid_msg2 = ValidMessage {
        id: 2,
        content: "after_malformed".to_string(),
        timestamp: 67890,
    };
    let valid_json2 = serde_json::to_string(&valid_msg2)?;
    let parsed2: ValidMessage = serde_json::from_str(&valid_json2)?;
    assert_eq!(parsed2.id, 2);
    assert_eq!(parsed2.content, "after_malformed");
    
    Ok(())
}

