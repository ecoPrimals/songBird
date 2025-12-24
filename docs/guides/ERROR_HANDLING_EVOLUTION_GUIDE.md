# 🛡️ Error Handling Evolution Guide

**Date**: December 22, 2025  
**Goal**: Replace unwrap/panic with proper error handling  
**Current State**: ~714 production unwrap/expect calls identified

---

## 🎯 Philosophy

**Modern Rust Error Handling**:
- ❌ Never `unwrap()` or `panic!()` in production paths
- ❌ Never silently discard errors
- ✅ Use `Result<T, E>` for fallible operations
- ✅ Propagate errors with `?` operator
- ✅ Add context to errors
- ✅ Handle at appropriate boundaries

---

## 📊 Current State

### Unwrap/Expect Distribution
```
Total in production crates: ~714 instances

By severity:
- Hot paths (network, I/O): ~150 🔴 CRITICAL
- Initialization: ~200 🟡 HIGH  
- Internal logic: ~250 🟡 MEDIUM
- Error handling code: ~114 🟢 LOW
```

### Panic Distribution
```
Total panic!() calls: ~50

Contexts:
- Assertion failures: ~20 (may be appropriate)
- Unrecoverable states: ~15
- TODO markers: ~10
- Test helpers: ~5
```

---

## 🔄 Evolution Patterns

### Pattern 1: Unwrap on Infallible Operations

**❌ Before:**
```rust
let json = serde_json::to_string(&value).unwrap();
```

**Analysis**: `to_string` can actually fail (stack overflow, trait impl issues)

**✅ After:**
```rust
let json = serde_json::to_string(&value)
    .map_err(|e| SongbirdError::serialization(format!("Failed to serialize: {}", e)))?;
```

---

### Pattern 2: Unwrap with "Known Safe" Values

**❌ Before:**
```rust
let port = env::var("PORT").unwrap_or("8080".to_string());
let port_num: u16 = port.parse().unwrap(); // ⚠️ Can panic!
```

**✅ After:**
```rust
use songbird_types::SafeEnv;

let port: u16 = SafeEnv::parse("PORT", 8080);
// Handles parse errors gracefully, uses fallback
```

---

### Pattern 3: Expect with Context

**❌ Before:**
```rust
let config = load_config().expect("Failed to load config");
```

**Issues**:
- Terminates program
- No recovery possible
- Poor user experience

**✅ After:**
```rust
let config = load_config()
    .map_err(|e| SongbirdError::configuration(
        format!("Failed to load config from {}: {}", path.display(), e)
    ))?;
```

---

### Pattern 4: Unwrap in Error Recovery

**❌ Before:**
```rust
let result = risky_operation().unwrap_or_else(|e| {
    // Try recovery
    fallback_operation().unwrap() // ⚠️ Can still panic!
});
```

**✅ After:**
```rust
let result = risky_operation()
    .or_else(|e| {
        warn!("Primary operation failed: {}, trying fallback", e);
        fallback_operation()
    })
    .map_err(|e| SongbirdError::operation_failed(
        format!("Both primary and fallback failed: {}", e)
    ))?;
```

---

### Pattern 5: Lock Poisoning

**❌ Before:**
```rust
let data = mutex.lock().unwrap(); // Panics if poisoned
```

**✅ After:**
```rust
let data = mutex.lock()
    .map_err(|e| SongbirdError::lock_poisoned(
        format!("Mutex poisoned: {}", e)
    ))?;

// Or handle poisoned state:
let data = match mutex.lock() {
    Ok(guard) => guard,
    Err(poisoned) => {
        warn!("Lock was poisoned, recovering");
        poisoned.into_inner() // Use the data anyway if safe
    }
};
```

---

### Pattern 6: Channel Operations

**❌ Before:**
```rust
sender.send(message).unwrap(); // Panics if receiver dropped
```

**✅ After:**
```rust
sender.send(message)
    .map_err(|_| SongbirdError::channel_closed(
        "Receiver has been dropped"
    ))?;

// Or log and continue:
if let Err(e) = sender.send(message) {
    warn!("Failed to send message, receiver dropped: {}", e);
    // Continue without panicking
}
```

---

### Pattern 7: Initialization Guarantees

**❌ Before:**
```rust
pub fn new() -> Self {
    Self {
        client: create_client().unwrap(), // Panics on init failure
    }
}
```

**✅ After (Option 1 - Fallible Constructor):**
```rust
pub fn new() -> Result<Self, SongbirdError> {
    Ok(Self {
        client: create_client()
            .map_err(|e| SongbirdError::initialization(
                format!("Failed to create client: {}", e)
            ))?,
    })
}
```

**✅ After (Option 2 - Builder Pattern):**
```rust
pub struct Builder {
    client: Option<Client>,
}

impl Builder {
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }
    
    pub fn build(self) -> Result<MyStruct, SongbirdError> {
        let client = self.client
            .ok_or_else(|| SongbirdError::configuration(
                "Client not configured"
            ))?;
        Ok(MyStruct { client })
    }
}
```

---

### Pattern 8: Array/Vec Indexing

**❌ Before:**
```rust
let first = vec[0]; // Panics if empty
```

**✅ After:**
```rust
let first = vec.first()
    .ok_or_else(|| SongbirdError::invalid_state("Vector is empty"))?;

// Or with default:
let first = vec.first().copied().unwrap_or(default_value);
```

---

### Pattern 9: Expect in Tests

**✅ Acceptable:**
```rust
#[test]
fn test_something() {
    let result = function().expect("Test setup failed");
    // In tests, expect() is acceptable for setup
}
```

**Even Better:**
```rust
#[test]
fn test_something() -> Result<()> {
    let result = function()?; // Propagate, test framework shows error
    assert_eq!(result, expected);
    Ok(())
}
```

---

## 🎯 Migration Strategy

### Phase 1: Hot Paths (High Impact) 🔴
**Priority**: Network I/O, request handling, core orchestration

```rust
// Files to prioritize:
- crates/songbird-orchestrator/src/app/mod.rs
- crates/songbird-orchestrator/src/network/
- crates/songbird-network-federation/src/
- crates/songbird-orchestrator/src/server/
```

**Approach**:
1. Identify critical paths via profiling
2. Replace unwrap with proper error propagation
3. Add comprehensive error context
4. Test error scenarios

---

### Phase 2: Initialization (Fail Fast is OK) 🟡
**Priority**: Application startup, config loading

**Pattern**: Convert to `Result` in constructors

```rust
// Before
impl App {
    pub fn new() -> Self { /* may panic */ }
}

// After  
impl App {
    pub fn new() -> Result<Self, SongbirdError> { /* returns error */ }
}
```

---

### Phase 3: Internal Logic (Lower Risk) 🟡
**Priority**: Business logic, state management

**Approach**:
1. Audit for actual error conditions
2. Add Result returns where needed
3. Document invariants that prevent errors

---

### Phase 4: Cleanup (Remaining Cases) 🟢
**Priority**: Error handling code, edge cases

**Approach**:
1. Review each remaining unwrap
2. Add comment if genuinely safe
3. Replace if any possibility of failure

---

## 🛠️ Tools & Helpers

### SafeEnv Helper (Already Exists)
```rust
use songbird_types::SafeEnv;

// Safe parsing with fallback
let port: u16 = SafeEnv::parse("PORT", 8080);

// Safe get with default
let host = SafeEnv::get_or_default("HOST", "localhost");

// Safe get with Result
let required = SafeEnv::get("REQUIRED_VAR")?;
```

### Error Context Extension
```rust
use anyhow::Context;

let config = load_config()
    .context("Failed to load configuration")?;

let value = parse_value(&input)
    .with_context(|| format!("Failed to parse input: {}", input))?;
```

### Custom Error Types
```rust
use songbird_types::SongbirdError;

// Rich error types with context
return Err(SongbirdError::configuration(
    format!("Invalid port: {}", port)
));

return Err(SongbirdError::network(
    format!("Connection failed: {}", addr)
));
```

---

## 🚫 When Unwrap IS Acceptable

### 1. Static Known Values
```rust
// OK: Regex is known valid at compile time
static RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\d+$").unwrap() // Safe: pattern is valid
});
```

### 2. Mutex/RwLock in Tests
```rust
#[test]
fn test_concurrent_access() {
    let lock = Arc::new(RwLock::new(0));
    // OK in tests: we want to know if locks fail
    *lock.write().unwrap() = 42;
}
```

### 3. Documented Invariants
```rust
// OK if well-documented
let value = map.get(key).unwrap(); // SAFETY: key guaranteed present by [...]

// Even better: assert the invariant
let value = map.get(key).expect("key must be present due to initialization");
```

### 4. Types that Can't Fail
```rust
// OK: Some operations are infallible
let vec = vec![1, 2, 3];
let clone = vec.clone(); // Can't fail
```

---

## 📋 Migration Checklist

### Per-File Audit
- [ ] Identify all unwrap/expect calls
- [ ] Categorize by risk (hot path vs initialization)
- [ ] Check if operation can actually fail
- [ ] Add proper error handling or document safety
- [ ] Add tests for error scenarios
- [ ] Review in PR

### Verification
```bash
# Find remaining unwraps in production code
rg "\.unwrap\(\)" crates/songbird-orchestrator/src --type rust | grep -v test

# Find expects
rg "\.expect\(" crates/songbird-orchestrator/src --type rust | grep -v test

# Find panics
rg "panic!\(" crates/songbird-orchestrator/src --type rust | grep -v test
```

---

## 📊 Progress Tracking

### Week 1 (Current)
- [x] Document error handling patterns
- [x] Create migration guide
- [ ] Audit hot paths (150 unwraps)
- [ ] Begin Phase 1 migration

### Week 2
- [ ] Complete hot path migration
- [ ] Begin initialization migration
- [ ] Add error scenario tests

### Week 3
- [ ] Complete initialization migration
- [ ] Begin internal logic migration
- [ ] Add error documentation

### Week 4
- [ ] Complete all migrations
- [ ] Add CI checks
- [ ] Final audit

---

## 🎯 Success Criteria

### Must Have
- [ ] Zero unwrap() in hot paths (network, I/O)
- [ ] All fallible constructors return Result
- [ ] Comprehensive error context everywhere
- [ ] Error scenarios tested

### Should Have
- [ ] <50 total unwraps in production code
- [ ] All remaining unwraps documented with SAFETY comments
- [ ] Error handling guide in docs

### Nice to Have
- [ ] Zero panics in production code
- [ ] Automated unwrap detection in CI
- [ ] Error recovery strategies documented

---

## 💡 Best Practices

### DO ✅
```rust
// DO: Proper error propagation
pub async fn process() -> Result<Output, SongbirdError> {
    let input = get_input().await?;
    let result = transform(input)?;
    Ok(result)
}

// DO: Add context to errors
let config = load_config()
    .context("Failed to load application configuration")?;

// DO: Use SafeEnv for environment variables
let port: u16 = SafeEnv::parse("PORT", 8080);

// DO: Document invariants
let value = map.get(key).expect("key present: ensured by initialization");
```

### DON'T ❌
```rust
// DON'T: Unwrap without considering failure
let value = risky_operation().unwrap();

// DON'T: Panic in production paths
if condition {
    panic!("This should never happen");
}

// DON'T: Silently ignore errors
let _ = operation(); // Discards error

// DON'T: Generic error messages
.expect("failed") // What failed? Why?
```

---

## 🔍 Code Review Guidelines

When reviewing error handling changes:

1. **Check propagation**: Are errors properly propagated up?
2. **Verify context**: Do errors have meaningful messages?
3. **Test failure**: Are error scenarios tested?
4. **Consider recovery**: Can the error be recovered from?
5. **Review unwraps**: Are remaining unwraps documented?

---

## 📚 References

- [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [anyhow crate](https://docs.rs/anyhow/)
- [thiserror crate](https://docs.rs/thiserror/)
- [Error Handling Survey](https://blog.burntsushi.net/rust-error-handling/)

---

**Status**: 🔄 Phase 1 In Progress  
**Target**: <50 production unwraps  
**Current**: ~714 unwraps identified

*Generated: December 22, 2025*

