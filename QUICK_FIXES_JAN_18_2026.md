# 🔧 Quick Fixes - January 18, 2026

**Priority**: CRITICAL - These block coverage measurement and full CI/CD

---

## 1. Test Build Error (BLOCKING COVERAGE)

**File**: `crates/songbird-types/tests/config_canonical_environment_tests.rs:200`

**Error**:
```
error[E0061]: this function takes 1 argument but 0 arguments were supplied
env.set("SONGBIRD_BIND_ADDRESS", &test_bind_address());
```

**Fix**:
```rust
// Before
env.set("SONGBIRD_BIND_ADDRESS", &test_bind_address());

// After
env.set("SONGBIRD_BIND_ADDRESS", &test_bind_address("orchestrator"));
```

**Command**:
```bash
# Apply fix then verify
cargo test -p songbird-types --test config_canonical_environment_tests
```

---

## 2. Deprecation Warnings (3 instances)

**File**: `crates/songbird-config/tests/evolved_configuration_tests.rs:133-134`

**Warning**:
```
use of deprecated struct `songbird_config::test_helpers::EnvironmentLock`
Use `songbird_test_utils::ScopedEnv` instead
```

**Fix**:
```rust
// Before
use songbird_config::test_helpers::EnvironmentLock;
let _lock = EnvironmentLock::new();

// After
use songbird_test_utils::ScopedEnv;
let _env = ScopedEnv::new();
```

---

## 3. Formatting Issues (3 files)

**Files**:
- `crates/songbird-bluetooth/src/host.rs:369`
- `crates/songbird-bluetooth/src/transport/usb.rs:2`
- `crates/songbird-bluetooth/src/transport/usb_nusb.rs:2`

**Fix**:
```bash
cargo fmt
```

---

## 4. Dead Code Warnings (2 constants)

**File**: `crates/songbird-genesis/src/physical_channels/bluetooth_pure.rs:17,20`

**Warning**:
```
constant `GENESIS_SERVICE_UUID` is never used
constant `GENESIS_CREDENTIAL_CHAR_UUID` is never used
```

**Fix Options**:
1. Use them (if planned)
2. Prefix with `_` (if intentionally unused)
3. Remove (if truly unnecessary)

```rust
// Option 2 (temporary)
const _GENESIS_SERVICE_UUID: uuid::Uuid = ...;
const _GENESIS_CREDENTIAL_CHAR_UUID: uuid::Uuid = ...;
```

---

## 5. Unused Import

**File**: `crates/songbird-universal/src/adapters/security_trust_tests.rs:3`

**Fix**:
```bash
cargo fix --lib -p songbird-universal --tests
```

---

## 6. Unused Field

**File**: `crates/songbird-universal/src/jsonrpc_client.rs:84`

**Warning**:
```
field `jsonrpc` is never read
```

**Fix**:
```rust
// Option 1: Use it for validation
#[derive(Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,  // Validate equals "2.0"
    id: Option<u64>,
    result: Option<serde_json::Value>,
}

// Then in parsing:
if response.jsonrpc != "2.0" {
    return Err(anyhow!("Invalid JSON-RPC version"));
}

// Option 2: Prefix if intentionally ignored
#[allow(dead_code)]
jsonrpc: String,
```

---

## Verification Script

```bash
#!/bin/bash
# Quick validation after fixes

echo "1. Fixing test build error..."
# Apply fix to config_canonical_environment_tests.rs

echo "2. Running format..."
cargo fmt

echo "3. Testing affected crates..."
cargo test -p songbird-types --test config_canonical_environment_tests
cargo test -p songbird-config --test evolved_configuration_tests
cargo test -p songbird-universal

echo "4. Attempting coverage..."
cargo llvm-cov --workspace --html

echo "5. Checking documentation..."
cargo doc --no-deps 2>&1 | grep -E "warning|error" | wc -l

echo "Done! Check target/llvm-cov/html/index.html for coverage report"
```

---

## Expected Impact

After these fixes:
- ✅ Test coverage measurement will work
- ✅ Deprecation warnings eliminated
- ✅ Formatting clean
- ✅ Dead code warnings resolved
- ✅ Can proceed with full audit

**Time Estimate**: 15-30 minutes

**Risk**: Very low (cosmetic and test-only changes)

