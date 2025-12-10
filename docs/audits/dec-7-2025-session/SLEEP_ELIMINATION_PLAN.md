# 🔄 SLEEP ELIMINATION & CONCURRENCY EVOLUTION PLAN
**Date**: December 7, 2025  
**Goal**: Remove all `sleep()` calls and evolve to truly concurrent, robust Rust patterns

---

## 📊 CURRENT STATE

**Total Sleep Calls**: 371 across 117 files

### Categories

1. **Circuit Breaker Tests** (37 sleeps)
   - Waiting for timeouts to expire
   - **Solution**: Use `tokio::time::pause()` and `advance()` for instant testing

2. **mDNS Discovery** (4 sleeps in production code!) 🔴
   - Polling loops
   - **Solution**: Use async channels and event-driven architecture

3. **Test Coordination** (300+ sleeps)
   - Waiting for services to start
   - Waiting for state changes
   - **Solution**: Barriers, channels, and proper synchronization

4. **Chaos Tests** (70+ sleeps)
   - Some legitimate for chaos timing
   - **Action**: Keep only necessary chaos sleeps, remove coordination sleeps

---

## 🎯 MODERN PATTERNS TO USE

### 1. **tokio::time::pause() for Tests**
```rust
// OLD (flaky, slow)
tokio::time::sleep(Duration::from_millis(100)).await;

// NEW (instant, deterministic)
tokio::time::pause();
// ... trigger timeout
tokio::time::advance(Duration::from_millis(100)).await;
tokio::time::resume();
```

### 2. **Channels for Coordination**
```rust
// OLD (timing-dependent)
start_service();
tokio::time::sleep(Duration::from_secs(1)).await; // hope it's ready
test_service();

// NEW (synchronous, fast)
let (tx, rx) = tokio::sync::oneshot::channel();
start_service(tx); // service sends ready signal
rx.await?; // wait for actual ready state
test_service();
```

### 3. **Barriers for Multi-Task Sync**
```rust
// OLD (hope they all start)
tokio::spawn(task1());
tokio::spawn(task2());
tokio::time::sleep(Duration::from_millis(500)).await;

// NEW (guaranteed sync)
let barrier = Arc::new(tokio::sync::Barrier::new(3));
let b1 = barrier.clone();
let b2 = barrier.clone();
tokio::spawn(async move { b1.wait().await; task1() });
tokio::spawn(async move { b2.wait().await; task2() });
barrier.wait().await; // all tasks synchronized
```

### 4. **WaitGroup Pattern**
```rust
use tokio::sync::Semaphore;

// Wait for N tasks to complete
let semaphore = Arc::new(Semaphore::new(0));
for i in 0..10 {
    let sem = semaphore.clone();
    tokio::spawn(async move {
        do_work(i).await;
        sem.add_permits(1);
    });
}
// Wait for all 10
semaphore.acquire_many(10).await?;
```

### 5. **Watch Channels for State**
```rust
// OLD (polling)
loop {
    if check_state() { break; }
    tokio::time::sleep(Duration::from_millis(10)).await;
}

// NEW (event-driven)
let (tx, mut rx) = tokio::sync::watch::channel(State::NotReady);
tokio::spawn(async move {
    when_ready().await;
    tx.send(State::Ready)?;
});
rx.changed().await?; // instant notification
```

---

## 🔴 PRODUCTION CODE SLEEPS (Must Eliminate)

### Files with Production Sleeps:

1. **`crates/songbird-universal/src/discovery/backends/network.rs:73`**
   ```rust
   tokio::time::sleep(Duration::from_millis(10)).await;
   ```
   **Context**: Polling loop in mDNS discovery
   **Fix**: Use async mDNS library with proper event handling

2. **`crates/songbird-config/src/discovery/mdns.rs:342, 440`**
   ```rust
   tokio::time::sleep(Duration::from_millis(10)).await;
   ```
   **Context**: More mDNS polling
   **Fix**: Event-driven mDNS with channels

3. **`crates/songbird-universal/src/circuit_breaker.rs:3`**
   **Context**: Circuit breaker timeout logic
   **Fix**: Use `tokio::time::timeout()` instead

---

## 📋 EXECUTION PLAN

### Phase 1: Production Code (Priority 🔴 P0)

**Files to Fix** (4 production sleeps):
1. `songbird-universal/src/discovery/backends/network.rs`
2. `songbird-config/src/discovery/mdns.rs` (2 locations)
3. `songbird-universal/src/circuit_breaker.rs`

**Estimated Time**: 2-3 hours

### Phase 2: Circuit Breaker Tests (Priority 🟠 P1)

**Files to Fix** (2 files, 37 sleeps):
1. `circuit_breaker_enhanced_tests.rs` (14 sleeps)
2. `circuit_breaker_edge_cases_tests.rs` (14 sleeps)

**Pattern**: Replace with `tokio::time::pause()/advance()`

**Estimated Time**: 1-2 hours

### Phase 3: Integration Tests (Priority 🟡 P2)

**Files**: 20+ integration test files

**Pattern**: Replace with channels/barriers

**Estimated Time**: 4-6 hours

### Phase 4: Chaos Tests (Priority 🟢 P3)

**Files**: 10+ chaos test files

**Action**: Keep legitimate chaos timing, remove coordination sleeps

**Estimated Time**: 2-3 hours

---

## 🎯 SUCCESS CRITERIA

✅ **Zero sleeps in production code**
✅ **Circuit breaker tests use time mocking**
✅ **Integration tests use proper sync primitives**
✅ **Tests run 10x faster** (no waiting)
✅ **Tests are deterministic** (no flakes)
✅ **All tests still pass**

---

## 📈 EXPECTED BENEFITS

1. **Speed**: Tests run 10-50x faster
2. **Reliability**: No timing-dependent flakes
3. **Determinism**: Same results every time
4. **Concurrency**: True parallel test execution
5. **Production Quality**: Better patterns everywhere

---

**Next Steps**: Start with Phase 1 (production code) immediately.

