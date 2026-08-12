// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Load Balancing for Universal Adapters
//!
//! Provides round-robin and health-based load balancing across multiple
//! capability providers.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::RwLock;

#[cfg(test)]
use songbird_types::SongbirdResult;

/// Load balancing strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Round-robin distribution
    RoundRobin,
    /// Route to least loaded endpoint
    LeastLoaded,
    /// Route to healthiest endpoint
    HealthBased,
    /// Random selection
    Random,
}

/// Endpoint with health and load tracking
#[derive(Debug, Clone)]
pub struct LoadBalancedEndpoint {
    /// Endpoint URL
    pub url: String,
    /// Current active connections
    pub active_connections: u32,
    /// Health score (0.0 - 1.0)
    pub health_score: f64,
    /// Is endpoint currently available?
    pub available: bool,
    /// Total requests handled
    pub total_requests: u64,
}

impl LoadBalancedEndpoint {
    /// Create a new endpoint
    #[must_use]
    pub const fn new(url: String) -> Self {
        Self {
            url,
            active_connections: 0,
            health_score: 1.0,
            available: true,
            total_requests: 0,
        }
    }

    /// Mark endpoint as unavailable
    pub const fn mark_unavailable(&mut self) {
        self.available = false;
        self.health_score = 0.0;
    }

    /// Mark endpoint as available
    pub fn mark_available(&mut self) {
        self.available = true;
        if self.health_score == 0.0 {
            self.health_score = 1.0;
        }
    }

    /// Update health score
    pub fn update_health(&mut self, score: f64) {
        self.health_score = score.clamp(0.0, 1.0);
        self.available = score > 0.0;
    }

    /// Increment active connections
    pub const fn increment_connections(&mut self) {
        self.active_connections = self.active_connections.saturating_add(1);
    }

    /// Decrement active connections
    pub const fn decrement_connections(&mut self) {
        self.active_connections = self.active_connections.saturating_sub(1);
    }

    /// Record a completed request
    pub const fn record_request(&mut self) {
        self.total_requests = self.total_requests.saturating_add(1);
    }
}

/// Load balancer for capability endpoints
pub struct LoadBalancer {
    /// Available endpoints
    endpoints: Arc<RwLock<Vec<LoadBalancedEndpoint>>>,
    /// Current strategy
    strategy: LoadBalancingStrategy,
    /// Round-robin counter
    round_robin_counter: Arc<RwLock<usize>>,
}

impl LoadBalancer {
    /// Create a new load balancer
    #[must_use]
    pub fn new(endpoints: Vec<String>, strategy: LoadBalancingStrategy) -> Self {
        let endpoints = endpoints.into_iter().map(LoadBalancedEndpoint::new).collect();

        Self {
            endpoints: Arc::new(RwLock::new(endpoints)),
            strategy,
            round_robin_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// Get the next endpoint based on load balancing strategy
    ///
    /// # Errors
    ///
    /// Returns an error if no endpoints are available.
    #[expect(clippy::unused_async, reason = "async for API stability")]
    pub async fn get_next_endpoint(&self) -> anyhow::Result<String> {
        let endpoints = self.endpoints.read().unwrap_or_else(std::sync::PoisonError::into_inner);
        anyhow::ensure!(!endpoints.is_empty(), "No endpoints configured");

        let available: Vec<&LoadBalancedEndpoint> =
            endpoints.iter().filter(|e| e.available).collect();
        anyhow::ensure!(!available.is_empty(), "No available endpoints");

        let selected = match self.strategy {
            LoadBalancingStrategy::RoundRobin => self.select_round_robin(&available),
            LoadBalancingStrategy::LeastLoaded => Self::select_least_loaded(&available),
            LoadBalancingStrategy::HealthBased => Self::select_healthiest(&available),
            LoadBalancingStrategy::Random => Self::select_random(&available),
        };

        Ok(selected.url.clone())
    }

    /// Select endpoint using round-robin
    fn select_round_robin<'a>(
        &self,
        available: &[&'a LoadBalancedEndpoint],
    ) -> &'a LoadBalancedEndpoint {
        let mut counter =
            self.round_robin_counter.write().unwrap_or_else(std::sync::PoisonError::into_inner);
        let index = *counter % available.len();
        *counter = counter.wrapping_add(1);
        available[index]
    }

    /// Select least loaded endpoint
    fn select_least_loaded<'a>(available: &[&'a LoadBalancedEndpoint]) -> &'a LoadBalancedEndpoint {
        available.iter().min_by_key(|e| e.active_connections).copied().unwrap_or(&available[0])
    }

    /// Select healthiest endpoint
    fn select_healthiest<'a>(available: &[&'a LoadBalancedEndpoint]) -> &'a LoadBalancedEndpoint {
        available
            .iter()
            .max_by(|a, b| {
                a.health_score.partial_cmp(&b.health_score).unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
            .unwrap_or(&available[0])
    }

    /// Select random endpoint
    fn select_random<'a>(available: &[&'a LoadBalancedEndpoint]) -> &'a LoadBalancedEndpoint {
        use rand::Rng;
        let index = rand::thread_rng().gen_range(0..available.len());
        available[index]
    }

    /// Mark an endpoint as unavailable
    #[expect(clippy::unused_async, reason = "async for API stability")]
    pub async fn mark_endpoint_unavailable(&self, url: &str) {
        let mut endpoints =
            self.endpoints.write().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(endpoint) = endpoints.iter_mut().find(|e| e.url == url) {
            endpoint.mark_unavailable();
        }
    }

    /// Mark an endpoint as available
    #[expect(clippy::unused_async, reason = "async for API stability")]
    pub async fn mark_endpoint_available(&self, url: &str) {
        let mut endpoints =
            self.endpoints.write().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(endpoint) = endpoints.iter_mut().find(|e| e.url == url) {
            endpoint.mark_available();
        }
    }

    /// Update health score for an endpoint
    #[expect(clippy::unused_async, reason = "async for API stability")]
    pub async fn update_endpoint_health(&self, url: &str, health_score: f64) {
        let mut endpoints =
            self.endpoints.write().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(endpoint) = endpoints.iter_mut().find(|e| e.url == url) {
            endpoint.update_health(health_score);
        }
    }

    /// Get all endpoints with their status
    ///
    /// **Note**: This method clones the entire endpoints vec. For read-only access
    /// in hot paths, consider adding a method that provides a read lock guard instead.
    #[expect(clippy::unused_async, reason = "async for API stability")]
    pub async fn get_endpoints(&self) -> Vec<LoadBalancedEndpoint> {
        self.endpoints.read().unwrap_or_else(std::sync::PoisonError::into_inner).clone()
    }

    /// Get count of healthy endpoints (zero-clone)
    #[expect(clippy::unused_async, reason = "async for API stability")]
    pub async fn healthy_count(&self) -> usize {
        self.endpoints
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter(|e| e.available && e.health_score > 0.5)
            .count()
    }

    /// Get count of available endpoints
    #[expect(clippy::unused_async, reason = "async for API stability")]
    pub async fn available_count(&self) -> usize {
        self.endpoints
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter(|e| e.available)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_types::SongbirdError;

    #[tokio::test]
    async fn test_round_robin_load_balancing() -> SongbirdResult<()> {
        let endpoints = vec![
            String::from("http://endpoint1:8080"),
            String::from("http://endpoint2:8080"),
            String::from("http://endpoint3:8080"),
        ];

        let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

        // Should cycle through endpoints in order
        let e1 = lb.get_next_endpoint().await.map_err(|e| {
            SongbirdError::configuration(format!(
                "Failed to get first endpoint in round-robin: {e}"
            ))
        })?;
        let e2 = lb.get_next_endpoint().await.map_err(|e| {
            SongbirdError::configuration(format!(
                "Failed to get second endpoint in round-robin: {e}"
            ))
        })?;
        let e3 = lb.get_next_endpoint().await.map_err(|e| {
            SongbirdError::configuration(format!(
                "Failed to get third endpoint in round-robin: {e}"
            ))
        })?;
        let e4 = lb.get_next_endpoint().await.map_err(|e| {
            SongbirdError::configuration(format!(
                "Failed to get wrapped endpoint in round-robin: {e}"
            ))
        })?;

        assert_eq!(e1, endpoints[0]);
        assert_eq!(e2, endpoints[1]);
        assert_eq!(e3, endpoints[2]);
        assert_eq!(e4, endpoints[0]); // Wrapped
        Ok(())
    }

    #[tokio::test]
    async fn test_least_loaded_selection() -> SongbirdResult<()> {
        let endpoints =
            vec![String::from("http://endpoint1:8080"), String::from("http://endpoint2:8080")];

        let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::LeastLoaded);

        // Manually set load on first endpoint
        {
            let mut eps = lb.endpoints.write().unwrap_or_else(std::sync::PoisonError::into_inner);
            eps[0].active_connections = 10;
            eps[1].active_connections = 2;
        }

        // Should select endpoint with fewer connections
        let selected = lb.get_next_endpoint().await.map_err(|e| {
            SongbirdError::configuration(format!("Failed to select least-loaded endpoint: {e}"))
        })?;
        assert_eq!(selected, endpoints[1]); // endpoint2 has fewer connections
        Ok(())
    }

    #[tokio::test]
    async fn test_health_based_selection() -> SongbirdResult<()> {
        let endpoints =
            vec![String::from("http://endpoint1:8080"), String::from("http://endpoint2:8080")];

        let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);

        // Set different health scores
        {
            let mut eps = lb.endpoints.write().unwrap_or_else(std::sync::PoisonError::into_inner);
            eps[0].health_score = 0.5;
            eps[1].health_score = 0.9;
        }

        // Should select healthiest endpoint
        let selected = lb.get_next_endpoint().await.map_err(|e| {
            SongbirdError::configuration(format!("Failed to select health-based endpoint: {e}"))
        })?;
        assert_eq!(selected, endpoints[1]); // endpoint2 is healthier
        Ok(())
    }

    #[tokio::test]
    async fn test_mark_endpoint_unavailable() -> SongbirdResult<()> {
        let endpoints =
            vec![String::from("http://endpoint1:8080"), String::from("http://endpoint2:8080")];

        let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

        // Mark first endpoint as unavailable
        lb.mark_endpoint_unavailable(&endpoints[0]).await;

        // Should only return endpoint2
        let selected = lb.get_next_endpoint().await.map_err(|e| {
            SongbirdError::configuration(format!(
                "Failed to get endpoint after marking one unavailable: {e}"
            ))
        })?;
        assert_eq!(selected, endpoints[1]);

        assert_eq!(lb.available_count().await, 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_all_endpoints_unavailable() {
        let endpoints = vec![String::from("http://endpoint1:8080")];

        let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

        // Mark all endpoints unavailable
        lb.mark_endpoint_unavailable(&endpoints[0]).await;

        // Should return error
        let result = lb.get_next_endpoint().await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("No available endpoints"));
    }

    #[tokio::test]
    async fn test_empty_endpoints() {
        let lb = LoadBalancer::new(vec![], LoadBalancingStrategy::RoundRobin);

        let result = lb.get_next_endpoint().await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("No endpoints configured"));
    }

    #[tokio::test]
    async fn test_endpoint_recovery() -> SongbirdResult<()> {
        let endpoints = vec![String::from("http://endpoint1:8080")];

        let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

        // Mark unavailable then available
        lb.mark_endpoint_unavailable(&endpoints[0]).await;
        assert_eq!(lb.available_count().await, 0);

        lb.mark_endpoint_available(&endpoints[0]).await;
        assert_eq!(lb.available_count().await, 1);

        // Should work again
        let selected = lb.get_next_endpoint().await.map_err(|e| {
            SongbirdError::configuration(format!("Failed to get endpoint after recovery: {e}"))
        })?;
        assert_eq!(selected, endpoints[0]);
        Ok(())
    }

    #[tokio::test]
    async fn test_health_score_update() {
        let endpoints = vec![String::from("http://endpoint1:8080")];

        let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);

        // Update health score
        lb.update_endpoint_health(&endpoints[0], 0.7).await;

        let eps = lb.get_endpoints().await;
        assert_eq!(eps[0].health_score, 0.7);
        assert!(eps[0].available); // Should still be available

        // Zero health should mark unavailable
        lb.update_endpoint_health(&endpoints[0], 0.0).await;
        let eps = lb.get_endpoints().await;
        assert!(!eps[0].available);
    }

    #[tokio::test]
    async fn test_random_strategy_returns_valid_endpoint() {
        let endpoints = vec![
            String::from("http://a:1"),
            String::from("http://b:2"),
            String::from("http://c:3"),
        ];
        let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::Random);
        let selected = lb.get_next_endpoint().await.unwrap();
        assert!(endpoints.contains(&selected));
    }

    #[tokio::test]
    async fn test_endpoint_connection_tracking() {
        let endpoints = vec![String::from("http://endpoint1:8080")];
        let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::LeastLoaded);
        {
            let mut eps = lb.endpoints.write().unwrap_or_else(std::sync::PoisonError::into_inner);
            eps[0].increment_connections();
            eps[0].increment_connections();
            eps[0].record_request();
            assert_eq!(eps[0].active_connections, 2);
            assert_eq!(eps[0].total_requests, 1);
            eps[0].decrement_connections();
            assert_eq!(eps[0].active_connections, 1);
        }
    }

    #[tokio::test]
    async fn test_mark_available_restores_zero_health_to_one() {
        let endpoints = vec![String::from("http://endpoint1:8080")];
        let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);
        lb.mark_endpoint_unavailable(&endpoints[0]).await;
        lb.mark_endpoint_available(&endpoints[0]).await;
        let eps = lb.get_endpoints().await;
        assert!(eps[0].available);
        assert_eq!(eps[0].health_score, 1.0);
    }

    #[tokio::test]
    async fn test_update_health_clamps_out_of_range_scores() {
        let endpoints = vec![String::from("http://endpoint1:8080")];
        let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);
        lb.update_endpoint_health(&endpoints[0], 1.5).await;
        let eps = lb.get_endpoints().await;
        assert_eq!(eps[0].health_score, 1.0);
        lb.update_endpoint_health(&endpoints[0], -0.5).await;
        let eps = lb.get_endpoints().await;
        assert_eq!(eps[0].health_score, 0.0);
        assert!(!eps[0].available);
    }

    #[tokio::test]
    async fn test_healthy_count_excludes_low_health_endpoints() {
        let endpoints = vec![String::from("http://a:1"), String::from("http://b:2")];
        let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);
        lb.update_endpoint_health(&endpoints[0], 0.9).await;
        lb.update_endpoint_health(&endpoints[1], 0.3).await;
        assert_eq!(lb.healthy_count().await, 1);
    }

    #[tokio::test]
    async fn test_load_balanced_endpoint_mark_unavailable() {
        let mut ep = LoadBalancedEndpoint::new(String::from("http://x:1"));
        ep.mark_unavailable();
        assert!(!ep.available);
        assert_eq!(ep.health_score, 0.0);
    }

    #[tokio::test]
    async fn test_round_robin_skips_unavailable_endpoints() {
        let endpoints = vec![String::from("http://a:1"), String::from("http://b:2")];
        let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);
        lb.mark_endpoint_unavailable(&endpoints[0]).await;
        let first = lb.get_next_endpoint().await.unwrap();
        let second = lb.get_next_endpoint().await.unwrap();
        assert_eq!(first, endpoints[1]);
        assert_eq!(second, endpoints[1]);
    }
}
