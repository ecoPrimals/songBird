# 🚀 CONCURRENT TEST MODERNIZATION - EXECUTION PLAN
## December 7, 2025

---

## 📊 **BASELINE METRICS**

**Scan Results**:
- Sleep instances: 202
- Serial references: 807 (mostly comments)
- Files with #[serial]: 1
- Files with sleep: 20

---

## 🎯 **MODERNIZATION STRATEGY**

### **Phase 1: Eliminate Sleeps** (Priority 1)
Replace `sleep()` with modern event-driven patterns:

**Pattern A - Event Polling**:
```rust
// OLD (bad - flaky)
tokio::time::sleep(Duration::from_secs(1)).await;
assert!(condition);

// NEW (good - deterministic)
async_polling::poll_until(|| async { condition }, Duration::from_secs(5)).await?;
```

**Pattern B - Channel Signals**:
```rust
// OLD (bad - arbitrary delay)
tokio::time::sleep(Duration::from_millis(100)).await;

// NEW (good - explicit synchronization)
let (tx, rx) = tokio::sync::oneshot::channel();
// Signal when ready
tx.send(()).unwrap();
rx.await?;
```

**Pattern C - Immediate Assertions**:
```rust
// OLD (bad - unnecessary delay)
tokio::time::sleep(Duration::from_millis(10)).await;
assert_eq!(result, expected);

// NEW (good - test what we mean)
assert_eq!(result, expected);  // No sleep needed!
```

### **Phase 2: Remove Serial Attributes** (Priority 2)
Only chaos/fault injection tests should be serial.

**Allowed**:
```rust
#[tokio::test]
#[serial]  // OK for chaos tests that manipulate global state
async fn chaos_test_resource_exhaustion() { ... }
```

**Not Allowed**:
```rust
#[tokio::test]
#[serial]  // BAD - make concurrent instead
async fn test_normal_functionality() { ... }
```

### **Phase 3: Modern Sync Primitives** (Priority 3)
Use the existing concurrent sync primitives in `songbird-test-utils`.

**Available Tools**:
- `async_polling::poll_until()` - Wait for condition
- `concurrent_sync::EventSync` - Multi-waiter coordination
- `concurrent_sync::StateSync` - State transitions
- `concurrent_sync::WorkerSync` - Worker coordination

---

## 📋 **FILES TO FIX** (Priority Order)

### **Tier 1: High Impact** (Fix First):
1. `metrics_integration_tests.rs` - Already partially modernized
2. `capability_integration_tests.rs` - 20+ sleeps
3. `federation_coordination_tests.rs` - Network tests with delays
4. `integration_tarpc.rs` - RPC tests with timeouts

### **Tier 2: Medium Impact**:
5. `e2e_multi_primal_workflows.rs` - E2E tests
6. `discovery_integration_comprehensive_tests.rs` - Discovery tests
7. `federation_state_comprehensive_tests.rs` - State tests

### **Tier 3: Lower Impact**:
8-20. Other files with occasional sleeps

---

## ⚡ **EXECUTION PLAN**

### **NOW** (Next 30 minutes):
1. ✅ Scan complete
2. ⏳ Fix Tier 1 files (4 files, ~60 sleeps)
3. ⏳ Verify tests still pass

### **Next Hour**:
4. Fix Tier 2 files (3 files, ~40 sleeps)
5. Remove serial attribute from non-chaos test
6. Run full test suite

### **Next Session**:
7. Fix remaining Tier 3 files
8. Add documentation on modern patterns
9. Update CONTRIBUTING.md with guidelines

---

## 🎓 **MODERNIZATION GUIDELINES**

### **DO**:
- ✅ Use `async_polling` for condition waiting
- ✅ Use channels for synchronization
- ✅ Use concurrent sync primitives
- ✅ Make tests deterministic
- ✅ Test actual behavior, not timing

### **DON'T**:
- ❌ Use `sleep()` or `delay()`
- ❌ Use `#[serial]` except for chaos tests
- ❌ Rely on timing for correctness
- ❌ Add arbitrary delays "to be safe"
- ❌ Use `thread::sleep` in async tests

---

## 🚀 **STARTING EXECUTION**

Beginning with highest-priority file modernization...

