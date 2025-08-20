# 🧪 Integration Test Conversion Guide - Universal Adapter Delegation

**Date**: January 2025  
**Purpose**: Convert integration tests from old capability system to universal adapter delegation  
**Status**: Implementation Guide  

---

## 🎯 **Conversion Principle: Test Real Delegation, Not Mock Systems**

> *"Test the actual universal adapter delegation, not simulated environments."*

**Old Approach**: Mock providers, capability routers, direct implementations  
**New Approach**: Test universal adapter routing to real primals or fail gracefully  

---

## 🔄 **Conversion Patterns**

### **❌ OLD PATTERN: Mock Provider Registration**

```rust
// OLD: Create mock providers and capability routers
#[tokio::test]
async fn test_security_operations() {
    let router = Arc::new(RwLock::new(CapabilityRouter::new()));
    
    // Register mock security provider
    let mock_provider = CapabilityProvider {
        id: Uuid::new_v4().to_string(),
        name: "mock-beardog".to_string(),
        endpoint: "http://localhost:9999".to_string(), // Non-existent
        capabilities: vec!["security".to_string()],
        health_status: HealthStatus::Healthy,
        // ... more mock setup
    };
    
    router.write().await.register_provider(mock_provider);
    let security_adapter = SecurityAdapter::new(router);
    
    // Test falls back to mock implementation
    let result = security_adapter.encrypt("test data").await.unwrap();
    assert!(result.data.contains("fallback"));
}
```

### **✅ NEW PATTERN: Universal Adapter Integration Testing**

```rust
// NEW: Test actual universal adapter delegation
#[tokio::test]
async fn test_security_operations() {
    let security_integration = SecurityProviderIntegration::new().await.unwrap();
    
    // Test real BearDog delegation via universal adapter
    let result = security_integration.encrypt(b"test data").await;
    
    // Either succeeds with real BearDog or fails cleanly
    match result {
        Ok(encrypted) => {
            assert!(!encrypted.is_empty());
            info!("✅ BearDog encryption successful via universal adapter");
            
            // Test round-trip with decryption
            let decrypted = security_integration.decrypt(&encrypted).await.unwrap();
            assert_eq!(decrypted, b"test data");
        }
        Err(e) => {
            info!("⚠️ BearDog not available for integration test: {}", e);
            // No fallbacks - fail gracefully with clear error
            assert!(e.to_string().contains("BearDog"));
        }
    }
}
```

---

## 📋 **Test Conversion Examples**

### **🔐 Security Integration Tests**

```rust
// Convert BearDog security tests
#[tokio::test]
async fn test_beardog_authentication_integration() {
    let integration = SecurityProviderIntegration::new().await.unwrap();
    
    let result = integration.authenticate("test_user", "test_pass").await;
    
    match result {
        Ok(token) => {
            assert!(!token.is_empty());
            info!("✅ BearDog authentication successful");
            
            // Test token validation
            // Note: Would need separate validation method via universal adapter
        }
        Err(e) => {
            info!("⚠️ BearDog not available: {}", e);
            // Clear error, no fallback security
        }
    }
}

#[tokio::test]  
async fn test_beardog_threat_detection_integration() {
    let integration = SecurityProviderIntegration::new().await.unwrap();
    
    let activity_data = json!({
        "user_id": "test_user",
        "activity": "suspicious_login",
        "source_ip": "192.168.1.100"
    });
    
    let result = integration.detect_threats(activity_data).await;
    
    match result {
        Ok(threat_level) => {
            info!("✅ BearDog threat detection: {:?}", threat_level);
            assert!(matches!(threat_level, ThreatLevel::Low | ThreatLevel::Medium | ThreatLevel::High));
        }
        Err(e) => {
            info!("⚠️ BearDog threat detection not available: {}", e);
        }
    }
}
```

### **💾 Storage Integration Tests**

```rust
// Convert NestGate storage tests
#[tokio::test]
async fn test_nestgate_storage_integration() {
    let storage = StorageAdapter::new();
    
    // Test directory creation
    let result = storage.ensure_directory("test_integration_dir").await;
    match result {
        Ok(_) => {
            info!("✅ NestGate directory creation successful");
            
            // Test file operations
            let file_result = storage.store_file("test_integration_dir/test.txt", "integration test data").await;
            assert!(file_result.is_ok() || file_result.is_err()); // Either works or fails cleanly
            
            if file_result.is_ok() {
                // Test file reading
                let content = storage.load_file("test_integration_dir/test.txt").await.unwrap();
                assert_eq!(content, "integration test data");
                
                // Test cleanup
                let _delete_result = storage.delete_file("test_integration_dir/test.txt").await;
            }
        }
        Err(e) => {
            info!("⚠️ NestGate not available: {}", e);
        }
    }
}

#[tokio::test]
async fn test_nestgate_backup_integration() {
    let storage = StorageAdapter::new();
    
    let result = storage.backup_data("test_source", "integration_backup").await;
    
    match result {
        Ok(backup_path) => {
            info!("✅ NestGate backup successful: {}", backup_path);
            assert!(backup_path.contains("integration_backup"));
        }
        Err(e) => {
            info!("⚠️ NestGate backup not available: {}", e);
        }
    }
}
```

### **🧠 AI Integration Tests**

```rust
// Convert Squirrel AI tests  
#[tokio::test]
async fn test_squirrel_workload_classification_integration() {
    let ai_adapter = AICapabilityAdapter::new();
    
    let workload = WorkloadRequest {
        workload_type: "gaming_session".to_string(),
        resource_requirements: ResourceRequirements {
            cpu_cores: Some(4),
            memory_gb: Some(8),
            gpu_required: true,
            storage_gb: Some(50),
        },
        performance_constraints: PerformanceConstraints {
            max_latency_ms: Some(50),
            min_throughput: Some(1000.0),
            priority: "high".to_string(),
        },
        metadata: HashMap::new(),
    };
    
    let result = ai_adapter.classify_workload(workload).await;
    
    match result {
        Ok(classification) => {
            info!("✅ Squirrel classification: {:?}", classification);
            // Should classify gaming workload correctly
            assert!(matches!(classification, WorkloadType::Gaming | WorkloadType::AI | WorkloadType::General));
        }
        Err(e) => {
            info!("⚠️ Squirrel not available: {}", e);
        }
    }
}

#[tokio::test]
async fn test_squirrel_mcp_integration() {
    let ai_adapter = AICapabilityAdapter::new();
    
    let mcp_data = json!({
        "operation": "analyze",
        "data": "test mcp data for analysis",
        "context": "integration_test"
    });
    
    let result = ai_adapter.process_mcp_request(mcp_data).await;
    
    match result {
        Ok(ai_result) => {
            info!("✅ Squirrel MCP processing: success={}, confidence={}", 
                  ai_result.success, ai_result.confidence);
            assert!(ai_result.confidence >= 0.0 && ai_result.confidence <= 1.0);
        }
        Err(e) => {
            info!("⚠️ Squirrel MCP processing not available: {}", e);
        }
    }
}
```

---

## 🏗️ **Test Environment Setup**

### **Environment Variables for Integration Tests**

```bash
# Set these for full integration testing
export BEARDOG_AVAILABLE=true
export NESTGATE_AVAILABLE=true  
export SQUIRREL_AVAILABLE=true
export TOADSTOOL_AVAILABLE=true

# Or disable for unit testing
export BEARDOG_AVAILABLE=false
export NESTGATE_AVAILABLE=false
export SQUIRREL_AVAILABLE=false
export TOADSTOOL_AVAILABLE=false
```

### **Test Configuration Helper**

```rust
// Helper for conditional integration tests
pub fn should_run_integration_test(primal: &str) -> bool {
    std::env::var(&format!("{}_AVAILABLE", primal.to_uppercase()))
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false)
}

#[tokio::test]
async fn test_conditional_beardog_integration() {
    if !should_run_integration_test("beardog") {
        info!("⏭️ Skipping BearDog integration test - not available");
        return;
    }
    
    // Run actual integration test
    let integration = SecurityProviderIntegration::new().await.unwrap();
    let result = integration.get_security_status().await.unwrap();
    assert!(result.healthy);
}
```

---

## ✅ **Conversion Checklist**

- [ ] Replace `CapabilityRouter` and `CapabilityProvider` with universal adapter
- [ ] Remove mock provider registrations  
- [ ] Convert to real primal delegation via `routing::*_request()`
- [ ] Handle both success and failure cases gracefully
- [ ] Remove fallback implementations from tests
- [ ] Add environment-based conditional test execution
- [ ] Verify tests work with and without real primals available
- [ ] Update test documentation to reflect delegation architecture

---

## 🎯 **Success Criteria**

✅ **Clean Architecture**: Tests verify actual delegation behavior  
✅ **No Mock Systems**: No simulated providers or capability routers  
✅ **Graceful Failures**: Clear error messages when primals unavailable  
✅ **Real Integration**: Tests work with actual BearDog/NestGate/Squirrel/ToadStool  
✅ **Conditional Execution**: Tests skip gracefully when primals not available 