// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! QoS-Aware Provider Selection
//!
//! Intelligent provider selection based on Quality of Service metrics including
//! latency, availability, load, and health status.
//!
//! **Benefits:**
//! - Better resource utilization
//! - Improved reliability
//! - Lower latency
//! - Automatic load balancing

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// `QoS` metrics for a provider
#[derive(Debug, Clone)]
pub struct ProviderQoSMetrics {
    /// Provider identifier
    pub provider_id: String,

    /// Average latency in milliseconds
    pub avg_latency_ms: f64,

    /// Current load (0.0 = idle, 1.0 = fully loaded)
    pub current_load: f64,

    /// Availability score (0.0 = unavailable, 1.0 = fully available)
    pub availability: f64,

    /// Recent success rate (0.0 = all failures, 1.0 = all successes)
    pub success_rate: f64,

    /// Last health check result
    pub health_status: ProviderHealth,

    /// Number of recent requests
    pub request_count: u64,

    /// Last update timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Provider health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderHealth {
    /// Provider is healthy and operational
    Healthy,

    /// Provider is experiencing degraded performance
    Degraded,

    /// Provider is unhealthy
    Unhealthy,

    /// Health status unknown
    Unknown,
}

impl ProviderHealth {
    /// Convert to score (0.0 to 1.0)
    #[must_use]
    pub const fn to_score(self) -> f64 {
        match self {
            Self::Healthy => 1.0,
            Self::Degraded => 0.5,
            Self::Unhealthy => 0.0,
            Self::Unknown => 0.7, // Give benefit of doubt
        }
    }
}

/// QoS-aware provider selector
#[derive(Debug)]
pub struct QoSProviderSelector {
    /// Provider metrics cache
    metrics: Arc<RwLock<HashMap<String, ProviderQoSMetrics>>>,

    /// Selection weights
    weights: SelectionWeights,
}

/// Configurable weights for provider selection
#[derive(Debug, Clone)]
#[expect(
    clippy::struct_field_names,
    reason = "_weight field suffix is intentional for selection scoring clarity"
)]
pub struct SelectionWeights {
    /// Weight for health status (0.0 to 1.0)
    pub health_weight: f64,

    /// Weight for latency (0.0 to 1.0)
    pub latency_weight: f64,

    /// Weight for load (0.0 to 1.0)
    pub load_weight: f64,

    /// Weight for availability (0.0 to 1.0)
    pub availability_weight: f64,

    /// Weight for success rate (0.0 to 1.0)
    pub success_rate_weight: f64,
}

impl Default for SelectionWeights {
    fn default() -> Self {
        Self {
            health_weight: 0.35,       // 35% - Most important
            latency_weight: 0.25,      // 25% - Performance critical
            load_weight: 0.15,         // 15% - Balance load
            availability_weight: 0.15, // 15% - Historical reliability
            success_rate_weight: 0.10, // 10% - Recent performance
        }
    }
}

impl QoSProviderSelector {
    /// Create a new QoS-aware provider selector
    #[must_use]
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            weights: SelectionWeights::default(),
        }
    }

    /// Create with custom weights
    #[must_use]
    pub fn with_weights(weights: SelectionWeights) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            weights,
        }
    }

    /// Update metrics for a provider
    pub async fn update_metrics(&self, metrics: ProviderQoSMetrics) {
        let mut cache = self.metrics.write().await;
        cache.insert(metrics.provider_id.clone(), metrics);
    }

    /// Select best provider from a list based on `QoS` metrics
    ///
    /// Returns the provider ID with the highest `QoS` score, or `None` if no suitable provider.
    pub async fn select_best_provider(&self, providers: &[String]) -> Option<String> {
        if providers.is_empty() {
            debug!("No providers available for selection");
            return None;
        }

        if providers.len() == 1 {
            debug!("Only one provider available: {}", providers[0]);
            return Some(providers[0].clone());
        }

        let metrics_cache = self.metrics.read().await;

        // Score each provider
        let mut scored_providers: Vec<(String, f64)> = Vec::new();

        for provider in providers {
            let score = metrics_cache.get(provider).map_or_else(
                || {
                    // No metrics yet, use default score
                    debug!("No metrics for provider {}, using default score", provider);
                    0.5 // Neutral score for unknown providers
                },
                |metrics| self.calculate_qos_score(metrics),
            );

            scored_providers.push((provider.clone(), score));
            debug!("Provider {} scored: {:.3}", provider, score);
        }

        // Select provider with highest score
        scored_providers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored_providers.first().map(|(id, score)| {
            info!("✅ Selected provider {} with QoS score: {:.3}", id, score);
            id.clone()
        })
    }

    /// Calculate `QoS` score for a provider (0.0 to 1.0)
    fn calculate_qos_score(&self, metrics: &ProviderQoSMetrics) -> f64 {
        // Health score
        let health_score = metrics.health_status.to_score();

        // Latency score (lower is better, normalize to 0-1)
        // Assume 0ms = 1.0, 1000ms = 0.0
        let latency_score = (1000.0 - metrics.avg_latency_ms.min(1000.0)) / 1000.0;

        // Load score (lower is better)
        let load_score = 1.0 - metrics.current_load;

        // Availability score (higher is better)
        let availability_score = metrics.availability;

        // Success rate score (higher is better)
        let success_rate_score = metrics.success_rate;

        // Weighted average
        let total_score = success_rate_score.mul_add(
            self.weights.success_rate_weight,
            availability_score.mul_add(
                self.weights.availability_weight,
                health_score.mul_add(
                    self.weights.health_weight,
                    latency_score * self.weights.latency_weight,
                ) + load_score * self.weights.load_weight,
            ),
        );

        total_score.clamp(0.0, 1.0)
    }

    /// Get current metrics for a provider
    pub async fn get_metrics(&self, provider_id: &str) -> Option<ProviderQoSMetrics> {
        let cache = self.metrics.read().await;
        cache.get(provider_id).cloned()
    }

    /// Get all provider metrics
    pub async fn get_all_metrics(&self) -> HashMap<String, ProviderQoSMetrics> {
        let cache = self.metrics.read().await;
        cache.clone()
    }

    /// Record a request result to update metrics
    pub async fn record_request_result(&self, provider_id: &str, latency_ms: f64, success: bool) {
        let mut cache = self.metrics.write().await;

        let metrics = cache.entry(provider_id.to_string()).or_insert_with(|| ProviderQoSMetrics {
            provider_id: provider_id.to_string(),
            avg_latency_ms: latency_ms,
            current_load: 0.0,
            availability: 1.0,
            success_rate: if success {
                1.0
            } else {
                0.0
            },
            health_status: ProviderHealth::Unknown,
            request_count: 0,
            last_updated: chrono::Utc::now(),
        });

        // Update running averages (exponential moving average)
        let alpha = 0.3; // Smoothing factor
        metrics.avg_latency_ms = alpha * latency_ms + (1.0 - alpha) * metrics.avg_latency_ms;
        metrics.success_rate = alpha
            * (if success {
                1.0
            } else {
                0.0
            })
            + (1.0 - alpha) * metrics.success_rate;
        metrics.request_count += 1;
        metrics.last_updated = chrono::Utc::now();

        // Update health based on success rate
        metrics.health_status = if metrics.success_rate > 0.9 {
            ProviderHealth::Healthy
        } else if metrics.success_rate > 0.5 {
            ProviderHealth::Degraded
        } else {
            ProviderHealth::Unhealthy
        };
    }

    /// Clear metrics for a provider (e.g., when it's removed)
    pub async fn clear_metrics(&self, provider_id: &str) {
        let mut cache = self.metrics.write().await;
        cache.remove(provider_id);
    }

    /// Clear all metrics
    pub async fn clear_all_metrics(&self) {
        let mut cache = self.metrics.write().await;
        cache.clear();
    }
}

impl Default for QoSProviderSelector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_qos_selection_basic() {
        let selector = QoSProviderSelector::new();

        // Add metrics for two providers
        selector
            .update_metrics(ProviderQoSMetrics {
                provider_id: "provider-a".to_string(),
                avg_latency_ms: 50.0,
                current_load: 0.3,
                availability: 0.99,
                success_rate: 0.95,
                health_status: ProviderHealth::Healthy,
                request_count: 100,
                last_updated: chrono::Utc::now(),
            })
            .await;

        selector
            .update_metrics(ProviderQoSMetrics {
                provider_id: "provider-b".to_string(),
                avg_latency_ms: 200.0,
                current_load: 0.8,
                availability: 0.85,
                success_rate: 0.75,
                health_status: ProviderHealth::Degraded,
                request_count: 50,
                last_updated: chrono::Utc::now(),
            })
            .await;

        let providers = vec!["provider-a".to_string(), "provider-b".to_string()];
        let best = selector.select_best_provider(&providers).await;

        assert_eq!(best, Some("provider-a".to_string()));
    }

    #[tokio::test]
    async fn test_qos_selection_no_metrics() {
        let selector = QoSProviderSelector::new();

        let providers = vec!["unknown-a".to_string(), "unknown-b".to_string()];
        let best = selector.select_best_provider(&providers).await;

        // Should still select one even without metrics
        assert!(best.is_some());
    }

    #[tokio::test]
    async fn test_record_request_updates_metrics() {
        let selector = QoSProviderSelector::new();

        // Record successful requests
        selector.record_request_result("provider-a", 100.0, true).await;
        selector.record_request_result("provider-a", 120.0, true).await;
        selector.record_request_result("provider-a", 80.0, true).await;

        let metrics = selector.get_metrics("provider-a").await;
        assert!(metrics.is_some());

        let metrics = metrics.unwrap();
        assert_eq!(metrics.request_count, 3);
        assert!(metrics.success_rate > 0.9);
        assert_eq!(metrics.health_status, ProviderHealth::Healthy);
    }

    #[tokio::test]
    async fn test_health_based_on_success_rate() {
        let selector = QoSProviderSelector::new();

        // Record mostly failures
        selector.record_request_result("provider-bad", 100.0, false).await;
        selector.record_request_result("provider-bad", 100.0, false).await;
        selector.record_request_result("provider-bad", 100.0, false).await;
        selector.record_request_result("provider-bad", 100.0, true).await;

        let metrics = selector.get_metrics("provider-bad").await.unwrap();
        assert_eq!(metrics.health_status, ProviderHealth::Unhealthy);
    }

    #[tokio::test]
    async fn test_custom_weights() {
        let weights = SelectionWeights {
            health_weight: 0.5,
            latency_weight: 0.5,
            load_weight: 0.0,
            availability_weight: 0.0,
            success_rate_weight: 0.0,
        };

        let selector = QoSProviderSelector::with_weights(weights);

        // Fast but degraded
        selector
            .update_metrics(ProviderQoSMetrics {
                provider_id: "fast-degraded".to_string(),
                avg_latency_ms: 10.0,
                current_load: 0.5,
                availability: 0.5,
                success_rate: 0.5,
                health_status: ProviderHealth::Degraded,
                request_count: 100,
                last_updated: chrono::Utc::now(),
            })
            .await;

        // Slow but healthy
        selector
            .update_metrics(ProviderQoSMetrics {
                provider_id: "slow-healthy".to_string(),
                avg_latency_ms: 500.0,
                current_load: 0.5,
                availability: 0.5,
                success_rate: 0.5,
                health_status: ProviderHealth::Healthy,
                request_count: 100,
                last_updated: chrono::Utc::now(),
            })
            .await;

        let providers = vec!["fast-degraded".to_string(), "slow-healthy".to_string()];
        let best = selector.select_best_provider(&providers).await;

        // With 50/50 weights, healthy should win despite slower
        assert_eq!(best, Some("slow-healthy".to_string()));
    }
}
