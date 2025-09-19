//! Route optimization and path finding for federation
//!
//! Handles route calculation, optimization, and caching for efficient
//! communication between federation nodes.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::types::*;
use songbird_errors::Result;

/// Route optimizer managing route calculation and caching
pub struct RouteOptimizer {
    /// Routing strategy
    strategy: RouteStrategy,
    /// Route cache
    route_cache: Arc<RwLock<HashMap<(Uuid, Uuid), RouteInfo>>>,
    /// Performance history
    performance_history: Arc<RwLock<HashMap<Uuid, Vec<PerformanceSnapshot>>>>,
}

impl RouteOptimizer {
    /// Create a new route optimizer
    pub fn new(strategy: RouteStrategy) -> Self {
        Self {
            strategy,
            route_cache: Arc::new(RwLock::new(HashMap::new())),
            performance_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get cached route if available
    pub async fn get_cached_route(&self, source: Uuid, destination: Uuid) -> Option<RouteInfo> {
        let cache = self.route_cache.read().await;
        cache.get(&(source, destination)).cloned()
    }

    /// Cache a route
    pub async fn cache_route(&self, source: Uuid, destination: Uuid, route: RouteInfo) {
        let mut cache = self.route_cache.write().await;
        cache.insert((source, destination), route);
    }

    /// Calculate optimal route between two nodes
    pub async fn calculate_optimal_route(
        &self,
        source: Uuid,
        destination: Uuid,
        topology: &NetworkTopology,
        nodes: &HashMap<Uuid, FederationNode>,
    ) -> Result<RouteInfo> {
        // Check cache first
        if let Some(cached_route) = self.get_cached_route(source, destination).await {
            // Check if cache is still valid (not too old)
            if cached_route.measured_at.elapsed().as_secs() < 300 {
                return Ok(cached_route);
            }
        }

        // Calculate new route based on strategy
        let route = match self.strategy {
            RouteStrategy::LowLatency => {
                self.calculate_low_latency_route(source, destination, topology, nodes)
                    .await?
            }
            RouteStrategy::HighBandwidth => {
                self.calculate_high_bandwidth_route(source, destination, topology, nodes)
                    .await?
            }
            RouteStrategy::LowCost => {
                self.calculate_low_cost_route(source, destination, topology, nodes)
                    .await?
            }
            RouteStrategy::Balanced => {
                self.calculate_balanced_route(source, destination, topology, nodes)
                    .await?
            }
        };

        // Cache the route
        self.cache_route(source, destination, route.clone()).await;

        Ok(route)
    }

    /// Calculate route optimized for low latency
    async fn calculate_low_latency_route(
        &self,
        source: Uuid,
        destination: Uuid,
        topology: &NetworkTopology,
        nodes: &HashMap<Uuid, FederationNode>,
    ) -> Result<RouteInfo> {
        // Use Dijkstra's algorithm with latency as weight
        let path = self
            .dijkstra_shortest_path(source, destination, topology, nodes, |node| {
                node.metrics.network_latency_ms as f32
            })
            .await?;

        let total_latency = path
            .iter()
            .skip(1)
            .map(|&node_id| {
                nodes
                    .get(&node_id)
                    .map(|n| n.metrics.network_latency_ms)
                    .unwrap_or(0)
            })
            .sum();

        let min_bandwidth = path
            .iter()
            .skip(1)
            .map(|&node_id| {
                nodes
                    .get(&node_id)
                    .map(|n| n.metrics.bandwidth_usage_mbps)
                    .unwrap_or(0)
            })
            .min()
            .unwrap_or(0);

        Ok(RouteInfo {
            path,
            expected_latency_ms: total_latency,
            expected_bandwidth_mbps: min_bandwidth,
            quality_score: self.calculate_quality_score(total_latency, min_bandwidth),
            measured_at: Instant::now(),
        })
    }

    /// Calculate route optimized for high bandwidth
    async fn calculate_high_bandwidth_route(
        &self,
        source: Uuid,
        destination: Uuid,
        topology: &NetworkTopology,
        nodes: &HashMap<Uuid, FederationNode>,
    ) -> Result<RouteInfo> {
        // Use modified Dijkstra with bandwidth as inverse weight
        let path = self
            .dijkstra_shortest_path(source, destination, topology, nodes, |node| {
                1.0 / (node.metrics.bandwidth_usage_mbps as f32 + 1.0)
            })
            .await?;

        let total_latency = path
            .iter()
            .skip(1)
            .map(|&node_id| {
                nodes
                    .get(&node_id)
                    .map(|n| n.metrics.network_latency_ms)
                    .unwrap_or(0)
            })
            .sum();

        let min_bandwidth = path
            .iter()
            .skip(1)
            .map(|&node_id| {
                nodes
                    .get(&node_id)
                    .map(|n| n.metrics.bandwidth_usage_mbps)
                    .unwrap_or(0)
            })
            .min()
            .unwrap_or(0);

        Ok(RouteInfo {
            path,
            expected_latency_ms: total_latency,
            expected_bandwidth_mbps: min_bandwidth,
            quality_score: self.calculate_quality_score(total_latency, min_bandwidth),
            measured_at: Instant::now(),
        })
    }

    /// Calculate route optimized for low cost
    async fn calculate_low_cost_route(
        &self,
        source: Uuid,
        destination: Uuid,
        topology: &NetworkTopology,
        nodes: &HashMap<Uuid, FederationNode>,
    ) -> Result<RouteInfo> {
        // Use load score as cost metric
        let path = self
            .dijkstra_shortest_path(source, destination, topology, nodes, |node| {
                node.metrics.load_score
            })
            .await?;

        let total_latency = path
            .iter()
            .skip(1)
            .map(|&node_id| {
                nodes
                    .get(&node_id)
                    .map(|n| n.metrics.network_latency_ms)
                    .unwrap_or(0)
            })
            .sum();

        let min_bandwidth = path
            .iter()
            .skip(1)
            .map(|&node_id| {
                nodes
                    .get(&node_id)
                    .map(|n| n.metrics.bandwidth_usage_mbps)
                    .unwrap_or(0)
            })
            .min()
            .unwrap_or(0);

        Ok(RouteInfo {
            path,
            expected_latency_ms: total_latency,
            expected_bandwidth_mbps: min_bandwidth,
            quality_score: self.calculate_quality_score(total_latency, min_bandwidth),
            measured_at: Instant::now(),
        })
    }

    /// Calculate balanced route
    async fn calculate_balanced_route(
        &self,
        source: Uuid,
        destination: Uuid,
        topology: &NetworkTopology,
        nodes: &HashMap<Uuid, FederationNode>,
    ) -> Result<RouteInfo> {
        // Combine latency, bandwidth, and load with equal weights
        let path = self
            .dijkstra_shortest_path(source, destination, topology, nodes, |node| {
                let latency_weight = node.metrics.network_latency_ms as f32 / 100.0;
                let bandwidth_weight = 1.0 / (node.metrics.bandwidth_usage_mbps as f32 + 1.0);
                let load_weight = node.metrics.load_score;

                (latency_weight + bandwidth_weight + load_weight) / 3.0
            })
            .await?;

        let total_latency = path
            .iter()
            .skip(1)
            .map(|&node_id| {
                nodes
                    .get(&node_id)
                    .map(|n| n.metrics.network_latency_ms)
                    .unwrap_or(0)
            })
            .sum();

        let min_bandwidth = path
            .iter()
            .skip(1)
            .map(|&node_id| {
                nodes
                    .get(&node_id)
                    .map(|n| n.metrics.bandwidth_usage_mbps)
                    .unwrap_or(0)
            })
            .min()
            .unwrap_or(0);

        Ok(RouteInfo {
            path,
            expected_latency_ms: total_latency,
            expected_bandwidth_mbps: min_bandwidth,
            quality_score: self.calculate_quality_score(total_latency, min_bandwidth),
            measured_at: Instant::now(),
        })
    }

    /// Dijkstra's algorithm implementation with custom weight function
    async fn dijkstra_shortest_path<F>(
        &self,
        source: Uuid,
        destination: Uuid,
        topology: &NetworkTopology,
        nodes: &HashMap<Uuid, FederationNode>,
        weight_fn: F,
    ) -> Result<Vec<Uuid>>
    where
        F: Fn(&FederationNode) -> f32,
    {
        use std::cmp::Ordering;
        use std::collections::BinaryHeap;

        #[derive(Debug, Clone)]
        struct State {
            node_id: Uuid,
            cost: f32,
            path: Vec<Uuid>,
        }

        impl Ord for State {
            fn cmp(&self, other: &Self) -> Ordering {
                other
                    .cost
                    .partial_cmp(&self.cost)
                    .unwrap_or(Ordering::Equal)
            }
        }

        impl PartialOrd for State {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl PartialEq for State {
            fn eq(&self, other: &Self) -> bool {
                self.cost == other.cost
            }
        }

        impl Eq for State {}

        let mut heap = BinaryHeap::new();
        let mut visited = std::collections::HashSet::new();

        // Start with source node
        heap.push(State {
            node_id: source,
            cost: 0.0,
            path: vec![source],
        });

        while let Some(State {
            node_id,
            cost,
            path,
        }) = heap.pop()
        {
            if node_id == destination {
                return Ok(path);
            }

            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id);

            // Explore neighbors
            if let Some(neighbors) = topology.graph.get(&node_id) {
                for &neighbor_id in neighbors {
                    if visited.contains(&neighbor_id) {
                        continue;
                    }

                    if let Some(neighbor_node) = nodes.get(&neighbor_id) {
                        let neighbor_cost = weight_fn(neighbor_node);
                        let new_cost = cost + neighbor_cost;

                        let mut new_path = path.clone();
                        new_path.push(neighbor_id);

                        heap.push(State {
                            node_id: neighbor_id,
                            cost: new_cost,
                            path: new_path,
                        });
                    }
                }
            }
        }

        // No path found
        Err(songbird_errors::SongbirdError::service_error(
            "routing",
            "No route found".to_string(),
        ))
    }

    /// Calculate quality score for a route
    fn calculate_quality_score(&self, latency: u32, bandwidth: u32) -> f32 {
        // Simple quality score calculation
        let latency_score = 1.0 / (latency as f32 + 1.0);
        let bandwidth_score = bandwidth as f32 / 1000.0; // Normalize to Gbps

        (latency_score + bandwidth_score) / 2.0
    }

    /// Record performance snapshot for a node
    pub async fn record_performance(&self, node_id: Uuid, snapshot: PerformanceSnapshot) {
        let mut history = self.performance_history.write().await;
        history
            .entry(node_id)
            .or_insert_with(Vec::new)
            .push(snapshot);

        // Keep only last 100 snapshots
        if let Some(snapshots) = history.get_mut(&node_id) {
            if snapshots.len() > 100 {
                snapshots.drain(0..snapshots.len() - 100);
            }
        }
    }

    /// Get performance history for a node
    pub async fn get_performance_history(&self, node_id: Uuid) -> Vec<PerformanceSnapshot> {
        let history = self.performance_history.read().await;
        history.get(&node_id).cloned().unwrap_or_default()
    }

    /// Clear expired routes from cache
    pub async fn cleanup_cache(&self) {
        let mut cache = self.route_cache.write().await;
        cache.retain(|_, route| route.measured_at.elapsed().as_secs() < 600);
    }
}
