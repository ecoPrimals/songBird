# 🌌 Universal Adapter Delegation Specification

**Date**: January 2025  
**Purpose**: Define delegation patterns for Songbird to route capabilities to specialized primals  
**Status**: Active Implementation Guide  

---

## 🎯 **Core Principle: Orchestration, Not Implementation**

> *"Songbird orchestrates; Primals specialize. Route capabilities, don't duplicate them."*

**Songbird's Role**: Service discovery, load balancing, federation orchestration, networking  
**Delegate To Primals**: Security → BearDog, Storage → NestGate, Compute → ToadStool, AI → Squirrel  

---

## 📋 **Delegation Categories**

### **🔐 Security Capabilities → BearDog**

**Current Problem**: Direct security implementations in Songbird
**Solution**: Route through universal adapter

```rust
// ❌ WRONG: Direct implementation
impl SecurityProvider {
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Direct crypto implementation - BELONGS TO BEARDOG
        ring::aead::seal(...)
    }
}

// ✅ CORRECT: Universal adapter delegation
impl SecurityProvider {
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let ctx = AdapterContext::new("security_encryption");
        routing::security_request(&ctx, "encrypt", serde_json::to_value(data)?).await
    }
}
```

**Delegation Targets**:
- Authentication/Authorization → BearDog
- Encryption/Decryption → BearDog  
- Threat Detection → BearDog
- Audit Logging → BearDog
- Compliance Checking → BearDog

### **💾 Storage Capabilities → NestGate**

**Current Problem**: Storage monitoring and management TODOs
**Solution**: Route through universal adapter

```rust
// ❌ WRONG: Direct implementation  
impl StorageMonitor {
    async fn get_storage_stats(&self) -> Result<StorageStats> {
        // TODO: Implement actual storage detection
        Ok(StorageStats::default()) // Placeholder
    }
}

// ✅ CORRECT: Universal adapter delegation
impl StorageMonitor {
    async fn get_storage_stats(&self) -> Result<StorageStats> {
        let ctx = AdapterContext::new("storage_monitoring");
        routing::storage_request(&ctx, "get_stats", serde_json::Value::Null).await
    }
}
```

**Delegation Targets**:
- Storage capacity monitoring → NestGate
- Backup management → NestGate
- Data persistence → NestGate
- Storage optimization → NestGate

### **🖥️ Compute Capabilities → ToadStool**

**Current Problem**: System monitoring TODOs returning placeholder values
**Solution**: Route through universal adapter

```rust
// ❌ WRONG: Placeholder implementation
impl SystemMonitor {
    async fn get_cpu_usage(&self) -> Result<f64> {
        // TODO: Implement actual CPU usage monitoring
        Ok(0.0) // Placeholder - PRODUCTION RISK
    }
}

// ✅ CORRECT: Universal adapter delegation  
impl SystemMonitor {
    async fn get_cpu_usage(&self) -> Result<f64> {
        let ctx = AdapterContext::new("compute_monitoring");
        let response = routing::compute_request(&ctx, "cpu_usage", serde_json::Value::Null).await?;
        Ok(response.as_f64().unwrap_or(0.0))
    }
}
```

**Delegation Targets**:
- CPU usage monitoring → ToadStool
- Memory usage monitoring → ToadStool
- Load average calculation → ToadStool
- Performance optimization → ToadStool

### **🧠 AI Capabilities → Squirrel**

**Current Problem**: AI integration mocks and placeholders
**Solution**: Route through universal adapter

```rust
// ❌ WRONG: Mock implementation
pub struct MockAIProvider {
    // Mock responses - NOT PRODUCTION READY
}

// ✅ CORRECT: Universal adapter delegation
impl AIProvider {
    async fn process_request(&self, input: AIRequest) -> Result<AIResponse> {
        let ctx = AdapterContext::new("ai_processing");
        routing::ai_request(&ctx, "process", serde_json::to_value(input)?).await
    }
}
```

**Delegation Targets**:
- AI inference → Squirrel
- Model management → Squirrel
- AI optimization → Squirrel
- MCP integration → Squirrel

---

## 🔧 **Implementation Conversion Plan**

### **Phase 1: Critical Security Delegation**

#### **Files to Update**:
```bash
# Find security implementations to convert
find crates/ -name "*.rs" -exec grep -l "TODO.*security\|Mock.*Security" {} \;

# Target patterns:
crates/songbird-security/src/security/universal_security_provider.rs
crates/songbird-federation/src/security/
```

#### **Conversion Pattern**:
```rust
// Replace ALL security TODOs with universal adapter calls
impl SecurityProvider {
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken> {
        let ctx = AdapterContext::new("security_auth");
        let payload = serde_json::json!({
            "credentials": credentials,
            "operation": "authenticate"
        });
        
        let response = routing::security_request(&ctx, "authenticate", payload).await?;
        Ok(serde_json::from_value(response)?)
    }
}
```

### **Phase 2: System Monitoring Delegation**

#### **Files to Update**:
```bash
# Find monitoring TODOs to convert
grep -r "TODO.*CPU\|TODO.*memory\|TODO.*monitoring" crates/songbird-federation/

# Target files:
crates/songbird-federation/src/mcp_handler/monitoring.rs:lines_with_TODOs
crates/songbird-federation/src/capability_based_monitoring.rs:placeholder_implementations
```

#### **Conversion Pattern**:
```rust
// Replace monitoring TODOs with universal adapter calls
impl MonitoringManager {
    async fn get_system_metrics(&self) -> Result<SystemMetrics> {
        let ctx = AdapterContext::new("system_monitoring");
        
        // Parallel requests to different capabilities
        let (cpu_result, memory_result, storage_result) = tokio::join!(
            routing::compute_request(&ctx, "cpu_usage", serde_json::Value::Null),
            routing::compute_request(&ctx, "memory_usage", serde_json::Value::Null),
            routing::storage_request(&ctx, "storage_stats", serde_json::Value::Null)
        );
        
        Ok(SystemMetrics {
            cpu_usage: cpu_result?.as_f64().unwrap_or(0.0),
            memory_usage: memory_result?.as_f64().unwrap_or(0.0),
            storage_available: storage_result?.as_f64().unwrap_or(0.0),
            timestamp: chrono::Utc::now(),
        })
    }
}
```

### **Phase 3: Network and Federation Delegation**

#### **Conversion Targets**:
```rust
// Convert federation communication to inter-primal routing
impl FederationManager {
    async fn broadcast_message(&self, message: &FederationMessage) -> Result<()> {
        let ctx = AdapterContext::new("federation_broadcast");
        
        // Route to orchestration capability providers
        routing::orchestration_request(&ctx, "broadcast", serde_json::to_value(message)?).await?;
        Ok(())
    }
}
```

---

## 📊 **Universal Adapter Extensions**

### **Additional Routing Functions Needed**:

```rust
// Add to crates/songbird-universal-primals/src/global_adapter.rs
pub mod routing {
    // Existing: security_request, storage_request, compute_request
    
    // Add these new routing functions:
    
    /// Route AI processing requests to Squirrel
    pub async fn ai_request(
        ctx: &AdapterContext,
        operation: &str,
        payload: Value,
    ) -> SongbirdResult<Value> {
        get_global_adapter()
            .send_capability_request("ai", operation, payload)
            .await
    }
    
    /// Route orchestration requests to coordination primals
    pub async fn orchestration_request(
        ctx: &AdapterContext,
        operation: &str,
        payload: Value,
    ) -> SongbirdResult<Value> {
        get_global_adapter()
            .send_capability_request("orchestration", operation, payload)
            .await
    }
    
    /// Route monitoring requests to metrics providers
    pub async fn monitoring_request(
        ctx: &AdapterContext,
        operation: &str,
        payload: Value,
    ) -> SongbirdResult<Value> {
        get_global_adapter()
            .send_capability_request("monitoring", operation, payload)
            .await
    }
}
```

---

## 🎯 **Success Criteria**

### **Completion Metrics**:
- [ ] **Zero security implementations** in Songbird (all delegated to BearDog)
- [ ] **Zero compute monitoring TODOs** (all delegated to ToadStool)  
- [ ] **Zero storage management code** (all delegated to NestGate)
- [ ] **Zero AI processing mocks** (all delegated to Squirrel)
- [ ] **Universal adapter coverage** for all non-core capabilities

### **Validation Tests**:
```rust
#[cfg(test)]
mod delegation_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_security_delegation() {
        // Verify security calls route to universal adapter
        let provider = SecurityProvider::new();
        let result = provider.encrypt(b"test").await;
        
        // Should succeed via universal adapter routing
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_no_direct_implementations() {
        // Verify no hardcoded security/compute/storage implementations exist
        // This test ensures we maintain delegation architecture
    }
}
```

---

## 📚 **Implementation Priority**

| Priority | Category | Impact | Timeline |
|----------|----------|--------|----------|
| **P0** | Security Delegation | Production Security | 1 week |
| **P1** | Monitoring Delegation | System Stability | 1 week |
| **P2** | Storage Delegation | Data Management | 1 week |
| **P3** | AI Delegation | Feature Completeness | 1 week |

**Total Estimated Time**: 3-4 weeks for complete delegation architecture 