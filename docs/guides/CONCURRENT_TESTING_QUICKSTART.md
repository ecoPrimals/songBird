# 🚀 CONCURRENT TESTING QUICK START GUIDE

**For the Songbird Development Team**

---

## 🎯 TL;DR - What Changed

**OLD WAY** (Flaky, slow):
```rust
loop {
    if registry.get_count().await >= 3 { break; }
    tokio::time::sleep(Duration::from_millis(100)).await;
}
```

**NEW WAY** (Fast, reliable):
```rust
use songbird_test_utils::async_polling::*;

poll_until_count(Duration::from_secs(5), 3, || async {
    registry.get_count().await
}).await?;
```

**Why**: Event-driven, deterministic, self-documenting!

---

## 📦 Available Primitives

### 1. `poll_until()` - Wait for any condition

**Use when**: Waiting for boolean condition to become true

```rust
use songbird_test_utils::async_polling::poll_until;
use std::time::Duration;

// Wait for service to be ready
poll_until(Duration::from_secs(5), || async {
    service.is_ready().await
}).await.expect("Service should become ready");
```

---

### 2. `poll_until_eq()` - Wait for specific value

**Use when**: Waiting for a value to match expected

```rust
use songbird_test_utils::async_polling::poll_until_eq;

// Wait for status to be "active"
poll_until_eq(
    Duration::from_secs(5),
    ServiceStatus::Active,
    || async { service.get_status().await }
).await.expect("Service should become active");
```

---

### 3. `poll_until_some()` - Wait for Option<T>

**Use when**: Waiting for an item to appear

```rust
use songbird_test_utils::async_polling::poll_until_some;

// Wait for provider to be registered
let provider = poll_until_some(Duration::from_secs(5), || async {
    registry.find_provider("my-provider").await
}).await.expect("Provider should register");

assert_eq!(provider.name, "my-provider");
```

---

### 4. `poll_until_ok()` - Wait for Result success

**Use when**: Waiting for an operation to succeed

```rust
use songbird_test_utils::async_polling::poll_until_ok;

// Wait for connection to succeed
let connection = poll_until_ok(Duration::from_secs(5), || async {
    service.connect().await
}).await.expect("Should connect eventually");
```

---

### 5. `poll_until_count()` - Wait for N items

**Use when**: Waiting for collection to reach size

```rust
use songbird_test_utils::async_polling::poll_until_count;

// Wait for 3 nodes to join
poll_until_count(Duration::from_secs(10), 3, || async {
    cluster.node_count().await
}).await.expect("3 nodes should join");
```

---

### 6. `poll_with_interval()` - Rate-limited polling

**Use when**: Polling external APIs that need rate limiting

```rust
use songbird_test_utils::async_polling::poll_with_interval;

// Poll external API every 500ms
poll_with_interval(
    Duration::from_secs(30),
    Duration::from_millis(500),
    || async {
        external_api.check_status().await == "ready"
    }
).await.expect("API should become ready");
```

---

## 🎓 Migration Patterns

### Pattern 1: Simple State Check

```rust
// ❌ OLD: Flaky, unclear timing
loop {
    if check_ready().await { break; }
    tokio::time::sleep(Duration::from_millis(100)).await;
}

// ✅ NEW: Clear, deterministic
poll_until(Duration::from_secs(5), check_ready).await?;
```

---

### Pattern 2: Waiting for Value

```rust
// ❌ OLD: Manual polling
loop {
    let status = get_status().await;
    if status == "ready" { break; }
    tokio::time::sleep(Duration::from_millis(50)).await;
}

// ✅ NEW: Semantic intent
poll_until_eq(Duration::from_secs(5), "ready", get_status).await?;
```

---

### Pattern 3: Waiting for Item

```rust
// ❌ OLD: Repetitive
loop {
    if let Some(item) = find_item("foo").await {
        return item;
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
}

// ✅ NEW: Concise
let item = poll_until_some(Duration::from_secs(5), || async {
    find_item("foo").await
}).await?;
```

---

### Pattern 4: Waiting for Count

```rust
// ❌ OLD: Verbose
loop {
    if get_providers().await.len() >= 3 { break; }
    tokio::time::sleep(Duration::from_millis(100)).await;
}

// ✅ NEW: Clear intent
poll_until_count(Duration::from_secs(5), 3, || async {
    get_providers().await.len()
}).await?;
```

---

## ⚠️ When NOT to Use These Helpers

### Legitimate Sleep Use Cases

**1. Testing Time-Based Behavior**
```rust
// ✅ GOOD: Testing circuit breaker timeout
let deadline = tokio::time::Instant::now() + timeout;
tokio::time::sleep_until(deadline).await;
assert_eq!(circuit_breaker.state(), Open);
```

**2. Performance Benchmarks**
```rust
// ✅ GOOD: Measuring actual timing
let result = benchmark_async("operation", 10, || async {
    tokio::time::sleep(Duration::from_millis(50)).await;
    Ok(())
}).await;
```

**3. Chaos Engineering**
```rust
// ✅ GOOD: Intentional delay for fault injection
chaos.inject_delay(Duration::from_millis(200)).await;
```

**4. Stability Tests**
```rust
// ✅ GOOD: Sampling state over time
for _ in 0..5 {
    let state = get_state().await;
    tokio::time::sleep(Duration::from_millis(50)).await;
}
```

---

## 🛠️ Common Mistakes & Fixes

### Mistake 1: Closure Capturing Issues

```rust
// ❌ WRONG: Captures variable incorrectly
let expected = 5;
poll_until_count(timeout, expected, || async {
    get_count().await  // 'expected' not accessible
}).await?;

// ✅ RIGHT: Use poll_until_count directly
poll_until_count(timeout, 5, get_count).await?;

// ✅ OR: Inline the comparison
poll_until(timeout, || async {
    get_count().await >= 5
}).await?;
```

---

### Mistake 2: Not Using Timeout

```rust
// ❌ WRONG: No timeout (hangs forever)
loop {
    if condition().await { break; }
    tokio::task::yield_now().await;
}

// ✅ RIGHT: Always have timeout
poll_until(Duration::from_secs(30), condition).await?;
```

---

### Mistake 3: Too Short Timeout

```rust
// ❌ RISKY: 100ms might not be enough
poll_until(Duration::from_millis(100), slow_condition).await?;

// ✅ BETTER: Be generous with timeouts
poll_until(Duration::from_secs(5), slow_condition).await?;
```

---

## 📋 Migration Checklist

When updating a test:

- [ ] Identify the waiting pattern (state, value, item, count)
- [ ] Choose appropriate `poll_until_*` helper
- [ ] Add `use songbird_test_utils::async_polling::*;` import
- [ ] Set reasonable timeout (5-10 seconds for most cases)
- [ ] Remove the old `loop { sleep; check }` pattern
- [ ] Add `.expect()` with descriptive message
- [ ] Run the test to verify it works
- [ ] Verify test is faster than before

---

## 🎯 Quick Reference Card

| Old Pattern | New Helper | Timeout |
|-------------|------------|---------|
| `loop { sleep; check }` | `poll_until()` | 5s |
| `loop { sleep; check == X }` | `poll_until_eq()` | 5s |
| `loop { sleep; find }` | `poll_until_some()` | 5s |
| `loop { sleep; try_op }` | `poll_until_ok()` | 5s |
| `loop { sleep; count >= N }` | `poll_until_count()` | 10s |
| `loop { sleep(N); check }` | `poll_with_interval()` | 30s |

---

## 💡 Pro Tips

### Tip 1: Descriptive Error Messages
```rust
poll_until_eq(timeout, Ready, get_status)
    .await
    .expect("Service should become ready within 5s");
```

### Tip 2: Chain Operations
```rust
let item = poll_until_some(timeout, find_item)
    .await
    .expect("Item should appear")?;

// Use item immediately
assert_eq!(item.status, "active");
```

### Tip 3: Combine with Existing Patterns
```rust
// Works great with tokio::test
#[tokio::test]
async fn test_service_startup() {
    let service = Service::new();
    service.start().await?;
    
    // Wait for ready state
    poll_until_eq(Duration::from_secs(5), Ready, || async {
        service.status().await
    }).await?;
    
    // Continue with test
    assert!(service.is_operational().await);
}
```

---

## 🚀 Getting Started

### Step 1: Import the Module
```rust
use songbird_test_utils::async_polling::*;
use std::time::Duration;
```

### Step 2: Replace One Test
Pick a simple test with a sleep loop and modernize it.

### Step 3: Run and Verify
```bash
cargo test test_name
```

### Step 4: Compare
- Is it clearer? ✓
- Is it faster? ✓
- Is it more reliable? ✓

### Step 5: Repeat
Apply the pattern to more tests!

---

## 📚 Additional Resources

- **Full Examples**: See `crates/songbird-test-utils/src/async_polling.rs`
- **Test Suite**: `crates/songbird-test-utils/tests/`
- **Architecture Docs**: `CONCURRENT_MODERNIZATION_SUMMARY.md`
- **Migration Guide**: `MODERNIZATION_COMPLETE_DEC_7_2025.md`

---

## 🆘 Need Help?

**Common Issues**:

1. **"Timeout expired"** → Increase timeout duration
2. **"Closure capturing error"** → Use direct function reference
3. **"Test slower than before"** → Check if using `poll_with_interval()` correctly

**Questions?** Check the comprehensive docs or ask the team!

---

## ✅ Success Criteria

You'll know you're using these correctly when:

- ✓ Test intent is crystal clear from the code
- ✓ Tests run faster than before
- ✓ No flaky failures
- ✓ Easier to debug when they do fail

---

**Happy Concurrent Testing! 🚀**

*Last Updated: December 7, 2025*

