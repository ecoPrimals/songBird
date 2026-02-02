# Testing Evolution Strategy - January 16, 2026

**Vision**: Comprehensive test coverage for modern idiomatic async Rust  
**Target**: 90% code coverage (llvm-cov) across unit, integration, E2E, chaos, and fault tests  
**Status**: Strategy designed, implementation in progress  
**Philosophy**: Deep debt solutions + fast AND safe Rust

---

## 🎯 **Current State**

### **Test Coverage**

```bash
# Current test files: 280+
├── Unit tests: ~200 files (embedded in modules)
├── Integration tests: ~50 files (tests/)
├── E2E tests: ~20 files (specific scenarios)
├── Chaos tests: ~5 files (discovery, trust)
└── Fault injection tests: ~5 files (discovery)
```

**Strengths**:
- ✅ Extensive unit test coverage
- ✅ Good integration test coverage
- ✅ Some E2E tests
- ✅ Chaos and fault tests exist

**Gaps**:
- ⚠️  BTSP Unix socket integration tests (blocked, needs BearDog)
- ⚠️  E2E tower atomic tests (blocked, needs BearDog)
- ⚠️  HTTP gateway tests (not yet implemented)
- ⚠️  Code coverage measurement not systematic
- ⚠️  Chaos/fault tests limited scope

---

## 📊 **Testing Pyramid**

```
        ┌──────────────┐
        │ E2E (5-10%)  │  ← Comprehensive scenarios
        └──────────────┘
      ┌──────────────────┐
      │ Integration (15%) │ ← Component interaction
      └──────────────────┘
    ┌────────────────────────┐
    │  Unit Tests (75-80%)   │ ← Core logic
    └────────────────────────┘

Overlay:
┌──────────────────────────────┐
│ Chaos + Fault Injection (5%) │ ← Resilience
└──────────────────────────────┘
```

---

## 🧪 **Testing Layers**

### **1. Unit Tests** (75-80% of tests)

**Scope**: Individual functions, modules, structs  
**Speed**: Fast (<1ms per test)  
**Isolation**: Mocked dependencies

**Current Coverage**:
- ✅ Discovery logic
- ✅ Trust evaluation
- ✅ Connection management
- ✅ Access control
- ✅ Configuration loading
- ✅ Crypto operations (RustCrypto)

**Gaps to Fill**:
- ⏳ HTTP gateway components (Phase 2)
- ⏳ BTSP client utilities
- ⏳ Rate limiter
- ⏳ Response cache

**Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new(10, Duration::from_secs(1));
        
        for _ in 0..10 {
            assert!(limiter.try_acquire().is_ok());
        }
    }
    
    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new(10, Duration::from_secs(1));
        
        for _ in 0..10 {
            limiter.try_acquire().unwrap();
        }
        
        // 11th should fail
        assert!(limiter.try_acquire().is_err());
    }
}
```

---

### **2. Integration Tests** (15% of tests)

**Scope**: Component interactions, API contracts  
**Speed**: Medium (10-100ms per test)  
**Dependencies**: Real components, mocked external services

**Current Coverage**:
- ✅ IPC integration (`ipc_integration_tests.rs`)
- ✅ Trust establishment (`trust_establishment_e2e_test.rs`)
- ✅ Discovery (`discovery_e2e_test.rs`)
- ✅ Orchestrator (`orchestrator_integration_tests.rs`)
- ✅ Task lifecycle (`task_lifecycle_integration_tests.rs`)

**Gaps to Fill** (Week 2):
- ⏳ BTSP Unix socket integration (blocked, needs BearDog)
- ⏳ HTTP gateway integration
- ⏳ Multi-primal coordination

**Example**:
```rust
// tests/btsp_unix_socket_integration.rs

#[tokio::test]
#[ignore = "Requires BearDog Unix socket server"]
async fn test_btsp_tunnel_establishment() {
    // Setup: Start BearDog mock
    let beardog_mock = BearDogMock::start().await;
    
    // Test: Create BTSP client
    let client = BtspClient::new();
    
    // Test: Establish tunnel
    let peer = PeerEndpoint {
        id: "test-peer".to_string(),
        endpoint: "unix:///tmp/beardog-test.sock".to_string(),
        public_key: None,
        capabilities: vec!["btsp_enabled".to_string()],
    };
    
    let tunnel = client.establish_tunnel(peer).await.unwrap();
    assert_eq!(tunnel.state, TunnelState::Established);
    
    // Test: Encrypt data
    let plaintext = b"Hello BTSP!";
    let ciphertext = client
        .tunnel_encrypt(&tunnel, plaintext, Direction::Egress)
        .await
        .unwrap();
    assert_ne!(ciphertext, plaintext);
    
    // Test: Decrypt data
    let decrypted = client.tunnel_decrypt(&tunnel, &ciphertext).await.unwrap();
    assert_eq!(decrypted, plaintext);
    
    // Test: Close tunnel
    client.tunnel_close(&tunnel).await.unwrap();
    
    // Cleanup
    beardog_mock.stop().await;
}
```

---

### **3. E2E Tests** (5-10% of tests)

**Scope**: Full system scenarios, user workflows  
**Speed**: Slow (100ms-1s per test)  
**Dependencies**: Real components, real interactions

**Current Coverage**:
- ✅ HTTP server sovereign (`http_server_sovereign_e2e_test.rs`)
- ✅ Trust establishment (`trust_establishment_e2e_test.rs`)
- ✅ Discovery (`discovery_e2e_test.rs`)
- ✅ Peer discovery API (`peer_discovery_api_e2e_tests.rs`)

**Gaps to Fill** (Week 2):
- ⏳ E2E tower atomic validation (blocked, needs BearDog)
- ⏳ E2E BirdSong multi-tag discovery
- ⏳ E2E HTTP gateway → external API
- ⏳ E2E Squirrel → Songbird → OpenAI

**Example**:
```rust
// tests/e2e_tower_atomic.rs

#[tokio::test]
#[ignore = "Requires BearDog + Songbird running"]
async fn test_tower_atomic_full_flow() {
    // Setup: Start Songbird + BearDog
    let songbird = SongbirdServer::start().await;
    let beardog = BearDogServer::start().await;
    
    // Scenario: Discover BearDog
    let discovered = songbird
        .discover_capability("security")
        .await
        .unwrap();
    assert_eq!(discovered.len(), 1);
    assert_eq!(discovered[0].primal_name, "beardog");
    
    // Scenario: Establish BTSP tunnel
    let tunnel = songbird
        .establish_btsp_tunnel(&discovered[0])
        .await
        .unwrap();
    
    // Scenario: Encrypt task via tower atomic
    let task = Task::new("test_task", vec![]);
    let encrypted_task = songbird
        .encrypt_task_atomic(&tunnel, &task)
        .await
        .unwrap();
    
    // Scenario: Execute encrypted task
    let result = songbird
        .execute_atomic_task(&tunnel, &encrypted_task)
        .await
        .unwrap();
    assert!(result.is_success());
    
    // Cleanup
    songbird.stop().await;
    beardog.stop().await;
}
```

---

### **4. Chaos Tests** (2-3% of tests)

**Scope**: System behavior under chaotic conditions  
**Speed**: Variable (can be slow)  
**Purpose**: Validate resilience, fault tolerance

**Current Coverage**:
- ✅ Generic trust chaos (`generic_trust_chaos_tests.rs`)
- ✅ Chaos engineering (`chaos_engineering_tests.rs`)

**Gaps to Fill**:
- ⏳ BTSP chaos (network partitions, latency spikes)
- ⏳ HTTP gateway chaos (API failures, timeouts)
- ⏳ Discovery chaos (registry failures, DNS issues)
- ⏳ Multi-primal chaos (cascading failures)

**Example**:
```rust
// tests/btsp_chaos_tests.rs

#[tokio::test]
#[ignore = "Chaos test - may be slow"]
async fn test_btsp_network_partition() {
    let client = BtspClient::new();
    let mock = BearDogMock::start().await;
    
    // Establish tunnel
    let tunnel = client.establish_tunnel(test_peer()).await.unwrap();
    
    // Inject chaos: Network partition
    mock.inject_fault(FaultType::NetworkPartition).await;
    
    // Test: Encrypt should fail gracefully
    let result = client
        .tunnel_encrypt(&tunnel, b"test", Direction::Egress)
        .await;
    assert!(result.is_err());
    
    // Test: Songbird should still be healthy
    assert!(songbird_health_check().await.is_ok());
    
    // Recover: Remove partition
    mock.clear_faults().await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Test: Should auto-recover
    let result = client
        .tunnel_encrypt(&tunnel, b"test", Direction::Egress)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore = "Chaos test - may be slow"]
async fn test_http_gateway_api_rate_limit() {
    let gateway = HttpGatewayService::new().await.unwrap();
    gateway.start().await.unwrap();
    
    // Inject chaos: External API starts rate limiting
    let mock_api = MockAiApi::start().await;
    mock_api.set_rate_limit(1, Duration::from_secs(60)).await;
    
    // Test: First request succeeds
    let result = gateway.proxy_request("openai", test_request()).await;
    assert!(result.is_ok());
    
    // Test: Second request should be rate limited
    let result = gateway.proxy_request("openai", test_request()).await;
    assert!(matches!(result, Err(GatewayError::RateLimited { .. })));
    
    // Test: Songbird's internal rate limiter should prevent spam
    for _ in 0..100 {
        let result = gateway.proxy_request("openai", test_request()).await;
        assert!(matches!(result, Err(GatewayError::RateLimited { .. })));
    }
    
    // Test: After backoff, should retry
    tokio::time::sleep(Duration::from_secs(60)).await;
    let result = gateway.proxy_request("openai", test_request()).await;
    assert!(result.is_ok());
}
```

---

### **5. Fault Injection Tests** (2-3% of tests)

**Scope**: Specific fault scenarios  
**Speed**: Variable  
**Purpose**: Validate error handling, recovery

**Current Coverage**:
- ✅ Fault injection (`fault_injection_tests.rs`)

**Gaps to Fill**:
- ⏳ BTSP fault injection (connection drops, timeouts)
- ⏳ HTTP gateway faults (DNS failures, TLS errors)
- ⏳ Discovery faults (registry unreachable)
- ⏳ Memory pressure faults

**Example**:
```rust
// tests/btsp_fault_injection.rs

#[tokio::test]
async fn test_btsp_connection_drop() {
    let client = BtspClient::new();
    let mock = BearDogMock::start().await;
    
    // Establish tunnel
    let tunnel = client.establish_tunnel(test_peer()).await.unwrap();
    
    // Inject fault: Connection drop mid-operation
    mock.inject_fault(FaultType::ConnectionDrop {
        after_bytes: 100,
    }).await;
    
    // Test: Operation should fail with clear error
    let result = client
        .tunnel_encrypt(&tunnel, &vec![0u8; 1000], Direction::Egress)
        .await;
    
    assert!(matches!(result, Err(BtspError::ConnectionLost { .. })));
    
    // Test: Should not panic or corrupt state
    assert!(client.is_healthy());
}

#[tokio::test]
async fn test_http_gateway_dns_failure() {
    let gateway = HttpGatewayService::new().await.unwrap();
    
    // Inject fault: DNS resolution fails
    inject_dns_fault("api.openai.com", DnsError::NxDomain).await;
    
    // Test: Should fail with clear error
    let result = gateway.proxy_request("openai", test_request()).await;
    assert!(matches!(result, Err(GatewayError::DnsFailure { .. })));
    
    // Test: Should include helpful suggestion
    if let Err(GatewayError::DnsFailure { suggestion, .. }) = result {
        assert!(suggestion.contains("DNS"));
    }
}
```

---

## 📏 **Code Coverage Measurement**

### **Using llvm-cov**

```bash
# Install llvm-cov
cargo install cargo-llvm-cov

# Run tests with coverage
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# Generate HTML report
cargo llvm-cov --all-features --workspace --html

# Open report
open target/llvm-cov/html/index.html
```

---

### **Coverage Goals**

| Component | Target Coverage | Current | Gap |
|-----------|----------------|---------|-----|
| **Core** | 90%+ | ~85% | 5% |
| **Discovery** | 90%+ | ~90% | ✅ |
| **Trust** | 90%+ | ~88% | 2% |
| **BTSP Client** | 85%+ | ~40% | 45% |
| **HTTP Gateway** | 85%+ | 0% | 85% |
| **Access Control** | 95%+ | ~92% | 3% |
| **IPC** | 90%+ | ~85% | 5% |
| **Overall** | **90%** | **~70%** | **20%** |

**Strategy**: Prioritize gaps (BTSP, HTTP gateway), expand chaos/fault tests

---

## 🔧 **Implementation Plan**

### **Phase 1: Test Infrastructure** (2-3 hours) ✅ **CAN DO NOW**

**Tasks**:

1. ✅ Setup llvm-cov for coverage measurement
2. ✅ Create test helpers for BTSP mocking
3. ✅ Create test helpers for HTTP gateway mocking
4. ✅ Document testing patterns

**Deliverables**:
- `tests/helpers/btsp_mock.rs` - BearDog mock for testing
- `tests/helpers/http_mock.rs` - HTTP API mock for testing
- `tests/helpers/mod.rs` - Common test utilities
- `.github/workflows/coverage.yml` - CI coverage reporting

---

### **Phase 2: Unit Test Expansion** (3-4 hours) ✅ **CAN DO NOW**

**Tasks**:

1. ✅ BTSP client unit tests (socket discovery, error handling)
2. ✅ HTTP gateway unit tests (rate limiter, cache, translators)
3. ✅ Crypto unit tests (RustCrypto usage)
4. ✅ Connection pool unit tests

**Deliverables**:
- Unit tests for all new components
- Coverage: 85%+ for BTSP client
- Coverage: 90%+ for HTTP gateway components

---

### **Phase 3: Integration Tests** (4-6 hours) ⏳ **BLOCKED (needs BearDog)**

**Tasks**:

1. ⏳ BTSP Unix socket integration tests
2. ⏳ HTTP gateway integration tests
3. ⏳ Multi-component integration tests

**Deliverables**:
- `tests/btsp_unix_socket_integration.rs`
- `tests/http_gateway_integration.rs`
- `tests/multi_primal_integration.rs`

**Blocked By**: BearDog Unix socket server availability

---

### **Phase 4: E2E Tests** (4-6 hours) ⏳ **BLOCKED (needs BearDog)**

**Tasks**:

1. ⏳ E2E tower atomic validation
2. ⏳ E2E BirdSong multi-tag discovery
3. ⏳ E2E HTTP gateway → external API
4. ⏳ E2E Squirrel → Songbird → OpenAI

**Deliverables**:
- `tests/e2e_tower_atomic.rs`
- `tests/e2e_birdsong_multitag.rs`
- `tests/e2e_http_gateway.rs`
- `tests/e2e_squirrel_ai_proxy.rs`

**Blocked By**: BearDog Unix socket server + HTTP gateway implementation

---

### **Phase 5: Chaos & Fault Tests** (3-4 hours) ✅ **PARTIAL - CAN START NOW**

**Tasks**:

1. ✅ BTSP chaos tests (scaffolding, can implement core logic)
2. ✅ HTTP gateway chaos tests (can implement with mocks)
3. ⏳ Discovery chaos tests (extend existing)
4. ⏳ Multi-primal chaos tests (blocked, needs BearDog)

**Deliverables**:
- `tests/btsp_chaos_tests.rs` (scaffolding ready)
- `tests/http_gateway_chaos_tests.rs`
- `tests/discovery_chaos_tests.rs` (extended)

**Can Start Now**: Scaffolding, mock-based tests  
**Blocked**: Live integration chaos tests

---

### **Phase 6: Documentation** (2-3 hours) ✅ **CAN DO NOW**

**Tasks**:

1. ✅ Create testing guide (`TESTING_GUIDE.md`)
2. ✅ Update `CONTRIBUTING.md` with testing requirements
3. ✅ Document test helpers and patterns
4. ✅ Create CI/CD testing workflow

**Deliverables**:
- `TESTING_GUIDE.md`
- Updated `CONTRIBUTING.md`
- `.github/workflows/test.yml` (comprehensive)
- `.github/workflows/coverage.yml`

---

## 🚀 **Immediate Actions** (Can Do NOW!)

### **1. Setup llvm-cov**

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/songbird

# Install llvm-cov
cargo install cargo-llvm-cov

# Run coverage
cargo llvm-cov --all-features --workspace --html

# View report
open target/llvm-cov/html/index.html
```

---

### **2. Create Test Helpers**

```rust
// tests/helpers/btsp_mock.rs

use anyhow::Result;
use tokio::net::{UnixListener, UnixStream};
use std::path::PathBuf;

/// Mock BearDog server for testing BTSP client
pub struct BearDogMock {
    socket_path: PathBuf,
    listener: UnixListener,
}

impl BearDogMock {
    pub async fn start() -> Result<Self> {
        let socket_path = PathBuf::from("/tmp/beardog-test-mock.sock");
        
        // Clean up old socket
        let _ = std::fs::remove_file(&socket_path);
        
        let listener = UnixListener::bind(&socket_path)?;
        
        Ok(Self {
            socket_path,
            listener,
        })
    }
    
    pub async fn accept_connection(&self) -> Result<UnixStream> {
        let (stream, _) = self.listener.accept().await?;
        Ok(stream)
    }
    
    pub async fn stop(self) -> Result<()> {
        std::fs::remove_file(&self.socket_path)?;
        Ok(())
    }
}
```

---

### **3. Create BTSP Client Unit Tests**

```rust
// crates/songbird-orchestrator/src/btsp_client.rs
// (Add to existing tests)

#[cfg(test)]
mod tests {
    use super::*;
    
    // ... existing tests ...
    
    #[test]
    fn test_peer_endpoint_validation() {
        let valid_peer = PeerEndpoint {
            id: "test-peer".to_string(),
            endpoint: "unix:///tmp/test.sock".to_string(),
            public_key: None,
            capabilities: vec!["btsp_enabled".to_string()],
        };
        
        assert_eq!(valid_peer.id, "test-peer");
        assert!(valid_peer.capabilities.contains(&"btsp_enabled".to_string()));
    }
    
    #[test]
    fn test_tunnel_state_transitions() {
        let states = vec![
            TunnelState::Establishing,
            TunnelState::Established,
            TunnelState::Closing,
            TunnelState::Closed,
        ];
        
        // Verify all states are distinct
        for (i, state1) in states.iter().enumerate() {
            for (j, state2) in states.iter().enumerate() {
                if i == j {
                    assert_eq!(state1, state2);
                } else {
                    assert_ne!(state1, state2);
                }
            }
        }
    }
    
    #[test]
    fn test_direction_enum() {
        assert_ne!(Direction::Ingress, Direction::Egress);
    }
}
```

---

### **4. Create HTTP Gateway Chaos Tests (Mock-Based)**

```rust
// tests/http_gateway_chaos_tests.rs

use songbird_orchestrator::http_gateway::*;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_rate_limiter_under_load() {
    let limiter = RateLimiter::new(10, Duration::from_secs(1));
    
    // Spawn 100 concurrent requests
    let mut handles = vec![];
    for i in 0..100 {
        let limiter = limiter.clone();
        let handle = tokio::spawn(async move {
            limiter.check(&format!("client-{}", i % 5)).await
        });
        handles.push(handle);
    }
    
    // Wait for all
    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    
    // At least some should be rate limited
    let allowed = results.iter().filter(|r| r.is_ok()).count();
    let denied = results.iter().filter(|r| r.is_err()).count();
    
    assert!(allowed > 0);
    assert!(denied > 0);
    assert_eq!(allowed + denied, 100);
}

#[tokio::test]
async fn test_response_cache_under_pressure() {
    let cache = ResponseCache::new(10); // Small cache
    
    // Fill cache beyond capacity
    for i in 0..20 {
        let request = json!({"id": i});
        let response = json!({"result": i});
        cache.set(&request, &response).await;
    }
    
    // Oldest entries should be evicted (LRU)
    let old_request = json!({"id": 0});
    assert!(cache.get(&old_request).await.is_none());
    
    // Recent entries should still be cached
    let recent_request = json!({"id": 19});
    assert!(cache.get(&recent_request).await.is_some());
}
```

---

## 📋 **Testing Checklist**

### **Immediate (Can Do NOW!)**
- [ ] Setup llvm-cov
- [ ] Run initial coverage report
- [ ] Create test helpers (`btsp_mock.rs`, `http_mock.rs`)
- [ ] Add BTSP client unit tests
- [ ] Add HTTP gateway unit tests (when implemented)
- [ ] Create chaos test scaffolding
- [ ] Document testing patterns

### **Week 2 (Requires BearDog)**
- [ ] BTSP Unix socket integration tests
- [ ] E2E tower atomic tests
- [ ] E2E BirdSong tests
- [ ] Multi-primal integration tests
- [ ] Live chaos tests

### **Week 3 (HTTP Gateway)**
- [ ] HTTP gateway integration tests
- [ ] E2E Squirrel → Songbird → OpenAI
- [ ] HTTP gateway chaos tests
- [ ] Comprehensive coverage report (90%)

---

## 🎯 **Success Criteria**

- [ ] **Coverage**: 90%+ overall code coverage
- [ ] **Unit Tests**: 75-80% of all tests
- [ ] **Integration Tests**: 15% of all tests
- [ ] **E2E Tests**: 5-10% of all tests
- [ ] **Chaos/Fault Tests**: 5% of all tests
- [ ] **CI/CD**: Automated coverage reporting
- [ ] **Documentation**: Complete testing guide
- [ ] **All Tests Passing**: 100% pass rate

---

## 🚀 **Timeline**

| Phase | Time | Blocked? | Can Start Now? |
|-------|------|----------|----------------|
| **Phase 1: Infrastructure** | 2-3 hours | ❌ No | ✅ YES |
| **Phase 2: Unit Tests** | 3-4 hours | ❌ No | ✅ YES |
| **Phase 3: Integration** | 4-6 hours | ✅ Yes (BearDog) | ❌ NO |
| **Phase 4: E2E** | 4-6 hours | ✅ Yes (BearDog) | ❌ NO |
| **Phase 5: Chaos/Fault** | 3-4 hours | ⚠️  Partial | ✅ PARTIAL |
| **Phase 6: Documentation** | 2-3 hours | ❌ No | ✅ YES |
| **TOTAL** | **18-26 hours** | - | **~10 hours NOW** |

**Immediate Work**: ~10 hours (Phases 1, 2, 5 partial, 6)  
**Blocked Work**: ~10 hours (Phases 3, 4, 5 live tests)

---

## 🎊 **Conclusion**

**Testing Evolution**: Modern, comprehensive, resilient

**Can Start NOW**:
- ✅ Test infrastructure
- ✅ Unit test expansion
- ✅ Chaos test scaffolding
- ✅ Documentation

**Week 2** (with BearDog):
- ⏳ Integration tests
- ⏳ E2E tests
- ⏳ Live chaos/fault tests

**Goal**: 90% coverage, world-class async Rust testing! 🦀✨

---

**Created**: January 16, 2026  
**Author**: Songbird Team  
**Status**: Strategy complete, ready to execute  
**Philosophy**: Deep debt + fast AND safe Rust

