# ⚠️  CRITICAL ISSUE: reqwest Transitive Dependency Problem

**Date**: January 18, 2026  
**Issue**: rustls/ring still in dependency tree via reqwest  
**Impact**: BLOCKS ecoBin validation  
**Priority**: CRITICAL

---

## 🔍 PROBLEM DISCOVERED

After removing rustls from `songbird-orchestrator/Cargo.toml`, dependency check reveals:
```
cargo tree | grep rustls
│   │   ├── rustls v0.23.35
│   │   │   ├── aws-lc-rs v1.15.1
│   │   │   │   ├── aws-lc-sys v0.34.0  ❌ C CODE
│   │   │   ├── ring v0.17.14  ❌ C CODE
```

**Root Cause**: reqwest is being pulled in with default features (rustls-tls) by OTHER workspace crates!

---

## 📋 INVESTIGATION

### Our Change (Correct):
```toml
# crates/songbird-orchestrator/Cargo.toml
reqwest = { version = "0.11", features = ["json"], default-features = false }
```

### The Problem:
**OTHER crates in workspace are pulling reqwest with default features!**

Candidates:
- songbird-network-federation?
- songbird-discovery?
- songbird-registry?
- songbird-universal?

---

## 🎯 SOLUTION OPTIONS

### Option A: Remove reqwest from ALL crates ✅ (BEST for ecoBin)
**Rationale**: Songbird shouldn't be making HTTP requests at all!
- Songbird is the HTTP/TLS **SERVER** (not client)
- Inter-primal communication: Unix sockets (BTSP)
- No external HTTP needed

**Action**: Remove reqwest from entire workspace

### Option B: Patch all workspace reqwest deps
**Rationale**: Ensure all reqwest uses are `default-features = false`
**Problem**: Still defeats ecoBin purpose (why use HTTP client?)

### Option C: Feature-gate reqwest
**Rationale**: Optional HTTP client for specific use cases
**Problem**: What use case? Songbird shouldn't make HTTP requests!

---

## 🚀 RECOMMENDED: Option A (Remove reqwest)

### Justification:
1. **Songbird is TLS primal** - serves HTTP, doesn't consume it
2. **Unix sockets for IPC** - BTSP already implemented
3. **No external HTTP** - violates architecture (Concentrated Gap)
4. **ecoBin compliance** - can't have transitive C deps

### Implementation:
1. Find all workspace crates using reqwest
2. Verify they don't need it (should use Unix sockets)
3. Remove from all Cargo.toml files
4. Update any HTTP client code to use Unix sockets

---

## 🔍 NEXT STEPS

1. **Audit workspace for reqwest usage**:
   ```bash
   find crates -name "Cargo.toml" -exec grep -l "reqwest" {} \;
   ```

2. **Check actual usage**:
   ```bash
   grep -r "reqwest::" crates/
   ```

3. **Remove reqwest** from all crates or ensure `default-features = false`

4. **Verify**: `cargo tree | grep rustls` → NO MATCHES

---

**Issue**: reqwest transitive dependency  
**Status**: BLOCKING ecoBin  
**Solution**: Remove reqwest from workspace  
**Priority**: CRITICAL (must fix before ecoBin validation)

---

🦀 **Pure Rust requires ZERO HTTP client dependencies!** 🦀
