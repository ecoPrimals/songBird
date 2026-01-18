# Deprecation Schedule - Songbird

**Last Updated**: January 17, 2026  
**Status**: Active deprecation management  
**Philosophy**: Gradual, well-communicated migrations with clear timelines

---

## 🎯 Overview

This document tracks all deprecated features in Songbird, their removal timelines, and migration paths.

**Principles**:
1. **6-Month Minimum**: All deprecations have at least 6 months notice
2. **Clear Migration Paths**: Every deprecated feature has a documented replacement
3. **Gradual Removal**: Features are deprecated → warned → removed
4. **Backward Compatibility**: Old data remains readable during migration period

---

## 📋 Active Deprecations

### 1. Hardcoded Primal Type Aliases

**Status**: ⚠️ OVERDUE (Deadline passed Jan 1, 2026)  
**Removal**: v0.10.0 (Q1 2026)  
**Impact**: Low (no active usage found)

**Deprecated Types**:
- `NestGateConfig` → `AgnosticPrimalConfig::storage_primal()`
- `ToadstoolConfig` → `AgnosticPrimalConfig::compute_primal()`
- `ToadstoolEndpoint` → `PrimalEndpoint`
- `BearDogConfig` (in `core/biome/modules/types.rs`) → `AgnosticPrimalConfig::security_primal()`
- `SquirrelConfig` → `AgnosticPrimalConfig::ai_primal()`
- `storage_provider_configConfig` → `AgnosticPrimalConfig::storage_primal()`

**Reason**: Hardcoded primal names violate zero-hardcoding philosophy

**Migration**:
```rust
// ❌ OLD
let config = NestGateConfig { ... };

// ✅ NEW
let config = AgnosticPrimalConfig::storage_primal("storage-provider-1", endpoint);
```

**Action Required**: Remove in next commit (Jan 17, 2026)

---

### 2. Environment Variables

#### 2.1 `BEARDOG_URL`

**Status**: Deprecated  
**Removal**: Q2 2026 (v3.16.0)  
**Impact**: Low (2 files)

**Replacement**: `SECURITY_PROVIDER` or `SONGBIRD_SECURITY_PROVIDER`

**Files**:
- `crates/songbird-orchestrator/src/trust/escalation.rs`
- `crates/songbird-orchestrator/src/app/security_setup.rs`

**Migration**:
```bash
# ❌ OLD
export BEARDOG_URL=http://localhost:8443

# ✅ NEW
export SECURITY_PROVIDER=http://localhost:8443
# or
export SONGBIRD_SECURITY_PROVIDER=http://localhost:8443
```

**Action Required**: Update deprecation warnings with Q2 2026 date

---

#### 2.2 `SONGBIRD_BEARDOG_URL`

**Status**: Deprecated  
**Removal**: v3.16.0 (Q2 2026)  
**Impact**: Low (2 files)

**Replacement**: `SONGBIRD_SECURITY_PROVIDER`

**Files**:
- `crates/songbird-orchestrator/src/app/security_setup.rs`
- `crates/songbird-orchestrator/src/app/discovery_startup.rs`

**Migration**:
```bash
# ❌ OLD
export SONGBIRD_BEARDOG_URL=http://localhost:8443

# ✅ NEW
export SONGBIRD_SECURITY_PROVIDER=http://localhost:8443
```

**Action Required**: Update deprecation warnings with Q2 2026 date

---

#### 2.3 `BEARDOG_2FA_ENDPOINT`

**Status**: Deprecated  
**Removal**: Q2 2026 (v3.16.0)  
**Impact**: Low (1 file)

**Replacement**: Universal Adapter capability discovery

**Files**:
- `crates/songbird-orchestrator/src/access_control/auth.rs`

**Migration**:
```rust
// ❌ OLD
let endpoint = env::var("BEARDOG_2FA_ENDPOINT")?;

// ✅ NEW
let client = UniversalAdapter::discover_capability("2fa").await?;
```

**Action Required**: Update deprecation warning with Q2 2026 date

---

### 3. Legacy Configuration Helpers

**Status**: Deprecated  
**Removal**: v0.3.0 (Q2 2026)  
**Impact**: Medium (multiple files)

**Deprecated Functions** (in `crates/songbird-config/src/canonical/constants.rs`):
- `DEFAULT_HOST` constant → `network::default_host()` function
- `get_bind_address_legacy()` → `get_bind_address()` function

**Migration**:
```rust
// ❌ OLD
use songbird_config::canonical::constants::DEFAULT_HOST;

// ✅ NEW
use songbird_config::canonical::constants::network::default_host;
let host = default_host();
```

**Action Required**: Create migration guide

---

### 4. Zstd Checkpoint Compatibility

**Status**: Migration compatibility (not deprecated)  
**Removal**: Q3 2026 (after 6-month migration period)  
**Impact**: Low (backward compatibility only)

**Context**: Zstd was migrated to flate2 (Pure Rust) on Jan 17, 2026

**Current Behavior**:
- New checkpoints use Gzip (Pure Rust)
- Old Zstd checkpoints readable (treated as uncompressed)
- No new Zstd checkpoints created

**Migration Timeline**:
- **Jan 17, 2026**: Zstd → flate2 migration complete
- **Q2 2026**: All active checkpoints migrated to Gzip/Zlib
- **Q3 2026**: Remove Zstd compatibility code

**Files**:
- `crates/songbird-orchestrator/src/task_lifecycle/storage.rs` (lines 380-388)
- `crates/songbird-orchestrator/src/task_lifecycle/checkpoint.rs` (comment only)

**Action Required**: Create checkpoint migration utility

---

### 5. Legacy BearDog SDK Module

**Status**: Deprecated  
**Removal**: v0.3.0 (Q2 2026)  
**Impact**: Low (module-level deprecation)

**Deprecated Module**:
- `songbird-primal-sdk::beardog` → `songbird-primal-sdk::security_capability_client`

**Files**:
- `crates/songbird-primal-sdk/src/beardog.rs` (entire module deprecated)

**Migration**:
```rust
// ❌ OLD
use songbird_primal_sdk::beardog::BearDogConfig;
let beardog = BearDogConfig { endpoint: "...", ... };

// ✅ NEW
use songbird_primal_sdk::security_capability_client::SecurityCapabilityClient;
let security = SecurityCapabilityClient::new().await?;
```

**Action Required**: None (already fully deprecated at module level)

---

## 📅 Removal Timeline

### Q1 2026 (Immediate)

- [x] Remove hardcoded primal type aliases (deadline passed Jan 1, 2026)
- [ ] Update all deprecation warnings with specific dates
- [ ] Create migration guides for all deprecated features

### Q2 2026 (April-June)

- [ ] Remove `BEARDOG_URL` environment variable
- [ ] Remove `SONGBIRD_BEARDOG_URL` environment variable
- [ ] Remove `BEARDOG_2FA_ENDPOINT` environment variable
- [ ] Remove legacy configuration helpers
- [ ] Remove legacy BearDog SDK module
- [ ] Migrate all active Zstd checkpoints to Gzip/Zlib

### Q3 2026 (July-September)

- [ ] Remove Zstd checkpoint compatibility code
- [ ] Clean up migration compatibility shims

### Q4 2026 (October-December)

- [ ] Migrate `rustls` to `rustls-rustcrypto` (Pure Rust TLS)
- [ ] Migrate internal JWT to Pure Rust (remove `jsonwebtoken` dependency)
- [ ] **Achieve 100% ecoBin!** 🎉

---

## 🔧 Migration Tools

### Planned Utilities

1. **Checkpoint Migration Tool** (Q1 2026)
   - Scan for Zstd checkpoints
   - Convert to Gzip format
   - Verify integrity
   - Update database records

2. **Environment Variable Migration Script** (Q1 2026)
   - Scan configuration files
   - Update environment variable names
   - Generate migration report

3. **Configuration Migration Tool** (Q2 2026)
   - Update legacy configuration patterns
   - Convert to capability-based discovery
   - Validate new configuration

---

## 📊 Deprecation Status Dashboard

| Feature | Deprecated | Removal | Impact | Migration Guide | Status |
|---------|-----------|---------|--------|----------------|--------|
| Hardcoded Primal Types | v0.9.0 | Q1 2026 | Low | ✅ Yes | ⚠️ Overdue |
| `BEARDOG_URL` | v3.15.0 | Q2 2026 | Low | ✅ Yes | 🟡 Active |
| `SONGBIRD_BEARDOG_URL` | v3.15.0 | Q2 2026 | Low | ✅ Yes | 🟡 Active |
| `BEARDOG_2FA_ENDPOINT` | v3.15.0 | Q2 2026 | Low | ✅ Yes | 🟡 Active |
| Legacy Config Helpers | v0.2.0 | Q2 2026 | Medium | ⏳ Pending | 🟡 Active |
| Zstd Compatibility | N/A | Q3 2026 | Low | ✅ Yes | 🟢 Migration |
| Legacy BearDog SDK | v0.2.0 | Q2 2026 | Low | ✅ Yes | 🟡 Active |

**Legend**:
- ✅ Complete
- ⏳ Pending
- 🟢 On Track
- 🟡 Active
- ⚠️ Overdue

---

## 📝 Communication Plan

### User Notifications

1. **Deprecation Warnings**: All deprecated features emit runtime warnings
2. **Changelog**: All deprecations documented in CHANGELOG.md
3. **Migration Guides**: Step-by-step guides for each deprecated feature
4. **Release Notes**: Prominent deprecation notices in release notes

### Developer Notifications

1. **Compiler Warnings**: `#[deprecated]` attributes on all deprecated items
2. **Documentation**: Clear deprecation notices in API docs
3. **Code Comments**: Inline comments explaining replacements
4. **CI/CD**: Automated checks for deprecated feature usage

---

## 🎯 Success Criteria

**For Each Deprecation**:
- [ ] Replacement feature is production-ready
- [ ] Migration guide is published
- [ ] Deprecation warnings are in place
- [ ] Timeline is communicated (6+ months notice)
- [ ] Migration tools are available (if needed)
- [ ] No active usage found before removal

**For Overall Process**:
- [ ] Zero breaking changes without notice
- [ ] All migrations have clear paths
- [ ] User feedback is incorporated
- [ ] Technical debt is reduced

---

## 📚 References

- **Code Cleanup Report**: `docs/sessions/jan-2026/week4-day5/CODE_CLEANUP_JAN_17_2026.md`
- **Vendor Hardcoding Elimination**: `VENDOR_HARDCODING_ELIMINATION_REPORT.md`
- **Migration Guides**: `docs/migrations/`
- **Changelog**: `CHANGELOG.md`

---

**Last Review**: January 17, 2026  
**Next Review**: February 1, 2026  
**Owner**: Songbird Core Team

---

🦀✨ **GRADUAL EVOLUTION + CLEAR COMMUNICATION = MAINTAINABLE EXCELLENCE!** ✨🦀

