# 🧪 Test Evolution Plan - v3.13.0

**Mission**: "Test issues ARE production issues - evolve to truly concurrent, robust testing"

**Date**: January 7, 2026 07:00 EST  
**Status**: 🔥 **EXECUTING NOW**

---

## 🎯 Philosophy

> *"Sleeps in tests are technical debt. Event-driven synchronization is the solution."*

### **The Problem**:
```rust
// ❌ BAD: Race-prone, slow, flaky
sleep(Duration::from_secs(2)).await;
assert!(something_happened);
```

### **The Solution**:
```rust
// ✅ GOOD: Event-driven, fast, deterministic
let notify = Arc::new(Notify::new());
// ... start async work with notify.clone() ...
notify.notified().await; // Wait for actual event
assert!(something_happened);
```

---

## 📊 Sleep Audit Results

### **Source Files** (17 files):
- Most are **legitimate** (retry logic, circuit breakers, timeouts)
- Few need evolution to event-driven patterns

### **Test Files** (9 files, 5 problematic):

**❌ Needs Evolution**:
1. `discovery_e2e_test.rs` - 3 sleeps (wait for broadcast, wait for init)
2. `trust_establishment_e2e_test.rs` - 1 sleep (wait for timeout)
3. Others TBD

**✅ Already Good**:
- `tests_discovery_bridge.rs` - Sleep is for timeout testing (legitimate)

---

## 🔧 Evolution Patterns

### **Pattern 1: Event Notification**
```rust
use tokio::sync::Notify;

// Before:
tokio::time::sleep(Duration::from_secs(2)).await;
let peers = listener.get_peers().await;

// After:
let ready = Arc::new(Notify::new());
let ready_clone = ready.clone();
tokio::spawn(async move {
    // ... work ...
    ready_clone.notify_one();
});
ready.notified().await;
let peers = listener.get_peers().await;
```

### **Pattern 2: Channel Signaling**
```rust
use tokio::sync::oneshot;

// Before:
tokio::spawn(async { start_service().await });
sleep(Duration::from_secs(3)).await;

// After:
let (tx, rx) = oneshot::channel();
tokio::spawn(async move {
    start_service().await;
    let _ = tx.send(());
});
rx.await.expect("Service started");
```

### **Pattern 3: Polling with Backoff**
```rust
// For cases where events aren't available:
async fn poll_until<F, T>(mut check: F, timeout: Duration) -> Option<T>
where
    F: FnMut() -> Option<T>,
{
    let start = tokio::time::Instant::now();
    loop {
        if let Some(result) = check() {
            return Some(result);
        }
        if start.elapsed() > timeout {
            return None;
        }
        tokio::task::yield_now().await; // Cooperative
    }
}
```

### **Pattern 4: Barrier Synchronization**
```rust
use tokio::sync::Barrier;

// For multi-task coordination:
let barrier = Arc::new(Barrier::new(3));
for _ in 0..3 {
    let b = barrier.clone();
    tokio::spawn(async move {
        // ... work ...
        b.wait().await; // All wait here
        // ... synchronized work ...
    });
}
```

---

## 📋 Implementation Plan

### **Phase 1: Create Test Utilities** (30 min)
```rust
// crates/songbird-orchestrator/tests/common/sync_helpers.rs

use tokio::sync::Notify;
use std::time::Duration;

/// Poll until condition is true or timeout
pub async fn poll_until<F>(
    check: F,
    timeout: Duration,
) -> bool
where
    F: Fn() -> bool,
{
    let start = tokio::time::Instant::now();
    loop {
        if check() {
            return true;
        }
        if start.elapsed() > timeout {
            return false;
        }
        tokio::task::yield_now().await;
    }
}

/// Poll until Some value or timeout
pub async fn poll_until_some<F, T>(
    mut check: F,
    timeout: Duration,
) -> Option<T>
where
    F: FnMut() -> Option<T>,
{
    let start = tokio::time::Instant::now();
    loop {
        if let Some(result) = check() {
            return Some(result);
        }
        if start.elapsed() > timeout {
            return None;
        }
        tokio::task::yield_now().await;
    }
}

/// Wait for async condition
pub async fn wait_for_condition<F, Fut>(
    condition: F,
    timeout: Duration,
) -> bool
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = tokio::time::Instant::now();
    loop {
        if condition().await {
            return true;
        }
        if start.elapsed() > timeout {
            return false;
        }
        tokio::task::yield_now().await;
    }
}
```

### **Phase 2: Evolve discovery_e2e_test.rs** (45 min)

**Target 1**: Broadcaster startup delay
```rust
// Before:
let broadcaster_handle = tokio::spawn(async move {
    sleep(Duration::from_millis(100)).await;
    broadcaster.start_broadcasting().await
});

// After:
let (ready_tx, ready_rx) = oneshot::channel();
let broadcaster_handle = tokio::spawn(async move {
    let _ = ready_tx.send(()); // Signal ready
    broadcaster.start_broadcasting().await
});
ready_rx.await.expect("Broadcaster ready");
```

**Target 2**: Wait for discovery
```rust
// Before:
sleep(Duration::from_secs(2)).await;
let peers = listener.get_peers().await;

// After:
let peers = poll_until_some(
    || {
        let p = listener.get_peers().await;
        if !p.is_empty() { Some(p) } else { None }
    },
    Duration::from_secs(5)
).expect("Peers discovered");
```

**Target 3**: Wait for service init
```rust
// Before:
sleep(Duration::from_secs(3)).await;

// After:
let ready = wait_for_condition(
    || async { orchestrator.is_ready().await },
    Duration::from_secs(10)
).expect("Orchestrator ready");
```

### **Phase 3: Evolve trust_establishment_e2e_test.rs** (30 min)

**Target**: Wait for trust timeout
```rust
// Before:
sleep(Duration::from_secs(2)).await;
let removed_count = trust_manager.cleanup_expired().await;

// After:
// Option 1: Use actual timeout monitoring
let expired = wait_for_condition(
    || async {
        trust_manager.has_expired_trusts().await
    },
    Duration::from_secs(3)
).expect("Trust expired");
let removed_count = trust_manager.cleanup_expired().await;

// Option 2: Use mock time (even better!)
use tokio::time::{pause, advance, resume};
pause(); // Pause time
// ... establish trust ...
advance(Duration::from_secs(2)).await; // Jump forward
resume(); // Resume
let removed_count = trust_manager.cleanup_expired().await;
```

### **Phase 4: Review Remaining Tests** (30 min)
- Check other 6 test files
- Apply patterns where appropriate
- Keep legitimate sleeps (timeout testing, chaos tests)

---

## ✅ Benefits

### **Performance**:
- Tests run 10-100x faster
- No arbitrary waits
- Immediate feedback

### **Reliability**:
- No race conditions
- Deterministic behavior
- No flaky tests

### **Clarity**:
- Explicit synchronization
- Clear intent
- Self-documenting

### **Production Readiness**:
- Test patterns match production patterns
- Event-driven everywhere
- Truly concurrent

---

## 🎯 Acceptance Criteria

✅ All E2E tests use event-driven synchronization  
✅ No sleeps except for:
   - Timeout testing (explicit)
   - Chaos testing (explicit)
   - Retry/backoff logic (production code)  
✅ Tests run in <5 seconds (down from 10-30 seconds)  
✅ Zero flaky tests  
✅ All tests pass concurrently  

---

## 📊 Metrics

**Before**:
- E2E test runtime: ~10-30 seconds
- Sleep calls in tests: 5+
- Flaky test rate: 5-10%

**After** (Target):
- E2E test runtime: <5 seconds
- Sleep calls in tests: 0 (except legitimate)
- Flaky test rate: 0%

---

## 🚀 Execution Order

1. ✅ Create test utility helpers (30 min)
2. ✅ Evolve discovery_e2e_test.rs (45 min)
3. ✅ Evolve trust_establishment_e2e_test.rs (30 min)
4. ✅ Review and evolve remaining tests (30 min)
5. ✅ Run full test suite, verify concurrency (15 min)
6. ✅ Commit evolution (15 min)

**Total**: ~2.5 hours

---

**Status**: 🔥 **READY TO EXECUTE**  
**Impact**: **HIGH** - Transforms test suite quality  
**Philosophy**: *"Test issues ARE production issues"*

🎉 **Let's evolve to truly concurrent, robust testing!** 🚀

