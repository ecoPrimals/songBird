# 🔄 **HARDCODING MIGRATION GUIDE**

**Date**: November 18, 2025  
**Scope**: Primal Name Elimination (1,030 instances)  
**Status**: STRATEGIC APPROACH

---

## 📊 **CURRENT SITUATION**

### Hardcoding Analysis
```
Total Primal References: 1,030 instances across 100 files

Breakdown:
  ToadStool:  280 instances
  BearDog:    245 instances
  NestGate:   255 instances  
  Squirrel:   250 instances

Context Distribution:
  Test Code:         ~600 instances (58%) - ACCEPTABLE
  Documentation:     ~200 instances (19%) - ACCEPTABLE
  Legacy Adapters:   ~150 instances (15%) - NEEDS MIGRATION
  SDK Integration:    ~80 instances (8%)  - NEEDS MIGRATION
```

---

## 🎯 **MIGRATION STRATEGY**

### Phase 1: PRODUCTION CODE ONLY (230 instances)

**Priority: HIGH** - Migrate production code that violates universal capability pattern

**Targets:**
1. Legacy adapter code (~150 instances)
2. SDK integration code (~80 instances)

### Phase 2: TEST/DOC CODE (800 instances)

**Priority: LOW** - Test and documentation code is acceptable

**Rationale:**
- Test mocks SHOULD reference specific primals (that's their purpose)
- Documentation examples need concrete names for clarity
- These don't violate sovereignty principles

---

## 🔄 **MIGRATION PATTERNS**

### Pattern 1: Direct Primal References

**❌ BEFORE (Hardcoded):**
```rust
let toadstool = get_primal_endpoint("toadstool");
let beardog = get_primal_endpoint("beardog");
```

**✅ AFTER (Capability-Based):**
```rust
use songbird_config::capability_endpoints::CapabilityEndpointResolver;

let resolver = CapabilityEndpointResolver::new();
let compute = resolver.get_endpoint(&CapabilityType::Compute).await?;
let security = resolver.get_endpoint(&CapabilityType::Security).await?;
```

### Pattern 2: Service Discovery

**❌ BEFORE:**
```rust
let providers = discovery.find_providers("ToadStool").await?;
```

**✅ AFTER:**
```rust
let providers = discovery.discover_capability("compute").await?;
```

### Pattern 3: Adapter Creation

**❌ BEFORE:**
```rust
let adapter = ComputeAdapter::new_for_toadstool(endpoint)?;
```

**✅ AFTER:**
```rust
let adapter = ComputeAdapter::from_discovery().await?;
// Uses capability-based discovery internally
```

### Pattern 4: Configuration

**❌ BEFORE:**
```rust
config.toadstool_endpoint = "http://toadstool:8080";
config.beardog_endpoint = "http://beardog:8081";
```

**✅ AFTER:**
```rust
// Set capability endpoints via environment
env::set_var("CAPABILITY_COMPUTE_ENDPOINT", "http://localhost:8080");
env::set_var("CAPABILITY_SECURITY_ENDPOINT", "http://localhost:8081");

// Or use discovery
let config = CapabilityConfig::from_discovery().await?;
```

---

## 📋 **ACCEPTABLE HARDCODING**

### Test Code ✅

```rust
// ✅ ACCEPTABLE: Test mocks should reference specific implementations
#[tokio::test]
async fn test_toadstool_integration() {
    let mock_toadstool = MockPrimalServer::new("toadstool", 8080);
    // Testing specific primal integration
}
```

**Reason:** Tests validate specific integrations. Mocks need concrete implementations.

### Documentation ✅

```rust
/// # Example
///
/// Connect to ToadStool for compute capabilities:
/// ```
/// let compute = discover_capability("compute").await?;
/// // Could be ToadStool, or any compute provider
/// ```
```

**Reason:** Documentation needs concrete examples for clarity.

### Test Fixtures ✅

```rust
pub fn create_test_beardog() -> MockPrimalServer {
    MockPrimalServer::new("beardog", test_security_port())
}
```

**Reason:** Test fixtures represent specific test scenarios.

---

## 🚫 **UNACCEPTABLE HARDCODING**

### Production Code ❌

```rust
// ❌ BAD: Production code should never hardcode primal names
pub fn production_workflow() -> Result<()> {
    let toadstool = connect_to("toadstool:8080")?;
    let beardog = connect_to("beardog:8081")?;
    // WRONG: Violates universal capability pattern
}
```

### Adapters ❌

```rust
// ❌ BAD: Adapters should be capability-based
impl ComputeAdapter {
    pub fn new_for_toadstool(endpoint: String) -> Self {
        // WRONG: Assumes ToadStool is compute provider
    }
}
```

### Discovery Logic ❌

```rust
// ❌ BAD: Discovery should not know primal names
pub fn find_compute_provider() -> Option<Endpoint> {
    if exists("toadstool") {
        // WRONG: Hardcodes primal name in discovery
    }
}
```

---

## 🛠️ **MIGRATION TOOLS**

### Available Tools

1. **CapabilityEndpointResolver**
   - Location: `crates/songbird-config/src/capability_endpoints.rs`
   - Purpose: Resolve capability types to endpoints
   - Usage: See Pattern 1 above

2. **ZeroHardcodingMigrator**
   - Location: `crates/songbird-config/src/zero_hardcoding_migration.rs`
   - Purpose: Automated detection and replacement
   - Status: Available but needs careful application

3. **Environment Variables**
   - `CAPABILITY_COMPUTE_ENDPOINT`
   - `CAPABILITY_SECURITY_ENDPOINT`
   - `CAPABILITY_STORAGE_ENDPOINT`
   - `CAPABILITY_AI_ENDPOINT`

---

## 📈 **MIGRATION PHASES**

### Phase 1: Identify Production Code (Complete ✅)

**Analysis Complete:**
- 230 instances in production code
- 150 in legacy adapters
- 80 in SDK integration

### Phase 2: Migrate Adapters (Week 1)

**Targets:**
- `crates/songbird-universal/src/adapters/*.rs`
- `crates/songbird-primal-sdk/src/*.rs`

**Approach:**
1. Review each hardcoded reference
2. Replace with capability-based pattern
3. Update tests
4. Verify functionality

**Estimated Effort:** 2-3 days

### Phase 3: Migrate SDK (Week 1-2)

**Targets:**
- `crates/songbird-primal-sdk/src/discovery/*.rs`
- SDK integration points

**Approach:**
1. Update SDK to use capability types
2. Maintain backward compatibility where needed
3. Add deprecation warnings
4. Update documentation

**Estimated Effort:** 1-2 days

### Phase 4: Validate (Week 2)

**Actions:**
1. Run full test suite
2. Verify no production hardcoding remains
3. Check capability discovery works
4. Update documentation

**Estimated Effort:** 1 day

---

## 🎯 **SUCCESS CRITERIA**

### Must Have ✅

1. **Zero Production Hardcoding**
   - No primal names in production adapter code
   - No primal names in production discovery logic
   - No primal names in production routing

2. **Capability-Based APIs**
   - All adapters use `from_discovery()`
   - All discovery uses capability types
   - All routing uses capability matching

3. **Tests Pass**
   - All 582 tests continue passing
   - Coverage remains at 62%+
   - No functionality regression

### Nice to Have 🎁

1. **Reduced Test Hardcoding**
   - Abstract common test patterns
   - Use capability-based test helpers
   - (But direct primal mocks still acceptable)

2. **Documentation Updates**
   - Clear examples of capability pattern
   - Migration guide for users
   - Architecture documentation

---

## 📊 **CURRENT STATUS**

### ✅ Already Migrated

**Good News:** Most production code already uses proper patterns!

```rust
// ✅ Already using capability-based discovery
impl SecurityAdapter {
    pub async fn from_discovery() -> SongbirdResult<Self> {
        let resolver = CapabilityEndpointResolver::new();
        match resolver.get_endpoint(&CapabilityType::Security).await {
            Ok(endpoint) => Self::new(endpoint),
            Err(_) => {
                // Fallback patterns available
            }
        }
    }
}
```

### 🔄 Needs Migration

**Legacy Code Locations:**
1. `crates/songbird-primal-sdk/src/toadstool.rs`
2. `crates/songbird-primal-sdk/src/beardog.rs`
3. `crates/songbird-primal-sdk/src/squirrel.rs`
4. Some test utilities in `songbird-test-utils`

---

## 💡 **RECOMMENDATIONS**

### Immediate Actions

1. **✅ ACCEPT Test/Doc Hardcoding**
   - 800 instances are appropriate (test mocks, examples)
   - Don't waste time changing these
   - Focus on production code only

2. **🔄 Migrate Legacy SDK**
   - 230 production instances to review
   - Systematic adapter updates
   - 1 week estimated effort

3. **📝 Document Pattern**
   - Clear before/after examples
   - Migration guide for contributors
   - Architecture decision record

### Long-term Strategy

1. **Lint Rule** (Future)
   - Create clippy lint for primal names in production code
   - Allow in test code
   - Enforce in CI

2. **Code Review**
   - Check for hardcoding in PRs
   - Ensure new code uses capability pattern
   - Maintain sovereignty principles

---

## 🎉 **CONCLUSION**

### Key Insight

**Only 22% of hardcoding needs migration (230/1,030 instances)**

The majority (78%) is in tests and documentation where it's appropriate. This makes the migration task much smaller and more focused than initially thought.

### Action Plan

1. **Week 1:** Migrate legacy adapter code (150 instances)
2. **Week 1-2:** Migrate SDK integration (80 instances)
3. **Week 2:** Validate and document

**Total Effort: 1-2 weeks**  
**Impact: Production code fully capability-based**

---

**Status**: STRATEGIC PLAN COMPLETE  
**Next**: Execute adapter migration (150 instances)  
**Timeline**: 1-2 weeks for completion

---

*Generated: November 18, 2025*  
*Migration Strategy: Focus on production code, accept test/doc patterns*

