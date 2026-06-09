// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! High-level discovery functions
//!
//! Convenience functions for common discovery patterns.

use crate::capability::provider::Provider;
use crate::capability::registry::CapabilityRegistry;
use crate::error::IpcResult;
use std::sync::OnceLock;

/// Global capability registry
static GLOBAL_REGISTRY: OnceLock<CapabilityRegistry> = OnceLock::new();

/// Initialize global capability registry
///
/// This should be called once at startup.
/// It's safe to call multiple times (subsequent calls are no-ops).
pub fn init_capability_registry() {
    GLOBAL_REGISTRY.get_or_init(CapabilityRegistry::new);
}

/// Get global capability registry
///
/// # Panics
/// Panics if registry hasn't been initialized.
/// Call `init_capability_registry()` first.
pub fn global_registry() -> &'static CapabilityRegistry {
    GLOBAL_REGISTRY
        .get()
        .expect("Capability registry not initialized. Call init_capability_registry() first!")
}

/// Discover a provider for the given capability
///
/// Convenience function using the global registry.
///
/// # Example
/// ```rust,no_run
/// # use songbird_universal_ipc::capability::discovery;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// discovery::init_capability_registry();
///
/// let provider = discovery::discover("crypto").await?;
/// println!("Found: {}", provider.id);
/// # Ok(())
/// # }
/// ```
pub async fn discover(capability: &str) -> IpcResult<Provider> {
    global_registry().discover(capability).await
}

/// Discover all providers for the given capability
///
/// Convenience function using the global registry.
pub async fn discover_all(capability: &str) -> IpcResult<Vec<Provider>> {
    global_registry().discover_all(capability).await
}

/// Clear discovery cache
///
/// Forces re-discovery on next request.
pub async fn clear_cache() {
    global_registry().clear_cache().await;
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::capability::strategy::EnvironmentStrategy;
    use songbird_process_env;

    #[tokio::test]
    async fn discover_finds_provider_via_environment() {
        use songbird_process_env::ScopedEnv;
        init_capability_registry();
        let cap = "sbipcdiscserial";
        let key = format!("{}_PROVIDER_SOCKET", cap.to_uppercase());
        let _env = ScopedEnv::new(&key, "/tmp/sbipc_disc_test.sock");
        let p = discover(cap).await.expect("discover");
        assert!(!p.id.is_empty());
    }

    #[tokio::test]
    async fn discover_all_returns_vec_from_environment() {
        use songbird_process_env::ScopedEnv;
        init_capability_registry();
        let cap = "sbipcdiscall";
        let key = format!("{}_PROVIDER_SOCKET", cap.to_uppercase());
        let _env = ScopedEnv::new(&key, "/tmp/sbipc_disc_all.sock");
        let v = discover_all(cap).await.expect("discover_all");
        assert_eq!(v.len(), 1);
    }

    #[tokio::test]
    async fn test_global_registry() {
        init_capability_registry();

        let providers = EnvironmentStrategy::discover_with("test", |k| {
            if k == "TEST_PROVIDER_SOCKET" {
                Ok("/tmp/test.sock".to_string())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        })
        .await
        .unwrap();
        assert!(!providers.is_empty());
    }

    #[test]
    fn init_capability_registry_is_idempotent() {
        init_capability_registry();
        init_capability_registry();
        let _ = global_registry();
    }

    #[tokio::test]
    async fn clear_cache_runs_after_init() {
        init_capability_registry();
        clear_cache().await;
    }
}
