# 🧪 **TEST COVERAGE EXPANSION PLAN**
## **Phase 2: Critical Path Testing**

---

## 📊 **CURRENT STATE**

### **Test Status**
- **Tests Passing**: 151 ✅
- **Test Files**: 10 files
- **Coverage**: 17% ⚠️
- **Target**: 90% ✅

### **Gap Analysis**
**Why is coverage so low despite 151 passing tests?**

Most tests only validate struct creation and configuration:
```rust
#[test]
fn test_create_adapter() {
    let adapter = create_universal_adapter();
    assert!(std::mem::size_of_val(&adapter) > 0); // Just checks it exists!
}
```

**Missing**: Tests for actual business logic (async operations, discovery, routing, etc.)

---

## 🎯 **CRITICAL FUNCTIONS NEEDING TESTS**

### **Priority 1: Discovery & Routing** (Core Business Logic)

#### `UnifiedUniversalAdapter::discover_services()` 
**Coverage**: ❌ Not tested  
**Lines**: ~35 lines of logic  
**Criticality**: CRITICAL - Foundation of system

**Needs**:
- ✅ Success case: discovers multiple services
- ✅ Partial success: some endpoints fail
- ✅ All endpoints fail scenario
- ✅ Timeout handling
- ✅ Network error handling
- ✅ Registry update verification
- ✅ Capability indexing correctness

#### `UnifiedUniversalAdapter::find_services_by_capability()`
**Coverage**: ❌ Not tested  
**Lines**: ~25 lines  
**Criticality**: CRITICAL - Routing foundation

**Needs**:
- ✅ Find services with capability
- ✅ No services found
- ✅ Multiple providers
- ✅ Filter by criteria

#### `UnifiedUniversalAdapter::send_request()`
**Coverage**: ❌ Not tested  
**Lines**: ~40 lines  
**Criticality**: CRITICAL - Request handling

**Needs**:
- ✅ Successful request/response
- ✅ Service selection logic
- ✅ Failover to backup service
- ✅ All services down
- ✅ Timeout handling
- ✅ Error responses

---

## 📋 **TESTING STRATEGY**

### **Week 1-2: Core Operations (17% → 30%)**

**Focus**: Discovery & basic routing

**Tests to Add**: ~30 tests
1. Service discovery (10 tests)
2. Capability routing (8 tests)
3. Basic request/response (7 tests)
4. Error handling (5 tests)

**Expected Coverage Gain**: +13%

### **Week 3-4: Advanced Operations (30% → 50%)**

**Focus**: Load balancing, health checks, failover

**Tests to Add**: ~40 tests
1. Load balancing algorithms (12 tests)
2. Health monitoring (10 tests)
3. Circuit breakers (8 tests)
4. Failover scenarios (10 tests)

**Expected Coverage Gain**: +20%

### **Week 5-6: Integration & E2E (50% → 70%)**

**Focus**: Complete workflows

**Tests to Add**: ~30 tests
1. Multi-service workflows (10 tests)
2. Federation scenarios (8 tests)
3. Adapter interactions (7 tests)
4. Real-world scenarios (5 tests)

**Expected Coverage Gain**: +20%

### **Week 7-8: Edge Cases & Chaos (70% → 90%)**

**Focus**: Resilience and edge cases

**Tests to Add**: ~40 tests
1. Chaos scenarios (15 tests)
2. Edge cases (10 tests)
3. Error paths (10 tests)
4. Performance tests (5 tests)

**Expected Coverage Gain**: +20%

---

## 🔧 **TESTING INFRASTRUCTURE NEEDED**

### **Mock HTTP Server**
```rust
// For testing discovery and requests
use mockito::{mock, Server};

#[tokio::test]
async fn test_discover_services_success() {
    let mut server = Server::new_async().await;
    
    let mock = server.mock("GET", "/capabilities")
        .with_status(200)
        .with_body(r#"[{"name": "test-service", ...}]"#)
        .create_async()
        .await;
    
    // Test discovery against mock server
}
```

### **Test Fixtures**
```rust
// Reusable test data
pub fn create_test_service_info() -> ServiceInfo { ... }
pub fn create_test_capability() -> Capability { ... }
pub fn create_test_request() -> UniversalRequest { ... }
```

### **Async Test Helpers**
```rust
// Timeout helpers for async tests
pub async fn with_timeout<F>(fut: F) -> Result<F::Output>
where F: Future { ... }
```

---

## 📈 **IMMEDIATE ACTIONS (Next Session)**

### **File to Create**: `discovery_comprehensive_tests.rs`

**Tests**:
1. `test_discover_services_all_endpoints_success()`
2. `test_discover_services_partial_failure()`
3. `test_discover_services_all_fail()`
4. `test_discover_services_timeout()`
5. `test_discover_services_network_error()`
6. `test_discover_services_invalid_json()`
7. `test_discover_services_updates_registry()`
8. `test_discover_services_indexes_capabilities()`
9. `test_discover_services_concurrent_safe()`
10. `test_discover_services_empty_response()`

**Expected Coverage**: +5%

### **File to Enhance**: `unified_adapter_core_tests.rs`

**Add**:
1. `test_find_services_by_capability_found()`
2. `test_find_services_by_capability_not_found()`
3. `test_find_services_by_capability_multiple()`
4. `test_send_request_success()`
5. `test_send_request_service_down()`
6. `test_send_request_failover()`
7. `test_send_request_timeout()`
8. `test_send_request_error_response()`

**Expected Coverage**: +4%

---

## 🎯 **SUCCESS METRICS**

### **Week 1 Target**
- [ ] 151 → 181 tests (+30 tests)
- [ ] 17% → 30% coverage (+13%)
- [ ] All discovery paths tested
- [ ] Basic routing tested

### **Week 2 Target**
- [ ] 181 → 211 tests (+30 tests)
- [ ] 30% → 40% coverage (+10%)
- [ ] Load balancing tested
- [ ] Health checks tested

### **Month Target**
- [ ] 151 → 261 tests (+110 tests)
- [ ] 17% → 70% coverage (+53%)
- [ ] All critical paths tested
- [ ] E2E workflows validated

### **Final Target (Week 8)**
- [ ] 151 → 291 tests (+140 tests)
- [ ] 17% → 90% coverage (+73%)
- [ ] Chaos testing complete
- [ ] Production ready

---

## 💡 **TESTING BEST PRACTICES**

### **Test Structure**
```rust
#[tokio::test]
async fn test_specific_scenario() {
    // ARRANGE: Set up test data and mocks
    let adapter = UnifiedUniversalAdapter::new();
    let mock_server = setup_mock_server().await;
    
    // ACT: Execute the operation
    let result = adapter.discover_services().await;
    
    // ASSERT: Verify outcomes
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 2);
    
    // CLEANUP: Verify side effects
    let registry = adapter.get_registry().await;
    assert_eq!(registry.service_count(), 2);
}
```

### **Test Naming**
- `test_<function>_<scenario>_<expected_outcome>()`
- Examples:
  - `test_discover_services_all_success()`
  - `test_send_request_timeout_returns_error()`
  - `test_find_capability_not_found_returns_empty()`

### **Coverage Focus**
1. **Happy paths** - Normal operation
2. **Error paths** - All error conditions
3. **Edge cases** - Boundary conditions
4. **Concurrent scenarios** - Thread safety
5. **Integration** - Component interactions

---

## 🚀 **DEPENDENCIES**

### **Add to Cargo.toml** (test dependencies)
```toml
[dev-dependencies]
mockito = "1.2"
tokio-test = "0.4"
proptest = "1.4"  # For property-based testing
```

### **Existing (Already Available)**
- ✅ tokio (async runtime)
- ✅ reqwest (HTTP client)
- ✅ serde_json (JSON handling)
- ✅ chrono (time handling)

---

## 📊 **ESTIMATED EFFORT**

| Phase | Tests | Coverage | Weeks | Effort |
|-------|-------|----------|-------|--------|
| Current | 151 | 17% | - | - |
| Phase 1 | 181 | 30% | 2 | 40h |
| Phase 2 | 221 | 50% | 2 | 40h |
| Phase 3 | 251 | 70% | 2 | 40h |
| Phase 4 | 291 | 90% | 2 | 40h |
| **Total** | **+140** | **+73%** | **8** | **160h** |

**Average**: 20h/week for 8 weeks = **2.5 hours per day**

---

## ✅ **READY TO BEGIN**

**Next Step**: Start implementing discovery tests  
**File**: `tests/discovery_comprehensive_tests.rs`  
**Target**: +10 tests, +5% coverage  
**Time**: 2-3 hours

---

**Plan Created**: October 21, 2025  
**Target Completion**: December 2025 (8 weeks)  
**Critical Path**: Discovery → Routing → Integration → Chaos  
**Success Criteria**: 90% coverage, all critical paths tested


