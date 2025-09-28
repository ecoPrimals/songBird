//! # 🔧 Canonical Traits
//!
//! **MODERN TRAIT SYSTEM** ✅
//!
//! This module provides the canonical trait definitions that all
//! Songbird components implement for consistency and interoperability.

use std::collections::HashMap;

/// Capability definition
#[derive(Debug, Clone)]
pub struct Capability {
    /// Capability name
    pub name: String,
    /// Capability version
    pub version: String,
    /// Capability metadata
    pub metadata: HashMap<String, String>,
}

// ============================================================================
// MIGRATION NOTICE
// ============================================================================

/// Migration notice for deprecated traits
pub const TRAIT_MIGRATION_NOTICE: &str = r#"
🚨 TRAIT MIGRATION COMPLETE 🚨

All provider traits have been migrated to the canonical trait system:

USE THESE CANONICAL TRAITS:
```rust
use songbird_types::traits::{
    Provider,
    ServiceProvider,
    PrimalProvider,
    DiscoveryProvider,
    CapabilityProvider,
    SecurityProvider,
    OrchestrationProvider,
    ObservabilityProvider,
};
```

All deprecated trait re-exports have been removed.
Update your imports to use songbird_types::traits directly.
"#;
