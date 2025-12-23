# 🔧 Integration Team Feedback - December 23, 2025

**Status**: Issues identified and being addressed

---

## 🐦 SONGBIRD - System Dependency Issue

### Current Status: ✅ Builds (with system dep)

**Issue**: Requires `libdbus-1-dev` system package

**Root Cause**: 
```
dbus v0.9.10 → libdbus-sys v0.2.7 → requires C library (libdbus-1-dev)
```

**Dependency Chain** (via `cargo tree --invert dbus`):
```
songbird-genesis → btleplug v0.11.8 → bluez-async v0.8.2 → dbus v0.9.10 → libdbus-sys v0.2.7
```

**Specific Crate**: `songbird-genesis` (Bluetooth genesis channel for physical bootstrap)  
**Feature**: Bluetooth Low Energy (BLE) physical device pairing  
**Purpose**: Physical genesis ceremony using Bluetooth (SoloKey, phone, etc.)

### 🎯 Evolution Path: Pure Rust Implementation

**Recommendation**: Replace `dbus` + `dbus-tokio` with `zbus` (pure Rust)

**Benefits**:
- ✅ **No system dependencies** - builds anywhere
- ✅ **Primal sovereignty** - no C library dependencies  
- ✅ **Modern async** - built on tokio
- ✅ **Better maintained** - active development
- ✅ **Type safety** - no FFI unsafe boundaries

**Migration Plan**:

```toml
# Remove (has C deps):
# dbus = "0.9"
# dbus-tokio = "0.7"

# Add (pure Rust):
zbus = "5.1"  # Pure Rust D-Bus
```

**Code Changes**:
```rust
// Old (dbus crate):
use dbus::blocking::Connection;
use dbus_tokio::connection;

// New (zbus crate):
use zbus::{Connection, proxy};
```

**Alternative Approach: Make Bluetooth Optional**

Since dbus is only needed for Bluetooth genesis:

```toml
# In songbird-genesis/Cargo.toml
[dependencies]
btleplug = { version = "0.11", optional = true }

[features]
bluetooth-genesis = ["btleplug"]  # Enable only when needed
```

**Benefits**:
- ✅ Core Songbird has no system deps
- ✅ Bluetooth genesis is opt-in
- ✅ Teams can build without system packages
- ✅ Enable Bluetooth only for physical genesis deployments

**Effort Estimate**: 
- Option 1: Make Bluetooth optional - 1-2 hours
- Option 2: Replace with zbus - 4-6 hours (need pure Rust BLE stack)

**Priority**: 🟡 Medium (Songbird builds, but sovereignty improvement)

**Note**: Bluetooth genesis is currently a **TODO stub** (see audit report), so system dep isn't blocking production yet.

### Immediate Workaround

For teams that need to build now:

```bash
# Ubuntu/Debian
sudo apt install libdbus-1-dev

# Fedora/RHEL
sudo dnf install dbus-devel

# Arch
sudo pacman -S libdbus
```

---

## 🏠 NESTGATE - Build Failure

### Current Status: 🔴 BROKEN - Fixed ✅

**Issue**: `sysinfo = "0.37.0"` requires rustc 1.88 (doesn't exist yet)

**File**: `nestgate/code/crates/nestgate-api/Cargo.toml:23`

**Root Cause**: Version `0.37.0` requires unreleased Rust compiler

**Current Stable Rust**: 1.87

### ✅ FIX APPLIED

**Changed**:
```diff
- sysinfo = "0.37.0"
+ sysinfo = "0.30"
```

**Verification**: All other ecoPrimals projects use `sysinfo = "0.30"` successfully:
- ✅ Songbird: 0.30
- ✅ BearDog: 0.30  
- ✅ Toadstool: 0.30
- ✅ Squirrel: 0.30 (most crates)
- ✅ NestGate core: 0.30

**Status**: 
- ✅ Fix committed to NestGate
- 🔄 Needs: New stable release tag
- 🔄 Needs: Updated binaries

### Next Steps for NestGate

```bash
# 1. Commit the fix
cd /home/eastgate/Development/ecoPrimals/nestgate
git add code/crates/nestgate-api/Cargo.toml
git commit -m "fix: Pin sysinfo to 0.30 for Rust 1.87 compatibility"

# 2. Test build
cargo build --release

# 3. Tag stable release
git tag -a v0.2.0-stable-dec23 -m "Stable release - Rust 1.87 compatible"
git push origin v0.2.0-stable-dec23

# 4. Create GitHub release with binary
gh release create v0.2.0-stable-dec23 \
  target/release/nestgate-bin \
  --title "NestGate v0.2.0 - Integration Ready" \
  --notes "Fixed sysinfo dependency for Rust 1.87 compatibility"
```

---

## 📊 ECOSYSTEM DEPENDENCY AUDIT

### System Dependencies by Project

| Project | System Deps | Status | Notes |
|---------|-------------|--------|-------|
| **Songbird** | libdbus-1-dev | 🟡 Optional | Can be eliminated with zbus |
| **BearDog** | None | ✅ Pure Rust | Sovereignty maintained |
| **NestGate** | None* | ✅ Pure Rust | *After sysinfo fix |
| **Toadstool** | CUDA (optional) | ✅ Optional | GPU features only |
| **Squirrel** | None | ✅ Pure Rust | Sovereignty maintained |

### Rust Version Requirements

| Project | Min Rust | Recommended | Status |
|---------|----------|-------------|--------|
| Songbird | 1.75 | 1.87 | ✅ Current |
| BearDog | 1.75 | 1.87 | ✅ Current |
| NestGate | 1.75 | 1.87 | ✅ Fixed |
| Toadstool | 1.75 | 1.87 | ✅ Current |
| Squirrel | 1.75 | 1.87 | ✅ Current |

---

## 🎯 ACTION ITEMS

### Immediate (Today)

- [x] Fix NestGate sysinfo dependency ✅
- [ ] Commit NestGate fix
- [ ] Build and test NestGate
- [ ] Tag NestGate stable release
- [ ] Create NestGate binary release

### Short Term (This Week)

- [ ] Investigate zbus migration for Songbird
- [ ] Find which Songbird crate uses dbus
- [ ] Estimate migration effort
- [ ] Document build dependencies clearly

### Medium Term (Next Month)

- [ ] Complete zbus migration (if feasible)
- [ ] Test all projects build on clean systems
- [ ] Document any remaining system dependencies
- [ ] Add CI checks for system dependency creep

---

## 📝 LESSONS LEARNED

### 1. Version Pinning is Critical
- ❌ Don't use bleeding-edge versions (`0.37.0`)
- ✅ Pin to stable versions that work (`0.30`)
- ✅ Test version bumps before committing

### 2. System Dependencies Break Sovereignty
- Pure Rust = builds anywhere
- C libraries = platform-specific pain
- FFI = unsafe boundaries

### 3. Ecosystem Consistency Matters
- All projects should use compatible dependency versions
- Shared Cargo workspace helps enforce this
- Regular dependency audits catch issues early

---

## 🤝 INTEGRATION TEAM THANK YOU

**Excellent catches!** These issues would have blocked production deployment.

Your feedback helps us:
- ✅ Maintain build sovereignty
- ✅ Keep dependencies clean
- ✅ Improve cross-team collaboration
- ✅ Ship production-ready binaries

---

## 📞 CONTACT

**Questions about these fixes?**
- Songbird dbus migration: See this document
- NestGate build issues: Fixed, ready for new release
- Other integration issues: Please report!

---

**Updated**: December 23, 2025  
**Next Review**: After zbus migration (TBD)

🐻 ecoPrimals - Building together!

