# 🔄 BiomeOS Universal Adapter Migration Guide

**Date**: January 2025  
**Migration**: Hardcoded BiomeOS Integration → Universal Capability Provider  
**Impact**: **BREAKING CHANGE** - Architectural transformation required

---

## 🎯 **MIGRATION OVERVIEW**

BiomeOS is now treated as a **capability provider** through the universal adapter system, eliminating hardcoded integration patterns. This aligns BiomeOS with the same universal architecture used for all other primals.

### **Before (Hardcoded - DEPRECATED)**
```rust
use songbird_core::biomeos::{BiomeOSClient, BiomeOSIntegration};

// Hardcoded client creation
let client = BiomeOSClient::new("http://biomeos:4000".to_string());
let integration = BiomeOSIntegration::new(config, orchestrator).await?;
```

### **After (Universal Adapter - RECOMMENDED)**
```rust
use songbird_core::biomeos::universal_adapter::{UniversalBiomeOSManager, BiomeOSCapabilityProvider};

// Universal capability provider approach
let manager = UniversalBiomeOSManager::new().await?;
// BiomeOS endpoint discovered automatically via environment or discovery
```

---

## 📋 **STEP-BY-STEP MIGRATION**

### **Step 1: Update Imports**

**OLD (Deprecated)**:
```rust
use songbird_core::biomeos::{
    BiomeOSClient,
    BiomeOSIntegration, 
    ServiceRegistrationManager,
};
```

**NEW (Universal)**:
```rust
use songbird_core::biomeos::universal_adapter::{
    UniversalBiomeOSManager,
    BiomeOSCapabilityProvider,
};
use songbird_core::biomeos::types::*; // Types remain the same
```

### **Step 2: Replace Initialization**

**OLD (Hardcoded)**:
```rust
let biomeos_endpoint = std::env::var("BIOMEOS_ENDPOINT")
    .unwrap_or_else(|_| "http://localhost:4000".to_string());
let client = BiomeOSClient::new(biomeos_endpoint);
let integration = BiomeOSIntegration::new(config, orchestrator).await?;
integration.initialize().await?;
```

**NEW (Universal)**:
```rust
// Endpoint discovery is automatic
let manager = UniversalBiomeOSManager::new().await?;
// No manual initialization needed - handled by capability provider
```

### **Step 3: Replace Service Registration**

**OLD (Direct Registration)**:
```rust
let registration = BiomeOSServiceRegistration {
    service_id: "my-service".to_string(),
    endpoints: BiomeOSEndpoints::default(),
    capabilities: BiomeOSCapabilities::default(),
    // ... other fields
};

integration.register_with_biomeos(registration).await?;
```

**NEW (Universal Registration)**:
```rust
let registration = BiomeOSServiceRegistration {
    service_id: "my-service".to_string(),
    endpoints: BiomeOSEndpoints::default(),
    capabilities: BiomeOSCapabilities::default(),
    // ... other fields (same as before)
};

let result = manager.register_service(registration).await?;
```

### **Step 4: Replace Health Checks**

**OLD (Direct Client)**:
```rust
let connectivity = client.test_connection().await;
if connectivity.is_connected() {
    // BiomeOS is available
}
```

**NEW (Universal Health)**:
```rust
let is_available = manager.is_available().await;
if is_available {
    // BiomeOS is available
}

// Or get detailed health info
let health = manager.check_health().await?;
```

### **Step 5: Replace Deployment Operations**

**OLD (Direct Integration)**:
```rust
let deployment_request = BiomeOSDeploymentRequest {
    // ... deployment configuration
};
integration.deploy_service(deployment_request).await?;
```

**NEW (Universal Deployment)**:
```rust
let deployment_request = serde_json::json!({
    // ... deployment configuration (same structure)
});
let result = manager.deploy_service(deployment_request).await?;
```

---

## 🔧 **CONFIGURATION CHANGES**

### **Environment Variables**
The universal adapter maintains backward compatibility with existing environment variables:

```bash
# Still supported for backward compatibility
BIOMEOS_ENDPOINT=http://biomeos.example.com:4000

# NEW: Universal discovery approach (future)
PRIMAL_BIOMEOS_ENDPOINT=http://biomeos.example.com:4000
PRIMAL_BIOMEOS_CAPABILITIES=os,deployment,coordination,registration,health
```

### **Configuration Files**
Update your configuration to use the universal primal system:

**OLD (songbird.toml)**:
```toml
[biomeos]
endpoint = "http://biomeos:4000"
enabled = true
```

**NEW (songbird.toml)**:
```toml
[primals.biomeos]
endpoint = "http://biomeos:4000"
capabilities = ["os", "deployment", "coordination", "registration", "health"]
enabled = true
discovery_enabled = true
```

---

## ⚠️ **BREAKING CHANGES**

### **1. Import Paths Changed**
- `songbird_core::biomeos::BiomeOSClient` → `songbird_core::biomeos::universal_adapter::BiomeOSCapabilityProvider`
- `songbird_core::biomeos::BiomeOSIntegration` → `songbird_core::biomeos::universal_adapter::UniversalBiomeOSManager`

### **2. Initialization Pattern Changed**
- No longer need to pass configuration and orchestrator to constructor
- Endpoint discovery is automatic
- Initialization is handled by the capability provider

### **3. Method Names Changed**
- `integration.register_with_biomeos()` → `manager.register_service()`
- `client.test_connection()` → `manager.is_available()` or `manager.check_health()`
- `integration.deploy_service()` → `manager.deploy_service()`

### **4. Error Handling**
Errors are now standardized through the universal adapter system:
```rust
match manager.register_service(registration).await {
    Ok(result) => {
        // Handle successful registration
    }
    Err(SongbirdError::Service { service, message, .. }) if service == "BiomeOS" => {
        // Handle BiomeOS-specific errors
    }
    Err(e) => {
        // Handle other errors
    }
}
```

---

## 🧪 **TESTING CHANGES**

### **Unit Tests**
**OLD**:
```rust
#[tokio::test]
async fn test_biomeos_integration() {
    let client = BiomeOSClient::new("http://test:4000".to_string());
    // ... test logic
}
```

**NEW**:
```rust
#[tokio::test]
async fn test_biomeos_universal() {
    let manager = UniversalBiomeOSManager::new().await.unwrap();
    // ... test logic (same structure)
}
```

### **Mock Testing**
The universal adapter provides better testing support:
```rust
#[tokio::test]
async fn test_biomeos_without_endpoint() {
    let provider = BiomeOSCapabilityProvider::new("test-biomeos".to_string());
    let connected = provider.test_connection().await.unwrap();
    assert!(!connected); // Graceful handling when BiomeOS unavailable
}
```

---

## 🚀 **BENEFITS OF MIGRATION**

### **1. Architectural Consistency**
- BiomeOS now follows the same universal pattern as all other primals
- No special-case hardcoded integration logic

### **2. Better Discovery**
- Automatic endpoint discovery through universal system
- Fallback to environment variables for backward compatibility

### **3. Improved Error Handling**
- Standardized error types through universal adapter
- Better error messages and recovery suggestions

### **4. Enhanced Testing**
- Easier to mock and test
- Better separation of concerns

### **5. Future-Proof**
- Ready for dynamic primal discovery
- Compatible with universal primal SDK patterns

---

## 📅 **MIGRATION TIMELINE**

### **Phase 1: Immediate (Current)**
- Universal adapter available alongside deprecated modules
- All existing code continues to work with deprecation warnings

### **Phase 2: Transition Period (Next 2-4 weeks)**
- Update your code to use universal adapter
- Test thoroughly in development environments

### **Phase 3: Deprecation Removal (Future release)**
- Hardcoded modules will be removed
- Only universal adapter approach will be available

---

## 🆘 **MIGRATION SUPPORT**

### **Gradual Migration**
You can migrate incrementally:
1. Start with new code using universal adapter
2. Gradually update existing code module by module
3. Both approaches work during transition period

### **Compatibility Layer**
A compatibility layer is available for complex migrations:
```rust
use songbird_core::biomeos::universal_adapter::UniversalBiomeOSManager;

// Compatibility wrapper for legacy code
impl BiomeOSIntegration {
    pub async fn from_universal_manager(manager: UniversalBiomeOSManager) -> Self {
        // Wrapper implementation for gradual migration
    }
}
```

### **Example Migration**
See `examples/biomeos_universal_migration.rs` for a complete migration example.

---

**Questions?** Check the universal adapter documentation or create an issue for migration support. 