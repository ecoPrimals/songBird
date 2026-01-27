# Test Port Hardcoding Cleanup Plan

## Current Status

The `songbird-universal` test suite contains **246 hardcoded port references** across 29 test files.

## Problem

Hardcoded ports in tests can cause:
- Test flakiness due to port conflicts
- Inability to run tests in parallel
- Failures in CI/CD environments with port restrictions
- Maintenance burden when ports need to change

## Examples of Hardcoding

```rust
// ❌ BAD: Hardcoded ports
let endpoints = vec![
    "http://endpoint1:8080".to_string(),
    "http://endpoint2:8080".to_string(),
];

// ✅ GOOD: Ephemeral ports via capability registry
use songbird_config::capability_port_config::CapabilityPortRegistry;

let registry = CapabilityPortRegistry::new();
let port1 = registry.register_ephemeral("test.endpoint1".into(), None)?;
let port2 = registry.register_ephemeral("test.endpoint2".into(), None)?;
let endpoints = vec![
    format!("http://localhost:{}", port1),
    format!("http://localhost:{}", port2),
];
```

## Affected Files (Top Offenders)

1. `load_balancer_error_paths_tests.rs` - 51 hardcoded ports
2. `load_balancer_async_integration_tests.rs` - 29 hardcoded ports
3. `security_adapter_comprehensive_coverage_tests.rs` - 18 hardcoded ports
4. `compute_adapter_comprehensive_coverage_tests.rs` - 18 hardcoded ports
5. `security_adapter_integration_tests.rs` - 18 hardcoded ports
6. `adapters_integration_tests.rs` - 18 hardcoded ports
7. `discovery_comprehensive_tests.rs` - 17 hardcoded ports

## Cleanup Strategy

### Phase 1: Infrastructure (✅ COMPLETE)
- [x] Create `CapabilityPortRegistry` in `songbird-config`
- [x] Add `to_capability_registry()` to `PortConfig`
- [x] Document capability-based port allocation

### Phase 2: Test Helper Module (DEFERRED)
- [ ] Create test-specific port allocator helper
- [ ] Handle Rust test module structure constraints
- [ ] Provide ergonomic API for test port allocation

### Phase 3: Migration (PENDING)
- [ ] Migrate `load_balancer_error_paths_tests.rs` (51 refs)
- [ ] Migrate `load_balancer_async_integration_tests.rs` (29 refs)
- [ ] Migrate security adapter tests (54 refs total)
- [ ] Migrate compute adapter tests (18 refs)
- [ ] Migrate remaining test files (116 refs)

### Phase 4: Validation (PENDING)
- [ ] Run full test suite with ephemeral ports
- [ ] Verify no port conflicts in parallel execution
- [ ] Update test documentation
- [ ] Remove this cleanup plan file

## Technical Challenges

1. **Test Module Structure**: Rust test files are compiled as separate crates, making shared test helpers complex
2. **Async Test Infrastructure**: Need to handle both sync and async test contexts
3. **Backward Compatibility**: Some tests may rely on specific port numbers for mocking
4. **CI/CD Integration**: Ensure ephemeral ports work in containerized environments

## Alternative Approaches

### Option A: Test-Specific Constants (Current)
```rust
// Each test file defines its own port range
const TEST_PORT_BASE: u16 = 8080;
```
**Pros**: Simple, no infrastructure needed
**Cons**: Still hardcoded, can conflict

### Option B: Environment Variables
```rust
// Tests read ports from environment
let port = env::var("TEST_PORT_1").unwrap_or("8080".to_string());
```
**Pros**: Configurable per test run
**Cons**: Requires test runner configuration, not truly dynamic

### Option C: Capability Registry (RECOMMENDED)
```rust
// Use CapabilityPortRegistry for ephemeral ports
let registry = CapabilityPortRegistry::new();
let port = registry.register_ephemeral("test.service".into(), None)?;
```
**Pros**: Truly dynamic, no conflicts, production-ready pattern
**Cons**: Requires infrastructure setup

## Decision

**DEFERRED TO FUTURE SESSION**

Reason: Test infrastructure complexity requires careful design to avoid breaking existing tests. The capability registry infrastructure is in place and ready for adoption when test refactoring is prioritized.

## Immediate Action

For new tests:
- ✅ Use `CapabilityPortRegistry::register_ephemeral()` for port allocation
- ✅ Avoid hardcoded ports
- ✅ Document port allocation strategy in test comments

For existing tests:
- ⏸️ Leave as-is until systematic refactoring session
- ⏸️ Mark as technical debt in STATUS.md
- ⏸️ Track in ROADMAP.md for future cleanup

## References

- `crates/songbird-config/src/capability_port_config.rs` - Port registry implementation
- `crates/songbird-config/src/canonical/hardcoded_elimination.rs` - Integration example
- `sessions/DEEP_EVOLUTION_SESSION_JAN_27_2026.md` - Session documentation

---

**Created**: January 27, 2026
**Status**: DEFERRED
**Priority**: Medium (technical debt, not blocking)
**Estimated Effort**: 4-6 hours for full migration

