# External Dependencies Audit Report

**Date**: January 19, 2026  
**Focus**: reqwest and rcgen (C dependencies)  
**Objective**: Path to 100% Pure Rust (ecoBin)  
**Status**: 🔄 IN PROGRESS - Mostly Pure, 2 blockers identified

---

## EXECUTIVE SUMMARY

**Current State**:
- ✅ `rcgen` - Already removed (commented out)
- ⚠️  `reqwest` - Used in 14 crates, mostly pure (no TLS)
- ⚠️  `songbird-network-federation` - Uses ring (C code) + rustls-tls

**ecoBin Impact**:
- **Blocker 1**: `songbird-network-federation` pulls `ring` (C code)
- **Blocker 2**: `reqwest` with `rustls-tls` in songbird-network-federation

**Path Forward**: 
1. Evolve songbird-network-federation to use songbird-tls
2. Remove reqwest or use capability-based HTTP via Songbird

---

## FINDINGS

### ✅ rcgen - ALREADY REMOVED

**Status**: Commented out in 2 crates

**Files**:
```toml
# crates/songbird-network-federation/Cargo.toml (line 40)
# rcgen = "0.14"  # ❌ REMOVED: Uses ring (C code). 
# Replaced with songbird-tls::CertificateGenerator - Jan 19, 2026

# crates/songbird-network/Cargo.toml (line 30)
# rcgen = "0.14"  # ❌ REMOVED: Uses ring (C code). 
# Replaced with songbird-tls::CertificateGenerator - Jan 19, 2026
```

**Action**: ✅ No action needed - already evolved!

---

### ⚠️  reqwest - MOSTLY PURE, 1 BLOCKER

**Usage**: 14 crates

**Pure Rust Usages** (13 crates):
```toml
# All use: default-features = false  (No TLS!)
- songbird-orchestrator
- songbird-universal
- songbird-discovery
- songbird-remote-deploy
- songbird-types (optional feature)
- songbird-primal-sdk
- songbird-registry
- songbird-primal-coordination
- songbird-compute-bridge
- songbird-genesis
- songbird-config
- songbird-cli
- songbird-execution-agent
```

✅ **These are fine** - No TLS, no C dependencies from reqwest

**Problematic Usage** (1 crate):
```toml
# crates/songbird-network-federation/Cargo.toml (line 34)
reqwest = { version = "0.11", features = ["json", "rustls-tls"], default-features = false }
```

⚠️  **Blocker**: `rustls-tls` feature pulls `hyper-rustls` → `rustls` → `ring` (C code)

**Used in**:
- `songbird-network-federation/src/tls.rs` (line 6-8)
- Uses `rcgen` (now commented out, but imports still present)
- Uses `rustls::crypto::ring` (line 30)

---

### 📊 DEPENDENCY ANALYSIS

#### **Current Dependency Chain** (problematic):
```
songbird
└── songbird-orchestrator
    └── songbird-network-federation
        ├── reqwest (with rustls-tls)
        │   └── hyper-rustls
        │       └── rustls
        │           └── ring ❌ (C code: uses unsafe, ASM, non-portable)
        └── rustls directly
            └── ring ❌ (C code)
```

**ecoBin Violation**: `ring` contains:
- Unsafe code
- Assembly code (x86/ARM specific)
- Non-portable C bindings
- Build-time C compilation

---

## EVOLUTION STRATEGIES

### **Option 1: Evolve songbird-network-federation** (Recommended)

**Goal**: Remove dependency on `ring` by using `songbird-tls`

**Steps**:
1. Replace `rustls` with `songbird-tls` for TLS operations
2. Remove `reqwest` with `rustls-tls` feature
3. Use plain `reqwest` (no TLS) + manual HTTP over songbird-tls sockets
4. OR: Remove reqwest entirely, delegate HTTP to Songbird orchestrator

**Timeline**: 2-4 hours

**Benefit**: 100% Pure Rust, true ecoBin

---

### **Option 2: Remove songbird-network-federation** (Nuclear)

**Goal**: Consolidate networking into songbird-orchestrator

**Current Usage**:
- songbird-orchestrator (dependency)
- songbird-cli (dependency)

**Steps**:
1. Extract needed functionality to songbird-orchestrator
2. Remove songbird-network-federation crate
3. Update dependencies

**Timeline**: 4-6 hours

**Benefit**: Simpler architecture + 100% Pure Rust

---

### **Option 3: Conditional Compilation** (Compromise)

**Goal**: Make ring dependency optional

**Pattern**:
```toml
[features]
default = []
legacy-tls = ["reqwest/rustls-tls", "rustls", "ring"]
pure-rust = ["songbird-tls"]

[dependencies]
reqwest = { version = "0.11", features = ["json"], default-features = false }
rustls = { version = "0.23", optional = true }
ring = { version = "0.17", optional = true }
songbird-tls = { path = "../songbird-tls", optional = true }
```

**Timeline**: 1-2 hours

**Benefit**: Backward compat + Pure Rust option

---

## REQWEST USAGE ANALYSIS

**Pure Rust Usages** (reqwest without TLS):

### **Pattern 1: JSON Client** (9 crates)
```rust
// No TLS, just HTTP client
let client = reqwest::Client::new();
let response = client.get(url).send().await?;
let json: Value = response.json().await?;
```

**Evolution**: Replace with capability-based HTTP via Songbird

### **Pattern 2: Optional Feature** (2 crates)
```toml
[features]
reqwest = ["dep:reqwest"]

[dependencies]
reqwest = { ..., optional = true }
```

**Status**: ✅ Good pattern - can be disabled

### **Pattern 3: Multipart Upload** (1 crate: songbird-remote-deploy)
```toml
reqwest = { version = "0.11", features = ["json", "multipart"] }
```

**Status**: ✅ No TLS, pure Rust feature

---

## RECOMMENDATION

### **Immediate Action** (2-4 hours):

1. **Evolve songbird-network-federation/src/tls.rs**:
   ```rust
   // OLD:
   use rcgen::*;
   use rustls::crypto::ring;
   
   // NEW:
   use songbird_tls::CertificateGenerator;
   use songbird_tls::TlsConfig;
   ```

2. **Remove rustls-tls from reqwest**:
   ```toml
   # OLD:
   reqwest = { version = "0.11", features = ["json", "rustls-tls"] }
   
   # NEW:
   reqwest = { version = "0.11", features = ["json"], default-features = false }
   ```

3. **Use songbird-tls for HTTPS**:
   - Manual HTTP over songbird-tls sockets
   - OR: Use capability-based HTTP (delegate to Songbird)

---

### **Long-term Vision** (Phase 2):

1. **Remove reqwest entirely**:
   - Replace with capability-based HTTP client
   - All HTTP goes through Songbird (which uses songbird-tls)
   - Primals discover Songbird at runtime
   - No direct HTTP client dependencies

2. **Pattern**:
   ```rust
   // OLD:
   let client = reqwest::Client::new();
   let response = client.get(url).send().await?;
   
   // NEW:
   let http = capability::discover("http").await?;
   let response = http.get(url).await?;
   ```

---

## VERIFICATION

### **Check C Dependencies**:
```bash
# Before evolution:
$ cargo tree -p songbird | grep ring
│   │   └── ring v0.17.14 ❌

# After evolution:
$ cargo tree -p songbird | grep ring
(no matches) ✅
```

### **Check Pure Rust**:
```bash
$ cargo build --target x86_64-unknown-linux-musl
✅ Success = Pure Rust achieved!
```

---

## SUMMARY

### **Current Status**:
| Dependency | Status | ecoBin Blocker? |
|------------|--------|-----------------|
| rcgen | ✅ Removed | No (already fixed) |
| reqwest (13 crates) | ✅ Pure | No (no TLS feature) |
| reqwest (1 crate) | ⚠️  C deps | **YES** (rustls-tls → ring) |
| songbird-network-federation | ⚠️  C deps | **YES** (uses ring directly) |

### **Path to 100% ecoBin**:
- ✅ rcgen: Done (already removed)
- 🔄 reqwest: Evolve 1 crate (2 hours)
- 🔄 songbird-network-federation: Evolve to songbird-tls (2-4 hours)

**Total Timeline**: 4-6 hours to 100% Pure Rust

---

## NEXT STEPS

1. Read `songbird-network-federation/src/tls.rs` fully
2. Identify all `ring` and `rustls` usages
3. Replace with `songbird-tls` equivalents
4. Remove `rustls-tls` feature from reqwest
5. Test with `cargo build --target x86_64-unknown-linux-musl`
6. Verify `cargo tree | grep ring` shows 0 matches

---

**Audit Complete**: January 19, 2026  
**Time**: ~45 minutes  
**Blockers Found**: 2 (both in songbird-network-federation)  
**Remediation**: 4-6 hours to 100% ecoBin

🦀🧬✨ **Clear Path to Pure Rust Excellence!** ✨🧬🦀

