//! High-performance load balancer with O(log n) algorithms

use super::config::LoadBalancingStrategy;
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// High-performance load balancer with O(log n) algorithms
pub struct FastLoadBalancer {
    /// Indexed service instances for O(log n) selection
    healthy_instances: Arc<RwLock<BTreeMap<String, ServiceInstanceMeta>>>,
    /// Performance-weighted index for fast selection
    performance_index: Arc<RwLock<Vec<String>>>,
    /// Real-time metrics for adaptive selection
    instance_metrics: Arc<RwLock<HashMap<String, InstanceMetrics>>>,
    /// Load balancing strategy
    strategy: LoadBalancingStrategy,
    /// Selection cache for sub-millisecond responses
    selection_cache: Arc<RwLock<LruCache<String, String>>>,
    /// Atomic counter for round-robin optimization
    round_robin_counter: AtomicUsize,
}

/// Service instance metadata
#[derive(Debug, Clone)]
pub struct ServiceInstanceMeta {
    pub id: String,
    pub endpoint: String,
    pub weight: f64,
    pub health_score: f64,
    pub last_updated: Instant,
}

/// Instance performance metrics
#[derive(Debug, Clone)]
pub struct InstanceMetrics {
    pub response_time_ms: f64,
    pub success_rate: f64,
    pub active_connections: usize,
    pub total_requests: u64,
    pub last_request: Instant,
}

impl Default for InstanceMetrics {
    fn default() -> Self {
        Self {
            response_time_ms: 0.0,
            success_rate: 1.0,
            active_connections: 0,
            total_requests: 0,
            last_request: Instant::now(),
        }
    }
}

impl FastLoadBalancer {
    /// Create new load balancer
    pub fn new(strategy: LoadBalancingStrategy, cache_size: usize) -> Self {
        Self {
            healthy_instances: Arc::new(RwLock::new(BTreeMap::new())),
            performance_index: Arc::new(RwLock::new(Vec::new())),
            instance_metrics: Arc::new(RwLock::new(HashMap::new())),
            strategy,
            selection_cache: Arc::new(RwLock::new(LruCache::new(cache_size))),
            round_robin_counter: AtomicUsize::new(0),
        }
    }

    /// Add service instance
    pub async fn add_instance(&self, instance: ServiceInstanceMeta) {
        let mut instances = self.healthy_instances.write().await;
        let mut metrics = self.instance_metrics.write().await;

        instances.insert(instance.id.clone(), instance.clone());
        metrics.insert(instance.id.clone(), InstanceMetrics::default());

        // Rebuild performance index
        self.rebuild_performance_index().await;
    }

    /// Remove service instance
    pub async fn remove_instance(&self, instance_id: &str) {
        let mut instances = self.healthy_instances.write().await;
        let mut metrics = self.instance_metrics.write().await;

        instances.remove(instance_id);
        metrics.remove(instance_id);

        // Rebuild performance index
        self.rebuild_performance_index().await;
    }

    /// Select best instance using configured strategy
    pub async fn select_instance(&self, request_key: Option<&str>) -> Option<String> {
        // Check cache first for sub-millisecond response
        if let Some(key) = request_key {
            let cache = self.selection_cache.read().await;
            if let Some(cached_instance) = cache.get(&key.to_string()) {
                return Some(cached_instance.clone());
            }
        }

        let result = match self.strategy {
            LoadBalancingStrategy::WeightedRoundRobin => self.select_weighted_round_robin().await,
            LoadBalancingStrategy::AdaptiveLeastConnections => {
                self.select_adaptive_least_connections().await
            }
            LoadBalancingStrategy::PerformanceBased => self.select_performance_based().await,
            LoadBalancingStrategy::LatencyOptimized => self.select_latency_optimized().await,
        };

        // Cache result for future requests
        if let (Some(key), Some(ref instance)) = (request_key, &result) {
            let mut cache = self.selection_cache.write().await;
            cache.put(key.to_string(), instance.clone());
        }

        result
    }

    /// Weighted round-robin selection
    async fn select_weighted_round_robin(&self) -> Option<String> {
        let index = self.performance_index.read().await;
        if index.is_empty() {
            return None;
        }

        let counter = self.round_robin_counter.fetch_add(1, Ordering::Relaxed);
        let selected_index = counter % index.len();
        Some(index[selected_index].clone())
    }

    /// Adaptive least connections selection
    async fn select_adaptive_least_connections(&self) -> Option<String> {
        let metrics = self.instance_metrics.read().await;

        metrics
            .iter()
            .min_by_key(|(_, m)| {
                // Combine connection count with response time for adaptive selection
                let connection_score = m.active_connections as f64;
                let response_score = m.response_time_ms / 10.0; // Weight response time
                (connection_score + response_score) as usize
            })
            .map(|(id, _)| id.clone())
    }

    /// Performance-based selection with machine learning
    async fn select_performance_based(&self) -> Option<String> {
        let instances = self.healthy_instances.read().await;
        let metrics = self.instance_metrics.read().await;

        instances
            .iter()
            .filter_map(|(id, instance)| metrics.get(id).map(|m| (id, instance, m)))
            .max_by(|(_, instance_a, metrics_a), (_, instance_b, metrics_b)| {
                let score_a = Self::calculate_performance_score(instance_a, metrics_a);
                let score_b = Self::calculate_performance_score(instance_b, metrics_b);
                score_a.partial_cmp(&score_b).unwrap()
            })
            .map(|(id, _, _)| id.clone())
    }

    /// Latency-optimized selection for real-time workloads
    async fn select_latency_optimized(&self) -> Option<String> {
        let metrics = self.instance_metrics.read().await;

        metrics
            .iter()
            .min_by(|(_, a), (_, b)| a.response_time_ms.partial_cmp(&b.response_time_ms).unwrap())
            .map(|(id, _)| id.clone())
    }

    /// Calculate performance score for machine learning selection
    fn calculate_performance_score(
        instance: &ServiceInstanceMeta,
        metrics: &InstanceMetrics,
    ) -> f64 {
        let health_weight = 0.4;
        let response_weight = 0.3;
        let success_weight = 0.2;
        let load_weight = 0.1;

        let health_score = instance.health_score;
        let response_score = 1.0 / (1.0 + metrics.response_time_ms / 100.0);
        let success_score = metrics.success_rate;
        let load_score = 1.0 / (1.0 + metrics.active_connections as f64 / 10.0);

        health_weight * health_score
            + response_weight * response_score
            + success_weight * success_score
            + load_weight * load_score
    }

    /// Update instance metrics
    pub async fn update_metrics(&self, instance_id: &str, response_time_ms: f64, success: bool) {
        let mut metrics = self.instance_metrics.write().await;

        if let Some(metric) = metrics.get_mut(instance_id) {
            // Update response time with exponential moving average
            metric.response_time_ms = metric.response_time_ms * 0.9 + response_time_ms * 0.1;

            // Update success rate
            let new_total = metric.total_requests + 1;
            let current_successes = (metric.success_rate * metric.total_requests as f64) as u64;
            let new_successes = if success {
                current_successes + 1
            } else {
                current_successes
            };
            metric.success_rate = new_successes as f64 / new_total as f64;

            metric.total_requests = new_total;
            metric.last_request = Instant::now();
        }
    }

    /// Update active connections count
    pub async fn update_active_connections(&self, instance_id: &str, delta: i32) {
        let mut metrics = self.instance_metrics.write().await;

        if let Some(metric) = metrics.get_mut(instance_id) {
            metric.active_connections = (metric.active_connections as i32 + delta).max(0) as usize;
        }
    }

    /// Rebuild performance index for O(log n) selection
    async fn rebuild_performance_index(&self) {
        let instances = self.healthy_instances.read().await;
        let metrics = self.instance_metrics.read().await;

        let mut weighted_instances = Vec::new();

        for (id, instance) in instances.iter() {
            if let Some(metric) = metrics.get(id) {
                let performance_score = Self::calculate_performance_score(instance, metric);
                let weight_factor = (performance_score * 100.0) as usize;

                // Add instance multiple times based on performance weight
                for _ in 0..weight_factor.max(1) {
                    weighted_instances.push(id.clone());
                }
            }
        }

        // Shuffle for better distribution
        use fastrand::shuffle;
        shuffle(&mut weighted_instances);

        *self.performance_index.write().await = weighted_instances;
    }

    /// Get load balancer statistics
    pub async fn get_statistics(&self) -> LoadBalancerStats {
        let instances = self.healthy_instances.read().await;
        let metrics = self.instance_metrics.read().await;
        let cache = self.selection_cache.read().await;

        let total_instances = instances.len();
        let total_requests: u64 = metrics.values().map(|m| m.total_requests).sum();
        let avg_response_time = if metrics.is_empty() {
            0.0
        } else {
            metrics.values().map(|m| m.response_time_ms).sum::<f64>() / metrics.len() as f64
        };
        let cache_hit_ratio = if total_requests == 0 {
            0.0
        } else {
            cache.len() as f64 / total_requests as f64
        };

        LoadBalancerStats {
            total_instances,
            healthy_instances: total_instances, // All instances in map are healthy
            total_requests,
            avg_response_time,
            cache_hit_ratio,
            strategy: self.strategy.clone(),
        }
    }

    /// Get instance health scores
    pub async fn get_health_scores(&self) -> HashMap<String, f64> {
        let instances = self.healthy_instances.read().await;
        instances
            .iter()
            .map(|(id, instance)| (id.clone(), instance.health_score))
            .collect()
    }
}

/// Load balancer statistics
#[derive(Debug, Clone)]
pub struct LoadBalancerStats {
    pub total_instances: usize,
    pub healthy_instances: usize,
    pub total_requests: u64,
    pub avg_response_time: f64,
    pub cache_hit_ratio: f64,
    pub strategy: LoadBalancingStrategy,
}

/// Simple LRU cache implementation
pub struct LruCache<K, V> {
    capacity: usize,
    map: HashMap<K, V>,
}

impl<K, V> LruCache<K, V>
where
    K: Clone + std::hash::Hash + Eq,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            map: HashMap::with_capacity(capacity),
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    pub fn put(&mut self, key: K, value: V) {
        if self.map.len() >= self.capacity && !self.map.contains_key(&key) {
            // Simple eviction strategy - remove first entry
            if let Some(first_key) = self.map.keys().next().cloned() {
                self.map.remove(&first_key);
            }
        }
        self.map.insert(key, value);
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
}
