# 🔄 reqwest Migration Guide - Discovery Backends

**Target**: `songbird-config` and `songbird-discovery` crates  
**Timeline**: Week 3-4 (16 hours estimated)  
**Priority**: P0 (Critical for TRUE ecoBin)

---

## 📋 Migration Checklist

### Phase 1: `songbird-config` (Week 3)

- [ ] **File 1**: `capability_discovery.rs` (1 usage, line 394)
  - Pattern: Health check HTTP call
  - Est: 30 minutes
  
- [ ] **File 2**: `capability_endpoints.rs` (2 usages, lines 267, 340)
  - Pattern: Endpoint validation
  - Est: 45 minutes

- [ ] **File 3**: `capability_based_runtime_discovery/service_registry.rs` (1 usage, line 110)
  - Pattern: Service registry query
  - Est: 30 minutes

### Phase 2: `songbird-discovery` (Week 3-4)

- [ ] **File 4**: `discovery/backends/service_discovery.rs` (2 usages, lines 270, 389)
  - Pattern: Service health checks
  - Est: 1 hour

- [ ] **File 5**: `discovery/backends/container_orchestration.rs` (1 usage, line 400)
  - Pattern: Container API calls
  - Est: 45 minutes

- [ ] **File 6**: `abstraction/adapters/consul_adapter.rs` (2 usages)
  - Pattern: Consul HTTP API
  - Est: 1 hour

- [ ] **File 7**: `abstraction/adapters/kubernetes_adapter.rs` (1 usage)
  - Pattern: K8s API calls
  - Est: 1 hour

- [ ] **File 8**: `production/real_service_discovery.rs` (1 usage, line 168)
  - Pattern: Production discovery
  - Est: 45 minutes

- [ ] **File 9**: `agnostic_service_mesh.rs` (1 usage, line 387)
  - Pattern: Service mesh integration
  - Est: 45 minutes

---

## 🔧 Migration Pattern

### Before (reqwest - C dependencies)

```rust
use reqwest::Client;

async fn discover_service(url: &str) -> Result<ServiceInfo> {
    let client = Client::new();
    let response = client.get(url).send().await?;
    let info: ServiceInfo = response.json().await?;
    Ok(info)
}
```

### After (IpcHttpClient - Pure Rust)

```rust
use songbird_http_client::IpcHttpClient;

async fn discover_service(url: &str) -> Result<ServiceInfo> {
    let client = IpcHttpClient::new().await?;
    let response = client.get(url).await?;
    let info: ServiceInfo = response.json().await?;
    Ok(info)
}
```

**Change**: Just swap `Client::new()` → `IpcHttpClient::new().await?`

---

## 📝 Detailed Migration: Example File

### File: `songbird-config/src/capability_discovery.rs`

#### Current Code (Line 394)

```rust
pub async fn check_service_health(endpoint: &str) -> Result<bool> {
    let client = reqwest::Client::new();  // ❌ C dependency
    
    match client.get(format!("{}/health", endpoint)).send().await {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}
```

#### Migrated Code

```rust
use songbird_http_client::IpcHttpClient;

pub async fn check_service_health(endpoint: &str) -> Result<bool> {
    let client = IpcHttpClient::new().await?;  // ✅ Pure Rust via IPC
    
    match client.get(format!("{}/health", endpoint)).await {
        Ok(response) => Ok(response.status() == 200),
        Err(_) => Ok(false),
    }
}
```

#### Changes Made
1. Import changed: `reqwest::Client` → `IpcHttpClient`
2. Client creation: `.new()` → `.new().await?`
3. Response method: `.status().is_success()` → `.status() == 200`

#### Testing
```bash
cargo test -p songbird-config check_service_health
```

---

## 🧪 Testing Strategy

### 1. Unit Tests (Per File)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running Songbird instance
    async fn test_ipc_http_discovery() {
        let client = IpcHttpClient::new().await.unwrap();
        let response = client.get("https://httpbin.org/get").await.unwrap();
        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn test_socket_discovery() {
        // Test socket path discovery logic
        std::env::set_var("SONGBIRD_SOCKET", "/tmp/test.sock");
        let path = IpcHttpClient::discover_socket_path().unwrap();
        assert_eq!(path.to_str().unwrap(), "/tmp/test.sock");
        std::env::remove_var("SONGBIRD_SOCKET");
    }
}
```

### 2. Integration Tests

```bash
# Start Songbird in background
songbird server --daemon &

# Run discovery tests
cargo test -p songbird-config --test integration_discovery
cargo test -p songbird-discovery --test integration_service_discovery

# Stop Songbird
pkill songbird
```

### 3. Performance Comparison

```rust
// Benchmark: reqwest vs IpcHttpClient
#[bench]
fn bench_reqwest(b: &mut Bencher) {
    b.iter(|| {
        // reqwest HTTP call
    });
}

#[bench]
fn bench_ipc_http(b: &mut Bencher) {
    b.iter(|| {
        // IpcHttpClient call
    });
}
```

**Expected Overhead**: <5% (IPC + JSON serialization)

---

## 🚀 Step-by-Step Execution

### Day 1: Setup & First File (4 hours)

```bash
# 1. Ensure Songbird compiles with new module
cd /home/eastgate/Development/ecoPrimals/phase1/songbird
cargo build -p songbird-http-client

# 2. Run IpcHttpClient tests
cargo test -p songbird-http-client test_socket_discovery

# 3. Start Songbird for integration testing
songbird server --daemon

# 4. Migrate capability_discovery.rs
vim crates/songbird-config/src/capability_discovery.rs
# Replace reqwest::Client with IpcHttpClient

# 5. Test
cargo test -p songbird-config capability_discovery
```

### Day 2: Config Crate Complete (4 hours)

```bash
# 6. Migrate capability_endpoints.rs
vim crates/songbird-config/src/capability_endpoints.rs

# 7. Migrate service_registry.rs
vim crates/songbird-config/src/capability_based_runtime_discovery/service_registry.rs

# 8. Full config crate test
cargo test -p songbird-config --all-features

# 9. Verify no reqwest in songbird-config
cargo tree -p songbird-config -i reqwest
# Expected: empty result
```

### Day 3-4: Discovery Crate (8 hours)

```bash
# 10. Migrate discovery backends (6 files)
vim crates/songbird-discovery/src/discovery/backends/*.rs
vim crates/songbird-discovery/src/abstraction/adapters/*.rs

# 11. Test each file
cargo test -p songbird-discovery --test service_discovery
cargo test -p songbird-discovery --test container_orchestration

# 12. Integration test
cargo test -p songbird-discovery --all-features

# 13. Verify no reqwest in songbird-discovery
cargo tree -p songbird-discovery -i reqwest
# Expected: empty result
```

---

## 📊 Progress Tracking

### Week 3 Goals

| Day | Files Migrated | Tests Passing | Hours |
|-----|----------------|---------------|-------|
| Mon | capability_discovery.rs | ✅ | 4 |
| Tue | capability_endpoints.rs + service_registry.rs | ✅ | 4 |
| Wed | service_discovery.rs + container_orchestration.rs | ✅ | 4 |
| Thu | consul_adapter.rs + kubernetes_adapter.rs | ✅ | 4 |
| Fri | real_service_discovery.rs + agnostic_service_mesh.rs | ✅ | 4 |

**Total**: 9 files, 20 hours, ~90% of discovery HTTP calls migrated

---

## ⚠️ Common Issues & Solutions

### Issue 1: Socket Not Found

**Error**: `Failed to connect to Songbird IPC socket`

**Solution**:
```bash
# Check if Songbird is running
ps aux | grep songbird

# Start Songbird if needed
songbird server --daemon

# Check socket exists
ls -l /tmp/songbird-*.sock
```

### Issue 2: Response Format Mismatch

**Error**: `Response parsing failed`

**Solution**:
```rust
// IpcHttpClient returns our Response type
let response = client.get(url).await?;

// For JSON, use .json()
let data: MyType = response.json().await?;

// For text, use .text()
let text = response.text().await?;
```

### Issue 3: Async/Await Changes

**Error**: `Cannot call async function`

**Solution**:
```rust
// Old (sync-ish)
let client = reqwest::Client::new();

// New (properly async)
let client = IpcHttpClient::new().await?;
```

---

## 🎯 Success Criteria

### Per File
- [ ] reqwest import removed
- [ ] IpcHttpClient import added
- [ ] All tests passing
- [ ] No compilation errors
- [ ] Documentation updated

### Per Crate
- [ ] `cargo tree -i reqwest` returns empty
- [ ] All integration tests passing
- [ ] Performance acceptable (<5% overhead)
- [ ] Documentation PR merged

### Overall (Week 3 End)
- [ ] 9 discovery files migrated
- [ ] 100% tests passing
- [ ] Discovery subsystem Pure Rust
- [ ] Metrics dashboard updated

---

## 📚 Resources

### Code References
- `IpcHttpClient`: `crates/songbird-http-client/src/ipc_client.rs`
- Migration examples: This document
- Test examples: `ipc_client.rs` tests

### Documentation
- Tower Atomic pattern: `docs/tower-atomic-pattern/`
- IPC protocol: `wateringHole/PRIMAL_IPC_PROTOCOL.md`
- Evolution plan: `REQWEST_ELIMINATION_EVOLUTION_PLAN.md`

### Support
- Questions: wateringHole discussions
- Issues: Create GitHub issue with `reqwest-migration` label
- Reviews: Request PR review from team

---

**Migration Guide Version**: 1.0  
**Last Updated**: January 25, 2026  
**Status**: Ready for execution  
**Next**: Start Day 1 migration

🦀🧬✨ **Let's eliminate reqwest and achieve TRUE ecoBin!** ✨🧬🦀

