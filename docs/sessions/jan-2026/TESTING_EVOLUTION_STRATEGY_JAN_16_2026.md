# 🧪 Testing Evolution Strategy - January 16, 2026

**Goal**: 90% code coverage across unit, E2E, chaos, and fault tests  
**Tool**: `llvm-cov` for coverage measurement  
**Status**: In Progress  

---

## 🎯 Testing Categories

### 1. Unit Tests
**Purpose**: Test individual functions/components in isolation  
**Coverage Target**: 95%+  
**Focus Areas**:
- Socket path derivation logic
- Family ID resolution
- Environment variable parsing
- Configuration validation
- Error handling paths

### 2. E2E (End-to-End) Tests
**Purpose**: Test complete workflows from start to finish  
**Coverage Target**: 85%+  
**Focus Areas**:
- BiomeOS Neural API deployment flow
- Multi-family deployment scenarios
- Inter-primal communication
- Service discovery workflows
- Health check integration

### 3. Chaos Tests
**Purpose**: Test system behavior under random failures  
**Coverage Target**: 70%+  
**Focus Areas**:
- Environment variable corruption
- Socket file deletion during runtime
- Permission changes
- Network disruptions
- Process crashes

### 4. Fault Injection Tests
**Purpose**: Test specific failure scenarios and recovery  
**Coverage Target**: 80%+  
**Focus Areas**:
- Missing environment variables
- Invalid socket paths
- Invalid family IDs
- Disk full scenarios
- Permission denied errors

---

## 📋 Testing Plan

### Phase 1: BiomeOS Socket Integration (Current)

#### Unit Tests ✅ (Complete)
- [x] Socket path priority order (3 tests, 11 scenarios)
- [x] Family ID priority order
- [x] Default behavior validation
- [x] Environment variable cleanup

#### E2E Tests (Add)
- [ ] Full BiomeOS deployment simulation
- [ ] Multi-primal socket coordination
- [ ] Socket path discovery by other primals
- [ ] Health check through Unix socket
- [ ] Graceful shutdown and cleanup

#### Chaos Tests (Add)
- [ ] Random environment variable corruption
- [ ] Socket file deleted during operation
- [ ] Rapid environment variable changes
- [ ] Concurrent socket creation attempts
- [ ] File system permission changes

#### Fault Injection Tests (Add)
- [ ] Missing critical environment variables
- [ ] Invalid socket paths (non-existent dirs)
- [ ] Invalid family IDs (special characters)
- [ ] Disk full during socket creation
- [ ] Permission denied scenarios
- [ ] Socket already in use errors

---

## 🔧 Implementation Strategy

### 1. Test Infrastructure

**Test Helpers** (to create):
```rust
// Environment variable isolation
pub struct EnvIsolation { ... }

// Chaos testing utilities
pub struct ChaosInjector { ... }

// Fault injection utilities
pub struct FaultInjector { ... }

// Socket testing utilities
pub struct SocketTestHarness { ... }
```

### 2. Coverage Measurement

**Commands**:
```bash
# Install llvm-cov
cargo install cargo-llvm-cov

# Run all tests with coverage
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# Generate HTML report
cargo llvm-cov --all-features --workspace --html

# View coverage by category
cargo llvm-cov --all-features --workspace --text
```

### 3. Test Organization

**Directory Structure**:
```
crates/songbird-orchestrator/
├── tests/
│   ├── unit/
│   │   └── biomeos_socket_env_vars.rs (✅ Complete)
│   ├── e2e/
│   │   └── biomeos_deployment.rs (TODO)
│   ├── chaos/
│   │   └── biomeos_socket_chaos.rs (TODO)
│   └── fault/
│       └── biomeos_socket_faults.rs (TODO)
```

---

## 📊 Coverage Targets

| Category | Current | Target | Priority |
|----------|---------|--------|----------|
| Unit Tests | ~60% | 95% | High |
| E2E Tests | ~40% | 85% | High |
| Chaos Tests | ~5% | 70% | Medium |
| Fault Tests | ~10% | 80% | High |
| **Overall** | **~45%** | **90%** | **Critical** |

---

## 🎯 Immediate Priorities

### Week 1 (Current)
1. ✅ BiomeOS socket unit tests (Complete)
2. 🔄 BiomeOS socket E2E tests (In Progress)
3. 🔄 BiomeOS socket fault injection (In Progress)

### Week 2
1. BiomeOS socket chaos tests
2. Service discovery E2E tests
3. Connection manager unit tests

### Week 3
1. Federation E2E tests
2. Genetic lineage chaos tests
3. IPC fault injection tests

### Week 4
1. Complete coverage gaps
2. Measure with llvm-cov
3. Target 90%+ overall coverage

---

## 🧪 Test Examples

### Unit Test Example (✅ Complete)
```rust
#[test]
fn test_socket_path_priority() {
    env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", "/tmp/test.sock");
    let path = UnixSocketServer::socket_path_from_env();
    assert_eq!(path, PathBuf::from("/tmp/test.sock"));
}
```

### E2E Test Example (TODO)
```rust
#[tokio::test]
async fn test_biomeos_deployment_flow() {
    // Setup BiomeOS environment
    let env = BiomeOsTestEnv::new();
    env.set_socket_path("/tmp/songbird-nat0.sock");
    env.set_family_id("nat0");
    
    // Start Songbird
    let server = start_songbird_server().await?;
    
    // Verify socket created at correct location
    assert!(Path::new("/tmp/songbird-nat0.sock").exists());
    
    // Simulate health check from BiomeOS
    let health = env.check_health().await?;
    assert_eq!(health.status, "healthy");
    
    // Cleanup
    server.shutdown().await?;
}
```

### Chaos Test Example (TODO)
```rust
#[tokio::test]
async fn test_socket_deletion_during_operation() {
    let server = start_songbird_server().await?;
    
    // Delete socket file while server is running
    tokio::fs::remove_file("/tmp/songbird-default.sock").await?;
    
    // Verify server detects and recovers
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert!(Path::new("/tmp/songbird-default.sock").exists());
}
```

### Fault Injection Test Example (TODO)
```rust
#[tokio::test]
async fn test_invalid_socket_path() {
    env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", "/non/existent/path/test.sock");
    
    let result = start_songbird_server().await;
    
    // Should fail gracefully with clear error
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("directory does not exist"));
}
```

---

## 📈 Progress Tracking

### Current Session (Jan 16, 2026)
- ✅ Unit tests: 3 functions, 11 scenarios (100% pass)
- 🔄 E2E tests: Starting
- 🔄 Fault tests: Starting
- ⏳ Chaos tests: Planned

### Coverage Measurement
```bash
# Before
Overall: ~45%
Unit: ~60%
E2E: ~40%
Chaos: ~5%
Fault: ~10%

# Target (4 weeks)
Overall: 90%+
Unit: 95%+
E2E: 85%+
Chaos: 70%+
Fault: 80%+
```

---

## 🎯 Success Metrics

### Code Coverage
- [ ] 90%+ overall coverage (llvm-cov)
- [ ] 95%+ unit test coverage
- [ ] 85%+ E2E test coverage
- [ ] 70%+ chaos test coverage
- [ ] 80%+ fault injection coverage

### Test Health
- [ ] All tests passing
- [ ] No flaky tests
- [ ] Fast test execution (< 5 min)
- [ ] CI/CD integration

### Documentation
- [ ] Every test has clear purpose
- [ ] Failure scenarios documented
- [ ] Recovery paths validated
- [ ] Edge cases covered

---

## 🚀 Next Steps

1. **Create E2E Test Suite** (This Session)
   - BiomeOS deployment simulation
   - Socket lifecycle validation
   - Health check integration

2. **Create Fault Injection Suite** (This Session)
   - Invalid paths
   - Missing env vars
   - Permission errors

3. **Create Chaos Test Suite** (Next Session)
   - Random failures
   - Concurrent operations
   - Resource exhaustion

4. **Measure Coverage** (Ongoing)
   - Run llvm-cov regularly
   - Track progress
   - Identify gaps

---

**Last Updated**: January 16, 2026  
**Status**: ✅ Strategy Complete, Implementation In Progress  
**Next**: Create E2E and Fault tests for BiomeOS socket integration

