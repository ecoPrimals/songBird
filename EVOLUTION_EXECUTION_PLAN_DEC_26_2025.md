# 🚀 Songbird Evolution Execution Plan - December 26, 2025

## Principles

1. **Deep Debt Solutions** - Not quick fixes, architectural improvements
2. **Modern Idiomatic Rust** - async/await, zero-cost abstractions, type safety
3. **Smart Refactoring** - Cohesive modules, not arbitrary splits
4. **Safe AND Fast** - Evolve unsafe to safe without performance loss
5. **Capability-Based** - Zero hardcoding, runtime discovery only
6. **Primal Self-Knowledge** - Each primal knows only itself
7. **Production Quality** - Replace mocks with real implementations
8. **Comprehensive Coverage** - E2E, chaos, fault injection

---

## Phase 1: Quick Wins (Hours 1-4)

### 1.1 Formatting ✅ (5 minutes)
```bash
cargo fmt
git diff  # Review changes
```

### 1.2 Clippy Fixes ✅ (30 minutes)
**Target**: 6 errors in bluetooth crate

1. Make 3 functions const (trivial)
2. Refactor CharacteristicProperties to bitflags (better design)
3. Tighten MutexGuard scope (performance + correctness)

### 1.3 Documentation HTML Tags (15 minutes)
Fix unclosed HTML tags in doc comments (13 instances)

**Total Phase 1**: ~1 hour, +3 grade points (92→95)

---

## Phase 2: Hardcoding Elimination (Week 1)

### 2.1 Port Configuration Evolution
**Current**: 4,688 hardcoded instances
**Target**: Environment-based with discovery fallback

**Approach**:
```rust
// OLD (hardcoded)
let url = "http://localhost:8080";

// NEW (capability-based discovery)
let endpoint = CapabilityDiscovery::find(CapabilityType::Orchestrator)
    .await?
    .first()
    .map(|e| e.endpoint.clone())
    .or_else(|| env::var("SONGBIRD_ORCHESTRATOR_URL").ok())
    .unwrap_or_else(|| "http://[::]:8080".to_string());
```

**Files to evolve** (Top 10 offenders):
1. `songbird-orchestrator/src/app/network.rs` - 15 instances
2. `songbird-primal-coordination/src/coordinator.rs` - 1 production instance
3. Test files - Leave as-is (acceptable in tests)

### 2.2 Primal Self-Knowledge Evolution
**Principle**: Primals discover, don't hardcode

**Current Pattern** (WRONG):
```rust
// Hardcoded dependency
let beardog_url = "http://localhost:8200";
let toadstool_url = "http://localhost:7000";
```

**Evolved Pattern** (CORRECT):
```rust
// Self-knowledge only
pub struct MyPrimalConfig {
    pub my_name: String,
    pub my_capabilities: Vec<Capability>,
    pub my_endpoint: SocketAddr,
    // NO references to other primals!
}

// Discovery at runtime
let security_primal = capability_registry
    .discover(CapabilityType::Security)
    .await?;
```

**Files to evolve**:
- All primal coordination code
- Federation discovery
- Config defaults

---

## Phase 3: Unwrap → Result Migration (Week 2)

### 3.1 Strategy
**Target**: 1,270 unwraps, focus on production hot paths

**Priority Order**:
1. Public API surfaces (20 unwraps)
2. Error propagation paths (50 unwraps)
3. Configuration loading (100 unwraps)
4. Internal helpers (remaining)

**Pattern Evolution**:
```rust
// OLD (panic risk)
let config = load_config().unwrap();

// NEW (proper error handling)
let config = load_config()
    .context("Failed to load configuration")?;

// Or with fallback
let config = load_config()
    .unwrap_or_else(|e| {
        warn!("Config load failed: {}, using defaults", e);
        Config::default()
    });
```

### 3.2 Custom Error Types
Create domain-specific errors:
```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {path}")]
    NotFound { path: PathBuf },
    
    #[error("Invalid configuration: {reason}")]
    Invalid { reason: String },
    
    #[error("Environment variable missing: {var}")]
    EnvVarMissing { var: String },
}
```

---

## Phase 4: Smart Refactoring (Week 3)

### 4.1 Large File: orchestrator/app/core.rs (1267 lines)

**Refactoring Strategy** (NOT just splitting):

**Identify Cohesive Modules**:
1. **Initialization** (~300 lines) → `core/initialization.rs`
2. **Discovery Integration** (~200 lines) → `core/discovery.rs`
3. **Federation Setup** (~200 lines) → `core/federation.rs`
4. **Server Management** (~300 lines) → `core/server.rs`
5. **Coordination** (~267 lines) → Keep in `core.rs` (orchestration logic)

**Module Structure**:
```
app/
├── core.rs (267 lines - main orchestration)
├── core/
│   ├── initialization.rs (server initialization)
│   ├── discovery.rs (discovery integration)
│   ├── federation.rs (federation setup)
│   └── server.rs (server lifecycle)
└── ...
```

**Benefits**:
- Each module has single responsibility
- Clear boundaries and dependencies
- Easier to test in isolation
- Maintainable (<1000 lines each)

### 4.2 Module Cohesion Analysis
Run for each large file:
1. Identify responsibilities
2. Map dependencies
3. Extract cohesive units
4. Preserve clear interfaces

---

## Phase 5: Unsafe → Safe+Fast Evolution (Week 4)

### 5.1 Current State
- 831 unsafe blocks (0.27% of code)
- Mostly in FFI and performance-critical paths
- All currently justified

### 5.2 Evolution Strategy

**For each unsafe block**:
1. **Document invariants** (already done mostly)
2. **Encapsulate in safe wrapper** (already done mostly)
3. **Consider safe alternatives**:
   - SIMD → `safe-simd` or `std::simd` (stable soon)
   - FFI → `safer-ffi` patterns
   - Raw pointers → `Pin`, `Rc`, `Arc`

**Example Evolution**:
```rust
// BEFORE (unsafe raw pointer)
unsafe {
    let ptr = data.as_mut_ptr();
    transmute::<_, &mut T>(ptr)
}

// AFTER (safe Pin-based)
let pinned = Pin::new(&mut data);
pinned.get_mut()
```

### 5.3 Zero-Copy Without Unsafe
Use safe abstractions:
- `bytes::Bytes` for shared ownership
- `Arc<[u8]>` for immutable sharing
- `Cow<'_, [u8]>` for copy-on-write

**Current**: 2,778 clones  
**Target**: Use zero-copy where semantically correct, clone where ownership needed

---

## Phase 6: Mock → Implementation Evolution (Week 5)

### 6.1 Production Mock Inventory
From audit: Minimal production mocks found

**Files to check**:
1. `network-federation/src/beardog/mock.rs` (28 TODOs)
2. `genesis/src/physical_channels/mock.rs` (23 TODOs)

### 6.2 Evolution Strategy

**Mock Pattern** (temporary):
```rust
#[cfg(not(feature = "mock"))]
pub use real_impl::*;

#[cfg(feature = "mock")]
pub use mock_impl::*;
```

**Evolution to Real**:
```rust
pub trait BearDogCrypto: Send + Sync {
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>>;
    async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>>;
}

// Real implementation (default)
pub struct BearDogClient { /* ... */ }

// Test double (tests only)
#[cfg(test)]
pub struct MockBearDogClient { /* ... */ }
```

### 6.3 Priority Order
1. BearDog integration (replace discovery mocks)
2. Genesis physical channels (hardware-dependent, keep test mocks)
3. Network mocks (use test fixtures)

---

## Phase 7: Test Coverage Expansion (Week 6)

### 7.1 Current State
- 2,261+ tests
- 63% coverage (estimated)
- Need: 90% target

### 7.2 Coverage Strategy

**Error Path Testing** (biggest gap):
```rust
#[tokio::test]
async fn test_config_load_missing_file() {
    let result = Config::load("nonexistent.toml").await;
    assert!(matches!(result, Err(ConfigError::NotFound { .. })));
}

#[tokio::test]
async fn test_network_connection_timeout() {
    let client = Client::new().with_timeout(Duration::from_millis(1));
    let result = client.connect("192.0.2.1:9999").await;
    assert!(matches!(result, Err(NetworkError::Timeout)));
}
```

**Chaos Testing**:
```rust
#[tokio::test]
async fn test_random_disconnects() {
    let mut network = TestNetwork::new();
    network.set_packet_loss(0.1); // 10% loss
    
    // System should recover
    let result = run_federation_test(network).await;
    assert!(result.is_ok());
}
```

**Fault Injection**:
```rust
#[tokio::test]
async fn test_disk_full_graceful_degradation() {
    let fs = TestFilesystem::new().set_disk_full(true);
    
    let result = save_state(&fs, &state).await;
    // Should fall back to memory-only
    assert!(matches!(result, Ok(SaveMode::MemoryOnly)));
}
```

---

## Phase 8: Idiomatic Rust Evolution (Ongoing)

### 8.1 Patterns to Apply

**1. Type-Driven Design**:
```rust
// Use newtypes for domain concepts
#[derive(Debug, Clone)]
pub struct PrimalId(Uuid);

#[derive(Debug, Clone)]
pub struct EndpointUrl(Url);

// Compiler enforces correct usage
fn connect(id: PrimalId, endpoint: EndpointUrl) { }
```

**2. Builder Pattern for Complex Types**:
```rust
let server = ServerBuilder::new()
    .with_port(8080)
    .with_tls(tls_config)
    .with_discovery(discovery_config)
    .build()?;
```

**3. Trait Objects for Extensibility**:
```rust
pub trait PhysicalChannel: Send + Sync {
    async fn initiate(&self) -> Result<Credentials>;
    async fn exchange(&self, their_creds: Credentials) -> Result<SharedSecret>;
}

// Easy to add new channels
pub struct BluetoothChannel { }
pub struct QrCodeChannel { }
pub struct NfcChannel { } // Future addition
```

**4. Error Context Chains**:
```rust
use anyhow::Context;

config.load()
    .context("Failed to load configuration")?
    .validate()
    .context("Configuration validation failed")?;
```

---

## Execution Timeline

```
Week 1: Formatting + Clippy + Hardcoding Start
├─ Day 1: Quick wins (formatting, clippy)
├─ Day 2-3: Port configuration evolution
├─ Day 4-5: Primal self-knowledge refactor
└─ Weekend: Review and test

Week 2: Unwrap Migration + Error Handling
├─ Day 1-2: Public API unwrap removal
├─ Day 3-4: Error type evolution
├─ Day 5: Configuration unwrap removal
└─ Weekend: Integration testing

Week 3: Smart Refactoring
├─ Day 1-2: Analyze core.rs cohesion
├─ Day 3-4: Extract and refactor modules
├─ Day 5: Verify and test
└─ Weekend: Documentation update

Week 4: Unsafe Evolution
├─ Day 1-2: Audit all unsafe blocks
├─ Day 3-4: Apply safe alternatives
├─ Day 5: Benchmark (ensure no regression)
└─ Weekend: Review

Week 5: Mock Evolution
├─ Day 1-2: BearDog integration
├─ Day 3-4: Genesis channels
├─ Day 5: Network fixtures
└─ Weekend: E2E testing

Week 6: Coverage Expansion
├─ Day 1-2: Error path tests
├─ Day 3: Chaos tests
├─ Day 4: Fault injection
├─ Day 5: Measure coverage (target: 90%)
└─ Weekend: Final review
```

---

## Success Criteria

### After Week 2:
- [ ] All formatting issues resolved
- [ ] All clippy errors fixed
- [ ] Hardcoding reduced by 50%
- [ ] Critical unwraps removed
- **Grade Target**: A (95/100)

### After Week 4:
- [ ] Large files refactored intelligently
- [ ] Unsafe code documented and minimized
- [ ] Hardcoding reduced by 90%
- **Grade Target**: A+ (97/100)

### After Week 6:
- [ ] Mocks evolved to implementations
- [ ] Test coverage at 90%+
- [ ] All TODOs triaged
- **Grade Target**: A+ (100/100) 🏆

---

## Monitoring

### Daily Metrics:
```bash
# Hardcoding count
grep -r "localhost\|127.0.0.1\|:8080" crates/ | wc -l

# Unwrap count
grep -r "\.unwrap()" crates/ | wc -l

# Coverage
cargo llvm-cov --html

# File sizes
find crates -name "*.rs" -exec wc -l {} + | awk '$1 > 1000'
```

### Weekly Review:
1. Grade assessment
2. Velocity check
3. Quality metrics
4. Adjust priorities

---

**Ready to Execute**: Starting with Phase 1 (Quick Wins)

🦀 **Modern. Idiomatic. Safe AND Fast. Human Dignity First.**

