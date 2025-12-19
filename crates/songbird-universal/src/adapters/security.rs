//! Security Capability Adapter
//!
//! **SOVEREIGNTY**: This adapter works with ANY security capability provider.
//! It does NOT know about specific primals (`BearDog` is just one example).
//! Discovery is capability-based through environment hints or zero-knowledge bootstrap.

// Allow async_fn_in_trait warning - our traits guarantee Send + Sync
#![allow(async_fn_in_trait)]

use serde::{Deserialize, Serialize};
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
    pub fn is_under_attack(&self) -> bool {
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

/// **CAPABILITY-BASED SECURITY ADAPTER**
///
/// Works with ANY security provider discovered through:
/// - Environment variable: `SONGBIRD_SECURITY_ENDPOINT`
/// - Capability discovery: `capability:security`
/// - Zero-knowledge bootstrap
pub struct SecurityAdapter {
    /// Endpoint URL for the security capability provider
    endpoint: String,
    /// HTTP client for requests
    client: reqwest::Client,
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
    /// ```no_run
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
                Self::new(endpoint)
            }
            Err(discovery_err) => {
                debug!("🔍 Primary discovery failed, trying legacy fallbacks: {}", discovery_err);

                // Fallback 1: Legacy environment variables
                if let Ok(endpoint) = SafeEnv::get_required("SONGBIRD_SECURITY_ENDPOINT")
                    .or_else(|_| SafeEnv::get_required("SECURITY_PROVIDER_ENDPOINT"))
                    .or_else(|_| SafeEnv::get_required("BEARDOG_ENDPOINT"))
                {
                    debug!("⚠️ Using legacy environment variable for security endpoint");
                    return Self::new(endpoint);
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
                Self::new(discovered_endpoint)
            }
        }
    }

    /// Create adapter with explicit endpoint (for testing or explicit configuration)
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Base URL of any security capability provider
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(endpoint: String) -> SongbirdResult<Self> {
        Ok(Self {
            endpoint,
            client: reqwest::Client::builder().timeout(Duration::from_secs(10)).build().map_err(
                |e| SongbirdError::configuration(format!("Failed to create HTTP client: {e}")),
            )?,
            timeout: Duration::from_secs(5),
        })
    }

    /// Set custom request timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Collect security metrics from the capability provider
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network request fails
    /// - Service returns non-success status
    /// - Response cannot be parsed
    pub async fn collect_metrics(&self) -> SongbirdResult<SecurityMetrics> {
        let url = format!("{}/metrics/security", self.endpoint);

        debug!("Collecting security metrics from: {}", url);

        let response = self.client.get(&url).timeout(self.timeout).send().await.map_err(|e| {
            warn!("Failed to reach security capability provider: {e}");
            songbird_types::SongbirdError::network(format!(
                "Failed to reach security provider: {e}"
            ))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            warn!("Security capability provider returned error status: {}", status);
            return Err(songbird_types::SongbirdError::security(format!(
                "HTTP {status}: Security metrics unavailable"
            )));
        }

        let mut metrics: SecurityMetrics = response.json().await.map_err(|e| {
            warn!("Failed to parse security metrics: {e}");
            songbird_types::SongbirdError::security(format!(
                "Failed to parse security metrics: {e}"
            ))
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
    /// # Errors
    ///
    /// Returns an error if the verification request fails
    pub async fn verify_auth(&self, token: &str) -> SongbirdResult<AuthResult> {
        let url = format!("{}/auth/verify", self.endpoint);

        debug!("Verifying authentication token");

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "token": token }))
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| {
                warn!("Auth verification request failed: {e}");
                songbird_types::SongbirdError::network(format!("Auth verification failed: {e}"))
            })?;

        if !response.status().is_success() {
            return Ok(AuthResult::Unauthorized);
        }

        let result: AuthResult = response.json().await.map_err(|e| {
            warn!("Failed to parse auth result: {e}");
            songbird_types::SongbirdError::security(format!("Failed to parse auth result: {e}"))
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
