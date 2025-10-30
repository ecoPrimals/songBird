# Explicit Export Guidelines

## 🎯 Purpose
Replace wildcard exports (`pub use module::*`) with explicit exports to:
- Improve API clarity and documentation
- Prevent naming conflicts
- Make dependencies explicit
- Enable better IDE support

## ✅ Good Patterns

```rust
// ✅ Explicit exports - clear what's available
pub use config::{UnifiedSongbirdConfig, GlobalConfig, NetworkConfig};
pub use traits::{PrimalProvider, ServiceDiscovery, HealthCheck};
pub use errors::{SongbirdError, SongbirdResult, success};
```

## ❌ Bad Patterns

```rust
// ❌ Wildcard exports - unclear what's available
pub use config::*;
pub use traits::*;
pub use errors::*;
```

## 📋 Migration Strategy

1. **Identify what's actually used**: Only export items that are genuinely needed by consumers
2. **Group related exports**: Organize exports by functional area
3. **Use type aliases for complex types**: Simplify common type combinations
4. **Document public API**: Add rustdoc comments for exported items

## 🔄 Common Replacements

| Wildcard | Explicit Alternative |
|----------|---------------------|
| `pub use config::*;` | `pub use config::{UnifiedSongbirdConfig, GlobalConfig};` |
| `pub use traits::*;` | `pub use traits::{PrimalProvider, ServiceDiscovery, HealthCheck};` |
| `pub use errors::*;` | `pub use errors::{SongbirdError, SongbirdResult, success};` |
| `pub use types::*;` | `pub use types::{ServiceInfo, HealthStatus, Config};` |

