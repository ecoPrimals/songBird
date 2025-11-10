# Config Migration Guide
**Version**: 1.0  
**Date**: November 7, 2025  
**Status**: Active Migration Period (Until Q2 2026)

---

## 📋 **OVERVIEW**

Songbird config is consolidating from fragmented modules (`config/`, root-level) into a unified `canonical/` module structure. This guide helps you migrate your code.

---

## 🎯 **QUICK MIGRATION**

### Before (Deprecated)
```rust
use songbird_config::config::NetworkConfig;
use songbird_config::config::environment::EnvironmentConfig;
use songbird_config::config::constants::get_default_bind_address;
```

### After (Canonical)
```rust
use songbird_config::canonical::{
    CanonicalNetworkConfig,  // or NetworkConfig (type alias)
    EnvironmentConfig,
    get_canonical_bind_address,
};
```

---

## 📦 **MODULE MAPPING**

| Old Location | New Location | Status |
|--------------|--------------|--------|
| `config::NetworkConfig` | `canonical::CanonicalNetworkConfig` | ✅ Available |
| `config::environment::EnvironmentConfig` | `canonical::EnvironmentConfig` | ✅ Available |
| `config::constants::*` | `canonical::constants::*` | ✅ Available |
| `config::security::SecurityConfig` | `canonical::SecurityConfig` | ✅ Available |
| `config::universal_primals::*` | `canonical::primals::*` | 🟡 In Progress |
| `config::paths::*` | `canonical::paths::*` | 🟡 Planned |
| `config::providers::*` | `canonical::service::*` | 🟡 Planned |

---

## 🔄 **DETAILED MIGRATIONS**

### Network Configuration

#### Before
```rust
use songbird_config::config::network::NetworkConfig;

let config = NetworkConfig::from_env()?;
```

#### After
```rust
use songbird_config::canonical::CanonicalNetworkConfig;

let config = CanonicalNetworkConfig::from_env()?;

// Or use the type alias for brevity:
use songbird_config::canonical::NetworkConfig;
let config = NetworkConfig::from_env()?;
```

**Changes:**
- Struct renamed to `CanonicalNetworkConfig` (explicit)
- Type alias `NetworkConfig` available for backward compatibility
- All methods remain the same

---

### Environment Configuration

#### Before
```rust
use songbird_config::config::environment::EnvironmentConfig;

let env = EnvironmentConfig::detect()?;
```

#### After
```rust
use songbird_config::canonical::EnvironmentConfig;

let env = EnvironmentConfig::detect()?;
```

**Changes:**
- Module path changed from `config::environment` to `canonical`
- Struct name unchanged
- All methods remain the same

---

### Constants and Defaults

#### Before
```rust
use songbird_config::config::constants::{
    get_default_bind_address,
    get_default_port,
};
```

#### After
```rust
use songbird_config::canonical::constants::{
    get_canonical_bind_address,
    get_canonical_orchestrator_endpoint,
};

// Or use module-level imports:
use songbird_config::canonical::{
    get_canonical_bind_address,
    get_canonical_orchestrator_endpoint,
};
```

**Changes:**
- Functions prefixed with `canonical_` for clarity
- Grouped in `canonical::constants` module
- New functions added (orchestrator, discovery, gaming endpoints)

---

### Security Configuration

#### Before
```rust
use songbird_config::config::security::SecurityConfig;
```

#### After
```rust
use songbird_config::canonical::SecurityConfig;
```

**Changes:**
- Module path changed only
- Struct and methods unchanged

---

## 🛠️ **AUTOMATION TOOLS**

### Find and Replace Script

```bash
#!/bin/bash
# migrate-config-imports.sh

# Network Config
find . -name "*.rs" -type f -exec sed -i \
  's/songbird_config::config::NetworkConfig/songbird_config::canonical::CanonicalNetworkConfig/g' {} +

# Environment Config
find . -name "*.rs" -type f -exec sed -i \
  's/songbird_config::config::environment::EnvironmentConfig/songbird_config::canonical::EnvironmentConfig/g' {} +

# Constants
find . -name "*.rs" -type f -exec sed -i \
  's/songbird_config::config::constants::/songbird_config::canonical::constants::/g' {} +

# Run cargo fix for additional cleanup
cargo fix --allow-dirty --allow-staged
```

### Regex Patterns for IDEs

**VSCode/IntelliJ Find & Replace:**

1. Network Config:
   - Find: `use songbird_config::config::NetworkConfig;`
   - Replace: `use songbird_config::canonical::CanonicalNetworkConfig;`

2. Environment:
   - Find: `use songbird_config::config::environment::(\w+);`
   - Replace: `use songbird_config::canonical::$1;`

3. Module imports:
   - Find: `use songbird_config::config::(\w+)::`
   - Replace: `use songbird_config::canonical::$1::`

---

## ⚠️ **BREAKING CHANGES**

### None (Yet!)

The migration maintains **100% backward compatibility** through:
- Re-exports in `lib.rs`
- Type aliases (`NetworkConfig` = `CanonicalNetworkConfig`)
- Deprecated but functional old modules

### Future Breaking Changes (Q2 2026)

After 6-month deprecation period:
- `config::` module will be removed
- Only `canonical::` imports will work
- Compiler errors will guide remaining migrations

---

## 🧪 **TESTING YOUR MIGRATION**

### 1. Check for Deprecated Imports

```bash
# Find files using old imports
grep -r "use songbird_config::config::" crates/ src/

# Count deprecated imports
grep -r "use songbird_config::config::" . | wc -l
```

### 2. Build with Warnings

```bash
# Enable deprecation warnings
RUSTFLAGS="-W deprecated" cargo build

# Or more verbose:
cargo build 2>&1 | grep "deprecated"
```

### 3. Run Tests

```bash
# Ensure all tests pass after migration
cargo test --workspace

# Run specific config tests
cargo test -p songbird-config
```

---

## 📚 **COMMON PATTERNS**

### Pattern 1: Wildcard Imports

#### Before
```rust
use songbird_config::config::*;
```

#### After
```rust
use songbird_config::canonical::*;
```

---

### Pattern 2: Nested Modules

#### Before
```rust
use songbird_config::config::{
    NetworkConfig,
    environment::EnvironmentConfig,
    constants::get_default_port,
};
```

#### After
```rust
use songbird_config::canonical::{
    CanonicalNetworkConfig as NetworkConfig,
    EnvironmentConfig,
    get_canonical_orchestrator_endpoint,
};
```

---

### Pattern 3: Re-exports

#### Before
```rust
// In your lib.rs
pub use songbird_config::config::NetworkConfig;
```

#### After
```rust
// In your lib.rs
pub use songbird_config::canonical::CanonicalNetworkConfig as NetworkConfig;
```

---

## 🎓 **LEARNING RESOURCES**

- **Consolidation Roadmap**: `CONFIG_CONSOLIDATION_ROADMAP.md`
- **Canonical Module Docs**: `cargo doc --open -p songbird-config`
- **Architecture Spec**: `specs/ARCHITECTURAL_CONSOLIDATION_SPECIFICATION.md`

---

## ❓ **FAQ**

### Q: Why consolidate?
**A:** The codebase had 4+ `NetworkConfig` definitions across different modules, causing confusion and maintenance burden. `canonical/` provides a single source of truth.

### Q: Do I have to migrate immediately?
**A:** No. Old imports work until Q2 2026 (6-month grace period). However, migrating early prevents future breaking changes.

### Q: What about `zero_touch/` modules?
**A:** `zero_touch/` is deployment-specific and separate from `canonical/`. No migration needed.

### Q: Will my tests break?
**A:** No. All tests should pass. If they don't, it's likely a bug in the consolidation (please report).

### Q: Can I use both old and new imports?
**A:** Yes, during the transition period. But we recommend migrating to `canonical::` for consistency.

---

## 📞 **SUPPORT**

- **Issues**: File in GitHub with `config-migration` label
- **Questions**: Ask in #songbird-dev channel
- **Documentation**: Check inline docs with `cargo doc`

---

**Last Updated**: November 7, 2025  
**Migration Period**: Nov 2025 - May 2026  
**Removal Date**: Q2 2026
