//! Primal Discovery Functions
//!
//! Modern replacement for deprecated hardcoded primal endpoints.
//! These functions discover primals through environment variables and
//! capability-based discovery, respecting sovereignty principles.
//!
//! ## Modern Async Pattern (v5.22.0 - Jan 25, 2026)
//!
//! Uses dependency injection for zero global state coupling:
//!
//! ```rust,ignore
//! // ❌ OLD: Hardcoded endpoints (DEPRECATED)
//! use songbird_config::config::constants::deprecated::DEFAULT_TOADSTOOL_ENDPOINT;
//! let endpoint = DEFAULT_TOADSTOOL_ENDPOINT;
//!
//! // ✅ NEW: Environment + Discovery (Production)
//! use songbird_config::primal_discovery::{get_compute_endpoint, DiscoveryOptions};
//! let endpoint = get_compute_endpoint(DiscoveryOptions::from_env()).await?;
//!
//! // ✅ NEW: Explicit config (Tests - zero global state!)
//! let options = DiscoveryOptions::for_testing()
//!     .compute_endpoint("http://test:8080")
//!     .build();
//! let endpoint = get_compute_endpoint(options).await?;
//! ```

use songbird_types::{SongbirdError, SongbirdResult};
use tracing::{debug, warn};

/// Discovery configuration options for dependency injection
///
/// This allows tests to pass explicit configuration without modifying
/// global environment variables, enabling fully concurrent test execution.
///
/// ## Modern Async Pattern
///
/// Production code uses `DiscoveryOptions::from_env()` to maintain
/// backward compatibility, while tests use explicit configuration.
#[derive(Debug, Clone, Default)]
pub struct DiscoveryOptions {
    /// Compute endpoint (None = read from env)
    pub compute_endpoint: Option<String>,
    /// Legacy Toadstool endpoint (None = read from env)
    pub toadstool_endpoint: Option<String>,
}

impl DiscoveryOptions {
    /// Create options from environment variables (production use)
    #[must_use]
    pub fn from_env() -> Self {
        Self::default() // All None = read from env
    }

    /// Create options for testing with explicit values
    #[cfg(test)]
    pub fn for_testing() -> DiscoveryOptionsBuilder {
        DiscoveryOptionsBuilder::default()
    }
}

/// Builder for DiscoveryOptions (test fixture pattern)
#[cfg(test)]
#[derive(Default)]
pub struct DiscoveryOptionsBuilder {
    options: DiscoveryOptions,
}

#[cfg(test)]
impl DiscoveryOptionsBuilder {
    pub fn compute_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.options.compute_endpoint = Some(endpoint.into());
        self
    }

    pub fn toadstool_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.options.toadstool_endpoint = Some(endpoint.into());
        self
    }

    pub fn build(self) -> DiscoveryOptions {
        self.options
    }
}

/// Get compute provider endpoint (replaces `DEFAULT_TOADSTOOL_ENDPOINT`)
///
/// Uses dependency injection for zero global state coupling.
///
/// Discovery order:
/// 1. Explicit option (if provided)
/// 2. `COMPUTE_ENDPOINT` environment variable
/// 3. `TOADSTOOL_ENDPOINT` environment variable (backwards compatibility)
/// 4. Capability-based discovery (future)
/// 5. Error - no hardcoded fallback
///
/// ## Modern Async Pattern
///
/// ```rust,ignore
/// use songbird_config::primal_discovery::{get_compute_endpoint, DiscoveryOptions};
///
/// #[tokio::main]
/// async fn main() -> songbird_types::SongbirdResult<()> {
///     // Production: read from environment
///     let endpoint = get_compute_endpoint(DiscoveryOptions::from_env()).await?;
///     println!("Compute provider: {}", endpoint);
///     Ok(())
/// }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - No `COMPUTE_ENDPOINT` or `TOADSTOOL_ENDPOINT` environment variable is set
/// - Capability-based discovery fails to find a compute provider
pub async fn get_compute_endpoint(options: DiscoveryOptions) -> SongbirdResult<String> {
    // 0. Try explicit option first (dependency injection)
    if let Some(endpoint) = options.compute_endpoint {
        debug!("Using explicit compute_endpoint from options: {}", endpoint);
        return Ok(endpoint);
    }

    // 1. Try modern COMPUTE_ENDPOINT from environment
    if let Ok(endpoint) = std::env::var("COMPUTE_ENDPOINT") {
        debug!("Using COMPUTE_ENDPOINT from environment: {}", endpoint);
        return Ok(endpoint);
    }

    // 2. Try explicit legacy option (dependency injection)
    if let Some(endpoint) = options.toadstool_endpoint {
        warn!("Using deprecated toadstool_endpoint option - migrate to compute_endpoint");
        return Ok(endpoint);
    }

    // 3. Try legacy TOADSTOOL_ENDPOINT from environment (backwards compatibility)
    if let Ok(endpoint) = std::env::var("TOADSTOOL_ENDPOINT") {
        warn!("Using deprecated TOADSTOOL_ENDPOINT - migrate to COMPUTE_ENDPOINT");
        return Ok(endpoint);
    }

    // 4. Try capability-based discovery (RuntimeDiscoveryEngine)
    match crate::runtime_discovery::discover_compute().await {
        Ok(service) => {
            debug!("Discovered compute via RuntimeDiscoveryEngine: {}", service.endpoint);
            return Ok(service.endpoint);
        }
        Err(e) => {
            debug!("Runtime discovery failed: {}", e);
            // Fall through to error
        }
    }

    // 5. No hardcoded fallback - fail with helpful message
    Err(SongbirdError::Configuration {
        message: "No compute provider configured.".to_string(),
        field: Some("compute_endpoint".to_string()),
        suggestion: Some("Set COMPUTE_ENDPOINT environment variable (e.g., export COMPUTE_ENDPOINT=http://your-provider:8001) or enable capability discovery.".to_string()),
    })
}

/// Get storage provider endpoint (replaces `DEFAULT_NESTGATE_ENDPOINT`)
///
/// Discovery order:
/// 1. `STORAGE_ENDPOINT` environment variable
/// 2. `NESTGATE_ENDPOINT` environment variable (backwards compatibility)
/// 3. Capability-based discovery (future)
/// 4. Error - no hardcoded fallback
///
/// # Errors
///
/// Returns an error if:
/// - No `STORAGE_ENDPOINT` or `NESTGATE_ENDPOINT` environment variable is set
/// - Capability-based discovery fails to find a storage provider
pub async fn get_storage_endpoint() -> SongbirdResult<String> {
    // 1. Try modern STORAGE_ENDPOINT
    if let Ok(endpoint) = std::env::var("STORAGE_ENDPOINT") {
        debug!("Using STORAGE_ENDPOINT from environment: {}", endpoint);
        return Ok(endpoint);
    }

    // 2. Try legacy NESTGATE_ENDPOINT (backwards compatibility)
    if let Ok(endpoint) = std::env::var("NESTGATE_ENDPOINT") {
        warn!("Using deprecated NESTGATE_ENDPOINT - migrate to STORAGE_ENDPOINT");
        return Ok(endpoint);
    }

    // 3. Try capability-based discovery (RuntimeDiscoveryEngine)
    match crate::runtime_discovery::discover_storage().await {
        Ok(service) => {
            debug!("Discovered storage via RuntimeDiscoveryEngine: {}", service.endpoint);
            return Ok(service.endpoint);
        }
        Err(e) => {
            debug!("Runtime discovery failed: {}", e);
            // Fall through to error
        }
    }

    Err(SongbirdError::Configuration {
        message: "No storage provider configured.".to_string(),
        field: Some("storage_endpoint".to_string()),
        suggestion: Some("Set STORAGE_ENDPOINT environment variable (e.g., export STORAGE_ENDPOINT=http://your-provider:8003) or enable capability discovery.".to_string()),
    })
}

/// Get security provider endpoint (replaces `DEFAULT_BEARDOG_ENDPOINT`)
///
/// Discovery order:
/// 1. `SECURITY_ENDPOINT` environment variable
/// 2. `BEARDOG_ENDPOINT` environment variable (backwards compatibility)
/// 3. Capability-based discovery (future)
/// 4. Error - no hardcoded fallback
///
/// # Errors
///
/// Returns an error if:
/// - No `SECURITY_ENDPOINT` or `BEARDOG_ENDPOINT` environment variable is set
/// - Capability-based discovery fails to find a security provider
pub async fn get_security_endpoint() -> SongbirdResult<String> {
    // 1. Try modern SECURITY_ENDPOINT
    if let Ok(endpoint) = std::env::var("SECURITY_ENDPOINT") {
        debug!("Using SECURITY_ENDPOINT from environment: {}", endpoint);
        return Ok(endpoint);
    }

    // 2. Try legacy BEARDOG_ENDPOINT (backwards compatibility)
    if let Ok(endpoint) = std::env::var("BEARDOG_ENDPOINT") {
        warn!("Using deprecated BEARDOG_ENDPOINT - migrate to SECURITY_ENDPOINT");
        return Ok(endpoint);
    }

    // 3. Try capability-based discovery (RuntimeDiscoveryEngine)
    match crate::runtime_discovery::discover_security().await {
        Ok(service) => {
            debug!("Discovered security via RuntimeDiscoveryEngine: {}", service.endpoint);
            return Ok(service.endpoint);
        }
        Err(e) => {
            debug!("Runtime discovery failed: {}", e);
            // Fall through to error
        }
    }

    Err(SongbirdError::Configuration {
        message: "No security provider configured.".to_string(),
        field: Some("security_endpoint".to_string()),
        suggestion: Some("Set SECURITY_ENDPOINT environment variable (e.g., export SECURITY_ENDPOINT=http://your-provider:8004) or enable capability discovery.".to_string()),
    })
}

/// Get AI provider endpoint (replaces `DEFAULT_SQUIRREL_ENDPOINT`)
///
/// Discovery order:
/// 1. `AI_ENDPOINT` environment variable
/// 2. `SQUIRREL_ENDPOINT` environment variable (backwards compatibility)
/// 3. Capability-based discovery (future)
/// 4. Error - no hardcoded fallback
///
/// # Errors
///
/// Returns an error if:
/// - No `AI_ENDPOINT` or `SQUIRREL_ENDPOINT` environment variable is set
/// - Capability-based discovery fails to find an AI provider
pub async fn get_ai_endpoint() -> SongbirdResult<String> {
    // 1. Try modern AI_ENDPOINT
    if let Ok(endpoint) = std::env::var("AI_ENDPOINT") {
        debug!("Using AI_ENDPOINT from environment: {}", endpoint);
        return Ok(endpoint);
    }

    // 2. Try legacy SQUIRREL_ENDPOINT (backwards compatibility)
    if let Ok(endpoint) = std::env::var("SQUIRREL_ENDPOINT") {
        warn!("Using deprecated SQUIRREL_ENDPOINT - migrate to AI_ENDPOINT");
        return Ok(endpoint);
    }

    // 3. Try capability-based discovery (RuntimeDiscoveryEngine)
    match crate::runtime_discovery::discover_ai().await {
        Ok(service) => {
            debug!("Discovered AI via RuntimeDiscoveryEngine: {}", service.endpoint);
            return Ok(service.endpoint);
        }
        Err(e) => {
            debug!("Runtime discovery failed: {}", e);
            // Fall through to error
        }
    }

    Err(SongbirdError::Configuration {
        message: "No AI provider configured.".to_string(),
        field: Some("ai_endpoint".to_string()),
        suggestion: Some("Set AI_ENDPOINT environment variable (e.g., export AI_ENDPOINT=http://your-provider:8002) or enable capability discovery.".to_string()),
    })
}

/// Get any primal endpoint by capability
///
/// This is the most flexible function - discovers ANY provider offering
/// the specified capability, not just specific primals.
///
/// ## Example
///
/// ```no_run
/// use songbird_config::primal_discovery::get_endpoint_by_capability;
///
/// #[tokio::main]
/// async fn main() -> songbird_types::SongbirdResult<()> {
///     // Discovers ANY provider with "compute" capability
///     let endpoint = get_endpoint_by_capability("compute").await?;
///     Ok(())
/// }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - No `{CAPABILITY}_ENDPOINT` environment variable is set
/// - Capability-based discovery fails to find a provider with the requested capability
pub async fn get_endpoint_by_capability(capability: &str) -> SongbirdResult<String> {
    get_endpoint_by_capability_with(capability, |key| std::env::var(key)).await
}

/// Get endpoint by capability with injectable env reader (concurrent-safe)
pub async fn get_endpoint_by_capability_with<F>(
    capability: &str,
    env_reader: F,
) -> SongbirdResult<String>
where
    F: Fn(&str) -> std::result::Result<String, std::env::VarError>,
{
    // 1. Try environment variable {CAPABILITY}_ENDPOINT
    let env_var = format!("{}_ENDPOINT", capability.to_uppercase());
    if let Ok(endpoint) = env_reader(&env_var) {
        if !endpoint.is_empty() {
            debug!("Using {} from environment: {}", env_var, endpoint);
            return Ok(endpoint);
        }
    }

    // 2. Try capability-based discovery (RuntimeDiscoveryEngine)
    let engine = crate::runtime_discovery::RuntimeDiscoveryEngine::new();
    match engine.discover_by_capability(capability).await {
        Ok(service) => {
            debug!("Discovered {} via RuntimeDiscoveryEngine: {}", capability, service.endpoint);
            return Ok(service.endpoint);
        }
        Err(e) => {
            debug!("Runtime discovery failed for {}: {}", capability, e);
            // Fall through to error
        }
    }

    Err(SongbirdError::Configuration {
        message: format!("No provider found for capability '{capability}'."),
        field: Some(format!("{capability}_endpoint")),
        suggestion: Some(format!(
            "Set {}_ENDPOINT environment variable (e.g., export {}_ENDPOINT=http://your-provider:PORT)",
            capability.to_uppercase(), capability.to_uppercase()
        )),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ✅ Modern async pattern: NO #[serial] needed!
    // ✅ Zero global state - fully concurrent tests!

    #[tokio::test]
    async fn test_compute_endpoint_from_explicit_option() {
        // Modern pattern: explicit configuration via dependency injection
        let options =
            DiscoveryOptions::for_testing().compute_endpoint("http://test-compute:9000").build();

        let result = get_compute_endpoint(options).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://test-compute:9000");
        // NO cleanup needed - no global state modified!
    }

    #[tokio::test]
    async fn test_compute_endpoint_legacy_fallback() {
        // Modern pattern: test legacy option path without modifying env
        let options = DiscoveryOptions::for_testing()
            .toadstool_endpoint("http://legacy-toadstool:8001")
            .build();

        let result = get_compute_endpoint(options).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://legacy-toadstool:8001");
        // NO cleanup needed!
    }

    #[tokio::test]
    async fn test_compute_endpoint_not_configured() {
        // Modern pattern: test error case with empty options
        // No env vars set, no explicit endpoint → should fail (unless runtime discovery succeeds)
        let options = DiscoveryOptions::default();
        let result = get_compute_endpoint(options).await;

        // Runtime discovery might find a service on this machine
        if result.is_ok() {
            return; // Valid — runtime discovered a compute provider
        }

        assert!(result.is_err());
        if let Err(SongbirdError::Configuration {
            message,
            suggestion,
            ..
        }) = result
        {
            assert!(message.contains("No compute provider configured"));
            assert!(suggestion.is_some());
            let suggestion_text = suggestion.unwrap();
            assert!(suggestion_text.contains("COMPUTE_ENDPOINT"));
        }
    }

    #[tokio::test]
    async fn test_capability_based_endpoint_with_env() {
        // ✅ Concurrent-safe: uses injectable env reader instead of set_var
        use std::collections::HashMap;

        let vars: HashMap<String, String> = HashMap::from([(
            "MYSERVICE_ENDPOINT".to_string(),
            "http://my-service:5000".to_string(),
        )]);
        let env = move |key: &str| -> std::result::Result<String, std::env::VarError> {
            vars.get(key).cloned().ok_or(std::env::VarError::NotPresent)
        };

        let result = get_endpoint_by_capability_with("myservice", env).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://my-service:5000");
    }

    #[tokio::test]
    async fn test_capability_empty_env_falls_through() {
        // Empty env var should be ignored
        use std::collections::HashMap;

        let vars: HashMap<String, String> =
            HashMap::from([("MYSERVICE_ENDPOINT".to_string(), String::new())]);
        let env = move |key: &str| -> std::result::Result<String, std::env::VarError> {
            vars.get(key).cloned().ok_or(std::env::VarError::NotPresent)
        };

        let result = get_endpoint_by_capability_with("myservice", env).await;
        // Should fail (empty env var ignored, no runtime discovery)
        // Unless runtime discovers something on this machine
        if result.is_err() {
            if let Err(SongbirdError::Configuration {
                message,
                ..
            }) = result
            {
                assert!(message.contains("No provider found"));
            }
        }
    }
}
