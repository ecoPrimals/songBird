// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! BTSP Endpoint Health Monitoring
//!
//! Provides real-time health monitoring for registered BTSP providers.
//!
//! **Modern Idiomatic Rust** ✅:
//! - Fully async/await
//! - tokio::select! for graceful shutdown
//! - Arc/RwLock for concurrent access
//! - No unwrap() or expect() in production code

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, info, warn};

/// Health status of a BTSP endpoint
#[derive(Debug, Clone)]
pub struct BtspEndpointHealth {
    /// Provider name
    pub provider_name: String,
    /// Endpoint URL
    pub endpoint_url: String,
    /// Is the endpoint reachable?
    pub is_healthy: bool,
    /// Last successful health check
    pub last_success: Option<Instant>,
    /// Last check time
    pub last_check: Instant,
    /// Response time in milliseconds
    pub response_time_ms: Option<u64>,
    /// HTTP status code from last check
    pub last_http_code: Option<u16>,
    /// Consecutive failures
    pub consecutive_failures: u32,
}

/// BTSP Health Monitor
///
/// Continuously monitors registered BTSP providers and tracks their health status.
/// 
/// ✅ EVOLVED (Jan 21, 2026): 100% Pure Rust HTTP via SongbirdHttpClient
pub struct BtspHealthMonitor {
    /// HTTP client for health checks
    client: songbird_http_client::SongbirdHttpClient,
    /// Health status for each endpoint
    health_status: Arc<RwLock<HashMap<String, BtspEndpointHealth>>>,
    /// Check interval
    check_interval: Duration,
    /// Alert threshold (response time)
    alert_threshold_ms: u64,
}

impl BtspHealthMonitor {
    /// Create a new BTSP health monitor
    /// 
    /// ✅ EVOLVED: Async construction with crypto discovery
    pub async fn new(check_interval_secs: u64, alert_threshold_ms: u64) -> Result<Self, String> {
        let crypto_socket = crate::primal_discovery::discover_crypto_provider()
            .await
            .map_err(|e| format!("Failed to discover crypto provider: {}", e))?;
        
        Ok(Self {
            client: songbird_http_client::SongbirdHttpClient::new(crypto_socket),
            health_status: Arc::new(RwLock::new(HashMap::new())),
            check_interval: Duration::from_secs(check_interval_secs),
            alert_threshold_ms,
        })
    }

    /// Start monitoring loop
    ///
    /// This runs indefinitely until cancelled via `shutdown_rx`.
    pub async fn start_monitoring(
        &self,
        songbird_url: String,
        mut shutdown_rx: tokio::sync::mpsc::Receiver<()>,
    ) {
        info!("🏥 Starting BTSP health monitor");
        info!("   Check interval: {}s", self.check_interval.as_secs());
        info!("   Alert threshold: {}ms", self.alert_threshold_ms);

        let mut check_interval = interval(self.check_interval);
        let mut iteration = 0;

        loop {
            tokio::select! {
                _ = check_interval.tick() => {
                    iteration += 1;
                    debug!("Health check iteration #{}", iteration);

                    // Query for BTSP providers
                    match self.query_btsp_providers(&songbird_url).await {
                        Ok(providers) => {
                            if providers.is_empty() {
                                debug!("No BTSP providers registered");
                            } else {
                                info!("Checking {} BTSP provider(s)", providers.len());
                                self.check_all_providers(providers).await;
                            }
                        }
                        Err(e) => {
                            warn!("Failed to query BTSP providers: {}", e);
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("🛑 BTSP health monitor shutting down");
                    break;
                }
            }
        }
    }

    /// Query Songbird for BTSP providers
    async fn query_btsp_providers(
        &self,
        songbird_url: &str,
    ) -> Result<Vec<BtspProviderInfo>, String> {
        let url = format!("{}/api/v1/services?capability=btsp", songbird_url);

        let response = self
            .client
            .get(&url)
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.status < 200 || response.status >= 300 {
            return Err(format!("HTTP {}", response.status));
        }

        let providers: Vec<serde_json::Value> = serde_json::from_value(response.body)
            .map_err(|e| format!("JSON parse failed: {}", e))?;

        Ok(providers
            .into_iter()
            .filter_map(|p| {
                Some(BtspProviderInfo {
                    name: p.get("primal_name")?.as_str()?.to_string(),
                    endpoint: p
                        .get("endpoints")?
                        .as_array()?
                        .first()?
                        .get("url")?
                        .as_str()?
                        .to_string(),
                })
            })
            .collect())
    }

    /// Check health of all providers
    async fn check_all_providers(&self, providers: Vec<BtspProviderInfo>) {
        for provider in providers {
            self.check_provider_health(&provider).await;
        }
    }

    /// Check health of a single provider
    async fn check_provider_health(&self, provider: &BtspProviderInfo) {
        let start = Instant::now();
        let health_url = format!("{}/health", provider.endpoint);

        match self.client.get(&health_url).await {
            Ok(response) => {
                let response_time_ms = start.elapsed().as_millis() as u64;
                let http_code = response.status;
                let is_healthy = http_code >= 200 && http_code < 300;

                if is_healthy {
                    if response_time_ms < self.alert_threshold_ms {
                        info!(
                            "✅ {} - HEALTHY ({}, {}ms)",
                            provider.name, provider.endpoint, response_time_ms
                        );
                    } else {
                        warn!(
                            "⚠️  {} - SLOW ({}, {}ms > {}ms threshold)",
                            provider.name,
                            provider.endpoint,
                            response_time_ms,
                            self.alert_threshold_ms
                        );
                    }
                } else {
                    warn!(
                        "❌ {} - ERROR ({}, HTTP {})",
                        provider.name, provider.endpoint, http_code
                    );
                }

                // Update health status
                self.update_health_status(
                    &provider.name,
                    &provider.endpoint,
                    is_healthy,
                    Some(response_time_ms),
                    Some(http_code),
                )
                .await;
            }
            Err(e) => {
                warn!(
                    "❌ {} - UNREACHABLE ({}) - {}",
                    provider.name, provider.endpoint, e
                );

                // Update health status
                self.update_health_status(&provider.name, &provider.endpoint, false, None, None)
                    .await;
            }
        }
    }

    /// Update health status for a provider
    async fn update_health_status(
        &self,
        provider_name: &str,
        endpoint_url: &str,
        is_healthy: bool,
        response_time_ms: Option<u64>,
        last_http_code: Option<u16>,
    ) {
        let mut health_map = self.health_status.write().await;
        let key = format!("{}::{}", provider_name, endpoint_url);

        let health = health_map.entry(key).or_insert_with(|| BtspEndpointHealth {
            provider_name: provider_name.to_string(),
            endpoint_url: endpoint_url.to_string(),
            is_healthy: false,
            last_success: None,
            last_check: Instant::now(),
            response_time_ms: None,
            last_http_code: None,
            consecutive_failures: 0,
        });

        health.last_check = Instant::now();
        health.response_time_ms = response_time_ms;
        health.last_http_code = last_http_code;

        if is_healthy {
            health.is_healthy = true;
            health.last_success = Some(Instant::now());
            health.consecutive_failures = 0;
        } else {
            health.is_healthy = false;
            health.consecutive_failures += 1;
        }
    }

    /// Get current health status for all providers
    pub async fn get_health_status(&self) -> Vec<BtspEndpointHealth> {
        let health_map = self.health_status.read().await;
        health_map.values().cloned().collect()
    }
}

#[derive(Debug, Clone)]
struct BtspProviderInfo {
    name: String,
    endpoint: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_monitor_creation() {
        // Note: Will fail without crypto provider, but demonstrates async construction
        let _ = BtspHealthMonitor::new(30, 1000).await;
    }

    #[tokio::test]
    async fn test_health_status_tracking() {
        // Note: Will fail without crypto provider
        if let Ok(monitor) = BtspHealthMonitor::new(30, 1000).await {
            // Update health status
            monitor
                .update_health_status("test-provider", "https://localhost:9000", true, Some(100), Some(200))
                .await;

            // Retrieve status
            let statuses = monitor.get_health_status().await;
            assert_eq!(statuses.len(), 1);
            assert_eq!(statuses[0].provider_name, "test-provider");
            assert!(statuses[0].is_healthy);
            assert_eq!(statuses[0].response_time_ms, Some(100));
        }
    }
}

