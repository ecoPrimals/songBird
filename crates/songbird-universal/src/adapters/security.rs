// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Security Capability Adapter
//!
//! **SOVEREIGNTY**: This adapter works with ANY security capability provider.
//! It does NOT know about specific primals (`BearDog` is just one example).
//! Discovery is capability-based through environment hints or zero-knowledge bootstrap.

// Allow async_fn_in_trait warning - our traits guarantee Send + Sync
#![allow(async_fn_in_trait)]

use serde::{Deserialize, Serialize};
use songbird_http_client::SongbirdHttpClient;
use songbird_types::{SafeEnv, SongbirdError, SongbirdResult};
use std::time::Duration;
use tracing::{debug, warn};

/// Security metrics from any security capability provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetrics {
    /// Number of active authenticated sessions
    pub active_sessions: u32,
    /// Number of failed authentication attempts in the last hour
    pub failed_auth_attempts: u32,
    /// Number of currently blocked IPs
    pub blocked_ips: u32,
    /// Security score (0.0 - 1.0)
    pub security_score: f64,
    /// Timestamp of metrics collection
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl SecurityMetrics {
    /// Check if security is under attack
    #[must_use]
    pub const fn is_under_attack(&self) -> bool {
        self.failed_auth_attempts > 100 || self.blocked_ips > 50
    }

    /// Get security health status
    #[must_use]
    pub fn health_status(&self) -> SecurityHealth {
        if self.security_score < 0.5 || self.is_under_attack() {
            SecurityHealth::Critical
        } else if self.security_score < 0.7 || self.failed_auth_attempts > 50 {
            SecurityHealth::Warning
        } else {
            SecurityHealth::Healthy
        }
    }

    /// Create healthy metrics for testing
    #[cfg(test)]
    pub fn healthy() -> Self {
        Self {
            active_sessions: 10,
            failed_auth_attempts: 2,
            blocked_ips: 0,
            security_score: 0.95,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Security health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityHealth {
    /// Security is healthy
    Healthy,
    /// Security warnings present
    Warning,
    /// Security is critical
    Critical,
}

#[cfg(test)]
mod security_types_tests {
    use super::*;

    #[test]
    fn test_security_metrics_is_under_attack_threshold_failed_auth() {
        let metrics = SecurityMetrics {
            active_sessions: 10,
            failed_auth_attempts: 101,
            blocked_ips: 5,
            security_score: 0.8,
            timestamp: chrono::Utc::now(),
        };
        assert!(metrics.is_under_attack());
    }

    #[test]
    fn test_security_metrics_is_under_attack_threshold_blocked_ips() {
        let metrics = SecurityMetrics {
            active_sessions: 10,
            failed_auth_attempts: 10,
            blocked_ips: 51,
            security_score: 0.8,
            timestamp: chrono::Utc::now(),
        };
        assert!(metrics.is_under_attack());
    }

    #[test]
    fn test_security_metrics_not_under_attack() {
        let metrics = SecurityMetrics::healthy();
        assert!(!metrics.is_under_attack());
    }

    #[test]
    fn test_security_metrics_health_status_healthy() {
        let metrics = SecurityMetrics {
            active_sessions: 10,
            failed_auth_attempts: 10,
            blocked_ips: 5,
            security_score: 0.9,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), SecurityHealth::Healthy);
    }

    #[test]
    fn test_security_metrics_health_status_warning() {
        let metrics = SecurityMetrics {
            active_sessions: 10,
            failed_auth_attempts: 60,
            blocked_ips: 10,
            security_score: 0.65,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), SecurityHealth::Warning);
    }

    #[test]
    fn test_security_metrics_health_status_critical_low_score() {
        let metrics = SecurityMetrics {
            active_sessions: 10,
            failed_auth_attempts: 10,
            blocked_ips: 5,
            security_score: 0.4,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), SecurityHealth::Critical);
    }

    #[test]
    fn test_security_metrics_health_status_critical_under_attack() {
        let metrics = SecurityMetrics {
            active_sessions: 10,
            failed_auth_attempts: 150,
            blocked_ips: 80,
            security_score: 0.9,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), SecurityHealth::Critical);
    }

    #[test]
    fn test_security_health_equality() {
        assert_eq!(SecurityHealth::Healthy, SecurityHealth::Healthy);
        assert_ne!(SecurityHealth::Healthy, SecurityHealth::Warning);
        assert_ne!(SecurityHealth::Warning, SecurityHealth::Critical);
    }

    #[test]
    fn test_auth_result_equality() {
        assert_eq!(AuthResult::Authorized, AuthResult::Authorized);
        assert_ne!(AuthResult::Authorized, AuthResult::Unauthorized);
        assert_ne!(AuthResult::Expired, AuthResult::Invalid);
    }

    #[test]
    fn test_security_metrics_serialization() {
        let metrics = SecurityMetrics::healthy();
        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("active_sessions"));
        assert!(json.contains("security_score"));
    }
}

/// Authentication result from security capability provider
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthResult {
    /// Authentication succeeded
    Authorized,
    /// Authentication failed
    Unauthorized,
    /// Token expired
    Expired,
    /// Invalid credentials
    Invalid,
}

/// Protocol used for communication with security provider (v3.12.0 - tarpc PRIMARY)
#[derive(Debug, Clone)]
enum SecurityProtocol {
    /// tarpc - HIGH-PERFORMANCE binary RPC (PRIMARY for primal-to-primal)
    Tarpc(crate::TarpcClient),
    /// JSON-RPC 2.0 over Unix socket (SECONDARY - port-free, universal)
    JsonRpc(crate::JsonRpcClient),
    /// HTTP/HTTPS protocol (FALLBACK - network only)
    Http(SongbirdHttpClient),
}

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
#[derive(Debug, Clone)]
pub struct SecurityAdapter {
    /// Endpoint URL for the security capability provider
    endpoint: String,
    /// Protocol-specific client
    protocol: SecurityProtocol,
    /// Request timeout
    timeout: Duration,
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
    /// // Discovers any security provider (could be BearDog, or anyone)
    /// let adapter = SecurityAdapter::from_discovery().await?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if no security capability can be discovered.
    pub async fn from_discovery() -> SongbirdResult<Self> {
        use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};

        // ✅ PHASE 1 INTEGRATION: Multi-tier capability discovery
        let resolver = CapabilityEndpointResolver::new();

        match resolver.get_endpoint(CapabilityType::Security).await {
            Ok(endpoint) => {
                debug!("✅ Security capability discovered via resolver: {}", endpoint);
                Self::new(endpoint).await
            }
            Err(discovery_err) => {
                debug!("🔍 Primary discovery failed, trying legacy fallbacks: {}", discovery_err);

                // Fallback 1: Legacy environment variables
                if let Ok(endpoint) = SafeEnv::get_required("SONGBIRD_SECURITY_ENDPOINT")
                    .or_else(|_| SafeEnv::get_required("SECURITY_PROVIDER_ENDPOINT"))
                    .or_else(|_| SafeEnv::get_required("BEARDOG_ENDPOINT"))
                {
                    debug!("⚠️ Using legacy environment variable for security endpoint");
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
    /// let adapter2 = SecurityAdapter::new("unix:///tmp/beardog.sock".to_string())?;
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
    #[allow(clippy::unused_async)] // async retained for API stability
    pub async fn new(endpoint: String) -> SongbirdResult<Self> {
        // Detect protocol based on endpoint scheme (v3.12.0 - tarpc PRIMARY)
        let protocol = if endpoint.starts_with("tarpc://") {
            // tarpc - HIGH-PERFORMANCE binary RPC (PRIMARY)
            debug!("🚀 Protocol detected: tarpc (PRIMARY - high-performance binary RPC)");
            let client = crate::TarpcClient::new(&endpoint).map_err(|e| {
                SongbirdError::configuration(format!("Failed to create tarpc client: {e}"))
            })?;
            SecurityProtocol::Tarpc(client)
        } else if endpoint.starts_with("unix://") {
            // JSON-RPC over Unix socket (SECONDARY)
            debug!("🔌 Protocol detected: JSON-RPC 2.0 over Unix socket (SECONDARY - port-free)");
            let client = crate::JsonRpcClient::new(&endpoint).map_err(|e| {
                SongbirdError::configuration(format!("Failed to create JSON-RPC client: {e}"))
            })?;
            SecurityProtocol::JsonRpc(client)
        } else {
            // HTTP/HTTPS protocol (FALLBACK - direct TCP connection)
            debug!("🌐 Protocol detected: HTTP (FALLBACK - direct connection)");
            SecurityProtocol::Http(SongbirdHttpClient::from_env())
        };

        debug!("✅ SecurityAdapter initialized: endpoint={}", endpoint);

        Ok(Self {
            endpoint,
            protocol,
            timeout: Duration::from_secs(5),
        })
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

        let mut metrics: SecurityMetrics = match &self.protocol {
            SecurityProtocol::Tarpc(client) => {
                // tarpc - HIGH-PERFORMANCE binary RPC (PRIMARY - ~10-20 μs latency!)
                debug!("🚀 Using tarpc (PRIMARY protocol)");
                let result =
                    client.call_method("get_security_metrics", None).await.map_err(|e| {
                        warn!("tarpc request failed: {e}");
                        songbird_types::SongbirdError::network(format!(
                            "Failed to reach security provider via tarpc: {e}"
                        ))
                    })?;

                serde_json::from_value(result).map_err(|e| {
                    warn!("Failed to parse security metrics: {e}");
                    songbird_types::SongbirdError::security(format!(
                        "Failed to parse security metrics: {e}"
                    ))
                })?
            }
            SecurityProtocol::JsonRpc(client) => {
                // JSON-RPC protocol (SECONDARY - ~50-100 μs latency)
                debug!("🔌 Using JSON-RPC (SECONDARY protocol)");
                let result = client
                    .call_method("get_metrics", Some(serde_json::json!({"type": "security"})))
                    .await
                    .map_err(|e| {
                        warn!("JSON-RPC request failed: {e}");
                        songbird_types::SongbirdError::network(format!(
                            "Failed to reach security provider: {e}"
                        ))
                    })?;

                serde_json::from_value(result).map_err(|e| {
                    warn!("Failed to parse security metrics: {e}");
                    songbird_types::SongbirdError::security(format!(
                        "Failed to parse security metrics: {e}"
                    ))
                })?
            }
            SecurityProtocol::Http(client) => {
                // HTTP protocol (FALLBACK - direct TCP connection)
                debug!("🌐 Using HTTP (FALLBACK protocol)");
                let url = format!("{}/metrics/security", self.endpoint);

                let response = tokio::time::timeout(self.timeout, client.get(&url))
                    .await
                    .map_err(|_| {
                        songbird_types::SongbirdError::network(format!(
                            "Timeout after {:?} reaching security provider",
                            self.timeout
                        ))
                    })?
                    .map_err(|e| {
                        warn!("Failed to reach security capability provider: {e}");
                        songbird_types::SongbirdError::network(format!(
                            "Failed to reach security provider: {e}"
                        ))
                    })?;

                if !(200..300).contains(&response.status) {
                    let status = response.status;
                    warn!("Security capability provider returned error status: {}", status);
                    return Err(songbird_types::SongbirdError::security(format!(
                        "HTTP {status}: Security metrics unavailable"
                    )));
                }

                serde_json::from_value(response.body).map_err(|e| {
                    warn!("Failed to parse security metrics: {e}");
                    songbird_types::SongbirdError::security(format!(
                        "Failed to parse security metrics: {e}"
                    ))
                })?
            }
        };

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

        let result: AuthResult = match &self.protocol {
            SecurityProtocol::Tarpc(client) => {
                // tarpc - HIGH-PERFORMANCE binary RPC (PRIMARY)
                debug!("🚀 Using tarpc (PRIMARY protocol)");
                let rpc_result = client
                    .call_method("verify_auth", Some(serde_json::json!({ "token": token })))
                    .await
                    .map_err(|e| {
                        warn!("tarpc auth verification failed: {e}");
                        songbird_types::SongbirdError::network(format!(
                            "Auth verification failed via tarpc: {e}"
                        ))
                    })?;

                serde_json::from_value(rpc_result).map_err(|e| {
                    warn!("Failed to parse auth result: {e}");
                    songbird_types::SongbirdError::security(format!(
                        "Failed to parse auth result: {e}"
                    ))
                })?
            }
            SecurityProtocol::JsonRpc(client) => {
                // JSON-RPC protocol (SECONDARY)
                debug!("🔌 Using JSON-RPC (SECONDARY protocol)");
                let rpc_result = client
                    .call_method("verify_auth", Some(serde_json::json!({ "token": token })))
                    .await
                    .map_err(|e| {
                        warn!("JSON-RPC auth verification failed: {e}");
                        songbird_types::SongbirdError::network(format!(
                            "Auth verification failed: {e}"
                        ))
                    })?;

                serde_json::from_value(rpc_result).map_err(|e| {
                    warn!("Failed to parse auth result: {e}");
                    songbird_types::SongbirdError::security(format!(
                        "Failed to parse auth result: {e}"
                    ))
                })?
            }
            SecurityProtocol::Http(client) => {
                // HTTP protocol (FALLBACK - direct TCP connection)
                debug!("🌐 Using HTTP (FALLBACK protocol)");
                let url = format!("{}/auth/verify", self.endpoint);

                let response = client
                    .post(&url, serde_json::json!({ "token": token }))
                    .await
                    .map_err(|e| {
                        warn!("Auth verification request failed: {e}");
                        songbird_types::SongbirdError::network(format!(
                            "Auth verification failed: {e}"
                        ))
                    })?;

                if !(200..300).contains(&response.status) {
                    return Ok(AuthResult::Unauthorized);
                }

                serde_json::from_value(response.body).map_err(|e| {
                    warn!("Failed to parse auth result: {e}");
                    songbird_types::SongbirdError::security(format!(
                        "Failed to parse auth result: {e}"
                    ))
                })?
            }
        };

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

        match &self.protocol {
            SecurityProtocol::Tarpc(client) => {
                debug!("🚀 Using tarpc for {} (PRIMARY - 10-100μs)", method);
                tokio::time::timeout(self.timeout, client.call_method(method, Some(params)))
                    .await
                    .map_err(|_| {
                    SongbirdError::network(format!("Timeout calling method '{method}'"))
                })?
            }
            SecurityProtocol::JsonRpc(client) => {
                debug!("🔌 Using JSON-RPC for {} (SECONDARY - 50-100μs)", method);
                tokio::time::timeout(self.timeout, client.call_method(method, Some(params)))
                    .await
                    .map_err(|_| {
                    SongbirdError::network(format!("Timeout calling method '{method}'"))
                })?
            }
            SecurityProtocol::Http(client) => {
                debug!("🌐 Using HTTP for {} (FALLBACK - 500-1000μs)", method);
                let url = format!("{}/{}", self.endpoint, method);

                let response = tokio::time::timeout(self.timeout, client.post(&url, params))
                    .await
                    .map_err(|_| {
                        SongbirdError::network(format!("Timeout calling method '{method}'"))
                    })?
                    .map_err(|e| {
                        SongbirdError::network(format!("HTTP request failed for '{method}': {e}"))
                    })?;

                if !(200..300).contains(&response.status) {
                    return Err(SongbirdError::network(format!(
                        "Method '{method}' failed: {status}",
                        status = response.status
                    )));
                }

                serde_json::from_value(response.body).map_err(|e| {
                    SongbirdError::serialization(format!(
                        "Failed to parse response for '{method}': {e}"
                    ))
                })
            }
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

        match &self.protocol {
            SecurityProtocol::Tarpc(client) => {
                // tarpc - HIGH-PERFORMANCE binary RPC (PRIMARY - ~10-20 μs latency!)
                debug!("🚀 Using tarpc for trust evaluation (PRIMARY protocol)");
                let result = client
                    .call_method("trust.evaluate_peer", Some(serde_json::to_value(request)?))
                    .await
                    .map_err(|e| {
                        warn!("tarpc trust evaluation failed: {e}");
                        SongbirdError::network(format!("Failed to evaluate trust via tarpc: {e}"))
                    })?;

                serde_json::from_value(result).map_err(|e| {
                    warn!("Failed to parse trust evaluation response: {e}");
                    SongbirdError::security(format!(
                        "Failed to parse trust evaluation response: {e}"
                    ))
                })
            }
            SecurityProtocol::JsonRpc(client) => {
                // JSON-RPC protocol (SECONDARY - ~50-100 μs latency)
                debug!("🔌 Using JSON-RPC for trust evaluation (SECONDARY protocol)");
                let result = client
                    .call_method("trust.evaluate_peer", Some(serde_json::to_value(request)?))
                    .await
                    .map_err(|e| {
                        warn!("JSON-RPC trust evaluation failed: {e}");
                        SongbirdError::network(format!(
                            "Failed to evaluate trust via JSON-RPC: {e}"
                        ))
                    })?;

                serde_json::from_value(result).map_err(|e| {
                    warn!("Failed to parse trust evaluation response: {e}");
                    SongbirdError::security(format!(
                        "Failed to parse trust evaluation response: {e}"
                    ))
                })
            }
            SecurityProtocol::Http(client) => {
                // HTTP protocol (FALLBACK - direct TCP connection)
                debug!("🌐 Using HTTP for trust evaluation (FALLBACK protocol)");
                let url = format!("{}/api/v1/trust/evaluate", self.endpoint);

                let response = client
                    .post(
                        &url,
                        serde_json::to_value(request)
                            .map_err(|e| SongbirdError::serialization(e.to_string()))?,
                    )
                    .await
                    .map_err(|e| {
                        warn!("Failed to reach security provider for trust evaluation: {e}");
                        SongbirdError::network(format!("Failed to reach security provider: {e}"))
                    })?;

                if !(200..300).contains(&response.status) {
                    let status = response.status;
                    warn!(
                        "Security provider returned error for trust evaluation: {} - {:?}",
                        status, response.body
                    );
                    return Err(SongbirdError::security(format!(
                        "Trust evaluation failed: {} - {}",
                        status, response.body
                    )));
                }

                serde_json::from_value(response.body).map_err(|e| {
                    warn!("Failed to parse trust evaluation response: {e}");
                    SongbirdError::security(format!(
                        "Failed to parse trust evaluation response: {e}"
                    ))
                })
            }
        }
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

        match &self.protocol {
            SecurityProtocol::Tarpc(client) => {
                // tarpc - HIGH-PERFORMANCE binary RPC (PRIMARY)
                debug!("🚀 Using tarpc for identity (PRIMARY protocol)");
                let result = client.call_method("identity", None).await.map_err(|e| {
                    warn!("tarpc identity request failed: {e}");
                    SongbirdError::network(format!("Failed to get identity via tarpc: {e}"))
                })?;

                serde_json::from_value(result).map_err(|e| {
                    warn!("Failed to parse identity response: {e}");
                    SongbirdError::security(format!("Failed to parse identity response: {e}"))
                })
            }
            SecurityProtocol::JsonRpc(client) => {
                // JSON-RPC protocol (SECONDARY)
                debug!("🔌 Using JSON-RPC for identity (SECONDARY protocol)");
                let result = client.call_method("identity", None).await.map_err(|e| {
                    warn!("JSON-RPC identity request failed: {e}");
                    SongbirdError::network(format!("Failed to get identity via JSON-RPC: {e}"))
                })?;

                serde_json::from_value(result).map_err(|e| {
                    warn!("Failed to parse identity response: {e}");
                    SongbirdError::security(format!("Failed to parse identity response: {e}"))
                })
            }
            SecurityProtocol::Http(client) => {
                // HTTP protocol (FALLBACK - direct TCP connection)
                debug!("🌐 Using HTTP for identity (FALLBACK protocol)");
                let url = format!("{}/api/v1/identity", self.endpoint);

                let response = client.get(&url).await.map_err(|e| {
                    warn!("Failed to reach security provider for identity: {e}");
                    SongbirdError::network(format!("Failed to reach security provider: {e}"))
                })?;

                if !(200..300).contains(&response.status) {
                    let status = response.status;
                    warn!(
                        "Security provider returned error for identity: {} - {:?}",
                        status, response.body
                    );
                    return Err(SongbirdError::security(format!(
                        "Identity request failed: {} - {}",
                        status, response.body
                    )));
                }

                serde_json::from_value(response.body).map_err(|e| {
                    warn!("Failed to parse identity response: {e}");
                    SongbirdError::security(format!("Failed to parse identity response: {e}"))
                })
            }
        }
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

#[cfg(test)]
#[path = "security_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests_protocol_detection.rs"]
mod protocol_detection_tests;

#[cfg(test)]
#[path = "security_trust_tests.rs"]
mod trust_tests;
