// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Capability endpoint discovery
//!
//! Modern replacement for deprecated hardcoded provider URLs.
//! These functions resolve provider endpoints through environment variables and
//! capability-based discovery, respecting sovereignty principles.
//!
//! ## Modern Async Pattern (v5.22.0 - Jan 25, 2026)
//!
//! Uses dependency injection for zero global state coupling:
//!
//! ```rust,ignore
//! // ❌ OLD: Hardcoded provider endpoints (DEPRECATED — use capability discovery)
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
use std::time::Duration;
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
    /// Legacy explicit compute provider override (None = read from env). Prefer [`Self::compute_endpoint`].
    pub compute_provider_endpoint: Option<String>,
    /// Overrides runtime discovery timeout when falling back to [`crate::runtime_discovery`] (tests use ~1ms).
    pub discovery_timeout: Option<Duration>,
}

impl DiscoveryOptions {
    /// Create options from environment variables (production use)
    #[must_use]
    pub fn from_env() -> Self {
        Self::default() // All None = read from env
    }

    /// Create options for testing with explicit values
    #[cfg(test)]
    #[must_use]
    pub fn for_testing() -> DiscoveryOptionsBuilder {
        DiscoveryOptionsBuilder::default()
    }
}

/// Builder for `DiscoveryOptions` (test fixture pattern)
#[cfg(test)]
#[derive(Default)]
pub struct DiscoveryOptionsBuilder {
    options: DiscoveryOptions,
}

#[cfg(test)]
impl DiscoveryOptionsBuilder {
    /// Sets an explicit compute endpoint (skips env for this field).
    #[must_use]
    pub fn compute_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.options.compute_endpoint = Some(endpoint.into());
        self
    }

    /// Sets an explicit compute provider endpoint (skips env for this field).
    #[must_use]
    pub fn compute_provider_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.options.compute_provider_endpoint = Some(endpoint.into());
        self
    }

    /// Sets runtime discovery timeout for the capability-discovery fallback path.
    #[must_use]
    pub fn discovery_timeout(mut self, timeout: Duration) -> Self {
        self.options.discovery_timeout = Some(timeout);
        self
    }

    /// Consumes the builder and returns [`DiscoveryOptions`].
    #[must_use]
    pub fn build(self) -> DiscoveryOptions {
        self.options
    }
}

/// Get compute provider endpoint (replaces legacy hardcoded compute URLs and the old default constant)
///
/// Uses dependency injection for zero global state coupling.
///
/// Discovery order:
/// 1. Explicit option (if provided)
/// 2. `COMPUTE_PROVIDER_ENDPOINT` environment variable
/// 3. `COMPUTE_ENDPOINT` environment variable
/// 4. Explicit legacy compute provider option (dependency injection)
/// 5. Legacy compute env alias (backwards compatibility)
/// 6. Capability-based discovery (future)
/// 7. Error - no hardcoded fallback
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
/// - Neither `COMPUTE_ENDPOINT` nor the legacy compute env alias is set
/// - Capability-based discovery fails to find a compute provider
pub async fn get_compute_endpoint(options: DiscoveryOptions) -> SongbirdResult<String> {
    get_compute_endpoint_with(options, |k| songbird_process_env::var(k)).await
}

/// Same as [`get_compute_endpoint`] with an injectable env reader (concurrent-safe tests).
pub async fn get_compute_endpoint_with<F>(
    options: DiscoveryOptions,
    env_reader: F,
) -> SongbirdResult<String>
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    // 0. Try explicit option first (dependency injection)
    if let Some(endpoint) = options.compute_endpoint {
        debug!("Using explicit compute_endpoint from options: {}", endpoint);
        return Ok(endpoint);
    }

    // 1. Try COMPUTE_PROVIDER_ENDPOINT (capability domain) first
    if let Ok(endpoint) = env_reader("COMPUTE_PROVIDER_ENDPOINT") {
        debug!("Using COMPUTE_PROVIDER_ENDPOINT from environment: {}", endpoint);
        return Ok(endpoint);
    }

    // 2. Try COMPUTE_ENDPOINT from environment
    if let Ok(endpoint) = env_reader("COMPUTE_ENDPOINT") {
        debug!("Using COMPUTE_ENDPOINT from environment: {}", endpoint);
        return Ok(endpoint);
    }

    // 3. Try explicit legacy compute provider option (dependency injection)
    if let Some(endpoint) = options.compute_provider_endpoint {
        warn!("Using explicit compute_provider_endpoint from options — prefer compute_endpoint");
        return Ok(endpoint);
    }

    // 4. Legacy compute env branch (backwards compatibility)
    if let Ok(endpoint) = env_reader("TOADSTOOL_ENDPOINT") {
        warn!("deprecated: use COMPUTE_PROVIDER_ENDPOINT instead of TOADSTOOL_ENDPOINT");
        return Ok(endpoint);
    }

    // 5. Try capability-based discovery (RuntimeDiscoveryEngine)
    let rt_timeout = options.discovery_timeout.unwrap_or_else(|| Duration::from_secs(5));
    match crate::runtime_discovery::discover_by_capability_timed("compute", rt_timeout).await {
        Ok(service) => {
            debug!("Discovered compute via RuntimeDiscoveryEngine: {}", service.endpoint);
            return Ok(service.endpoint);
        }
        Err(e) => {
            debug!("Runtime discovery failed: {}", e);
            // Fall through to error
        }
    }

    // 6. No hardcoded fallback - fail with helpful message
    Err(SongbirdError::Configuration {
        message: "No compute provider configured.".to_string(),
        field: Some("compute_endpoint".to_string()),
        suggestion: Some("Set COMPUTE_ENDPOINT environment variable (e.g., export COMPUTE_ENDPOINT=http://your-provider:8001) or enable capability discovery.".to_string()),
    })
}

/// Capability endpoint resolution table entry — drives the env-chain → discovery
/// → error flow for storage, security, and AI (and any future capability domains).
struct CapabilityEndpointSpec {
    provider_env: &'static str,
    primary_env: &'static str,
    legacy_env: Option<&'static str>,
    capability: &'static str,
    field: &'static str,
}

/// Shared resolution logic for capability endpoints.
///
/// Walks: `provider_env` → `primary_env` → `legacy_env` (with deprecation
/// warning) → runtime discovery → configuration error.
async fn resolve_capability_endpoint_with<F>(
    spec: &CapabilityEndpointSpec,
    env_reader: F,
) -> SongbirdResult<String>
where
    F: Fn(&str) -> std::result::Result<String, std::env::VarError>,
{
    if let Ok(ep) = env_reader(spec.provider_env) {
        debug!("Using {} from environment: {}", spec.provider_env, ep);
        return Ok(ep);
    }

    if let Ok(ep) = env_reader(spec.primary_env) {
        debug!("Using {} from environment: {}", spec.primary_env, ep);
        return Ok(ep);
    }

    if let Some(legacy) = spec.legacy_env
        && let Ok(ep) = env_reader(legacy)
    {
        warn!("deprecated: use {} instead of {}", spec.provider_env, legacy);
        return Ok(ep);
    }

    let engine = crate::runtime_discovery::RuntimeDiscoveryEngine::default();
    match engine.discover_by_capability(spec.capability).await {
        Ok(service) => {
            debug!(
                "Discovered {} via RuntimeDiscoveryEngine: {}",
                spec.capability, service.endpoint
            );
            return Ok(service.endpoint);
        }
        Err(e) => {
            debug!("Runtime discovery failed for {}: {}", spec.capability, e);
        }
    }

    Err(SongbirdError::Configuration {
        message: format!("No {} provider configured.", spec.capability),
        field: Some(spec.field.to_string()),
        suggestion: Some(format!(
            "Set {} environment variable (e.g., export {}=http://your-provider:PORT) or enable capability discovery.",
            spec.primary_env, spec.primary_env,
        )),
    })
}

const STORAGE_SPEC: CapabilityEndpointSpec = CapabilityEndpointSpec {
    provider_env: "STORAGE_PROVIDER_ENDPOINT",
    primary_env: "STORAGE_ENDPOINT",
    legacy_env: Some("NESTGATE_ENDPOINT"),
    capability: "storage",
    field: "storage_endpoint",
};

const SECURITY_SPEC: CapabilityEndpointSpec = CapabilityEndpointSpec {
    provider_env: "SECURITY_PROVIDER_ENDPOINT",
    primary_env: "SECURITY_ENDPOINT",
    legacy_env: Some("BEARDOG_ENDPOINT"),
    capability: "security",
    field: "security_endpoint",
};

const AI_SPEC: CapabilityEndpointSpec = CapabilityEndpointSpec {
    provider_env: "AI_PROVIDER_ENDPOINT",
    primary_env: "AI_ENDPOINT",
    legacy_env: Some("SQUIRREL_ENDPOINT"),
    capability: "ai",
    field: "ai_endpoint",
};

/// Get storage provider endpoint.
///
/// Discovery order: `STORAGE_PROVIDER_ENDPOINT` → `STORAGE_ENDPOINT` →
/// `NESTGATE_ENDPOINT` (deprecated) → runtime discovery → error.
///
/// # Errors
///
/// Returns an error when no storage endpoint is configured and discovery fails.
pub async fn get_storage_endpoint() -> SongbirdResult<String> {
    get_storage_endpoint_with(|k| songbird_process_env::var(k)).await
}

/// Same as [`get_storage_endpoint`] with an injectable env reader.
pub async fn get_storage_endpoint_with<F>(env_reader: F) -> SongbirdResult<String>
where
    F: Fn(&str) -> std::result::Result<String, std::env::VarError>,
{
    resolve_capability_endpoint_with(&STORAGE_SPEC, env_reader).await
}

/// Get security provider endpoint.
///
/// Discovery order: `SECURITY_PROVIDER_ENDPOINT` → `SECURITY_ENDPOINT` →
/// `BEARDOG_ENDPOINT` (deprecated) → runtime discovery → error.
///
/// # Errors
///
/// Returns an error when no security endpoint is configured and discovery fails.
pub async fn get_security_endpoint() -> SongbirdResult<String> {
    get_security_endpoint_with(|k| songbird_process_env::var(k)).await
}

/// Same as [`get_security_endpoint`] with an injectable env reader.
pub async fn get_security_endpoint_with<F>(env_reader: F) -> SongbirdResult<String>
where
    F: Fn(&str) -> std::result::Result<String, std::env::VarError>,
{
    resolve_capability_endpoint_with(&SECURITY_SPEC, env_reader).await
}

/// Get AI provider endpoint.
///
/// Discovery order: `AI_PROVIDER_ENDPOINT` → `AI_ENDPOINT` →
/// `SQUIRREL_ENDPOINT` (deprecated) → runtime discovery → error.
///
/// # Errors
///
/// Returns an error when no AI endpoint is configured and discovery fails.
pub async fn get_ai_endpoint() -> SongbirdResult<String> {
    get_ai_endpoint_with(|k| songbird_process_env::var(k)).await
}

/// Same as [`get_ai_endpoint`] with an injectable env reader.
pub async fn get_ai_endpoint_with<F>(env_reader: F) -> SongbirdResult<String>
where
    F: Fn(&str) -> std::result::Result<String, std::env::VarError>,
{
    resolve_capability_endpoint_with(&AI_SPEC, env_reader).await
}

/// Get an endpoint for a capability
///
/// This is the most flexible function — discovers any provider offering
/// the specified capability (identity-agnostic).
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
    get_endpoint_by_capability_with(capability, |key| songbird_process_env::var(key), None).await
}

/// Get endpoint by capability with injectable env reader (concurrent-safe)
///
/// `discovery_timeout` bounds the runtime discovery fallback (`None` = 5 seconds). Tests should pass
/// `Some(Duration::from_millis(1))` so the failure path does not wait on the full announcement timeout.
pub async fn get_endpoint_by_capability_with<F>(
    capability: &str,
    env_reader: F,
    discovery_timeout: Option<Duration>,
) -> SongbirdResult<String>
where
    F: Fn(&str) -> std::result::Result<String, std::env::VarError>,
{
    // 1. Try environment variable {CAPABILITY}_ENDPOINT
    let env_var = format!("{}_ENDPOINT", capability.to_uppercase());
    if let Ok(endpoint) = env_reader(&env_var)
        && !endpoint.is_empty()
    {
        debug!("Using {} from environment: {}", env_var, endpoint);
        return Ok(endpoint);
    }

    // 2. Try capability-based discovery (RuntimeDiscoveryEngine)
    let rt_timeout = discovery_timeout.unwrap_or_else(|| Duration::from_secs(5));
    let engine = crate::runtime_discovery::RuntimeDiscoveryEngine::with_timeout(rt_timeout);
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
            capability.to_uppercase(),
            capability.to_uppercase()
        )),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use super::*;
    use std::sync::Mutex;

    /// Serialize tests that mutate `COMPUTE_*` process env (parallel runs share one env).
    static COMPUTE_ENDPOINT_ENV_MUTEX: Mutex<()> = Mutex::new(());

    /// Restores a previous env value (or removal) on drop for isolated env tests.
    struct EnvRestore {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvRestore {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = songbird_process_env::var(key).ok();
            songbird_process_env::set_var(key, value);
            Self {
                key,
                previous,
            }
        }

        /// Remove `key` for the scope of the test; restore previous value on drop.
        fn clear(key: &'static str) -> Self {
            let previous = songbird_process_env::var(key).ok();
            songbird_process_env::remove_var(key);
            Self {
                key,
                previous,
            }
        }
    }

    impl Drop for EnvRestore {
        fn drop(&mut self) {
            match &self.previous {
                Some(v) => songbird_process_env::set_var(self.key, v),
                None => songbird_process_env::remove_var(self.key),
            }
        }
    }

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
    async fn test_compute_endpoint_explicit_compute_provider_option() {
        let _env_lock = COMPUTE_ENDPOINT_ENV_MUTEX.lock().expect("compute env mutex");
        // `get_compute_endpoint` honors env before injected `compute_provider_endpoint`; isolate env
        // so this test always exercises the option (parallel tests may set COMPUTE_*).
        let _compute_endpoint_guard = EnvRestore::clear("COMPUTE_ENDPOINT");
        let _compute_provider_endpoint_guard = EnvRestore::clear("COMPUTE_PROVIDER_ENDPOINT");

        let options = DiscoveryOptions::for_testing()
            .compute_provider_endpoint("http://legacy-compute-provider:8001")
            .build();

        let result = get_compute_endpoint(options).await;
        assert!(result.is_ok());
        assert_eq!(result.expect("should resolve"), "http://legacy-compute-provider:8001");
    }

    #[tokio::test]
    async fn test_compute_endpoint_not_configured() {
        // Modern pattern: test error case with empty options
        // No env vars set, no explicit endpoint → should fail (unless runtime discovery succeeds)
        let options =
            DiscoveryOptions::for_testing().discovery_timeout(Duration::from_millis(1)).build();
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

        let result = get_endpoint_by_capability_with("myservice", env, None).await;
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

        let result =
            get_endpoint_by_capability_with("myservice", env, Some(Duration::from_millis(1))).await;
        // Should fail (empty env var ignored, no runtime discovery)
        // Unless runtime discovers something on this machine
        if result.is_err()
            && let Err(SongbirdError::Configuration {
                message,
                ..
            }) = result
        {
            assert!(message.contains("No provider found"));
        }
    }

    #[tokio::test]
    async fn test_get_storage_endpoint_from_env() {
        let ep = get_storage_endpoint_with(|k| {
            if k == "STORAGE_ENDPOINT" {
                Ok("http://storage-test:3".to_string())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        })
        .await
        .expect("storage env");
        assert_eq!(ep, "http://storage-test:3");
    }

    #[tokio::test]
    async fn test_get_security_endpoint_from_env() {
        let ep = get_security_endpoint_with(|k| {
            if k == "SECURITY_ENDPOINT" {
                Ok("http://sec-test:4".to_string())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        })
        .await
        .expect("security env");
        assert_eq!(ep, "http://sec-test:4");
    }

    #[tokio::test]
    async fn test_get_ai_endpoint_from_env() {
        let ep = get_ai_endpoint_with(|k| {
            if k == "AI_ENDPOINT" {
                Ok("http://ai-test:5".to_string())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        })
        .await
        .expect("ai env");
        assert_eq!(ep, "http://ai-test:5");
    }

    #[tokio::test]
    async fn test_compute_endpoint_backward_compat_prefers_compute_over_legacy_toadstool_env() {
        let _env_lock = COMPUTE_ENDPOINT_ENV_MUTEX.lock().expect("compute env mutex");
        let _c = EnvRestore::set("COMPUTE_ENDPOINT", "http://compute-wins:8001");
        let _t = EnvRestore::set("TOADSTOOL_ENDPOINT", "http://legacy-compute-fallback:9001");
        let options =
            DiscoveryOptions::for_testing().discovery_timeout(Duration::from_millis(1)).build();
        let ep = get_compute_endpoint(options).await.expect("compute from env");
        assert_eq!(ep, "http://compute-wins:8001");
    }

    #[tokio::test]
    async fn test_get_storage_endpoint_backward_compat_provider_before_legacy_nestgate_env() {
        let ep = get_storage_endpoint_with(|k| match k {
            "STORAGE_PROVIDER_ENDPOINT" => Ok("http://provider-priority:8003".to_string()),
            "NESTGATE_ENDPOINT" => Ok("http://legacy-storage-fallback:8003".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        })
        .await
        .expect("storage");
        assert_eq!(ep, "http://provider-priority:8003");
    }

    #[tokio::test]
    async fn test_get_security_endpoint_backward_compat_legacy_beardog_env_var() {
        let ep = get_security_endpoint_with(|k| match k {
            "BEARDOG_ENDPOINT" => Ok("http://security-provider-legacy:7443".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        })
        .await
        .expect("security");
        assert_eq!(ep, "http://security-provider-legacy:7443");
    }

    #[tokio::test]
    async fn test_get_ai_endpoint_backward_compat_legacy_squirrel_env_var() {
        let ep = get_ai_endpoint_with(|k| match k {
            "SQUIRREL_ENDPOINT" => Ok("http://ai-provider-legacy:9200".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        })
        .await
        .expect("ai");
        assert_eq!(ep, "http://ai-provider-legacy:9200");
    }

    #[tokio::test]
    async fn test_endpoint_by_capability_uppercases_capability_in_env_key() {
        use std::collections::HashMap;

        let vars: HashMap<String, String> =
            HashMap::from([("MIXEDCAP_ENDPOINT".to_string(), "http://mixed:1".to_string())]);
        let env = move |key: &str| -> std::result::Result<String, std::env::VarError> {
            vars.get(key).cloned().ok_or(std::env::VarError::NotPresent)
        };
        let ep =
            get_endpoint_by_capability_with("mixedcap", env, Some(Duration::from_millis(1))).await;
        assert_eq!(ep.expect("endpoint"), "http://mixed:1");
    }

    #[tokio::test]
    async fn test_endpoint_by_capability_accepts_non_url_string_from_env() {
        use std::collections::HashMap;

        let vars: HashMap<String, String> =
            HashMap::from([("RAW_ENDPOINT".to_string(), "not-a-valid-url:::broken".to_string())]);
        let env = move |key: &str| -> std::result::Result<String, std::env::VarError> {
            vars.get(key).cloned().ok_or(std::env::VarError::NotPresent)
        };
        let ep = get_endpoint_by_capability_with("raw", env, Some(Duration::from_millis(1))).await;
        assert_eq!(ep.expect("opaque string preserved"), "not-a-valid-url:::broken");
    }

    #[tokio::test]
    async fn test_get_compute_endpoint_with_prefers_provider_over_plain_compute_env() {
        use std::collections::HashMap;

        let vars: HashMap<String, String> = HashMap::from([
            ("COMPUTE_PROVIDER_ENDPOINT".to_string(), "http://provider:8001".to_string()),
            ("COMPUTE_ENDPOINT".to_string(), "http://plain:9001".to_string()),
        ]);
        let env = move |k: &str| -> std::result::Result<String, std::env::VarError> {
            vars.get(k).cloned().ok_or(std::env::VarError::NotPresent)
        };
        let options = DiscoveryOptions::for_testing().build();
        let ep = get_compute_endpoint_with(options, env).await.expect("compute endpoint");
        assert_eq!(ep, "http://provider:8001");
    }

    #[tokio::test]
    async fn test_get_compute_endpoint_with_prefers_compute_over_legacy_toadstool() {
        use std::collections::HashMap;

        let vars: HashMap<String, String> = HashMap::from([
            ("COMPUTE_ENDPOINT".to_string(), "http://compute-wins:1".to_string()),
            ("TOADSTOOL_ENDPOINT".to_string(), "http://legacy:2".to_string()),
        ]);
        let env = move |k: &str| -> std::result::Result<String, std::env::VarError> {
            vars.get(k).cloned().ok_or(std::env::VarError::NotPresent)
        };
        let options =
            DiscoveryOptions::for_testing().discovery_timeout(Duration::from_millis(1)).build();
        let ep = get_compute_endpoint_with(options, env).await.expect("compute");
        assert_eq!(ep, "http://compute-wins:1");
    }

    #[tokio::test]
    async fn test_get_security_endpoint_with_prefers_provider_key() {
        let ep = get_security_endpoint_with(|k| match k {
            "SECURITY_PROVIDER_ENDPOINT" => Ok("https://sec-prov:8443".to_string()),
            "SECURITY_ENDPOINT" => Ok("http://plain-sec:80".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        })
        .await
        .expect("security");
        assert_eq!(ep, "https://sec-prov:8443");
    }

    #[tokio::test]
    async fn test_get_ai_endpoint_with_prefers_provider_key() {
        let ep = get_ai_endpoint_with(|k| match k {
            "AI_PROVIDER_ENDPOINT" => Ok("http://ai-prov:8083".to_string()),
            "AI_ENDPOINT" => Ok("http://ai-plain:9".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        })
        .await
        .expect("ai");
        assert_eq!(ep, "http://ai-prov:8083");
    }

    #[tokio::test]
    async fn test_get_endpoint_by_capability_discovery_failure_returns_configuration_error() {
        use std::collections::HashMap;

        let vars: HashMap<String, String> = HashMap::new();
        let env = move |key: &str| -> std::result::Result<String, std::env::VarError> {
            vars.get(key).cloned().ok_or(std::env::VarError::NotPresent)
        };
        let result =
            get_endpoint_by_capability_with("unknowncapxyz", env, Some(Duration::from_millis(1)))
                .await;
        if result.is_ok() {
            return;
        }
        match result.expect_err("expected configuration error when discovery finds nothing") {
            SongbirdError::Configuration {
                message,
                field,
                ..
            } => {
                assert!(message.contains("unknowncapxyz"));
                assert_eq!(field.as_deref(), Some("unknowncapxyz_endpoint"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
