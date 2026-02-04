//! `BearDog` Security Adapter
//!
//! **LEGACY EXAMPLE**: This example uses `reqwest` for demonstration purposes.
//! For TRUE Pure Rust production code, use `IpcHttpClient` instead (no C dependencies).
//!
//! See modern implementation:
//! - `crates/songbird-http-client/examples/ipc_http_client_demo.rs`
//! - Migration guide: `ecoPrimals/sessions/feb-2026/reqwest-removal/`
//!
//! ---
//!
//! Coordinates with `BearDog` primal for security, authentication, and deployment operations.
//! This adapter is capability-based and works with any service providing security
//! capabilities in the expected format.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
use std::time::Duration;
use tracing::{debug, warn};

/// Security metrics from `BearDog` or any security primal
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

/// Authentication result from security primal
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

/// Adapter for `BearDog` security coordination
pub struct BearDogSecurityAdapter {
    /// Endpoint URL for the security service
    endpoint: String,
    /// HTTP client for requests
    client: reqwest::Client,
    /// Request timeout
    timeout: Duration,
}

impl BearDogSecurityAdapter {
    /// Create a new `BearDog` security adapter with default endpoint from configuration
    ///
    /// Uses environment variables for endpoint configuration:
    /// - `BEARDOG_ENDPOINT` - Direct endpoint override
    /// - `PRIMAL_BEARDOG_ENDPOINT` - Alternative format
    /// - Falls back to `$SONGBIRD_HOST:$BEARDOG_PORT`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use songbird_universal::adapters::BearDogSecurityAdapter;
    ///
    /// // Uses environment-configured endpoint
    /// let adapter = BearDogSecurityAdapter::new_default().unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new_default() -> SongbirdResult<Self> {
        let endpoint = songbird_config::endpoints::get_primal_endpoint("beardog");
        Self::new(endpoint)
    }

    /// Create a new `BearDog` security adapter with custom endpoint
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Base URL of the security service
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use songbird_universal::adapters::BearDogSecurityAdapter;
    ///
    /// // Custom endpoint
    /// let adapter = BearDogSecurityAdapter::new(format!("http://security-service:{}", 
    ///     songbird_config::defaults::ports::discovery_port())).unwrap();
    ///
    /// // Or use default from configuration
    /// let adapter = BearDogSecurityAdapter::new_default().unwrap();
    /// ```
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

    /// Collect security metrics from the service
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
            warn!("Failed to reach security service: {e}");
            songbird_types::SongbirdError::network(format!("Failed to reach security service: {e}"))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            warn!("Security service returned error status: {}", status);
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

/// Trait for security coordination
#[async_trait]
pub trait SecurityProvider {
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

#[async_trait]
impl SecurityProvider for BearDogSecurityAdapter {
    async fn collect_security_metrics(&self) -> SongbirdResult<SecurityMetrics> {
        self.collect_metrics().await
    }

    async fn verify_authentication(&self, token: &str) -> SongbirdResult<AuthResult> {
        self.verify_auth(token).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_metrics_calculations() {
        let metrics = SecurityMetrics {
            active_sessions: 50,
            failed_auth_attempts: 10,
            blocked_ips: 2,
            security_score: 0.95,
            timestamp: chrono::Utc::now(),
        };

        assert!(!metrics.is_under_attack());
        assert_eq!(metrics.health_status(), SecurityHealth::Healthy);
    }

    #[test]
    fn test_security_under_attack() {
        let metrics = SecurityMetrics {
            active_sessions: 100,
            failed_auth_attempts: 150,
            blocked_ips: 60,
            security_score: 0.45,
            timestamp: chrono::Utc::now(),
        };

        assert!(metrics.is_under_attack());
        assert_eq!(metrics.health_status(), SecurityHealth::Critical);
    }

    #[test]
    fn test_security_warning() {
        let metrics = SecurityMetrics {
            active_sessions: 75,
            failed_auth_attempts: 60,
            blocked_ips: 10,
            security_score: 0.65,
            timestamp: chrono::Utc::now(),
        };

        assert!(!metrics.is_under_attack());
        assert_eq!(metrics.health_status(), SecurityHealth::Warning);
    }

    #[test]
    fn test_adapter_creation() {
        // Test uses localhost - acceptable for unit tests
        let port = songbird_config::defaults::ports::discovery_port();
        let endpoint = format!("http://localhost:{}", port);
        let adapter = BearDogSecurityAdapter::new(endpoint.clone())
            .expect("Test: adapter creation should succeed");
        assert_eq!(
            adapter.endpoint(), // Test uses localhost - acceptable for unit tests
            &endpoint
        );
    }

    #[test]
    fn test_adapter_with_timeout() {
        let adapter = BearDogSecurityAdapter::new(
            // Test uses localhost - acceptable for unit tests
            format!("http://localhost:{}", songbird_config::defaults::ports::discovery_port()),
        )
        .expect("Test: adapter creation should succeed")
        .with_timeout(Duration::from_secs(10));
        assert_eq!(adapter.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_auth_result_equality() {
        assert_eq!(AuthResult::Authorized, AuthResult::Authorized);
        assert_ne!(AuthResult::Authorized, AuthResult::Unauthorized);
        assert_eq!(AuthResult::Expired, AuthResult::Expired);
    }
}
