# Coverage Improvement Plan - Path to 90%

## Current Status (January 27, 2026)

### Measured Coverage

| Crate | Current | Target | Gap |
|-------|---------|--------|-----|
| songbird-config | 46.05% | 90% | +43.95% |
| songbird-http-client | 43.22% | 90% | +46.78% |
| songbird-orchestrator | Not measured | 90% | TBD |
| **Overall** | ~78% (STATUS.md) | 90% | +12% |

## Coverage Gaps Analysis

### songbird-http-client (43.22%)

#### Critical Gaps (Production Code)
1. **tls/server_complete.rs** - 12.96% coverage (1,088 lines)
   - TLS 1.3 server implementation
   - Complex protocol state machine
   - **Priority**: HIGH (production-critical)

2. **tls/record.rs** - 15.13% coverage (813 lines)
   - TLS record layer protocol
   - Encryption/decryption logic
   - **Priority**: HIGH (security-critical)

3. **tls/handshake_refactored/record_io.rs** - 24.47% coverage (470 lines)
   - Handshake record I/O operations
   - Buffer management
   - **Priority**: MEDIUM

4. **tls/server.rs** - 39.61% coverage (207 lines)
   - Server connection handling
   - **Priority**: MEDIUM

#### Strong Coverage (Keep Maintaining)
- tls/handshake_v2/protocol.rs - 97.03% ✅
- tls/session.rs - 95.77% ✅
- types.rs - 93.51% ✅
- tls/handshake_v2/client_hello.rs - 89.22% ✅

### songbird-config (46.05%)

#### Critical Gaps
1. **zero_touch/infant_config.rs** - 30.51% coverage (531 lines)
   - Zero-touch deployment configuration
   - **Priority**: MEDIUM

2. **unified/core.rs** - 0% coverage (35 lines)
   - Unified configuration core
   - **Priority**: HIGH (zero coverage!)

3. **unified/federation.rs** - 0% coverage (119 lines)
   - Federation configuration
   - **Priority**: HIGH (zero coverage!)

4. **primal_discovery.rs** - 51.66% coverage (271 lines)
   - Primal service discovery
   - **Priority**: MEDIUM

5. **runtime_discovery.rs** - 58.38% coverage (370 lines)
   - Runtime service discovery
   - **Priority**: MEDIUM

#### Strong Coverage
- capability_port_config.rs - 100% ✅ (NEW!)
- test_helpers.rs - 95.89% ✅
- env_override.rs - 87.38% ✅

## Strategy to Reach 90%

### Phase 1: Zero Coverage Elimination (Week 1)
**Goal**: Eliminate all 0% coverage modules

1. Add tests for `songbird-config/unified/core.rs`
2. Add tests for `songbird-config/unified/federation.rs`
3. Add tests for `songbird-config/unified/robustness.rs`
4. Add tests for `songbird-config/zero_touch/mod.rs`

**Expected Gain**: +5-10% overall coverage

### Phase 2: TLS Critical Paths (Week 2-3)
**Goal**: Cover critical TLS security paths

1. **tls/record.rs** (15% → 70%)
   - Test record encryption/decryption
   - Test record parsing and validation
   - Test error handling for malformed records

2. **tls/server_complete.rs** (13% → 60%)
   - Test server handshake happy path
   - Test server handshake error paths
   - Test certificate validation
   - Test cipher negotiation

3. **tls/server.rs** (40% → 80%)
   - Test server connection lifecycle
   - Test concurrent connections
   - Test error recovery

**Expected Gain**: +15-20% overall coverage

### Phase 3: Discovery & Configuration (Week 4)
**Goal**: Cover discovery and runtime configuration paths

1. **primal_discovery.rs** (52% → 85%)
   - Test discovery initialization
   - Test service registration
   - Test service lookup
   - Test failure scenarios

2. **runtime_discovery.rs** (58% → 85%)
   - Test dynamic endpoint resolution
   - Test configuration updates
   - Test fallback mechanisms

3. **zero_touch/infant_config.rs** (31% → 75%)
   - Test zero-touch initialization
   - Test configuration discovery
   - Test bootstrap scenarios

**Expected Gain**: +10-15% overall coverage

### Phase 4: Integration & E2E (Week 5)
**Goal**: Add integration tests for coverage gaps

1. TLS handshake integration tests
2. Service discovery integration tests
3. Configuration lifecycle tests
4. Error recovery scenarios

**Expected Gain**: +5-10% overall coverage

## Test Types Needed

### Unit Tests (70% of coverage)
- ✅ Already strong in most modules
- ❌ Missing in unified/* modules
- ❌ Missing in TLS server paths

### Integration Tests (20% of coverage)
- ✅ Some exist in songbird-universal
- ❌ Need more TLS integration tests
- ❌ Need discovery integration tests

### E2E Tests (10% of coverage)
- ✅ Mentioned in specs
- ❌ Not measured in current coverage
- ❌ Need chaos testing
- ❌ Need fault injection

## Tools & Infrastructure

### Current Setup
- ✅ `cargo-llvm-cov` installed
- ✅ Can generate HTML reports
- ✅ Can measure per-crate coverage

### Needed Improvements
- [ ] CI/CD integration for coverage tracking
- [ ] Coverage regression prevention
- [ ] Per-PR coverage diff
- [ ] Coverage badge in README
- [ ] Automated coverage reports

## Success Criteria

### Minimum Viable (by Feb 15, 2026)
- [ ] All modules > 50% coverage
- [ ] Zero modules with 0% coverage
- [ ] TLS security paths > 70% coverage
- [ ] Overall coverage > 75%

### Target (by Mar 15, 2026)
- [ ] All production modules > 80% coverage
- [ ] Security-critical paths > 90% coverage
- [ ] Overall coverage > 85%

### Stretch Goal (by Apr 15, 2026)
- [ ] All production modules > 90% coverage
- [ ] 100% coverage of public APIs
- [ ] Overall coverage > 90%
- [ ] Chaos tests integrated
- [ ] Fault injection tests integrated

## Quick Wins (This Week)

### High-Value, Low-Effort Tests
1. ✅ `capability_port_config.rs` - Already 100%!
2. Test `unified/core.rs` (35 lines, 0% → 90%)
3. Test `unified/federation.rs` (119 lines, 0% → 80%)
4. Test `unified/robustness.rs` (14 lines, 0% → 90%)
5. Test `zero_touch/mod.rs` (19 lines, 0% → 90%)

**Expected Gain**: +3-5% overall with ~4 hours effort

## Coverage Best Practices

### What to Test
✅ Public APIs
✅ Error paths
✅ Edge cases
✅ Security-critical paths
✅ Complex logic paths

### What Not to Test
❌ Simple getters/setters
❌ Deprecated legacy code
❌ Auto-generated code
❌ Trivial implementations

### Test Quality Over Quantity
- **Meaningful assertions**: Test behavior, not just code execution
- **Edge cases**: Test boundaries and error conditions
- **Integration**: Test components working together
- **Regression**: Test bug fixes stay fixed

## Blockers & Risks

### Technical Blockers
1. **TLS Testing Complexity**: Need mock crypto operations
2. **Async Testing**: Requires careful setup
3. **Network Testing**: Need mock sockets/connections
4. **Race Conditions**: Tests must be deterministic

### Risk Mitigation
- Use test helpers for common setup
- Mock external dependencies
- Isolate tests with proper cleanup
- Use deterministic test data

## Next Steps

1. **Immediate** (Today):
   - [x] Establish baseline coverage measurements
   - [x] Create this improvement plan
   - [ ] Add tests for zero-coverage modules

2. **This Week**:
   - [ ] Eliminate all 0% coverage modules
   - [ ] Add TLS record layer tests
   - [ ] Add discovery tests
   - [ ] Reach 50% coverage in all modules

3. **This Month**:
   - [ ] Implement Phase 1 & 2 of strategy
   - [ ] Reach 75% overall coverage
   - [ ] Set up CI/CD coverage tracking

## References

- `STATUS.md` - Current status (~78% coverage overall)
- `UNIFIED_TESTING_FRAMEWORK_SPECIFICATION_2025.md` - Testing standards
- `sessions/DEEP_EVOLUTION_SESSION_JAN_27_2026.md` - Current session log
- Coverage reports: `target/llvm-cov/html/index.html`

---

**Created**: January 27, 2026
**Status**: ACTIVE
**Owner**: Songbird Team
**Target Completion**: March 15, 2026 (85%), April 15, 2026 (90%)

