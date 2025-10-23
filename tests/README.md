# Songbird Test Suite

Comprehensive testing infrastructure for the Songbird orchestrator.

## Test Structure

```
tests/
├── e2e/           # End-to-End integration tests
├── chaos/         # Chaos engineering tests
└── fault/         # Fault injection tests
```

## Test Categories

### End-to-End Tests (`tests/e2e/`)

Tests complete system behavior including:
- **Orchestration**: Full system lifecycle and coordination
- **Service Discovery**: Registration, discovery, and health monitoring
- **Capability Routing**: Request routing and load balancing
- **Fault Tolerance**: Recovery and resilience

**Run with:**
```bash
cargo test --test e2e
```

### Chaos Tests (`tests/chaos/`)

Chaos engineering tests that inject random failures:
- **Network Chaos**: Packet loss, latency, connection resets
- **Resource Chaos**: Memory pressure, CPU saturation, disk full
- **Timing Chaos**: Clock skew, timeouts, race conditions
- **State Chaos**: Corrupted configuration, inconsistent state

**Run with:**
```bash
cargo test --test chaos -- --ignored
```

### Fault Tests (`tests/fault/`)

Deterministic fault injection for specific scenarios:
- **Component Failures**: Individual component error handling
- **Integration Failures**: Cross-component failure scenarios
- **Recovery Scenarios**: System recovery paths

**Run with:**
```bash
cargo test --test fault
```

## Test Philosophy

### E2E Tests
- **Purpose**: Verify system works as expected in normal conditions
- **When**: Every PR, before release
- **Coverage**: Happy paths + common error cases

### Chaos Tests
- **Purpose**: Verify system resilience under random failures
- **When**: Weekly, before major releases
- **Coverage**: Unexpected failure modes

### Fault Tests
- **Purpose**: Verify specific error handling code paths
- **When**: Every PR, CI/CD
- **Coverage**: All error paths

## Running Tests

### All Tests
```bash
cargo test --workspace
```

### Only Unit Tests
```bash
cargo test --lib
```

### Only Integration Tests
```bash
cargo test --test '*'
```

### Only E2E Tests
```bash
cargo test --test e2e
```

### Only Chaos Tests (with ignored)
```bash
cargo test --test chaos -- --ignored
```

### With Coverage
```bash
cargo tarpaulin --workspace --out Html
```

## Writing New Tests

### E2E Test Template
```rust
#[tokio::test]
async fn test_my_feature() {
    // Arrange: Set up test environment
    let env = TestEnvironment::new().await;
    
    // Act: Execute the feature
    let result = env.execute_feature().await;
    
    // Assert: Verify expected behavior
    assert!(result.is_ok());
}
```

### Chaos Test Template
```rust
#[tokio::test]
#[ignore] // Chaos tests are ignored by default
async fn chaos_test_my_scenario() {
    let config = ChaosConfig::default();
    
    // 1. Start system normally
    // 2. Inject chaos
    // 3. Verify system behavior
    // 4. Stop chaos
    // 5. Verify recovery
}
```

### Fault Test Template
```rust
#[tokio::test]
async fn fault_test_specific_failure() {
    // 1. Set up scenario
    // 2. Inject specific fault
    // 3. Verify error handling
    // 4. Verify recovery
}
```

## Test Status

### Current Coverage
- Unit Tests: ✅ 523 passing
- Integration Tests: ✅ Present in crates
- E2E Tests: 🚧 Framework created, implementation pending
- Chaos Tests: 🚧 Framework created, implementation pending
- Fault Tests: 🚧 Framework created, implementation pending

### Implementation Plan

**Phase 1: E2E Tests** (Current)
- [ ] Implement test environment setup
- [ ] Implement orchestration tests
- [ ] Implement service discovery tests
- [ ] Implement capability routing tests
- [ ] Implement fault tolerance tests

**Phase 2: Fault Tests**
- [ ] Implement component failure tests
- [ ] Implement integration failure tests
- [ ] Implement recovery scenario tests

**Phase 3: Chaos Tests**
- [ ] Set up chaos infrastructure
- [ ] Implement network chaos tests
- [ ] Implement resource chaos tests
- [ ] Implement timing chaos tests
- [ ] Implement state chaos tests

## Contributing

When adding new tests:
1. Choose the appropriate test category
2. Follow the test templates
3. Add documentation explaining what you're testing
4. Update this README if adding new test categories

## CI/CD Integration

### PR Checks
- All unit tests must pass
- All integration tests must pass
- E2E tests must pass
- Fault tests must pass
- Coverage must be ≥90%

### Weekly Checks
- All chaos tests must pass
- Performance benchmarks must pass

### Release Checks
- Full test suite (including ignored tests)
- Comprehensive chaos testing
- Performance validation

