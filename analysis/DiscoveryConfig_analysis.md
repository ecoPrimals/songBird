# DiscoveryConfig Consolidation Analysis

**Date**: November 10, 2025  
**Analyst**: AI Pair Programming Session  
**Status**: Analysis Complete - Ready for Consolidation

---

## Executive Summary

**Total Instances Found**: 14 actual DiscoveryConfig structs (16 references, 2 are builder/docs)
**Consolidation Strategy**: Complex - Multiple architectural patterns
**Estimated Complexity**: HIGH (more complex than HealthCheckConfig/CircuitBreakerConfig)
**Canonical Source**: `crates/songbird-config/src/canonical/discovery.rs` (378 lines, well-structured)

### Key Challenge
Unlike HealthCheckConfig and CircuitBreakerConfig which had similar field sets, DiscoveryConfig instances show **3 distinct architectural patterns**:
1. **Mechanism-based** (enum-driven discovery methods)
2. **Flag-based** (enable_* boolean flags for discovery types)
3. **Nested structured** (service, capability, network as separate configs) ← **Canonical uses this**

---

## Canonical Definition (TARGET)

**Location**: `crates/songbird-config/src/canonical/discovery.rs`

The canonical version uses a modern, nested structure:

```rust
pub struct DiscoveryConfig {
    pub service_discovery: ServiceDiscoveryConfig,
    pub capability_discovery: CapabilityDiscoveryConfig,
    pub network_discovery: NetworkDiscoveryConfig,
    pub auto_discovery: bool,
    pub common_ports: Vec<u16>,
    pub scan_timeout_secs: u64,
}

pub struct ServiceDiscoveryConfig {
    pub enabled: bool,
    pub discovery_interval_secs: u64,
    pub max_concurrent_discoveries: usize,
    pub discovery_timeout_secs: u64,
}

pub struct CapabilityDiscoveryConfig {
    pub enabled: bool,
    pub cache_ttl_secs: u64,
    pub discovery_batch_size: usize,
    pub max_retry_attempts: usize,
}

pub struct NetworkDiscoveryConfig {
    pub enabled: bool,
    pub scan_local_network: bool,
    pub scan_ports: Vec<u16>,
    pub discovery_protocols: Vec<String>,
}
```

**Advantages**:
- Clear separation of concerns
- Env-var driven defaults
- Comprehensive test coverage
- Modern Rust patterns (derive PartialEq, Eq where applicable)
- Excellent documentation

---

## All DiscoveryConfig Instances

### 1. `crates/songbird-config/src/config/mod.rs` (Line 462)
**Pattern**: Mechanism-based with enums
```rust
pub struct DiscoveryConfig {
    pub mechanism: DiscoveryMechanism,        // Enum: Dns, Consul, etc.
    pub interval_seconds: u64,
    pub health_check: HealthCheckConfig,      // ✅ Uses consolidated HC
    pub registration: RegistrationConfig,
}
```
**Consolidation**: **HARD** - Different pattern (enum-based vs nested)
**Notes**: Uses already-consolidated HealthCheckConfig

---

### 2. `crates/songbird-network-federation/src/network/mod.rs` (Line 293)
**Pattern**: Methods-based (Vec of enums)
```rust
pub struct DiscoveryConfig {
    pub enabled: bool,
    pub methods: Vec<DiscoveryMethod>,        // Vec of enum variants
    pub interval: Duration,
    pub timeout: Duration,
}
```
**Consolidation**: **MEDIUM** - Can map to canonical with some effort
**Notes**: Network-specific, may be a specialized variant to keep

---

### 3. `crates/songbird-primal-sdk/src/adaptive_discovery.rs` (Line 766)
**Pattern**: Flag-based (many enable_* fields)
```rust
pub struct DiscoveryConfig {
    pub discovery_interval_secs: u64,
    pub health_check_interval_secs: u64,
    pub max_discovery_timeout_secs: u64,
    pub enable_network_discovery: bool,
    pub enable_registry_discovery: bool,
    pub enable_filesystem_discovery: bool,
    pub enable_community_discovery: bool,
    pub enable_environment_discovery: bool,
}
```
**Consolidation**: **EASY-MEDIUM** - Can map boolean flags to canonical nested configs
**Field Mappings**:
- `discovery_interval_secs` → `service_discovery.discovery_interval_secs`
- `enable_network_discovery` → `network_discovery.enabled`
- `enable_registry_discovery` → `service_discovery.enabled`

---

### 4. `crates/songbird-primal-sdk/src/discovery/universal_discovery/types.rs` (Line 10)
**Pattern**: Flag-based + network scanning
```rust
pub struct DiscoveryConfig {
    pub enable_auto_discovery: bool,
    pub discovery_interval: Duration,
    pub enable_network_scanning: bool,
    pub network_scan_ranges: Vec<String>,
    pub discovery_ports: Vec<u16>,
    pub enable_dns_discovery: bool,
    pub dns_discovery_domains: Vec<String>,
    pub enable_multicast_discovery: bool,
    // ... more fields
}
```
**Consolidation**: **MEDIUM** - Detailed network config maps to canonical NetworkDiscoveryConfig
**Field Mappings**:
- `enable_auto_discovery` → `auto_discovery`
- `discovery_ports` → `common_ports`
- `enable_network_scanning` → `network_discovery.scan_local_network`
- `dns_discovery_domains` → `network_discovery.discovery_protocols`

---

### 5. `crates/songbird-primal-sdk/src/discovery/types.rs` (Line 88)
**Pattern**: Flag-based + ecosystem awareness
```rust
pub struct DiscoveryConfig {
    pub enable_network_scan: bool,
    pub enable_service_registry: bool,
    pub enable_broadcast: bool,
    pub enable_federation: bool,
    pub enable_ecosystem_discovery: bool,        // ← Unique!
    pub discovery_timeout_secs: u64,
    pub max_concurrent_operations: usize,
    pub network_scan_port_ranges: Vec<(u16, u16)>,
}
```
**Consolidation**: **MEDIUM** - Most flags map to canonical
**Notes**: `enable_ecosystem_discovery` is unique feature (beardog, toadstool integration)
**Field Mappings**:
- `enable_service_registry` → `service_discovery.enabled`
- `enable_federation` → `service_discovery.enabled` (federation-aware)
- `max_concurrent_operations` → `service_discovery.max_concurrent_discoveries`

---

### 6. `crates/songbird-config/src/zero_touch/infant_config.rs` (Line 127)
**Pattern**: Methods-based + caching
```rust
pub struct DiscoveryConfig {
    pub methods: Vec<DiscoveryMethod>,
    pub timeout: Duration,
    pub refresh_interval: Duration,
    pub enable_cache: bool,
    pub cache_ttl: Duration,
}
```
**Consolidation**: **MEDIUM** - Cache features map to CapabilityDiscoveryConfig
**Field Mappings**:
- `methods` → Can derive from enabled flags in canonical
- `cache_ttl` → `capability_discovery.cache_ttl_secs`
- `refresh_interval` → `service_discovery.discovery_interval_secs`

---

### 7. `crates/songbird-discovery/src/traits/discovery.rs` (Line 252)
**Pattern**: Backend-based with retry logic
```rust
pub struct DiscoveryConfig {
    pub backend: DiscoveryBackend,              // Enum: Songbird, Consul, etc.
    pub health_check_interval: Duration,
    pub connection_timeout: Duration,
    pub retry_attempts: u32,
    pub retry_delay: Duration,
}
```
**Consolidation**: **MEDIUM-HARD** - Backend enum may need to stay
**Notes**: May be a specialized variant for discovery trait implementations
**Field Mappings**:
- `retry_attempts` → `capability_discovery.max_retry_attempts`
- `connection_timeout` → `network_discovery.timeout` (needs to be added to canonical)

---

### 8. `crates/songbird-universal/src/discovery.rs` (Line 46)
**Pattern**: Nested mechanisms struct
```rust
pub struct DiscoveryConfig {
    pub mechanisms: DiscoveryMechanisms,        // Nested struct
    pub timeout: Duration,
}

pub struct DiscoveryMechanisms {
    pub enable_environment_scan: bool,
    pub enable_network_scanning: bool,
    pub enable_container_discovery: bool,
}
```
**Consolidation**: **EASY** - Similar pattern to canonical!
**Field Mappings**:
- `mechanisms.enable_environment_scan` → `capability_discovery.enabled`
- `mechanisms.enable_network_scanning` → `network_discovery.enabled`
- `mechanisms.enable_container_discovery` → `service_discovery.enabled`

---

### 9. `crates/songbird-universal/src/capabilities/types.rs` (Line 56)
**Pattern**: Capability-focused with intervals
```rust
pub struct DiscoveryConfig {
    pub refresh_interval: Duration,
    pub discovery_timeout: Duration,
    pub max_concurrent_discoveries: usize,
    pub auto_discovery: bool,
    pub enable_network_discovery: bool,
}
```
**Consolidation**: **EASY** - Maps almost directly to canonical
**Field Mappings**:
- `refresh_interval` → `service_discovery.discovery_interval_secs`
- `max_concurrent_discoveries` → `service_discovery.max_concurrent_discoveries`
- `auto_discovery` → `auto_discovery`
- `enable_network_discovery` → `network_discovery.enabled`

---

### 10. `crates/songbird-universal/src/agnostic_service_discovery.rs` (Line 124)
**Pattern**: Network scanning + caching
```rust
pub struct DiscoveryConfig {
    pub discovery_timeout_ms: u64,              // ← Note: milliseconds
    pub enable_network_scanning: bool,
    pub scan_ranges: Vec<String>,
    pub probe_ports: Vec<u16>,
    pub enable_caching: bool,
    pub cache_expiry_seconds: u64,
}
```
**Consolidation**: **EASY** - Maps to canonical with unit conversion
**Field Mappings**:
- `discovery_timeout_ms` → `scan_timeout_secs` (convert ms→secs)
- `probe_ports` → `common_ports`
- `scan_ranges` → Could be part of `network_discovery` config
- `cache_expiry_seconds` → `capability_discovery.cache_ttl_secs`

---

### 11. `crates/songbird-universal/src/infant_discovery.rs` (Line 133)
**Pattern**: Network-focused + aggressive mode
```rust
pub struct DiscoveryConfig {
    pub network_ranges: Vec<String>,
    pub probe_ports: Vec<u16>,
    pub discovery_timeout: Duration,
    pub max_concurrent_discoveries: usize,
    pub aggressive_discovery: bool,             // ← Unique!
}
```
**Consolidation**: **EASY** - Mostly maps to NetworkDiscoveryConfig
**Notes**: `aggressive_discovery` could map to higher concurrency/shorter timeouts
**Field Mappings**:
- `probe_ports` → `common_ports`
- `max_concurrent_discoveries` → `service_discovery.max_concurrent_discoveries`
- `discovery_timeout` → `scan_timeout_secs`

---

### Non-Struct Matches (2)

#### 12. `crates/songbird-discovery/src/abstraction/modernized_factory.rs` (Line 261)
**Type**: `DiscoveryConfigBuilder` (not a config struct)
**Action**: No consolidation needed - builder pattern for factory

#### 13-14. Documentation files
**Type**: Analysis/report docs mentioning DiscoveryConfig
**Action**: No consolidation needed

---

## Consolidation Strategy

### Phase 1: Easy Wins (4-5 instances) - PRIORITY
Target instances that closely match canonical pattern:
1. ✅ `universal/capabilities/types.rs` - Almost direct mapping
2. ✅ `universal/discovery.rs` - Nested pattern similar to canonical
3. ✅ `universal/infant_discovery.rs` - Network-focused, clean mapping
4. ✅ `universal/agnostic_service_discovery.rs` - With unit conversions
5. ✅ `primal-sdk/adaptive_discovery.rs` - Flag-based, straightforward mapping

### Phase 2: Medium Complexity (4-5 instances)
Instances requiring field restructuring:
1. `primal-sdk/discovery/universal_discovery/types.rs` - Detailed network config
2. `primal-sdk/discovery/types.rs` - Ecosystem discovery feature
3. `config/zero_touch/infant_config.rs` - Methods → flags conversion
4. `network-federation/src/network/mod.rs` - May be specialized variant
5. `discovery/traits/discovery.rs` - Backend enum consideration

### Phase 3: Hard/Specialized (2 instances)
May need to keep as specialized variants:
1. `config/mod.rs` - Enum-based mechanism (different pattern)
2. `discovery/traits/discovery.rs` - Trait-specific backend config

### Phase 4: Enhancement
Add missing features to canonical if needed:
- `connection_timeout` for network operations
- `aggressive_discovery` mode (or map to higher concurrency)
- `ecosystem_discovery` integration points

---

## Field Mapping Reference

### Common Mappings to Canonical

| Original Field | Canonical Mapping | Notes |
|----------------|-------------------|-------|
| `discovery_interval_secs` | `service_discovery.discovery_interval_secs` | Direct |
| `discovery_interval` | Same, convert Duration→u64 | Unit conversion |
| `enable_network_discovery` | `network_discovery.enabled` | Direct |
| `enable_network_scanning` | `network_discovery.scan_local_network` | Semantic match |
| `enable_service_registry` | `service_discovery.enabled` | Service-level |
| `max_concurrent_discoveries` | `service_discovery.max_concurrent_discoveries` | Direct |
| `max_concurrent_operations` | Same | Alias |
| `discovery_timeout_secs` | `service_discovery.discovery_timeout_secs` | Direct |
| `discovery_timeout_ms` | Convert ms→secs | Unit conversion |
| `probe_ports` | `common_ports` | Network ports |
| `discovery_ports` | `common_ports` | Same |
| `cache_ttl` | `capability_discovery.cache_ttl_secs` | Caching |
| `cache_expiry_seconds` | Same | Direct |
| `enable_caching` | `capability_discovery.enabled` | Capability-level |
| `auto_discovery` | `auto_discovery` | Top-level |
| `enable_auto_discovery` | Same | Direct |
| `retry_attempts` | `capability_discovery.max_retry_attempts` | Retry logic |

### Fields Not in Canonical (Need Decision)

| Field | Location | Proposal |
|-------|----------|----------|
| `aggressive_discovery` | infant_discovery.rs | Map to higher concurrency |
| `enable_ecosystem_discovery` | discovery/types.rs | Add to canonical? |
| `connection_timeout` | traits/discovery.rs | Add to NetworkDiscoveryConfig |
| `mechanism: Enum` | config/mod.rs | Keep as specialized variant |
| `backend: Enum` | traits/discovery.rs | Keep as specialized variant |
| `methods: Vec<Enum>` | various | Derive from enabled flags |

---

## Recommendations

### 1. Start with Universal Crate (High Success Rate)
The `songbird-universal` crate has 4 instances that are all relatively easy to consolidate:
- `universal/capabilities/types.rs` ⭐ (EASIEST)
- `universal/discovery.rs` ⭐
- `universal/infant_discovery.rs` ⭐
- `universal/agnostic_service_discovery.rs` ⭐

**Rationale**: Quick wins, builds momentum, 4/14 = 29% progress in one go

### 2. Tackle Primal-SDK Next (3 instances)
All three are flag-based and map well:
- `primal-sdk/adaptive_discovery.rs`
- `primal-sdk/discovery/universal_discovery/types.rs`
- `primal-sdk/discovery/types.rs`

**Rationale**: After universal consolidation, these become easier to see patterns

### 3. Evaluate Specialized Variants
Before consolidating `config/mod.rs` and `traits/discovery.rs`, verify if they should remain as specialized variants:
- **`config/mod.rs`**: May be legacy config system
- **`traits/discovery.rs`**: May be trait-specific configuration
- **`network-federation/network/mod.rs`**: Network-specific, consider keeping

### 4. Enhance Canonical if Needed
If valuable features found (e.g., `ecosystem_discovery`, `connection_timeout`):
- Add to canonical version
- Document migration strategy
- Update defaults

### 5. Maintain Zero Breaking Changes
Use same pattern as Week 2 Day 1:
- `pub use` re-exports
- Document field mappings
- Keep local copies in foundational crates if needed

---

## Risk Assessment

### Low Risk (4-5 instances)
- Direct field mappings
- Similar patterns
- Clear canonical equivalents

### Medium Risk (4-5 instances)
- Field restructuring needed
- Unit conversions required
- Some semantic interpretation

### High Risk (2 instances)
- Different architectural patterns
- May be specialized variants
- Require architectural decision

### Mitigation Strategy
1. **Test after each consolidation** (maintain 100% success rate)
2. **Document all assumptions** in field mapping comments
3. **Preserve functionality** - if unsure, keep as specialized variant
4. **Incremental approach** - do 1-2 per commit, verify compilation

---

## Estimated Effort

| Phase | Instances | Time Estimate | Complexity |
|-------|-----------|---------------|------------|
| Phase 1: Easy | 4-5 | 2-3 hours | LOW |
| Phase 2: Medium | 4-5 | 3-4 hours | MEDIUM |
| Phase 3: Hard | 2 | 2-3 hours | HIGH |
| Phase 4: Enhancement | Canonical | 1-2 hours | MEDIUM |
| **Total** | **12-14** | **8-12 hours** | **MIXED** |

Compare to Week 2 Day 1:
- HealthCheckConfig: 9 instances, ~2 hours
- CircuitBreakerConfig: 11 instances, ~3 hours
- **DiscoveryConfig**: 14 instances, ~8-12 hours (more complex)

---

## Success Criteria

- [ ] 10+ instances consolidated (70%+ success rate)
- [ ] Zero breaking changes
- [ ] Workspace compiles cleanly
- [ ] All field mappings documented
- [ ] Specialized variants identified and documented
- [ ] Canonical enhanced if valuable features found
- [ ] Grade improves (98 → 99/100?)

---

## Next Steps

1. **Start with universal/capabilities/types.rs** (easiest)
2. **Continue with other universal crate instances** (3 more)
3. **Move to primal-sdk instances** (3 instances)
4. **Evaluate specialized variants** (2-3 instances)
5. **Document decisions and field mappings**
6. **Update progress tracker**
7. **Celebrate success!** 🎉

---

**Status**: Ready to execute Phase 1! 🚀

**Prepared by**: AI Pair Programming Session  
**Date**: November 10, 2025  
**Document Version**: 1.0

