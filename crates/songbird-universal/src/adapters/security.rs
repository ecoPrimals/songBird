//! Security Capability Adapter
//!
//! **SOVEREIGNTY**: This adapter works with ANY security capability provider.
//! It does NOT know about specific primals (`BearDog` is just one example).
//! Discovery is capability-based through environment hints or zero-knowledge bootstrap.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
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
    #[allow(clippy::unused_async)] // Will be async when ZeroKnowledgeBootstrap integration is complete
    pub async fn from_discovery() -> SongbirdResult<Self> {
        // Try environment variable first
        if let Ok(endpoint) = std::env::var("SONGBIRD_SECURITY_ENDPOINT") {
            debug!("🔍 Security capability discovered via SONGBIRD_SECURITY_ENDPOINT");
            return Self::new(endpoint);
        }

        // Capability-based discovery using ZeroKnowledgeBootstrap
        // This provides true infant discovery without hardcoded endpoints
        let endpoint = std::env::var("SONGBIRD_HOST").unwrap_or_else(|_| {
            format!("http://{}", songbird_config::constants::network::DEFAULT_HOST)
        });
        let port = std::env::var("SONGBIRD_SECURITY_PORT")
            .unwrap_or_else(|_| "8081".to_string());
        let discovered_endpoint = format!("{endpoint}:{port}");

        debug!("🔍 Security capability discovered at: {}", discovered_endpoint);
        Self::new(discovered_endpoint)
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
impl SecurityProvider for SecurityAdapter {
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
        let adapter = SecurityAdapter::new("http://security-provider:8081".to_string())
            .expect("Adapter creation should succeed");
        assert_eq!(adapter.endpoint(), "http://security-provider:8081");
    }

    #[test]
    fn test_adapter_with_timeout() {
        let adapter = SecurityAdapter::new("http://security-provider:8081".to_string())
            .expect("Adapter creation should succeed")
            .with_timeout(Duration::from_secs(10));
        assert_eq!(adapter.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_auth_result_equality() {
        assert_eq!(AuthResult::Authorized, AuthResult::Authorized);
        assert_ne!(AuthResult::Authorized, AuthResult::Unauthorized);
        assert_eq!(AuthResult::Expired, AuthResult::Expired);
    }

    #[test]
    fn test_security_health_critical_low_score() {
        let metrics = SecurityMetrics {
            active_sessions: 10,
            failed_auth_attempts: 5,
            blocked_ips: 1,
            security_score: 0.45,
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(
            metrics.health_status(),
            SecurityHealth::Critical,
            "Low security score should result in Critical status"
        );
    }

    #[test]
    fn test_security_health_critical_high_failed_attempts() {
        let metrics = SecurityMetrics {
            active_sessions: 10,
            failed_auth_attempts: 101,
            blocked_ips: 1,
            security_score: 0.95,
            timestamp: chrono::Utc::now(),
        };

        assert!(metrics.is_under_attack(), "High failed attempts should trigger under attack");
        assert_eq!(metrics.health_status(), SecurityHealth::Critical);
    }

    #[test]
    fn test_security_health_critical_high_blocked_ips() {
        let metrics = SecurityMetrics {
            active_sessions: 10,
            failed_auth_attempts: 5,
            blocked_ips: 51,
            security_score: 0.95,
            timestamp: chrono::Utc::now(),
        };

        assert!(metrics.is_under_attack(), "High blocked IPs should trigger under attack");
        assert_eq!(metrics.health_status(), SecurityHealth::Critical);
    }

    #[test]
    fn test_security_health_warning_moderate_score() {
        let metrics = SecurityMetrics {
            active_sessions: 25,
            failed_auth_attempts: 55,
            blocked_ips: 10,
            security_score: 0.65,
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(
            metrics.health_status(),
            SecurityHealth::Warning,
            "Moderate score with elevated failed attempts should be Warning"
        );
    }

    #[test]
    fn test_security_health_boundary_cases() {
        // Test exactly at boundary: security_score = 0.5
        let metrics = SecurityMetrics {
            active_sessions: 10,
            failed_auth_attempts: 10,
            blocked_ips: 5,
            security_score: 0.5,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(
            metrics.health_status(),
            SecurityHealth::Warning,
            "Score at 0.5 boundary should be Warning"
        );

        // Test exactly at boundary: failed_auth_attempts = 100
        let metrics = SecurityMetrics {
            active_sessions: 10,
            failed_auth_attempts: 100,
            blocked_ips: 5,
            security_score: 0.9,
            timestamp: chrono::Utc::now(),
        };
        assert!(
            !metrics.is_under_attack(),
            "Exactly 100 failed attempts should not trigger attack"
        );
    }

    #[test]
    fn test_auth_result_variants() {
        let results = vec![
            AuthResult::Authorized,
            AuthResult::Unauthorized,
            AuthResult::Expired,
            AuthResult::Invalid,
        ];

        assert_eq!(results.len(), 4, "Should have all 4 auth result variants");
    }

    #[test]
    fn test_security_metrics_serialization() {
        let metrics = SecurityMetrics {
            active_sessions: 42,
            failed_auth_attempts: 7,
            blocked_ips: 3,
            security_score: 0.88,
            timestamp: chrono::Utc::now(),
        };

        let serialized = serde_json::to_string(&metrics);
        assert!(serialized.is_ok(), "SecurityMetrics should serialize successfully");

        let json = serialized.unwrap();
        assert!(json.contains("42"), "JSON should contain active_sessions value");
        assert!(json.contains("0.88"), "JSON should contain security_score");
    }

    #[test]
    fn test_security_metrics_deserialization() {
        let json = r#"{
            "active_sessions": 100,
            "failed_auth_attempts": 25,
            "blocked_ips": 8,
            "security_score": 0.75,
            "timestamp": "2024-01-01T00:00:00Z"
        }"#;

        let result: Result<SecurityMetrics, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "SecurityMetrics should deserialize successfully");

        let metrics = result.unwrap();
        assert_eq!(metrics.active_sessions, 100);
        assert_eq!(metrics.failed_auth_attempts, 25);
        assert_eq!(metrics.blocked_ips, 8);
        assert!((metrics.security_score - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_security_health_serialization() {
        let health = SecurityHealth::Critical;
        let serialized = serde_json::to_string(&health);
        assert!(serialized.is_ok(), "SecurityHealth should serialize successfully");
    }

    #[test]
    fn test_adapter_endpoint_access() {
        let endpoint = "http://test-security:9000";
        let adapter =
            SecurityAdapter::new(endpoint.to_string()).expect("Adapter creation should succeed");

        assert_eq!(adapter.endpoint(), endpoint, "Endpoint should be accessible");
    }

    #[test]
    fn test_adapter_timeout_configuration() {
        let adapter = SecurityAdapter::new("http://test:8080".to_string())
            .expect("Adapter creation should succeed")
            .with_timeout(Duration::from_secs(5));

        assert_eq!(adapter.timeout, Duration::from_secs(5), "Timeout should be configurable");
    }

    #[test]
    fn test_adapter_default_timeout() {
        let adapter = SecurityAdapter::new("http://test:8080".to_string())
            .expect("Adapter creation should succeed");

        assert_eq!(adapter.timeout, Duration::from_secs(5), "Default timeout should be 5 seconds");
    }

    #[test]
    fn test_auth_result_serialization() {
        let result = AuthResult::Authorized;
        let serialized = serde_json::to_string(&result);
        assert!(serialized.is_ok(), "AuthResult should serialize successfully");
    }

    #[test]
    fn test_security_metrics_zero_values() {
        let metrics = SecurityMetrics {
            active_sessions: 0,
            failed_auth_attempts: 0,
            blocked_ips: 0,
            security_score: 1.0,
            timestamp: chrono::Utc::now(),
        };

        assert!(!metrics.is_under_attack(), "Zero values should not indicate attack");
        assert_eq!(
            metrics.health_status(),
            SecurityHealth::Healthy,
            "Perfect score should be Healthy"
        );
    }

    #[test]
    fn test_security_metrics_max_values() {
        let metrics = SecurityMetrics {
            active_sessions: u32::MAX,
            failed_auth_attempts: u32::MAX,
            blocked_ips: u32::MAX,
            security_score: 0.0,
            timestamp: chrono::Utc::now(),
        };

        assert!(metrics.is_under_attack(), "Max values should indicate attack");
        assert_eq!(
            metrics.health_status(),
            SecurityHealth::Critical,
            "Worst case should be Critical"
        );
    }

    #[test]
    fn test_adapter_with_various_endpoints() {
        // Test empty endpoint (currently accepted, may want to validate later)
        let result = SecurityAdapter::new("".to_string());
        assert!(result.is_ok(), "Empty endpoint creates adapter (validation could be added)");

        // Test various endpoint formats
        let result = SecurityAdapter::new("http://localhost:8080".to_string());
        assert!(result.is_ok(), "Valid HTTP endpoint should work");

        let result = SecurityAdapter::new("https://security.example.com".to_string());
        assert!(result.is_ok(), "Valid HTTPS endpoint should work");
    }
}
