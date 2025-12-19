# 🔥 IMMEDIATE ACTION ITEMS - December 19, 2025

**Priority:** P0/P1 Issues - Must fix before staging deployment  
**Timeline:** 1-2 weeks  
**Status:** Ready to execute

---

## ✅ CHECKLIST (P0 - BLOCKING)

### 1. Fix Test Compilation Errors ⏰ 4 hours
**Priority:** P0 - BLOCKING  
**Status:** ❌ NOT STARTED

**Issue:**
```rust
error[E0412]: cannot find type `TaskStatus` in this scope
error[E0433]: failed to resolve: use of undeclared type `TaskStatus`

Location: crates/songbird-orchestrator/src/orchestrator.rs
Errors: 8 compilation errors in lib tests
```

**Action:**
```bash
cd crates/songbird-orchestrator

# Add missing imports in src/orchestrator.rs tests module
# Add: use crate::task_lifecycle::TaskStatus;

# Test the fix
cargo test --lib

# Expected: All tests compile and pass
```

**Acceptance Criteria:**
- [ ] cargo test --lib passes without compilation errors
- [ ] All 491 tests still passing
- [ ] No new warnings introduced

---

### 2. Fix Formatting Violations ⏰ 5 minutes
**Priority:** P0 - BLOCKING  
**Status:** ❌ NOT STARTED

**Issue:**
```
Diff in crates/songbird-orchestrator/src/access_control/mod.rs:294
Diff in crates/songbird-orchestrator/src/access_control/tokens.rs:124, 181
```

**Action:**
```bash
# Run formatter
cargo fmt

# Verify
cargo fmt -- --check

# Commit
git add -A
git commit -m "chore: apply rustfmt formatting"
```

**Acceptance Criteria:**
- [ ] cargo fmt -- --check passes
- [ ] No formatting violations
- [ ] Changes committed

---

### 3. Fix Showcase Clippy Failures ⏰ 1 hour
**Priority:** P0 - BLOCKING (for -D warnings)  
**Status:** ❌ NOT STARTED

**Issue:**
```
showcase/05-albatross-multiplex/benchmark:
- Dead code warnings (unused struct fields)
- Unused imports
- Missing Cargo.toml metadata (description, license, repository)
```

**Action:**
```bash
cd showcase/05-albatross-multiplex/benchmark

# Option 1: Fix the issues
# Add #[allow(dead_code)] to test structs
# Remove unused imports
# Add metadata to Cargo.toml

# Option 2: Exclude from workspace clippy
# Add to root .cargo/config.toml or use --exclude

# Test
cargo clippy -- -D warnings

# Verify workspace
cd ../../..
cargo clippy --workspace -- -D warnings
```

**Acceptance Criteria:**
- [ ] cargo clippy --workspace -- -D warnings passes
- [ ] No dead code warnings
- [ ] Cargo.toml has required metadata

---

## ⚠️ CHECKLIST (P1 - CRITICAL)

### 4. Fix Production Unwraps ⏰ 2-3 days
**Priority:** P1 - CRITICAL  
**Status:** ❌ NOT STARTED

**Issue:**
5 production unwrap/expect calls that should use proper error handling

**Instances:**

1. **tarpc_server.rs:390-391**
```rust
// BAD
let serialized = serde_json::to_string(&info).unwrap();
let deserialized: ServiceInfo = serde_json::from_str(&serialized).unwrap();

// GOOD
let serialized = serde_json::to_string(&info)
    .map_err(|e| SongbirdError::serialization(format!("Failed to serialize: {}", e)))?;
let deserialized: ServiceInfo = serde_json::from_str(&serialized)
    .map_err(|e| SongbirdError::serialization(format!("Failed to deserialize: {}", e)))?;
```

2. **orchestrator.rs:546, 570**
```rust
// BAD
let task = orchestrator.get_task(task_id).await?.unwrap();

// GOOD
let task = orchestrator.get_task(task_id).await?
    .ok_or_else(|| SongbirdError::not_found(format!("Task {} not found", task_id)))?;
```

3. **orchestrator.rs:579**
```rust
// BAD
let mut rx = orchestrator.subscribe_events().expect("Events enabled");

// GOOD
let mut rx = orchestrator.subscribe_events()
    .map_err(|e| SongbirdError::internal(format!("Failed to subscribe to events: {}", e)))?;
```

**Action:**
```bash
# Find all production unwraps
./check_production_unwraps.sh > unwraps_to_fix.txt

# Review and fix each instance
# Replace .unwrap() with ?
# Replace .expect("msg") with .context("msg")?

# Test after each fix
cargo test --lib

# Final verification
./check_production_unwraps.sh
# Expected: 0 production unwraps (test unwraps OK)
```

**Acceptance Criteria:**
- [ ] 0 unwrap() calls in crates/*/src/**/*.rs (excluding tests)
- [ ] 0 expect() calls in crates/*/src/**/*.rs (excluding tests)
- [ ] All tests still passing
- [ ] Proper error types used (SongbirdError::*)

---

### 5. Fix Deprecation Warnings ⏰ 30 minutes
**Priority:** P1 - CRITICAL  
**Status:** ❌ NOT STARTED

**Issue:**
```rust
warning: use of deprecated constant `DEFAULT_HOST`
  --> Use network::default_host() function instead

Locations (5 instances):
- crates/songbird-discovery/src/discovery/backends/container_orchestration.rs:337
- crates/songbird-discovery/src/discovery/backends/container_orchestration.rs:667
- crates/songbird-discovery/src/discovery/backends/service_discovery.rs:234
- crates/songbird-discovery/src/discovery/backends/service_discovery.rs:240
- crates/songbird-discovery/src/discovery/backends/service_discovery.rs:502
```

**Action:**
```bash
# Find all uses of deprecated constant
grep -r "constants::network::DEFAULT_HOST" crates/*/src/

# Replace with function call
# OLD: constants::network::DEFAULT_HOST
# NEW: network::default_host()

# Test
cargo doc --no-deps 2>&1 | grep -i "deprecated"
# Expected: 0 deprecation warnings

cargo build --workspace
```

**Acceptance Criteria:**
- [ ] 0 deprecation warnings in cargo doc
- [ ] All uses of DEFAULT_HOST replaced
- [ ] Tests still passing

---

### 6. Complete Critical TODOs ⏰ 1 week
**Priority:** P1 - CRITICAL  
**Status:** ❌ NOT STARTED

**Critical TODOs (Security-related):**

1. **JWT Validation** (access_control/auth.rs:37)
```rust
// TODO: Decode and validate JWT
// Current: Returns mock token

// Action: Implement real JWT validation
// - Decode JWT
// - Verify signature
// - Check expiration
// - Validate claims
```

2. **Registry Implementation** (rpc/tarpc_server.rs:185, 200)
```rust
// TODO: Call actual registry implementation
// Current: Placeholder response

// Action: Connect to real service registry
// - Query capability providers
// - Get service health
// - Return actual data
```

3. **2FA Verification** (access_control/mod.rs:285)
```rust
// TODO: Verify 2FA/hardware key for infrastructure access
// Current: Checks role but not 2FA

// Action: Implement 2FA check
// - Verify token has 2FA claim
// - Check hardware key presence
// - Validate entropy level
```

4. **Real Uptime Tracking** (rpc/tarpc_server.rs:232)
```rust
// TODO: Real uptime tracking
// Current: Returns hardcoded 3600

// Action: Track actual uptime
// - Store start time
// - Calculate elapsed
// - Return real value
```

5. **Blacklist Implementation** (access_control/tokens.rs:214)
```rust
// TODO: Blacklist implementation tracking
// Current: Issue link placeholder

// Action: Implement token blacklist
// - Store revoked tokens
// - Check on validation
// - Clean up expired entries
```

**Action:**
```bash
# Audit all critical TODOs
grep -r "TODO.*JWT\|TODO.*2FA\|TODO.*hardware\|TODO.*registry" crates/*/src/

# Create GitHub issues for each
# Assign priority and timeline
# Implement in order of security impact

# Test each implementation
cargo test --workspace
```

**Acceptance Criteria:**
- [ ] JWT validation implemented and tested
- [ ] Registry calls using real implementation
- [ ] 2FA verification functional
- [ ] Uptime tracking accurate
- [ ] Token blacklist implemented
- [ ] All security TODOs resolved
- [ ] Tests added for new functionality

---

## 📊 PROGRESS TRACKING

### P0 Tasks (BLOCKING)
- [ ] 1. Fix test compilation (4 hours)
- [ ] 2. Fix formatting (5 minutes)
- [ ] 3. Fix showcase clippy (1 hour)

**Total P0 Time:** ~5 hours  
**Estimated Completion:** Day 1

### P1 Tasks (CRITICAL)
- [ ] 4. Fix production unwraps (2-3 days)
- [ ] 5. Fix deprecation warnings (30 minutes)
- [ ] 6. Complete critical TODOs (1 week)

**Total P1 Time:** 8-10 days  
**Estimated Completion:** Week 2

### Overall Timeline
- **Day 1:** Complete P0 tasks (5 hours)
- **Day 2-4:** Fix production unwraps and deprecations (3 days)
- **Day 5-12:** Complete critical TODOs (1 week)
- **Day 13:** Final testing and validation

**Target:** Week 2 completion → **STAGING DEPLOYMENT READY**

---

## 🧪 VALIDATION CHECKLIST

After completing all items, verify:

### Build System
- [ ] `cargo build --workspace` passes
- [ ] `cargo test --workspace` passes (491 tests)
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo fmt -- --check` passes
- [ ] `cargo doc --workspace --no-deps` passes (0 warnings)

### Code Quality
- [ ] 0 production unwrap() calls
- [ ] 0 production expect() calls
- [ ] 0 deprecation warnings
- [ ] 0 critical TODOs in security code
- [ ] All P0/P1 issues resolved

### Testing
- [ ] All 491 tests passing
- [ ] No new test failures
- [ ] Test coverage baseline established
- [ ] Integration tests functional

---

## 📋 COMMANDS FOR QUICK VERIFICATION

```bash
# Quick health check (run after each fix)
cd /home/eastgate/Development/ecoPrimals/songbird

# 1. Compilation
cargo build --workspace --all-features

# 2. Tests
cargo test --workspace --lib

# 3. Linting
cargo clippy --workspace -- -D warnings

# 4. Formatting
cargo fmt -- --check

# 5. Documentation
cargo doc --workspace --no-deps 2>&1 | grep -i "warning\|error"

# 6. Production unwraps
./check_production_unwraps.sh

# All clean? Ready for staging! ✅
```

---

## 🎯 SUCCESS CRITERIA

**Ready for Staging Deployment when:**
- ✅ All P0 tasks complete
- ✅ All P1 tasks complete
- ✅ All validation checks pass
- ✅ No compilation errors
- ✅ No P0/P1 technical debt
- ✅ Documentation updated

**Timeline:** 8-10 business days  
**Next Phase:** Test coverage expansion (Week 3-8)

---

**Created:** December 19, 2025  
**Owner:** Development Team  
**Review Date:** End of Week 2  
**Status:** 🟡 READY TO EXECUTE

