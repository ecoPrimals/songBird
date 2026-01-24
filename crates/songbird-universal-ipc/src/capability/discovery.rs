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
    global_registry().clear_cache().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_global_registry() {
        init_capability_registry();

        std::env::set_var("TEST_PROVIDER_SOCKET", "/tmp/test.sock");

        let provider = discover("test").await;
        assert!(provider.is_ok());

        std::env::remove_var("TEST_PROVIDER_SOCKET");
    }
}
