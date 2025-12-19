# Hardcoding Evolution Progress - December 18, 2025

## ✅ Completed Actions

### 1. MdnsDiscovery API Fixed
- **Issue**: Missing `discover_by_capability` method
- **Solution**: Implemented complete method with proper signature
- **Status**: ✅ Compiling successfully
- **File**: `crates/songbird-config/src/capability_based_runtime_discovery/mdns.rs`

### 2. Code Formatting
- **Action**: Ran `cargo fmt --all`
- **Result**: All formatting inconsistencies resolved
- **Status**: ✅ Complete

### 3. Clippy Fixes
- **Location**: `showcase/05-albatross-multiplex/benchmark/`
- **Action**: Ran `cargo clippy --fix`
- **Result**: Auto-fixable issues resolved
- **Remaining**: Minor dead code warnings (acceptable for benchmarks)
- **Status**: ✅ Complete

## 📊 Hardcoding Analysis

### Current State (Per Audit):
- **Production Code**: 356 hardcoded instances
- **Test Code**: ~1,200 instances (acceptable)
- **Mock Code**: ~1,571 instances (acceptable)

### Categories of Hardcoding:

#### 1. **Configuration Constants** (Medium Risk)
**File**: `crates/songbird-config/src/canonical/constants.rs`

**Status**: 🟡 PARTIALLY EVOLVED
- ✅ Most functions use environment-first approach
- ✅ Dynamic calculation based on environment detection
- ⚠️ Still has hardcoded fallbacks for defaults

**Remaining Issues**:
```rust
// Lines 18-25: Hardcoded constants for "backwards compatibility with tests"
pub const LOCALHOST_IPV4: &str = "127.0.0.1";
pub const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1:8080";
pub const DEFAULT_LOCALHOST: &str = "127.0.0.1";

// Lines 693-708: Network module with hardcoded values
pub mod network {
    pub const DEFAULT_HOST: &str = "localhost";
    pub const DEFAULT_HOST_V4: &str = "127.0.0.1";
    pub const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1:8080";
    pub const DEFAULT_ORCHESTRATOR_PORT: u16 = 8080;
    // ...
}

// Lines 636-641: Development CORS origins hardcoded
vec![
    "http://127.0.0.1:3000".to_string(),
    "http://127.0.0.1:8080".to_string(),
    "http://localhost:3000".to_string(),
]
```

**Evolution Strategy**:
1. Make constants functions that return environment-aware values
2. Remove "backwards compatibility" constants - update tests instead
3. CORS origins should be discovered or explicitly configured
4. Network defaults should be calculated, not hardcoded

#### 2. **Hardcoded Elimination File** (High Priority - Ironic)
**File**: `crates/songbird-config/src/canonical/hardcoded_elimination.rs`

**Status**: ⚠️ NOT YET REVIEWED

This file ironically contains hardcoded values while being named "hardcoded_elimination". It has 62 hardcoded instances.

**Priority**: P0 - Must be evolved to pure discovery

#### 3. **Discovery Core** (Medium Risk)
**File**: `crates/songbird-discovery/src/discovery/core.rs`

**Status**: ⚠️ Contains 14 hardcoded instances

Needs review to ensure no hardcoded service locations.

#### 4. **Network Federation** (Medium Risk)
**Files**: Various in `crates/songbird-network-federation/src/`

**Status**: ⚠️ Needs audit

Multiple files with hardcoded references.

## 🎯 Evolution Principles

### Sovereignty-First Design:

1. **Zero Primal Knowledge**
   - Each primal only knows itself
   - All other primals discovered at runtime
   - No hardcoded endpoints, ports, or IPs

2. **Environment-First Configuration**
   - Try environment variables first
   - Fall back to discovery (NOT hardcoded defaults)
   - Log warnings when discovery is needed

3. **Discovery > Calculation > Error**
   - First: Try discovery via RuntimeDiscoveryEngine
   - Second: Calculate from environment context
   - Third: Return error with helpful message (NO silent fallbacks)

4. **Test Isolation**
   - Hardcoding only in test code (#[cfg(test)])
   - Tests set their own environment
   - No production code should assume test values

### Evolution Pattern:

```rust
// ❌ OLD: Hardcoded fallback
pub fn get_endpoint() -> String {
    env::var("ENDPOINT").unwrap_or("http://localhost:8080".to_string())
}

// ⚠️ INTERMEDIATE: Environment-aware but still has fallback
pub fn get_endpoint() -> String {
    env::var("ENDPOINT").unwrap_or_else(|_| {
        if is_production() {
            "http://service:8080".to_string()
        } else {
            "http://localhost:8080".to_string()
        }
    })
}

// ✅ NEW: Discovery-first, error on failure
pub async fn get_endpoint() -> Result<String, SongbirdError> {
    // 1. Try explicit configuration
    if let Ok(endpoint) = env::var("ENDPOINT") {
        return Ok(endpoint);
    }

    // 2. Try discovery
    let discovery = RuntimeDiscoveryEngine::new();
    match discovery.discover_by_capability("target_capability").await {
        Ok(service) => Ok(service.endpoint),
        Err(e) => {
            tracing::warn!(
                "Failed to discover endpoint. Set ENDPOINT environment variable or ensure service is advertising via mDNS/registry: {}",
                e
            );
            Err(SongbirdError::discovery(format!(
                "No endpoint configured and discovery failed: {}",
                e
            )))
        }
    }
}
```

## 📋 Next Steps

### Immediate (This Session):

1. ✅ Fix MdnsDiscovery compilation
2. ✅ Run cargo fmt
3. ✅ Fix clippy errors
4. 🔄 **IN PROGRESS**: Review and evolve constants.rs
5. ⏳ **PENDING**: Evolve hardcoded_elimination.rs (ironic)
6. ⏳ **PENDING**: Audit discovery/core.rs
7. ⏳ **PENDING**: Document hardcoding-free patterns

### Short Term (This Week):

1. Complete TaskLifecycleManager (Week 1)
2. Activate integration tests
3. Reach <100 hardcoded instances in production
4. Update all tests to set their own environment

### Medium Term (This Month):

1. Zero hardcoded endpoints in production code
2. All discovery via RuntimeDiscoveryEngine
3. Comprehensive hardcoding audit tool
4. Documentation update with examples

## 🔍 Files Requiring Evolution

### Priority 0 (Immediate):
- [ ] `crates/songbird-config/src/canonical/constants.rs` - Remove const fallbacks
- [ ] `crates/songbird-config/src/canonical/hardcoded_elimination.rs` - Ironic name

### Priority 1 (High):
- [ ] `crates/songbird-discovery/src/discovery/core.rs` - 14 instances
- [ ] `crates/songbird-config/src/canonical/hardcoded_elimination.rs` - 62 instances
- [ ] Network federation files - Various instances

### Priority 2 (Medium):
- [ ] Test files that leak into production imports
- [ ] Example/demo files with hardcoded values
- [ ] Documentation with hardcoded examples

## 📊 Metrics

### Before Evolution (Dec 18, 2025):
- Production hardcoding: 356 instances
- Coverage: 19%
- Linting errors: 8
- Unsafe blocks: 7 (justified)

### Targets:
- Production hardcoding: 0 instances
- Coverage: 90%
- Linting errors: 0
- Unsafe blocks: 0 (or all with safe alternatives)

## 🎓 Reference Documents

- `SAFE_PATTERNS.md` - Best practices for zero hardcoding
- `COMPREHENSIVE_AUDIT_REPORT_DEC_18_2025.md` - Full audit results
- `docs/audits/` - Historical audit reports
- `specs/INDIVIDUAL_HUMAN_DIGNITY_SPECIFICATION.md` - Sovereignty principles

---

**Status**: In Progress  
**Next Review**: After completing constants.rs evolution  
**Last Updated**: December 18, 2025

