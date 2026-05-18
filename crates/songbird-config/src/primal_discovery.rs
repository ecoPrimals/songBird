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
#[path = "primal_discovery_tests.rs"]
mod tests;
