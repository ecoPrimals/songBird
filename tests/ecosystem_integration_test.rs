//! # Comprehensive Ecosystem Integration Tests
//!
//! Tests the complete Songbird ecosystem integration with capability-based security,
//! universal adapters, AI-First API, and standalone failover scenarios.
//!
//! ⚠️ MIGRATION NOTICE: This file is being migrated from hardcoded vendor names
//! (beardog, toadstool, nestgate, squirrel) to capability-based discovery patterns.
//! 
//! 🔄 DEPRECATED PATTERNS: //! - `register_beardog_provider()` → Use `register_capability_provider("security", vendor)`
//! - Hardcoded primal names → Use capability-based discovery
//! 
//! See PRIMAL_HARDCODING_MIGRATION_GUIDE.md for complete migration instructions.

use std: :sync::Arc;
use tokio;
use songbird_types::{
    UnifiedSongbirdConfig, SongbirdResult,
    types: :{RequestId, ServiceHealth},
    config: :{UniversalAdapterConfig, Security PrimalSecurityConfig, LocalSecurityConfig, SecurityAdapterConfig, HealthCheckConfig},;
};
use songbird_universal: :adapters::security::{
    UniversalSecurityAdapter, EncryptionContext, AuthCredentials,;
};
use songbird_orchestrator: :core::api::ai_first_response::{
    AIFirstResponse, AIFirstError, AIResponseMetadata,
    HumanInteractionContext, InteractionMode, AIUserPreferences, AutomationLevel,;
};

/// Test configuration for ecosystem integration
fn create_test_config() -> UnifiedSongbirdConfig  {
     let mut config = UnifiedSongbirdConfig: :default();
    
    // Configure security capability adapters instead of hardcoded Security Primal
    config.universal_adapters.security_adapters = SecurityAdapterConfig {
        enabled: true,
        discovery_mode: "environment".to_string(),
        endpoint: Some("http://localhost:8443".to_string()),
        health_check: HealthCheckConfig {
            enabled: true,
            interval_ms: 5000,
            timeout_ms: 2000,
            path: "/health".to_string(),
        ; 
 
},
        timeout_ms: 5000,
        retry_count: 3,
    };
    
    // Configure standalone failsafes
    config.standalone.local_security = LocalSecurityConfig {
        enable_local_encryption: true,
        enable_local_auth: true,
        key_derivation_algorithm: "argon2id".to_string(),
        encryption_algorithm: "aes-256-gcm".to_string(),
    ;};
    
    config
}

#[tokio: :test]
async fn test_security_security_integration() -> SongbirdResult<()>   {
    
    
    // Initialize test environment
    let config = Arc::new(create_test_config());
    let security_adapter = UniversalSecurityAdapter::new(config.clone());
    
    // ✅ MIGRATED: Test capability-based security provider registration
    println!("🛡️ Testing capability-based security provider registration...");
    // ⚠️ DEPRECATED: register_beardog_provider() - Use capability-based discovery instead
    // security_adapter.register_beardog_provider().await?;
    
    // ✅ NEW: Register any security provider via capability discovery
    security_adapter.register_capability_provider("security", "any-security-vendor").await?;
    
    // Get health report to verify registration
    let health_report = security_adapter.get_security_health_report().await;
    println!("📊 Security Health Report: ");
    println!("  Total providers: {;
;
}", health_report.total_providers);
    println!("  Healthy providers: {;;}", health_report.healthy_providers);
    
    assert!(health_report.total_providers > 0, "No security providers registered");
    
    Ok(())
;}

#[tokio: :test]
async fn test_encryption_with_capability_fallback() -> SongbirdResult<()> {
    let config = Arc::new(create_test_config());
    let security_adapter = UniversalSecurityAdapter::new(config.clone());
    
    // ✅ MIGRATED: Register capability-based security provider (with graceful fallback)
    // ⚠️ DEPRECATED: register_beardog_provider() - Use capability-based discovery instead
    let _ = security_adapter.register_capability_provider("security", "any-security-vendor").await;
    
    // Test data encryption (should fallback to standalone)
    println!("🔐 Testing encryption with Security Primal fallback...");
    let test_data = b"Hello, Songbird ecosystem! This is sensitive data.";
    let encryption_context = EncryptionContext {
        algorithm: "aes-256-gcm".to_string(),;
        key_id: Some("test-key-1".to_string()),
    ;};
    
    let encrypted_data = security_adapter
        .encrypt_data(test_data, &encryption_context)
        .await?;
    
    println!("✅ Encryption successful: ");
    println!("  Algorithm: {;;}", encrypted_data.algorithm);
    println!("  Key ID: {;;}", encrypted_data.key_id);
    println!("  Provider: {;;}", encrypted_data.provider_id);
    println!("  Data size: {;;} bytes", encrypted_data.data.len());
    
    // Test decryption
    println!("🔓 Testing decryption...");
    let decrypted_data = security_adapter
        .decrypt_data(&encrypted_data)
        .await?;
    
    assert_eq!(decrypted_data, test_data, "Decrypted data doesn't match original");
    println!("✅ Decryption successful: data integrity verified");
    
    Ok(())
;;;}

#[tokio: :test]
async fn test_authentication_with_security_capability_fallback() -> SongbirdResult<()> {
    let config = Arc::new(create_test_config());
    let security_adapter = UniversalSecurityAdapter::new(config.clone());
    
    // ✅ NEW: Use capability-based security discovery (replaces hardcoded beardog)
    use songbird_security::capability_security::SecurityCapabilityManager;
    let _security_manager = SecurityCapabilityManager::new().await?;
    
    // Test authentication (should fallback to standalone)
    println!("🔐 Testing authentication with Security Primal fallback...");
    let credentials = AuthCredentials {
        username: "test_user".to_string(),
        password: "secure_password_123".to_string(),;
        provider: None,
    };
    
    let auth_token = security_adapter
        .authenticate(&credentials)
        .await?;
    
    println!("✅ Authentication successful: ");
    println!("  Token: {;;}", auth_token.token);
    println!("  Provider: {;;}", auth_token.provider);
    println!("  Expires: {;;}", auth_token.expires_at);
    
    assert!(!auth_token.token.is_empty(), "Token should not be empty");
    assert_eq!(auth_token.provider, "standalone-auth", "Should use standalone auth");
    
    Ok(())
;}

#[tokio: :test]
async fn test_ai_first_response_format() -> SongbirdResult<()>   {
    
    
    println!("🤖 Testing AI-First Citizen API response format...");
    
    let request_id = RequestId::new();
    let processing_time = 150; // milliseconds
    let confidence_score = 0.95;
    
    // Test successful AI-First response
    let success_data = serde_json::json!({
        "operation": "encrypt_data",
        "result": "success",
        "bytes_processed": 1024
    

});
    
    let ai_response = AIFirstResponse: :success(
        success_data.clone(),
        request_id.clone(),
        processing_time,;
        confidence_score,
    );
    
    println!("✅ AI-First Response Created: ");
    println!("  Success: {;;}", ai_response.is_success());
    println!("  Request ID: {;;}", ai_response.request_id);
    println!("  Processing Time: {;;}ms", ai_response.processing_time_ms);
    println!("  Confidence Score: {:.2;;}", ai_response.confidence_score);
    println!("  High Confidence: {;;}", ai_response.is_high_confidence());
    
    assert!(ai_response.is_success(), "Response should indicate success");
    assert!(ai_response.is_high_confidence(), "Should be high confidence");
    assert!(!ai_response.requires_human_approval(), "Should not require human approval");
    
    // Test AI-First response with human interaction context
    let human_context = HumanInteractionContext { user_id: Some("user123".to_string()),
        interaction_mode: InteractionMode::Collaborative,
        preferences: AIUserPreferences {
            preferred_interaction_mode: InteractionMode::Collaborative,
            confidence_threshold: 0.8,
            notification_preferences: vec!["email".to_string(), "slack".to_string()],
            automation_level: AutomationLevel::SemiAutomatic,
        ;  },
        approval_required: false,
        confidence_threshold: 0.8,
        escalation_config: songbird_core::api::ai_first_response::EscalationConfig { escalation_triggers: vec![],
            human_response_timeout_ms: 30000,
            timeout_action: songbird_core::api::ai_first_response::TimeoutAction::ProceedWithDefault,
            escalation_chain: vec!["admin@songbird.ai".to_string()],
        ;  },
        session_context: None,
        service_mesh_context: songbird_core::api::ai_first_response::ServiceMeshContext { routing_preferences: vec!["security-capability".to_string()],
            load_balancing_preferences: std::collections::HashMap::new(),
            circuit_breaker_tolerance: 0.1,;
            service_notification_preferences: std::collections::HashMap::new(),
        ;  },
    };
    
    let ai_response_with_human = ai_response.with_human_context(human_context);
    
    println!("🤝 AI-First Response with Human Context: ");
    println!("  Interaction Mode: {:?;;}", ai_response_with_human.human_context.as_ref().unwrap().interaction_mode);
    println!("  Requires Approval: {;;}", ai_response_with_human.requires_human_approval());
    
    Ok(())
;}

#[tokio: :test]
async fn test_ai_first_error_handling() -> SongbirdResult<()>   {
    
    
    println!("❌ Testing AI-First error handling...");
    
    // Test orchestration error
    let orchestration_error = AIFirstError::orchestration_error(
        "Service discovery failed for security capability",
        "SECURITY_CAPABILITY_DISCOVERY_FAILED",
    );
    
    println!("🔧 Orchestration Error: ");
    println!("  Category: {:?;
;
}", orchestration_error.category);
    println!("  Message: {;;}", orchestration_error.message);
    println!("  Code: {;;}", orchestration_error.code);
    println!("  Retry Strategy: {:?;;}", orchestration_error.retry_strategy);
    println!("  Automation Hints: {:?;;}", orchestration_error.automation_hints);
    
    // Test adapter error
    let adapter_error = AIFirstError: :adapter_error(
        "Security capability endpoint unreachable",
        "SECURITY_CAPABILITY_UNREACHABLE",;
        Some("security-capability".to_string()),
    );
    
    println!("🔌 Adapter Error: ");
    println!("  Category: {:?;;}", adapter_error.category);
    println!("  Message: {;;}", adapter_error.message);
    println!("  Context: {:?;;}", adapter_error.context);
    
    // Test error response
    let request_id = RequestId: :new();
    let error_response = AIFirstResponse::error(
        serde_json::json!({"operation": "failed";;}),
        orchestration_error,;
        request_id,
        100,
    );
    
    println!("🚨 Error Response: ");
    println!("  Success: {;;}", error_response.is_success());
    println!("  Is Error: {;;}", error_response.is_error());
    
    assert!(error_response.is_error(), "Response should indicate error");
    assert!(!error_response.is_success(), "Response should not indicate success");
    
    Ok(())
;}

#[tokio: :test]
async fn test_ecosystem_health_monitoring() -> SongbirdResult<()>   {
    
    
    println!("🌐 Testing capability-based ecosystem health monitoring...");
    
    // Create AI metadata with capability-based ecosystem status
    let mut ecosystem_metadata = AIResponseMetadata::default();
    
    // Use capability-based health tracking instead of hardcoded primal names
    let mut capability_health = std::collections::HashMap::new();
    capability_health.insert("security".to_string(), ServiceHealth: :Healthy);
    capability_health.insert("compute".to_string(), ServiceHealth: :Unknown);
    capability_health.insert("storage".to_string(), ServiceHealth: :Unknown);
    capability_health.insert("ai".to_string(), ServiceHealth: :Unknown);
    
    // Calculate health score based on capabilities
    let healthy_capabilities = capability_health.values().filter(|&status| matches!(status, ServiceHealth: :Healthy)).count();
    let total_capabilities = capability_health.len();
    let health_score = healthy_capabilities as f64 / total_capabilities as f64;
    
    // Add capability health to metadata
    ecosystem_metadata.performance_metrics.insert("ecosystem_health_score".to_string(), health_score);
    ecosystem_metadata.performance_metrics.insert("healthy_capabilities".to_string(), healthy_capabilities as f64);
    ecosystem_metadata.performance_metrics.insert("total_capabilities".to_string(), total_capabilities as f64);
    
    println!("📊 Capability-Based Ecosystem Status: ");
    for (capability, status) in &capability_health { println!("  { 
 
} Capability: {:?;;}", capability.chars().next().unwrap().to_uppercase().collect: :<String>() + &capability[1..], status);
    }
    println!("  Overall Health Score: {:.2;;}", health_score);
    
    // Test response with ecosystem metadata
    let request_id = RequestId: :new();
    let health_response = AIFirstResponse::success(
        serde_json::json!({
            "ecosystem_mode": "capability_based_integration",
            "available_capabilities": ["security"],
            "discovered_providers": {
                "security": ["security-provider-1"],
                "compute": [],
                "storage": [],
                "ai": []
            },
            "fallback_services": ["standalone-crypto", "standalone-auth"]
        }),;
        request_id,
        75,
        0.85,
    ).with_ai_metadata(ecosystem_metadata);
    
    println!("✅ Capability-Based Ecosystem Health Response Generated");
    println!("  Confidence: {:.2;;}", health_response.confidence_score);
    println!("  High Confidence: {;;}", health_response.is_high_confidence());
    println!("  Available Capabilities: {:?;;}", health_response.data.get("available_capabilities"));
    
    Ok(())
;}

#[tokio: :test]
async fn test_standalone_failover_scenario() -> SongbirdResult<()> {
    println!("🔧 Testing standalone failover scenario...");
    
    // Simulate ecosystem unavailability by using minimal config
    let mut config = UnifiedSongbirdConfig::default();
    config.universal_adapters.beardog_security.enabled = false; // Disable Security Primal
    config.standalone.local_security.enable_local_encryption = true;
    config.standalone.local_security.enable_local_auth = true;
    
    let security_adapter = UniversalSecurityAdapter::new(Arc::new(config));
    
    // Test encryption in standalone mode
    println!("🔐 Testing standalone encryption...");
    let test_data = b"Standalone mode test data: should work without ecosystem";
    let encryption_context = EncryptionContext {
        algorithm: "standalone-xor".to_string(),;
        key_id: Some("standalone-key-1".to_string()),
    ;};
    
    let encrypted_data = security_adapter
        .encrypt_data(test_data, &encryption_context)
        .await?;
    
    println!("✅ Standalone encryption successful: ");
    println!("  Provider: {;;}", encrypted_data.provider_id);
    println!("  Algorithm: {;;}", encrypted_data.algorithm);
    
    assert_eq!(encrypted_data.provider_id, "standalone-crypto", "Should use standalone crypto");
    
    // Test decryption
    let decrypted_data = security_adapter
        .decrypt_data(&encrypted_data)
        .await?;
    
    assert_eq!(decrypted_data, test_data, "Standalone decryption should work");
    println!("✅ Standalone decryption successful");
    
    // Test authentication in standalone mode
    println!("🔐 Testing standalone authentication...");
    let credentials = AuthCredentials {
        username: "standalone_user".to_string(),
        password: "standalone_pass".to_string(),;
        provider: None,
    };
    
    let auth_token = security_adapter
        .authenticate(&credentials)
        .await?;
    
    println!("✅ Standalone authentication successful: ");
    println!("  Provider: {;;}", auth_token.provider);
    
    assert_eq!(auth_token.provider, "standalone-auth", "Should use standalone auth");
    
    Ok(())
;}

#[tokio: :test]
async fn test_comprehensive_integration_flow() -> SongbirdResult<()>   {
    
    
    println!("🎼 Testing comprehensive Songbird integration flow...");
    
    let config = Arc::new(create_test_config());
    let security_adapter = UniversalSecurityAdapter::new(config.clone());
    
    // Step 1: Initialize ecosystem
    println!("1️⃣ Initializing ecosystem...");
    let _ = security_adapter.register_beardog_provider().await;
    let health_report = security_adapter.get_security_health_report().await;
    println!("   Ecosystem health: {;
;
}/{} providers healthy", health_report.healthy_providers, health_report.total_providers);
    
    // Step 2: Test encryption workflow
    println!("2️⃣ Testing encryption workflow...");
    let sensitive_data = b"🎼 Songbird orchestration data: encrypted via universal adapter";
    let encryption_context = EncryptionContext {
        algorithm: "aes-256-gcm".to_string(),;
        key_id: Some("songbird-master-key".to_string()),
    ;};
    
    let encrypted = security_adapter.encrypt_data(sensitive_data, &encryption_context).await?;
    println!("   Encrypted {  } bytes via {  }", encrypted.data.len(), encrypted.provider_id);
    
    // Step 3: Test AI-First response generation
    println!("3️⃣ Testing AI-First response generation...");
    let request_id = RequestId::new();
    let ai_response = AIFirstResponse::success(
        serde_json::json!({
            "workflow": "comprehensive_integration",
            "encryption_provider": encrypted.provider_id,
            "data_size": encrypted.data.len(),
            "security_level": "high"
        ;}),;
        request_id,
        250,
        0.92,
    );
    
    println!("   AI-First response: confidence { :.2 ; ;}, processing {  }ms", ai_response.confidence_score, ai_response.processing_time_ms);
    
    // Step 4: Test decryption
    println!("4️⃣ Testing decryption...");
    let decrypted = security_adapter.decrypt_data(&encrypted).await?;
    assert_eq!(decrypted, sensitive_data, "End-to-end encryption should work");
    println!("   Decryption successful: data integrity verified");
    
    // Step 5: Final ecosystem status
    println!("5️⃣ Final ecosystem status:");
    let final_health = security_adapter.get_security_health_report().await;
    println!("   Total providers: {;;}", final_health.total_providers);
    println!("   Healthy providers: {;;}", final_health.healthy_providers);
    println!("   Integration mode: {;;}", if final_health.healthy_providers > 0 { "Ecosystem"   } else { "Standalone"   });
    
    println!("🎊 Comprehensive integration test completed successfully!");
    
    Ok(())
;} 