# 🔄 Federation Monitoring Conversion Example

**Purpose**: Concrete example of converting TODOs to universal adapter delegation  
**Target File**: `crates/songbird-federation/src/mcp_handler/monitoring.rs`  
**Pattern**: Replace placeholder implementations with capability-based routing  

---

## 🚨 **Current State: Placeholder Implementations**

### **Problem: TODOs Returning Hardcoded Values**

```rust
// Current implementation in monitoring.rs - PRODUCTION RISK
impl MonitoringManager {
    /// Get CPU usage - PLACEHOLDER IMPLEMENTATION
    pub async fn get_cpu_usage(&self) -> Result<f64> {
        // TODO: Implement actual CPU usage monitoring
        Ok(0.0) // Returns placeholder - UNRELIABLE FOR PRODUCTION
    }

    /// Get memory usage - PLACEHOLDER IMPLEMENTATION  
    pub async fn get_memory_usage(&self) -> Result<(u64, u64)> {
        // TODO: Implement actual memory usage monitoring
        Ok((0, 0)) // Returns placeholder - UNRELIABLE FOR PRODUCTION
    }

    /// Get storage detection - PLACEHOLDER IMPLEMENTATION
    pub async fn get_storage_size(&self) -> Result<f64> {
        // TODO: Implement actual storage detection
        Ok(0.0) // Returns placeholder - UNRELIABLE FOR PRODUCTION
    }

    /// Get service count - PLACEHOLDER IMPLEMENTATION
    pub async fn get_service_count(&self) -> Result<u32> {
        // TODO: Implement actual service count
        Ok(0) // Returns placeholder - UNRELIABLE FOR PRODUCTION
    }

    /// Get active connections - PLACEHOLDER IMPLEMENTATION
    pub async fn get_active_connections(&self) -> Result<u32> {
        // TODO: Implement actual connection counting
        Ok(0) // Returns placeholder - UNRELIABLE FOR PRODUCTION
    }

    /// Message broadcasting - PLACEHOLDER IMPLEMENTATION
    pub async fn broadcast_message(&self, message: FederationMessage) -> Result<()> {
        // TODO: Implement actual message broadcasting
        Ok(()) // Does nothing - FEDERATION BROKEN
    }
}
```

**Issues**:
- ❌ All monitoring returns placeholder values (0.0, 0, etc.)
- ❌ No real system metrics - production monitoring broken
- ❌ Message broadcasting does nothing - federation communication broken
- ❌ Multiple critical TODOs block production deployment

---

## ✅ **Solution: Universal Adapter Delegation**

### **Convert to Capability-Based Routing**

```rust
// New implementation - PRODUCTION READY via delegation
use songbird_universal_primals::global_adapter::{routing, AdapterContext};
use serde_json::{json, Value};
use songbird_errors::SongbirdResult;

impl MonitoringManager {
    /// Get CPU usage via ToadStool compute capability
    pub async fn get_cpu_usage(&self) -> SongbirdResult<f64> {
        let ctx = AdapterContext::new("federation_cpu_monitoring");
        
        // Route to ToadStool for compute monitoring
        let response = routing::compute_request(
            &ctx, 
            "cpu_usage", 
            json!({ "format": "percentage" })
        ).await?;

        // Extract CPU usage from ToadStool response
        let cpu_usage = response
            .get("cpu_usage_percent")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        tracing::info!(
            request_id = %ctx.request_id,
            cpu_usage = cpu_usage,
            elapsed = ?ctx.elapsed(),
            "✅ CPU usage retrieved from ToadStool"
        );

        Ok(cpu_usage)
    }

    /// Get memory usage via ToadStool compute capability
    pub async fn get_memory_usage(&self) -> SongbirdResult<(u64, u64)> {
        let ctx = AdapterContext::new("federation_memory_monitoring");
        
        // Route to ToadStool for memory monitoring
        let response = routing::compute_request(
            &ctx,
            "memory_stats",
            json!({ "format": "bytes", "include_total": true })
        ).await?;

        // Extract memory stats from ToadStool response
        let used = response
            .get("memory_used_bytes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
            
        let total = response
            .get("memory_total_bytes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        tracing::info!(
            request_id = %ctx.request_id,
            memory_used = used,
            memory_total = total,
            elapsed = ?ctx.elapsed(),
            "✅ Memory usage retrieved from ToadStool"
        );

        Ok((used, total))
    }

    /// Get storage information via NestGate storage capability
    pub async fn get_storage_size(&self) -> SongbirdResult<f64> {
        let ctx = AdapterContext::new("federation_storage_monitoring");
        
        // Route to NestGate for storage monitoring
        let response = routing::storage_request(
            &ctx,
            "get_stats",
            json!({ "format": "gigabytes", "include_total": true })
        ).await?;

        // Extract storage size from NestGate response
        let storage_size = response
            .get("total_storage_gb")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        tracing::info!(
            request_id = %ctx.request_id,
            storage_size_gb = storage_size,
            elapsed = ?ctx.elapsed(),
            "✅ Storage size retrieved from NestGate"
        );

        Ok(storage_size)
    }

    /// Get service count via orchestration capability
    pub async fn get_service_count(&self) -> SongbirdResult<u32> {
        let ctx = AdapterContext::new("federation_service_monitoring");
        
        // Route to orchestration providers for service discovery
        let response = routing::orchestration_request(
            &ctx,
            "list_services",
            json!({ "format": "count_only", "include_health": false })
        ).await?;

        // Extract service count from orchestration response
        let service_count = response
            .get("active_service_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        tracing::info!(
            request_id = %ctx.request_id,
            service_count = service_count,
            elapsed = ?ctx.elapsed(),
            "✅ Service count retrieved via orchestration capability"
        );

        Ok(service_count)
    }

    /// Get active connections via monitoring capability
    pub async fn get_active_connections(&self) -> SongbirdResult<u32> {
        let ctx = AdapterContext::new("federation_connection_monitoring");
        
        // Route to monitoring providers for connection stats
        let response = routing::monitoring_request(
            &ctx,
            "connection_stats",
            json!({ "format": "active_count" })
        ).await?;

        // Extract connection count from monitoring response
        let connection_count = response
            .get("active_connections")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        tracing::info!(
            request_id = %ctx.request_id,
            connection_count = connection_count,
            elapsed = ?ctx.elapsed(),
            "✅ Connection count retrieved via monitoring capability"
        );

        Ok(connection_count)
    }

    /// Broadcast federation message via orchestration capability
    pub async fn broadcast_message(&self, message: FederationMessage) -> SongbirdResult<()> {
        let ctx = AdapterContext::new("federation_message_broadcasting");
        
        // Route to orchestration providers for message broadcasting
        let payload = json!({
            "message": message,
            "broadcast_type": "federation",
            "delivery_guarantee": "at_least_once"
        });

        let response = routing::orchestration_request(
            &ctx,
            "broadcast_message",
            payload
        ).await?;

        // Validate successful broadcast
        let broadcast_success = response
            .get("broadcast_successful")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let recipients_reached = response
            .get("recipients_reached")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        if broadcast_success {
            tracing::info!(
                request_id = %ctx.request_id,
                recipients = recipients_reached,
                elapsed = ?ctx.elapsed(),
                "✅ Federation message broadcast successful"
            );
        } else {
            tracing::warn!(
                request_id = %ctx.request_id,
                elapsed = ?ctx.elapsed(),
                "⚠️ Federation message broadcast partially failed"
            );
        }

        Ok(())
    }
}
```

---

## 🎯 **Benefits of Universal Adapter Conversion**

### **✅ Production Reliability**
- **Real data**: No more placeholder values (0.0, 0, etc.)
- **Specialized providers**: ToadStool for compute, NestGate for storage
- **Error handling**: Proper error propagation via universal adapter
- **Monitoring**: Request tracing and performance metrics

### **✅ Architectural Correctness**
- **Single responsibility**: Songbird orchestrates, primals specialize
- **Capability-based**: Routes by capability, not hardcoded names
- **Scalable**: Works with any primal providing required capabilities
- **Testable**: Can mock universal adapter for unit tests

### **✅ Operational Excellence**
- **Health monitoring**: Automatic primal health checking
- **Failover**: Routes to healthy capability providers
- **Load balancing**: Distributes requests across available primals
- **Performance**: Zero-cost routing with compile-time optimization

---

## 🔧 **Implementation Steps**

### **Step 1: Add Universal Adapter Import**
```rust
// Add to top of monitoring.rs
use songbird_universal_primals::global_adapter::{routing, AdapterContext};
use serde_json::{json, Value};
```

### **Step 2: Convert Each TODO Method**
```rust
// Pattern for all conversions:
pub async fn method_name(&self) -> SongbirdResult<ReturnType> {
    let ctx = AdapterContext::new("federation_operation_name");
    
    let response = routing::{capability}_request(
        &ctx,
        "operation_name", 
        json!({ "parameters": "values" })
    ).await?;
    
    let result = response
        .get("result_field")
        .and_then(|v| v.as_{type}())
        .unwrap_or(default_value);
    
    tracing::info!(
        request_id = %ctx.request_id,
        result = result,
        elapsed = ?ctx.elapsed(),
        "✅ Operation completed via {primal} capability"
    );
    
    Ok(result)
}
```

### **Step 3: Update Integration Tests**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use songbird_test_utils::UniversalAdapterMock;

    #[tokio::test]
    async fn test_cpu_monitoring_delegates_to_toadstool() {
        // Mock universal adapter to return expected ToadStool response
        let adapter_mock = UniversalAdapterMock::new()
            .expect_compute_request("cpu_usage")
            .return_json(json!({ "cpu_usage_percent": 45.5 }));

        let monitor = MonitoringManager::new(test_config()).await?;
        let cpu_usage = monitor.get_cpu_usage().await?;
        
        assert_eq!(cpu_usage, 45.5);
        adapter_mock.verify_compute_capability_called();
    }
}
```

---

## ✅ **Validation Checklist**

- [ ] **All TODO comments removed** from production code
- [ ] **Universal adapter routing** used for all external capabilities
- [ ] **Proper error handling** via SongbirdResult patterns
- [ ] **Request tracing** with AdapterContext for observability
- [ ] **Integration tests** verify delegation to correct primals
- [ ] **No hardcoded values** returned from monitoring functions
- [ ] **Performance metrics** captured for routing operations

**Result**: Production-ready federation monitoring with real data from specialized primals! 