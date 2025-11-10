# Discovery & Service Config Analysis - November 10, 2025

## 📊 ANALYSIS RESULTS

### 1. ServiceConfig - NOT DUPLICATES ✅ **KEEP BOTH**

**Location 1**: `crates/songbird-config/src/canonical/service.rs` (lines 10-22)
```rust
pub struct ServiceConfig {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub metadata: HashMap<String, String>,
    pub health_check: Option<HealthCheckConfig>,
}
```
**Purpose**: Service discovery/registration - network-level service info

**Location 2**: `crates/songbird-config/src/unified/core.rs` (lines 24-35)
```rust
pub struct ServiceConfig {
    pub name: String,
    pub version: String,
    pub instance_id: String,
    pub tags: Vec<String>,
}
```
**Purpose**: Service metadata - application-level service info

**Decision**: These serve **different purposes**. One is for network discovery (address/port), the other is for application metadata (version/instance_id). **NOT duplicates - keep both.**

---

### 2. RegistryConfig - SPECIALIZED ✅ **KEEP BOTH**

**Location 1**: `crates/songbird-registry/src/persistence/production_registry.rs` (lines 59-70)
```rust
pub struct RegistryConfig {
    pub service_ttl: Duration,
    pub health_check_interval: Duration,
    pub max_services: usize,
    pub enable_events: bool,
    pub persistence_type: PersistenceType,
}
```
**Purpose**: Production-grade service registry with persistence and events

**Location 2**: `crates/songbird-orchestrator/src/core/mod.rs` (lines 177-192)
```rust
pub struct RegistryConfig {
    pub discovery_interval: u64,
    pub service_timeout: u64,
    pub max_services: u32,
}
```
**Purpose**: Orchestrator-level simple registry configuration

**Decision**: **Specialized configurations** for different contexts. The production registry is much more sophisticated with persistence backends, event broadcasting, etc. The orchestrator version is simpler and fits its use case. **Keep both.**

---

### 3. LoadBalancerConfig - ⚠️ **TRUE DUPLICATES FOUND!**

#### ✅ **Comprehensive Version** (KEEP):
**Location**: `crates/songbird-config/src/unified/robustness.rs` (lines 148-168)
```rust
pub struct LoadBalancerConfig {
    pub algorithm: LoadBalancingAlgorithm,
    pub health_check: HealthCheckConfig,
    pub sticky_sessions: bool,
    pub session_timeout: Duration,
    pub max_connections_per_backend: usize,
    pub connection_timeout: Duration,
    pub fail_fast: bool,
}
```
**Purpose**: Comprehensive load balancing with advanced features (sticky sessions, connection pooling, fail-fast)

#### ❌ **Duplicate 1** (CONSOLIDATE):
**Location**: `crates/songbird-orchestrator/src/core/mod.rs` (lines 137-152)
```rust
pub struct LoadBalancingConfig {  // Note: Different name but same concept
    pub strategy: LoadBalancingStrategy,
    pub health_check_interval: u64,
    pub max_retries: u32,
}
```

#### ❌ **Duplicate 2** (CONSOLIDATE):
**Location**: `crates/songbird-orchestrator/src/core/load_balancer/types.rs` (lines 50-63)
```rust
pub struct LoadBalancerConfig {
    pub strategy: LoadBalancerStrategy,
    pub health_check_interval_secs: u64,
    pub max_retries: u32,
    pub timeout_seconds: u64,
}
```

**Decision**: Duplicates 1 and 2 are nearly identical simple configs. **Consolidate** these two into the comprehensive version from unified/robustness.rs.

---

### 4. CapabilityConfig - NOT A DUPLICATE ✅ **KEEP**

**Location**: `crates/songbird-primal-sdk/src/traits/capabilities.rs` (lines 435-456)
```rust
pub struct CapabilityConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub enabled: bool,
    pub parameters: HashMap<String, String>,
    pub tags: Vec<String>,
}
```
**Purpose**: Universal Primals capability configuration

**Decision**: Only **ONE instance** found. Not a duplicate. **Keep as-is.**

---

## 🎯 CONSOLIDATION PLAN

### Phase 1: LoadBalancingConfig in Orchestrator Module
1. Import `LoadBalancerConfig` from `songbird_config::unified::robustness`
2. Update `ConsolidatedOrchestratorConfig` to use the comprehensive version
3. Add migration comment explaining field mappings

### Phase 2: LoadBalancerConfig in Load Balancer Types
1. Replace simple `LoadBalancerConfig` with comprehensive version
2. Update references in load balancer implementation
3. Add migration guide for field mappings

---

## 📈 EXPECTED IMPACT

```
Configs to Consolidate: 2 (LoadBalancingConfig variants)
Configs to Keep:        5 (specialized/unique configs)
Lines to Remove:        ~40 lines
Build Impact:           Minimal (re-exports and imports)
Grade Improvement:      +0.01-0.02 points
```

---

## 🔍 FIELD MAPPINGS

### LoadBalancingConfig → LoadBalancerConfig

**From Orchestrator Module** (`LoadBalancingConfig`):
```
strategy                → algorithm
health_check_interval   → health_check.interval (HealthCheckConfig)
max_retries            → (handled at usage site or via retry config)
```

**From Load Balancer Types** (`LoadBalancerConfig`):
```
strategy                    → algorithm
health_check_interval_secs  → health_check.interval (HealthCheckConfig)
max_retries                 → (handled at usage site or via retry config)
timeout_seconds             → connection_timeout
```

**NEW fields from comprehensive version**:
- `sticky_sessions: bool` - default false
- `session_timeout: Duration` - default 300s
- `max_connections_per_backend: usize` - default 100
- `fail_fast: bool` - default false

---

## ✅ NEXT STEPS

1. Consolidate `LoadBalancingConfig` in `songbird-orchestrator/src/core/mod.rs`
2. Consolidate `LoadBalancerConfig` in `songbird-orchestrator/src/core/load_balancer/types.rs`
3. Verify build passes
4. Document migration in comments
5. Move to Phase 2 (Constants consolidation)

---

**Session**: November 10, 2025  
**Phase**: Discovery & Service Configs Analysis  
**Status**: Analysis complete, ready for consolidation  
**Confidence**: HIGH (clear duplicates identified)

