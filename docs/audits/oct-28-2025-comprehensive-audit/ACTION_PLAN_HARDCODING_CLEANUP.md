# 🔧 HARDCODING CLEANUP ACTION PLAN
**Current**: 266 production instances  
**Target**: <50 instances  
**Timeline**: 2-3 weeks  
**Priority**: P1 - HIGH PRIORITY

---

## 🎯 OBJECTIVE

Centralize all hardcoded values (IPs, ports, hosts, primal names) to configuration system, reducing production hardcoding from 266 to <50 instances.

**Impact**: +1-2 points to overall grade, improved configurability

---

## 📊 CURRENT STATE

### Hardcoding Breakdown
- **Total instances**: 1,016 (IPs, ports, hosts)
- **Test files**: ~750 instances (73%) - ✅ Acceptable
- **Production**: ~266 instances (27%) - ⚠️ Need fixing

### Primal Name Hardcoding
- **Total**: 225 instances (BearDog, ToadStool, NestGate, Squirrel)
- **Files**: 43 files
- **Issue**: Need capability-based routing instead

### Current Config Infrastructure ✅
```
crates/songbird-config/src/
├── defaults/
│   ├── hosts.rs      ✅ (exists, needs expansion)
│   ├── ports.rs      ✅ (exists, needs expansion)
│   ├── endpoints.rs  ✅ (exists, good)
│   └── timeouts.rs   ✅ (exists, good)
├── config/
│   ├── constants.rs  ✅ (exists)
│   └── network.rs    ✅ (exists)
└── environment.rs    ✅ (exists)
```

---

## 📋 WEEK 1: IP & HOST CENTRALIZATION

**Goal**: Move all production IPs/hosts to config  
**Target**: 850+ instances → centralized

### Day 1-2: Audit & Categorize
**Action**: Identify all hardcoded IPs/hosts in production code
```bash
# Generate list of production hardcoding
grep -r "127\.0\.0\.1\|localhost" crates/*/src/ \
  --include="*.rs" \
  ! -path "*/tests/*" \
  ! -name "*test*.rs" \
  > hardcoded_ips_production.txt

# Categorize by usage
# - Service endpoints
# - Database connections
# - Discovery targets
# - Health check endpoints
# - Load balancer targets
```

**Deliverables**:
- Complete list of production hardcoded IPs
- Categorization by usage type
- Priority ranking

### Day 3-4: Expand Config Infrastructure
**Action**: Enhance `crates/songbird-config/src/defaults/hosts.rs`

**Add These Constants**:
```rust
// In crates/songbird-config/src/defaults/hosts.rs

/// Service hosts
pub const DEFAULT_ORCHESTRATOR_HOST: &str = "localhost";
pub const DEFAULT_REGISTRY_HOST: &str = "localhost";
pub const DEFAULT_DISCOVERY_HOST: &str = "localhost";
pub const DEFAULT_FEDERATION_HOST: &str = "localhost";

/// Database hosts
pub const DEFAULT_POSTGRES_HOST: &str = "localhost";
pub const DEFAULT_MYSQL_HOST: &str = "localhost";
pub const DEFAULT_REDIS_HOST: &str = "localhost";
pub const DEFAULT_SQLITE_PATH: &str = "./data/songbird.db";

/// Load balancer targets
pub const DEFAULT_BACKEND_HOST: &str = "localhost";
pub const DEFAULT_BACKUP_HOST: &str = "localhost";

/// Health check endpoints
pub const DEFAULT_HEALTH_CHECK_HOST: &str = "localhost";

/// Environment-aware host resolution
pub fn get_host_for_environment(env: &str) -> &str {
    match env {
        "production" => std::env::var("SONGBIRD_HOST")
            .as_deref()
            .unwrap_or("0.0.0.0"),
        "development" => "localhost",
        "testing" => "127.0.0.1",
        _ => "localhost",
    }
}
```

**Add Environment Config**:
```rust
// In crates/songbird-config/src/environment.rs

use std::env;

pub struct EnvironmentConfig {
    pub orchestrator_host: String,
    pub registry_host: String,
    pub database_host: String,
    pub enable_tls: bool,
}

impl EnvironmentConfig {
    pub fn from_env() -> Self {
        Self {
            orchestrator_host: env::var("SONGBIRD_ORCHESTRATOR_HOST")
                .unwrap_or_else(|_| "localhost".to_string()),
            registry_host: env::var("SONGBIRD_REGISTRY_HOST")
                .unwrap_or_else(|_| "localhost".to_string()),
            database_host: env::var("SONGBIRD_DATABASE_HOST")
                .unwrap_or_else(|_| "localhost".to_string()),
            enable_tls: env::var("SONGBIRD_ENABLE_TLS")
                .map(|v| v == "true")
                .unwrap_or(false),
        }
    }
}
```

**Deliverables**:
- Enhanced host configuration
- Environment-aware resolution
- Documentation for all constants

### Day 5: Replace Production Hardcoding (Batch 1)
**Action**: Replace first 100 hardcoded instances

**Pattern**:
```rust
// BEFORE (hardcoded)
let host = "127.0.0.1";
let endpoint = format!("http://{}:8080", host);

// AFTER (config-driven)
use songbird_config::defaults::hosts;
let host = hosts::get_host_for_environment(&config.environment);
let endpoint = format!("http://{}:8080", host);
```

**Files to Update** (priority order):
1. `crates/songbird-orchestrator/src/app/mod.rs`
2. `crates/songbird-registry/src/production/persistent_registry.rs`
3. `crates/songbird-universal/src/unified_adapter.rs`
4. `crates/songbird-discovery/src/production/real_service_discovery.rs`
5. `crates/songbird-network-federation/src/network/mod.rs`

**Deliverables**:
- 100 instances migrated
- All affected tests passing
- No regressions

### Week 1 Checkpoint
- [ ] All production IPs audited
- [ ] Config infrastructure enhanced
- [ ] First 100 instances migrated
- [ ] Tests passing
- **Progress**: 266 → 166 instances

---

## 📋 WEEK 2: PORT & PRIMAL NAME CENTRALIZATION

**Goal**: Centralize ports and eliminate primal name hardcoding  
**Target**: 166 → 50 instances

### Day 6-7: Port Centralization
**Action**: Expand `crates/songbird-config/src/defaults/ports.rs`

**Add Missing Ports**:
```rust
// In crates/songbird-config/src/defaults/ports.rs

// Current ports (already defined - verify)
pub const ORCHESTRATOR_PORT: u16 = 8080;
pub const REGISTRY_PORT: u16 = 8081;
pub const DISCOVERY_PORT: u16 = 8082;

// Add these:
pub const FEDERATION_PORT: u16 = 8083;
pub const HEALTH_CHECK_PORT: u16 = 8090;
pub const METRICS_PORT: u16 = 9090;

// Database ports
pub const POSTGRES_PORT: u16 = 5432;
pub const MYSQL_PORT: u16 = 3306;
pub const REDIS_PORT: u16 = 6379;
pub const MONGODB_PORT: u16 = 27017;

// Service mesh ports
pub const ELASTICSEARCH_PORT: u16 = 9200;
pub const CONSUL_PORT: u16 = 8500;
pub const ETCD_PORT: u16 = 2379;

/// Get port for service from environment
pub fn get_service_port(service: &str) -> u16 {
    std::env::var(format!("SONGBIRD_{}_PORT", service.to_uppercase()))
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| default_port_for_service(service))
}

fn default_port_for_service(service: &str) -> u16 {
    match service {
        "orchestrator" => ORCHESTRATOR_PORT,
        "registry" => REGISTRY_PORT,
        "discovery" => DISCOVERY_PORT,
        "federation" => FEDERATION_PORT,
        _ => 8080,
    }
}
```

**Replace Hardcoded Ports**:
```rust
// BEFORE
let port = 8080;

// AFTER
use songbird_config::defaults::ports;
let port = ports::get_service_port("orchestrator");
```

**Deliverables**:
- All ports centralized
- Environment variable support
- 50 port instances migrated

### Day 8-9: Primal Name Elimination (Phase 1)
**Action**: Replace primal names with capability-based discovery

**Current Problem**:
```rust
// HARDCODED - BAD
if primal_name == "BearDog" {
    // Use BearDog for security
}

// CAPABILITY-BASED - GOOD
if primal.has_capability("authentication") {
    // Use any primal that provides authentication
}
```

**Files to Update** (225 instances across 43 files):
```
Priority 1 - Adapters (direct references):
- crates/songbird-universal/src/adapters/*.rs (11 files)
- crates/songbird-config/src/config/constants.rs
- crates/songbird-config/src/endpoints.rs

Priority 2 - Test utilities (acceptable, but improve):
- crates/songbird-test-utils/src/mocks/*.rs (5 files)
- crates/songbird-test-utils/src/fixtures/*.rs (2 files)

Priority 3 - Configuration files:
- crates/songbird-config/src/config/universal_primals.rs
- crates/songbird-config/src/environment_config_clean.rs
- crates/songbird-config/src/defaults/ports.rs
```

**Migration Pattern**:
```rust
// BEFORE - Hardcoded primal name
let beardog_endpoint = format!("http://beardog:8080");

// AFTER - Capability-based discovery
let security_providers = discovery
    .find_by_capability("authentication")
    .await?;
let endpoint = security_providers
    .first()
    .map(|p| &p.endpoint)
    .ok_or(SongbirdError::NoProviderFound)?;
```

**Deliverables**:
- 100 primal name references migrated to capability-based
- Adapter files fully capability-driven
- Tests updated

### Day 10: Final Production Cleanup
**Action**: Migrate remaining production hardcoding

**Targets**:
- Remaining IPs in network/federation code
- Any remaining ports in orchestrator
- Timeouts and retry values
- Buffer sizes and limits

**Pattern for Constants**:
```rust
// Create: crates/songbird-config/src/defaults/constants.rs
pub const DEFAULT_TIMEOUT_MS: u64 = 30_000;
pub const DEFAULT_RETRY_COUNT: u32 = 3;
pub const DEFAULT_BUFFER_SIZE: usize = 8192;
pub const DEFAULT_MAX_CONNECTIONS: usize = 100;

// Use with environment override
pub fn get_timeout_ms() -> u64 {
    std::env::var("SONGBIRD_TIMEOUT_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_TIMEOUT_MS)
}
```

**Deliverables**:
- All magic numbers extracted
- All remaining hardcoding centralized
- **Progress**: 166 → 50 instances ✅

### Week 2 Checkpoint
- [ ] All ports centralized
- [ ] 100 primal names migrated
- [ ] Magic numbers extracted
- [ ] Tests passing
- **Final Count**: <50 production instances ✅

---

## 📋 WEEK 3: PRIMAL NAME COMPLETION & VALIDATION

**Goal**: Complete primal name migration, validate everything  
**Target**: Full capability-based routing

### Day 11-12: Complete Primal Name Migration
**Action**: Migrate remaining 125 primal name instances

**Strategy**:
1. Update test utilities (acceptable but improve)
2. Update configuration references
3. Add capability mapping

**Create Capability Mapping**:
```rust
// In crates/songbird-config/src/capability_mapping.rs

use std::collections::HashMap;

/// Map legacy primal names to capabilities (for migration)
pub fn get_capabilities_for_primal(primal: &str) -> Vec<&'static str> {
    match primal.to_lowercase().as_str() {
        "beardog" => vec!["authentication", "encryption", "key-management"],
        "toadstool" => vec!["compute", "container-orchestration", "workload"],
        "nestgate" => vec!["storage", "backup", "caching"],
        "squirrel" => vec!["ai", "ml", "natural-language"],
        _ => vec![],
    }
}

/// Reverse mapping: capability to suggested providers
pub fn get_suggested_providers_for_capability(capability: &str) -> Vec<&'static str> {
    match capability {
        "authentication" | "encryption" => vec!["beardog", "any-security-provider"],
        "compute" | "container-orchestration" => vec!["toadstool", "any-compute-provider"],
        "storage" | "backup" => vec!["nestgate", "any-storage-provider"],
        "ai" | "ml" => vec!["squirrel", "any-ai-provider"],
        _ => vec!["any-provider"],
    }
}
```

**Migration Helper Script**:
```bash
#!/bin/bash
# migrate_primal_names.sh

echo "Migrating primal names to capability-based discovery..."

# Find all primal name references
for primal in "BearDog" "ToadStool" "NestGate" "Squirrel"; do
    echo "Finding $primal references..."
    grep -r "\"$primal\"" crates/*/src/ \
        --include="*.rs" \
        ! -path "*/tests/*" \
        ! -name "*test*.rs"
done

echo ""
echo "Manual migration required for context-specific replacements"
echo "Use capability_mapping.rs for guidance"
```

**Deliverables**:
- All primal names mapped to capabilities
- Migration helper tools created
- Remaining instances migrated

### Day 13-14: Validation & Testing
**Action**: Validate all changes, comprehensive testing

**Validation Checklist**:
```bash
# 1. Verify hardcoding reduction
echo "=== Hardcoding Audit ==="
echo "Production IPs/hosts:"
grep -r "127\.0\.0\.1\|localhost" crates/*/src/ \
  --include="*.rs" \
  ! -path "*/tests/*" \
  ! -name "*test*.rs" | wc -l

echo "Production primal names:"
grep -r "BearDog\|ToadStool\|NestGate\|Squirrel" crates/*/src/ \
  --include="*.rs" \
  ! -path "*/tests/*" \
  ! -name "*test*.rs" | wc -l

# 2. Run all tests
echo "=== Running Tests ==="
cargo test --workspace

# 3. Build check
echo "=== Build Check ==="
cargo build --workspace --release

# 4. Clippy check
echo "=== Clippy Check ==="
cargo clippy --workspace -- -D warnings
```

**Test Scenarios**:
1. **Environment Variable Override**
   ```bash
   export SONGBIRD_ORCHESTRATOR_HOST="production.example.com"
   export SONGBIRD_ORCHESTRATOR_PORT="9090"
   cargo run --bin songbird-orchestrator
   # Verify it uses overridden values
   ```

2. **Capability-Based Discovery**
   ```bash
   # Test finding providers by capability
   cargo test test_capability_discovery -- --nocapture
   ```

3. **Configuration Loading**
   ```bash
   # Test config loads from environment
   cargo test test_environment_config -- --nocapture
   ```

**Deliverables**:
- All tests passing ✅
- Validation script created
- Documentation updated

### Day 15: Documentation & Cleanup
**Action**: Document changes, create migration guide

**Create Migration Guide**:
```markdown
# Configuration Migration Guide

## For Operators

### Environment Variables
Set these to override defaults:
- `SONGBIRD_ORCHESTRATOR_HOST` (default: localhost)
- `SONGBIRD_ORCHESTRATOR_PORT` (default: 8080)
- `SONGBIRD_DATABASE_HOST` (default: localhost)
- `SONGBIRD_ENABLE_TLS` (default: false)

### Configuration Files
Use `.env` files or `config.toml`:
```toml
[network]
orchestrator_host = "prod.example.com"
orchestrator_port = 9090
enable_tls = true
```

## For Developers

### Using Configuration
```rust
use songbird_config::{defaults, environment};

// Get hosts
let host = defaults::hosts::get_host_for_environment(&env);

// Get ports
let port = defaults::ports::get_service_port("orchestrator");

// Environment config
let config = environment::EnvironmentConfig::from_env();
```

### Capability-Based Discovery
```rust
// DON'T: Hardcode primal names
let provider = find_provider("BearDog")?;

// DO: Use capability-based discovery
let providers = discovery.find_by_capability("authentication").await?;
let provider = providers.first().ok_or(NoProviderFound)?;
```
```

**Deliverables**:
- Migration guide written
- API documentation updated
- Examples updated

### Week 3 Checkpoint
- [ ] All primal names migrated (225 → 0 hardcoded)
- [ ] Validation complete
- [ ] Documentation updated
- [ ] Final count: <50 production instances ✅
- **COMPLETE**: Hardcoding cleanup done 🎉

---

## 📊 SUCCESS METRICS

### Before
- Hardcoded IPs/hosts: ~850 production instances
- Hardcoded primal names: 225 instances
- Configurability: Low
- Environment support: Minimal

### After
- Hardcoded IPs/hosts: <30 instances (production)
- Hardcoded primal names: 0 instances (all capability-based)
- Configurability: High ✅
- Environment support: Complete ✅

### Impact
- Configuration flexibility: 10x improvement
- Deployment environments: 1 → unlimited
- Primal provider swapping: Hard → Easy
- Grade improvement: +1-2 points

---

## 🛠️ TOOLS CREATED

1. **`scripts/audit_hardcoding.sh`** - Audit current hardcoding
2. **`scripts/migrate_primal_names.sh`** - Migration helper
3. **`crates/songbird-config/src/capability_mapping.rs`** - Capability mapping
4. **`docs/CONFIGURATION_MIGRATION_GUIDE.md`** - Migration guide
5. **Validation test suite** - Comprehensive testing

---

## 🚧 RISKS & MITIGATION

### Risk 1: Breaking Changes
**Mitigation**: 
- Extensive testing at each stage
- Keep backward compatibility where possible
- Use environment variables for override

### Risk 2: Test Failures
**Mitigation**:
- Update tests incrementally
- Fix immediately when found
- Add new tests for edge cases

### Risk 3: Performance Impact
**Mitigation**:
- Config loaded once at startup
- Cache resolved values
- Benchmark before/after

---

## ✅ COMPLETION CRITERIA

- [ ] Production hardcoding: <50 instances
- [ ] Primal name hardcoding: 0 instances
- [ ] All tests passing
- [ ] Documentation complete
- [ ] Migration guide written
- [ ] Validation script created
- [ ] Grade improvement: +1-2 points

---

**Plan Created**: October 28, 2025  
**Status**: Ready to execute  
**Priority**: P1 - HIGH PRIORITY  
**Timeline**: 2-3 weeks  
**Outcome**: Fully configurable system 🎉

