// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Security Capability Adapter
//!
//! **SOVEREIGNTY**: This adapter works with ANY security capability provider.
//! It does NOT assume a specific vendor implementation (only the capability contract).
//! Discovery is capability-based through environment hints or zero-knowledge bootstrap.

// Allow async_fn_in_trait warning - our traits guarantee Send + Sync
#![expect(async_fn_in_trait, reason = "async fn in trait (edition / trait-object compatibility)")]

use super::transport::{
    AdapterTransportKind, CapabilityTransport, build_default_transport, transport_kind_for_endpoint,
};
use songbird_types::{SafeEnv, SongbirdError, SongbirdResult};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

#[path = "security_types.rs"]
mod security_types;
pub use security_types::{AuthResult, SecurityHealth, SecurityMetrics};

#[cfg(test)]
#[path = "security_types_tests.rs"]
mod security_types_tests;

#[cfg(test)]
#[path = "security_tests.rs"]
mod security_tests;

#[cfg(test)]
#[path = "security_adapter_tests.rs"]
mod security_adapter_tests;

/// **CAPABILITY-BASED SECURITY ADAPTER**
///
/// Works with ANY security provider discovered through:
/// - Environment variable: `SONGBIRD_SECURITY_ENDPOINT`
/// - Capability discovery: `capability:security`
/// - Zero-knowledge bootstrap
///
/// **PROTOCOL AGNOSTIC** (v3.10.4):
/// - Automatically detects protocol based on endpoint URL scheme
/// - `unix://` → JSON-RPC 2.0 over Unix socket (port-free)
/// - `http://` or `https://` → HTTP protocol
/// - Enables fractal, isomorphic deployment patterns
#[derive(Clone)]
pub struct SecurityAdapter {
    /// Endpoint URL for the security capability provider
    endpoint: String,
    transport: Arc<dyn CapabilityTransport>,
    transport_kind: AdapterTransportKind,
    /// Request timeout
    timeout: Duration,
}

impl fmt::Debug for SecurityAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let protocol = match self.transport_kind {
            AdapterTransportKind::Tarpc => "Tarpc",
            AdapterTransportKind::JsonRpc => "JsonRpc",
            AdapterTransportKind::Http => "Http",
        };
        f.debug_struct("SecurityAdapter")
            .field("endpoint", &self.endpoint)
            .field("protocol", &protocol)
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl SecurityAdapter {
    /// Create adapter from discovered security capability
    ///
    /// Uses capability-based discovery:
    /// 1. Check `SONGBIRD_SECURITY_ENDPOINT` environment variable
    /// 2. Fall back to capability discovery
    /// 3. No hardcoded primal names anywhere
    ///
    /// **Runtime Discovery**: No compile-time knowledge of which security provider exists
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # tokio_test::block_on(async {
    /// use songbird_universal::adapters::SecurityAdapter;
    ///
    /// // Discovers any security provider implementation (vendor-specific or custom)
    /// let adapter = SecurityAdapter::from_discovery().await?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if no security capability can be discovered.
    pub async fn from_discovery() -> SongbirdResult<Self> {
        Self::from_discovery_with_resolver(
            songbird_config::capability_endpoints::CapabilityEndpointResolver::new(),
        )
        .await
    }

    /// Like [`Self::from_discovery`], but uses an explicit [`songbird_config::capability_endpoints::CapabilityEndpointResolver`].
    ///
    /// # Errors
    ///
    /// Returns an error if capability endpoint resolution or adapter construction fails.
    pub async fn from_discovery_with_resolver(
        resolver: songbird_config::capability_endpoints::CapabilityEndpointResolver,
    ) -> SongbirdResult<Self> {
        use songbird_config::capability_endpoints::CapabilityType;

        match resolver.get_endpoint(CapabilityType::Security).await {
            Ok(endpoint) => {
                debug!("✅ Security capability discovered via resolver: {}", endpoint);
                Self::new(endpoint).await
            }
            Err(discovery_err) => {
                debug!("🔍 Primary discovery failed, trying legacy fallbacks: {}", discovery_err);

                // Fallback 1: Environment variables (capability-first, then deprecated primal)
                if let Ok(endpoint) = SafeEnv::get_required("SONGBIRD_SECURITY_ENDPOINT") {
                    debug!("Using SONGBIRD_SECURITY_ENDPOINT after resolver miss");
                    return Self::new(endpoint).await;
                }
                if let Ok(endpoint) = SafeEnv::get_required("SECURITY_ENDPOINT") {
                    debug!("Using SECURITY_ENDPOINT after resolver miss");
                    return Self::new(endpoint).await;
                }
                if let Ok(endpoint) = SafeEnv::get_required("SECURITY_PROVIDER_ENDPOINT") {
                    debug!("Using SECURITY_PROVIDER_ENDPOINT after resolver miss");
                    return Self::new(endpoint).await;
                }
                if let Ok(endpoint) = SafeEnv::get_required("BEARDOG_ENDPOINT") {
                    warn!(
                        "BEARDOG_ENDPOINT is deprecated — migrate to SECURITY_ENDPOINT or SECURITY_PROVIDER_ENDPOINT"
                    );
                    return Self::new(endpoint).await;
                }

                // Fallback 2: Construct from host + port
                let endpoint = SafeEnv::get_or_default(
                    "SONGBIRD_HOST",
                    format!(
                        "http://{}",
                        &songbird_config::canonical::constants::get_bind_address()
                    ),
                );
                let port = SafeEnv::get_port("SONGBIRD_SECURITY_PORT", 8081).to_string();
                let discovered_endpoint = format!("{endpoint}:{port}");

                debug!("🔄 Using fallback security endpoint: {}", discovered_endpoint);
                Self::new(discovered_endpoint).await
            }
        }
    }

    /// Create adapter with explicit endpoint (for testing or explicit configuration)
    ///
    /// **Protocol Detection** (v3.12.0 - tarpc PRIMARY):
    /// - `tarpc://` → tarpc binary RPC (PRIMARY - 10-100x faster, type-safe)
    /// - `unix://` → JSON-RPC 2.0 over Unix socket (SECONDARY - port-free, universal)
    /// - `http://` or `https://` → HTTP protocol (FALLBACK - network only)
    /// - Automatic detection, zero configuration needed
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Base URL of any security capability provider
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # tokio_test::block_on(async {
    /// use songbird_universal::adapters::SecurityAdapter;
    ///
    /// // tarpc (PRIMARY - fastest)
    /// let adapter1 = SecurityAdapter::new("tarpc://localhost:9001".to_string())?;
    ///
    /// // Unix socket (SECONDARY - port-free)
    /// let adapter2 = SecurityAdapter::new("unix:///tmp/biomeos/security.sock".to_string())?;
    ///
    /// // HTTP endpoint (FALLBACK)
    /// let adapter3 = SecurityAdapter::new("http://localhost:9000".to_string())?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the client cannot be created.
    #[expect(clippy::unused_async, reason = "unused bindings/imports in this compilation unit")] // async retained for API stability
    pub async fn new(endpoint: String) -> SongbirdResult<Self> {
        let transport_kind = transport_kind_for_endpoint(&endpoint);
        if endpoint.starts_with("tarpc://") {
            debug!("🚀 Protocol detected: tarpc (PRIMARY - high-performance binary RPC)");
        } else if endpoint.starts_with("unix://") {
            debug!("🔌 Protocol detected: JSON-RPC 2.0 over Unix socket (SECONDARY - port-free)");
        } else {
            debug!("🌐 Protocol detected: HTTP (FALLBACK - direct connection)");
        }

        let transport = build_default_transport(&endpoint)?;

        debug!("✅ SecurityAdapter initialized: endpoint={}", endpoint);

        Ok(Self {
            endpoint,
            transport,
            transport_kind,
            timeout: Duration::from_secs(5),
        })
    }

    /// Construct with an explicit transport (crate tests and advanced injection).
    #[cfg(test)]
    pub(crate) fn with_transport(
        endpoint: String,
        transport: Arc<dyn CapabilityTransport>,
        transport_kind: AdapterTransportKind,
        timeout: Duration,
    ) -> Self {
        Self {
            endpoint,
            transport,
            transport_kind,
            timeout,
        }
    }

    /// Set custom request timeout
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Collect security metrics from the capability provider
    ///
    /// **Protocol Agnostic** (v3.10.4):
    /// - Automatically uses HTTP or JSON-RPC based on endpoint
    /// - Zero configuration, seamless protocol switching
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network request fails
    /// - Service returns non-success status
    /// - Response cannot be parsed
    pub async fn collect_metrics(&self) -> SongbirdResult<SecurityMetrics> {
        debug!("Collecting security metrics from: {}", self.endpoint);

        let result = tokio::time::timeout(self.timeout, self.transport.get("metrics/security"))
            .await
            .map_err(|_| {
                SongbirdError::network(format!(
                    "Timeout after {:?} reaching security provider",
                    self.timeout
                ))
            })?
            .map_err(|e| {
                warn!("Failed to collect security metrics: {e}");
                match self.transport_kind {
                    AdapterTransportKind::Tarpc => SongbirdError::network(format!(
                        "Failed to reach security provider via tarpc: {e}"
                    )),
                    AdapterTransportKind::JsonRpc => {
                        SongbirdError::network(format!("Failed to reach security provider: {e}"))
                    }
                    AdapterTransportKind::Http => e,
                }
            })?;

        let mut metrics: SecurityMetrics = serde_json::from_value(result).map_err(|e| {
            warn!("Failed to parse security metrics: {e}");
            SongbirdError::security(format!("Failed to parse security metrics: {e}"))
        })?;

        // Set timestamp if not provided
        if metrics.timestamp.timestamp() == 0 {
            metrics.timestamp = chrono::Utc::now();
        }

        debug!(
            "Collected security metrics: Sessions={}, FailedAuth={}, BlockedIPs={}, Score={}",
            metrics.active_sessions,
            metrics.failed_auth_attempts,
            metrics.blocked_ips,
            metrics.security_score
        );

        Ok(metrics)
    }

    /// Verify authentication token
    ///
    /// **Protocol Agnostic** (v3.12.0 - tarpc PRIMARY):
    /// - Automatically uses tarpc, JSON-RPC, or HTTP based on endpoint
    ///
    /// # Errors
    ///
    /// Returns an error if the verification request fails
    pub async fn verify_auth(&self, token: &str) -> SongbirdResult<AuthResult> {
        debug!("Verifying authentication token");

        let rpc_result = tokio::time::timeout(
            self.timeout,
            self.transport.post("auth/verify", serde_json::json!({ "token": token })),
        )
        .await
        .map_err(|_| {
            SongbirdError::network(format!("Timeout verifying auth after {:?}", self.timeout))
        })?
        .map_err(|e| {
            warn!("Auth verification failed: {e}");
            match self.transport_kind {
                AdapterTransportKind::Tarpc => {
                    SongbirdError::network(format!("Auth verification failed via tarpc: {e}"))
                }
                AdapterTransportKind::JsonRpc | AdapterTransportKind::Http => {
                    SongbirdError::network(format!("Auth verification failed: {e}"))
                }
            }
        })?;

        let result: AuthResult = serde_json::from_value(rpc_result).map_err(|e| {
            warn!("Failed to parse auth result: {e}");
            SongbirdError::security(format!("Failed to parse auth result: {e}"))
        })?;

        debug!("Auth verification result: {:?}", result);

        Ok(result)
    }

    /// Check security health
    ///
    /// # Errors
    ///
    /// Returns an error if the health check fails
    pub async fn check_health(&self) -> SongbirdResult<SecurityHealth> {
        let metrics = self.collect_metrics().await?;
        Ok(metrics.health_status())
    }

    /// Get the endpoint URL
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Generic method for calling security provider endpoints (v3.16.0)
    ///
    /// **Modern Idiomatic Rust**: Protocol-agnostic, zero hardcoding
    ///
    /// This method enables BTSP integration and future extensibility.
    /// Works with ANY security provider via automatic protocol negotiation.
    ///
    /// **Protocol Hierarchy**:
    /// - tarpc (PRIMARY): 10-100μs - High-performance binary RPC
    /// - JSON-RPC (SECONDARY): 50-100μs - Port-free, complementary
    /// - HTTP (FALLBACK): 500-1000μs - Network compatibility
    ///
    /// # Arguments
    ///
    /// * `method` - The RPC method name (e.g., "btsp/contact/exchange")
    /// * `params` - The parameters as a JSON value
    ///
    /// # Returns
    ///
    /// * `SongbirdResult<serde_json::Value>` - The response from the security provider
    ///
    /// # Errors
    ///
    /// Returns error if the call fails, times out, or the response cannot be parsed.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use songbird_universal::adapters::SecurityAdapter;
    /// use serde_json::json;
    ///
    /// # async fn example() -> songbird_types::SongbirdResult<()> {
    /// let adapter = SecurityAdapter::new("tarpc://localhost:8765".to_string())?;
    ///
    /// let params = json!({
    ///     "target_peer_id": "tower-b",
    ///     "max_hops": 3
    /// });
    ///
    /// let response = adapter.call_generic("btsp/contact/exchange", params).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn call_generic(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> SongbirdResult<serde_json::Value> {
        debug!("📡 Generic call to security provider: method={}", method);

        let response =
            tokio::time::timeout(self.timeout, self.transport.call_method(method, Some(params)))
                .await
                .map_err(|_| SongbirdError::network(format!("Timeout calling method '{method}'")))?
                .map_err(|e| {
                    if self.transport_kind == AdapterTransportKind::Http {
                        SongbirdError::network(format!("HTTP request failed for '{method}': {e}"))
                    } else {
                        e
                    }
                })?;

        if self.transport_kind == AdapterTransportKind::Http {
            serde_json::from_value(response).map_err(|e| {
                SongbirdError::serialization(format!(
                    "Failed to parse response for '{method}': {e}"
                ))
            })
        } else {
            Ok(response)
        }
    }

    /// Evaluate peer trust (protocol-agnostic) - v3.12.3
    ///
    /// **Protocol Agnostic**: Automatically uses tarpc/JSON-RPC/HTTP based on endpoint
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network request fails
    /// - Service returns non-success status
    /// - Response cannot be parsed
    pub async fn evaluate_trust(
        &self,
        request: &crate::trust_types::TrustEvaluationRequest,
    ) -> SongbirdResult<crate::trust_types::TrustEvaluationResponse> {
        debug!("Evaluating trust for peer: {}", request.peer_id);

        let result = self
            .transport
            .post(
                "api/v1/trust/evaluate",
                serde_json::to_value(request)
                    .map_err(|e| SongbirdError::serialization(e.to_string()))?,
            )
            .await
            .map_err(|e| {
                warn!("Trust evaluation request failed: {e}");
                match self.transport_kind {
                    AdapterTransportKind::Tarpc => {
                        SongbirdError::network(format!("Failed to evaluate trust via tarpc: {e}"))
                    }
                    AdapterTransportKind::JsonRpc => SongbirdError::network(format!(
                        "Failed to evaluate trust via JSON-RPC: {e}"
                    )),
                    AdapterTransportKind::Http => {
                        SongbirdError::network(format!("Failed to reach security provider: {e}"))
                    }
                }
            })?;

        serde_json::from_value(result).map_err(|e| {
            warn!("Failed to parse trust evaluation response: {e}");
            SongbirdError::security(format!("Failed to parse trust evaluation response: {e}"))
        })
    }

    /// Get identity from security provider (protocol-agnostic) - v3.12.3
    ///
    /// **Protocol Agnostic**: Automatically uses tarpc/JSON-RPC/HTTP based on endpoint
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network request fails
    /// - Service returns non-success status
    /// - Response cannot be parsed
    pub async fn get_identity(&self) -> SongbirdResult<crate::trust_types::IdentityResponse> {
        debug!("Getting identity from security provider: {}", self.endpoint);

        let result = self.transport.get("api/v1/identity").await.map_err(|e| {
            warn!("Identity request failed: {e}");
            match self.transport_kind {
                AdapterTransportKind::Tarpc => {
                    SongbirdError::network(format!("Failed to get identity via tarpc: {e}"))
                }
                AdapterTransportKind::JsonRpc => {
                    SongbirdError::network(format!("Failed to get identity via JSON-RPC: {e}"))
                }
                AdapterTransportKind::Http => {
                    SongbirdError::network(format!("Failed to reach security provider: {e}"))
                }
            }
        })?;

        serde_json::from_value(result).map_err(|e| {
            warn!("Failed to parse identity response: {e}");
            SongbirdError::security(format!("Failed to parse identity response: {e}"))
        })
    }
}

/// Trait for security capability providers
pub trait SecurityProvider: Send + Sync {
    /// Collect current security metrics
    async fn collect_security_metrics(&self) -> SongbirdResult<SecurityMetrics>;

    /// Verify authentication
    async fn verify_authentication(&self, token: &str) -> SongbirdResult<AuthResult>;

    /// Check security health
    async fn check_security_health(&self) -> SongbirdResult<SecurityHealth> {
        let metrics = self.collect_security_metrics().await?;
        Ok(metrics.health_status())
    }
}

impl SecurityProvider for SecurityAdapter {
    async fn collect_security_metrics(&self) -> SongbirdResult<SecurityMetrics> {
        self.collect_metrics().await
    }

    async fn verify_authentication(&self, token: &str) -> SongbirdResult<AuthResult> {
        self.verify_auth(token).await
    }
}
