// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Health Checking for Discovered Primals
//!
//! Performs health checks on discovered primals.
//! Part of the smart refactoring of discovery.rs

use super::types::{DiscoveredPrimal, DiscoveryError, PrimalHealth};
use tokio::time::{Duration, timeout};
use tracing::{debug, warn};

/// Health check paths to try
const HEALTH_PATHS: &[&str] = &["/health", "/api/v1/health", "/api/health", "/status"];

/// Health checker for discovered primals
pub struct HealthChecker {
    /// Timeout for health checks
    timeout: Duration,
    // Note: HTTP client created on-demand to support async initialization
}

impl HealthChecker {
    /// Create a new health checker
    #[must_use]
    pub const fn new(check_timeout: Duration) -> Self {
        Self {
            timeout: check_timeout,
        }
    }

    /// Check health of a discovered primal
    ///
    /// # Errors
    ///
    /// Returns an error if the primal is unreachable or returns invalid responses.
    pub async fn check_health(
        &self,
        primal: &mut DiscoveredPrimal,
    ) -> Result<PrimalHealth, DiscoveryError> {
        let base_url = &primal.endpoint;

        // Create client on-demand
        let client = songbird_http_client::IpcHttpClient::new().await.map_err(|e| {
            DiscoveryError::NetworkError(format!("Failed to create HTTP client: {e}"))
        })?;

        for health_path in HEALTH_PATHS {
            let url = format!("{base_url}{health_path}");

            // Pass the future (not awaited) to timeout, then await the whole thing
            match timeout(self.timeout, client.get(&url)).await {
                Ok(Ok(response)) if response.is_success() => {
                    debug!("✅ Health check passed for {} at {}", primal.name, url);
                    primal.health = PrimalHealth::Healthy;
                    return Ok(PrimalHealth::Healthy);
                }
                Ok(Ok(response)) => {
                    debug!("⚠️ Health check returned {} for {}", response.status(), primal.name);
                    primal.health = PrimalHealth::Degraded;
                    return Ok(PrimalHealth::Degraded);
                }
                Ok(Err(e)) => {
                    debug!("Health check error for {}: {}", primal.name, e);
                }
                Err(_) => {
                    debug!("Health check timeout for {} at {}", primal.name, url);
                }
            }
        }

        warn!("❌ All health checks failed for {}", primal.name);
        primal.health = PrimalHealth::Unhealthy;

        Err(DiscoveryError::HealthCheckFailed {
            primal: primal.name.clone(),
            reason: "All health check paths failed".to_string(),
        })
    }

    /// Batch health check for multiple primals
    pub async fn check_all(&self, primals: &mut [DiscoveredPrimal]) {
        for primal in primals {
            let _ = self.check_health(primal).await;
        }
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new(Duration::from_secs(5))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::Capability;
    use crate::discovery::types::*;
    use crate::types::PrimalType;

    #[tokio::test]
    async fn test_health_checker_creation() {
        let checker = HealthChecker::new(Duration::from_secs(10));
        assert_eq!(checker.timeout, Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_health_check_invalid_endpoint() {
        let checker = HealthChecker::default();
        let mut primal = DiscoveredPrimal {
            name: "test".to_string(),
            primal_type: PrimalType::new("security"),
            endpoint: "http://invalid-endpoint-12345:9999".to_string(),
            capabilities: vec![Capability {
                capability_type: "security".to_string(),
                name: "security".to_string(),
                version: "1.0".to_string(),
                parameters: Default::default(),
                qos_metrics: Default::default(),
                available: true,
            }],
            health: PrimalHealth::Unknown,
            discovery_method: DiscoveryMethod::Manual,
            metadata: Default::default(),
        };

        let result = checker.check_health(&mut primal).await;
        assert!(result.is_err());
        assert_eq!(primal.health, PrimalHealth::Unhealthy);
    }
}
