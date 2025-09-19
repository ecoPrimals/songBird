//! Production Load Balancing Implementation
//!
//! Advanced load balancing algorithms replacing simple round-robin

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use songbird_errors::{NetworkResult, SongbirdError, SongbirdResult, success};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Load balancing algorithms
#[derive(Debug, Clone, PartialEq)]
pub enum LoadBalancingAlgorithm {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    WeightedLeastConnections,
    HealthAware,
    LatencyBased,
    ResourceBased,
    Adaptive,
}

/// Node for load balancing
#[derive(Debug, Clone)]
pub struct LoadBalancerNode {
    /// Node identifier
    pub node_id: String,
    /// Node endpoint
    pub endpoint: String,
    /// Node weight (for weighted algorithms)
    pub weight: u32,
    /// Current active connections
    pub active_connections: u32,
    /// Health status
    pub is_healthy: bool,
    /// Average response time
    pub avg_response_time: Duration,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage percentage
    pub memory_usage: f64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Load balancing decision result
#[derive(Debug, Clone)]
pub struct LoadBalancingDecision {
    /// Selected node
    pub selected_node: LoadBalancerNode,
    /// Algorithm used
    pub algorithm_used: LoadBalancingAlgorithm,
    /// Decision confidence (0.0 to 1.0)
    pub confidence: f64,
    /// Alternative nodes (fallbacks)
    pub alternatives: Vec<LoadBalancerNode>,
    /// Decision timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Load balancing statistics
#[derive(Debug, Default, Clone)]
pub struct LoadBalancingStats {
    /// Total routing decisions made
    pub total_decisions: u64,
    /// Successful routes
    pub successful_routes: u64,
    /// Failed routes
    pub failed_routes: u64,
    /// Average decision time
    pub avg_decision_time: Duration,
    /// Node utilization stats
    pub node_utilization: HashMap<String, NodeUtilization>,
}

/// Node utilization statistics
#[derive(Debug, Clone)]
pub struct NodeUtilization {
    pub requests_routed: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time: Duration,
    pub current_load: f64,
}

/// Production load balancer
pub struct ProductionLoadBalancer {
    /// Available nodes
    nodes: Arc<RwLock<HashMap<String, LoadBalancerNode>>>,
    /// Current algorithm
    algorithm: LoadBalancingAlgorithm,
    /// Round-robin state
    round_robin_index: Arc<RwLock<usize>>,
    /// Load balancing statistics
    stats: Arc<RwLock<LoadBalancingStats>>,
    /// Algorithm weights for adaptive balancing
    algorithm_weights: HashMap<LoadBalancingAlgorithm, f64>,
}

impl ProductionLoadBalancer {
    /// Create new production load balancer
    pub fn new(algorithm: LoadBalancingAlgorithm) -> Self {
        let mut algorithm_weights = HashMap::new();
        algorithm_weights.insert(LoadBalancingAlgorithm::RoundRobin, 1.0);
        algorithm_weights.insert(LoadBalancingAlgorithm::WeightedRoundRobin, 1.2);
        algorithm_weights.insert(LoadBalancingAlgorithm::LeastConnections, 1.5);
        algorithm_weights.insert(LoadBalancingAlgorithm::HealthAware, 2.0);
        algorithm_weights.insert(LoadBalancingAlgorithm::LatencyBased, 1.8);
        algorithm_weights.insert(LoadBalancingAlgorithm::ResourceBased, 1.6);
        algorithm_weights.insert(LoadBalancingAlgorithm::Adaptive, 2.5);
        
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            algorithm,
            round_robin_index: Arc::new(RwLock::new(0)),
            stats: Arc::new(RwLock::new(LoadBalancingStats::default())),
            algorithm_weights,
        }
    }
    
    /// Update node list
    pub async fn update_nodes(&self, nodes: Vec<LoadBalancerNode>) -> NetworkResult<()> {
        let mut node_map = self.nodes.write().await;
        node_map.clear();
        
        for node in nodes {
            node_map.insert(node.node_id.clone(), node);
        }
        
        info!("🔄 Updated load balancer nodes: {} available", node_map.len());
        Ok(())
    }
    
    /// Select best node for request
    pub async fn select_node(&self) -> NetworkResult<LoadBalancingDecision> {
        let decision_start = Instant::now();
        
        let nodes = self.nodes.read().await;
        let available_nodes: Vec<LoadBalancerNode> = nodes
            .values()
            .filter(|node| node.is_healthy)
            .cloned()
            .collect();
        drop(nodes);
        
        if available_nodes.is_empty() {
            return Err(SongbirdError::internal_error(network_error("No healthy nodes available"));
        }
        
        // Select node based on algorithm
        let selected_node = match self.algorithm {
            LoadBalancingAlgorithm::RoundRobin => {
                self.select_round_robin(&available_nodes).await?
            }
            LoadBalancingAlgorithm::WeightedRoundRobin => {
                self.select_weighted_round_robin(&available_nodes).await?
            }
            LoadBalancingAlgorithm::LeastConnections => {
                self.select_least_connections(&available_nodes)?
            }
            LoadBalancingAlgorithm::WeightedLeastConnections => {
                self.select_weighted_least_connections(&available_nodes)?
            }
            LoadBalancingAlgorithm::HealthAware => {
                self.select_health_aware(&available_nodes)?
            }
            LoadBalancingAlgorithm::LatencyBased => {
                self.select_latency_based(&available_nodes)?
            }
            LoadBalancingAlgorithm::ResourceBased => {
                self.select_resource_based(&available_nodes)?
            }
            LoadBalancingAlgorithm::Adaptive => {
                self.select_adaptive(&available_nodes).await?
            }
        };
        
        // Calculate alternatives
        let alternatives: Vec<LoadBalancerNode> = available_nodes
            .into_iter()
            .filter(|node| node.node_id != selected_node.node_id)
            .take(3) // Top 3 alternatives
            .collect();
        
        // Calculate confidence based on node health and performance
        let confidence = self.calculate_decision_confidence(&selected_node);
        
        let decision = LoadBalancingDecision {
            selected_node: selected_node.clone(),
            algorithm_used: self.algorithm.clone(),
            confidence,
            alternatives,
            timestamp: chrono::Utc::now(),
        };
        
        // Update statistics
        self.update_routing_stats(&selected_node.node_id, decision_start.elapsed()).await;
        
        debug!(
            "🎯 Selected node {} using {:?} (confidence: {:.2})",
            selected_node.node_id, self.algorithm, confidence
        );
        
        Ok(songbird_errors::evolved_success(decision))
    }
    
    /// Round-robin selection
    async fn select_round_robin(&self, nodes: &[LoadBalancerNode]) -> NetworkResult<LoadBalancerNode> {
        let mut index = self.round_robin_index.write().await;
        let selected = &nodes[*index % nodes.len()];
        *index = (*index + 1) % nodes.len();
        Ok(songbird_errors::evolved_success(selected.clone()))
    }
    
    /// Weighted round-robin selection
    async fn select_weighted_round_robin(&self, nodes: &[LoadBalancerNode]) -> NetworkResult<LoadBalancerNode> {
        // Create weighted list based on node weights
        let mut weighted_nodes = Vec::new();
        for node in nodes {
            for _ in 0..node.weight {
                weighted_nodes.push(node.clone());
            }
        }
        
        if weighted_nodes.is_empty() {
            return self.select_round_robin(nodes).await;
        }
        
        let mut index = self.round_robin_index.write().await;
        let selected = &weighted_nodes[*index % weighted_nodes.len()];
        *index = (*index + 1) % weighted_nodes.len();
        Ok(songbird_errors::evolved_success(selected.clone()))
    }
    
    /// Least connections selection
    fn select_least_connections(&self, nodes: &[LoadBalancerNode]) -> NetworkResult<LoadBalancerNode> {
        let selected = nodes
            .iter()
            .min_by_key(|node| node.active_connections)
            .ok_or_else(|| SongbirdError::network("No nodes available"))?;
        
        Ok(songbird_errors::evolved_success(selected.clone()))
    }
    
    /// Weighted least connections selection
    fn select_weighted_least_connections(&self, nodes: &[LoadBalancerNode]) -> NetworkResult<LoadBalancerNode> {
        let selected = nodes
            .iter()
            .min_by(|a, b| {
                let a_ratio = a.active_connections as f64 / a.weight as f64;
                let b_ratio = b.active_connections as f64 / b.weight as f64;
                a_ratio.partial_cmp(&b_ratio).unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| SongbirdError::network("No nodes available"))?;
        
        Ok(songbird_errors::evolved_success(selected.clone()))
    }
    
    /// Health-aware selection
    fn select_health_aware(&self, nodes: &[LoadBalancerNode]) -> NetworkResult<LoadBalancerNode> {
        let selected = nodes
            .iter()
            .filter(|node| node.is_healthy && node.success_rate > 0.95)
            .max_by(|a, b| {
                let a_score = a.success_rate * (1.0 - a.cpu_usage / 100.0) * (1.0 - a.memory_usage / 100.0);
                let b_score = b.success_rate * (1.0 - b.cpu_usage / 100.0) * (1.0 - b.memory_usage / 100.0);
                a_score.partial_cmp(&b_score).unwrap_or(std::cmp::Ordering::Equal)
            })
            .or_else(|| nodes.first()) // Fallback to first node
            .ok_or_else(|| SongbirdError::network("No nodes available"))?;
        
        Ok(songbird_errors::evolved_success(selected.clone()))
    }
    
    /// Latency-based selection
    fn select_latency_based(&self, nodes: &[LoadBalancerNode]) -> NetworkResult<LoadBalancerNode> {
        let selected = nodes
            .iter()
            .min_by_key(|node| node.avg_response_time)
            .ok_or_else(|| SongbirdError::network("No nodes available"))?;
        
        Ok(songbird_errors::evolved_success(selected.clone()))
    }
    
    /// Resource-based selection
    fn select_resource_based(&self, nodes: &[LoadBalancerNode]) -> NetworkResult<LoadBalancerNode> {
        let selected = nodes
            .iter()
            .min_by(|a, b| {
                let a_load = (a.cpu_usage + a.memory_usage) / 2.0;
                let b_load = (b.cpu_usage + b.memory_usage) / 2.0;
                a_load.partial_cmp(&b_load).unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| SongbirdError::network("No nodes available"))?;
        
        Ok(songbird_errors::evolved_success(selected.clone()))
    }
    
    /// Adaptive selection (combines multiple algorithms)
    async fn select_adaptive(&self, nodes: &[LoadBalancerNode]) -> NetworkResult<LoadBalancerNode> {
        // Calculate scores for each algorithm
        let mut algorithm_scores = HashMap::new();
        
        // Health-aware score
        if let Ok(songbird_errors::evolved_success(health_node)) = self.select_health_aware(nodes) {
            algorithm_scores.insert(health_node.node_id.clone(), 2.0);
        }
        
        // Latency-based score
        if let Ok(songbird_errors::evolved_success(latency_node)) = self.select_latency_based(nodes) {
            *algorithm_scores.entry(latency_node.node_id.clone()).or_insert(0.0) += 1.8;
        }
        
        // Resource-based score
        if let Ok(songbird_errors::evolved_success(resource_node)) = self.select_resource_based(nodes) {
            *algorithm_scores.entry(resource_node.node_id.clone()).or_insert(0.0) += 1.6;
        }
        
        // Least connections score
        if let Ok(songbird_errors::evolved_success(connections_node)) = self.select_least_connections(nodes) {
            *algorithm_scores.entry(connections_node.node_id.clone()).or_insert(0.0) += 1.5;
        }
        
        // Select node with highest combined score
        let best_node_id = algorithm_scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(node_id, _)| node_id)
            .ok_or_else(|| SongbirdError::network("No suitable node found"))?;
        
        let selected_node = nodes
            .iter()
            .find(|node| &node.node_id == best_node_id)
            .ok_or_else(|| SongbirdError::network("Selected node not found"))?;
        
        Ok(songbird_errors::evolved_success(selected_node.clone()))
    }
    
    /// Calculate decision confidence
    fn calculate_decision_confidence(&self, node: &LoadBalancerNode) -> f64 {
        let mut confidence = 1.0;
        
        // Health impact
        if !node.is_healthy {
            confidence *= 0.3;
        }
        
        // Success rate impact
        confidence *= node.success_rate;
        
        // Resource usage impact
        let resource_load = (node.cpu_usage + node.memory_usage) / 200.0; // Average of CPU and memory
        confidence *= (1.0 - resource_load).max(0.1);
        
        // Response time impact (penalize high latency)
        if node.avg_response_time > Duration::from_millis(1000) {
            confidence *= 0.7;
        } else if node.avg_response_time > Duration::from_millis(500) {
            confidence *= 0.9;
        }
        
        confidence.max(0.0).min(1.0)
    }
    
    /// Update node metrics
    pub async fn update_node_metrics(
        &self,
        node_id: &str,
        active_connections: u32,
        is_healthy: bool,
        avg_response_time: Duration,
        cpu_usage: f64,
        memory_usage: f64,
        success_rate: f64,
    ) -> NetworkResult<()> {
        let mut nodes = self.nodes.write().await;
        
        if let Some(node) = nodes.get_mut(node_id) {
            node.active_connections = active_connections;
            node.is_healthy = is_healthy;
            node.avg_response_time = avg_response_time;
            node.cpu_usage = cpu_usage;
            node.memory_usage = memory_usage;
            node.success_rate = success_rate;
            node.last_updated = chrono::Utc::now();
            
            debug!(
                "📊 Updated metrics for {}: connections={}, health={}, latency={:?}",
                node_id, active_connections, is_healthy, avg_response_time
            );
        }
        
        Ok(())
    }
    
    /// Record successful request
    pub async fn record_success(&self, node_id: &str, response_time: Duration) -> NetworkResult<()> {
        let mut stats = self.stats.write().await;
        stats.successful_routes += 1;
        
        let utilization = stats.node_utilization
            .entry(node_id.to_string())
            .or_insert_with(|| NodeUtilization {
                requests_routed: 0,
                successful_requests: 0,
                failed_requests: 0,
                avg_response_time: Duration::from_millis(0),
                current_load: 0.0,
            });
        
        utilization.successful_requests += 1;
        utilization.requests_routed += 1;
        
        // Update average response time (exponential moving average)
        let alpha = 0.1; // Smoothing factor
        let new_avg = utilization.avg_response_time.as_millis() as f64 * (1.0 - alpha)
            + response_time.as_millis() as f64 * alpha;
        utilization.avg_response_time = Duration::from_millis(new_avg as u64);
        
        Ok(())
    }
    
    /// Record failed request
    pub async fn record_failure(&self, node_id: &str) -> NetworkResult<()> {
        let mut stats = self.stats.write().await;
        stats.failed_routes += 1;
        
        let utilization = stats.node_utilization
            .entry(node_id.to_string())
            .or_insert_with(|| NodeUtilization {
                requests_routed: 0,
                successful_requests: 0,
                failed_requests: 0,
                avg_response_time: Duration::from_millis(0),
                current_load: 0.0,
            });
        
        utilization.failed_requests += 1;
        utilization.requests_routed += 1;
        
        Ok(())
    }
    
    /// Update routing statistics
    async fn update_routing_stats(&self, node_id: &str, decision_time: Duration) {
        let mut stats = self.stats.write().await;
        stats.total_decisions += 1;
        
        // Update average decision time (exponential moving average)
        let alpha = 0.1;
        let new_avg = stats.avg_decision_time.as_millis() as f64 * (1.0 - alpha)
            + decision_time.as_millis() as f64 * alpha;
        stats.avg_decision_time = Duration::from_millis(new_avg as u64);
    }
    
    /// Get load balancing statistics
    pub async fn get_statistics(&self) -> LoadBalancingStats {
        let stats = self.stats.read().await;
        stats.clone()
    }
    
    /// Get node status summary
    pub async fn get_node_status(&self) -> NetworkResult<Vec<LoadBalancerNode>> {
        let nodes = self.nodes.read().await;
        Ok(songbird_errors::evolved_success(nodes.values()).cloned().collect())
    }
    
    /// Switch load balancing algorithm
    pub async fn set_algorithm(&mut self, algorithm: LoadBalancingAlgorithm) -> NetworkResult<()> {
        self.algorithm = algorithm.clone();
        info!("🔄 Switched to load balancing algorithm: {:?}", algorithm);
        Ok(())
    }
    
    /// Get current algorithm
    pub fn get_current_algorithm(&self) -> LoadBalancingAlgorithm {
        self.algorithm.clone()
    }
    
    /// Analyze and recommend optimal algorithm
    pub async fn recommend_algorithm(&self) -> NetworkResult<LoadBalancingAlgorithm> {
        let stats = self.stats.read().await;
        
        // Analyze current performance
        let success_rate = if stats.total_decisions > 0 {
            stats.successful_routes as f64 / stats.total_decisions as f64
        } else {
            1.0
        };
        
        let avg_decision_time_ms = stats.avg_decision_time.as_millis() as f64;
        
        // Recommend algorithm based on performance characteristics
        let recommendation = if success_rate < 0.95 {
            LoadBalancingAlgorithm::HealthAware // Focus on reliability
        } else if avg_decision_time_ms > 100.0 {
            LoadBalancingAlgorithm::LatencyBased // Focus on speed
        } else if stats.node_utilization.values().any(|u| u.current_load > 0.8) {
            LoadBalancingAlgorithm::ResourceBased // Focus on resource distribution
        } else {
            LoadBalancingAlgorithm::Adaptive // Use adaptive for optimal performance
        };
        
        info!(
            "💡 Algorithm recommendation: {:?} (success_rate: {:.2}, avg_decision_time: {:.2}ms)",
            recommendation, success_rate, avg_decision_time_ms
        );
        
        Ok(songbird_errors::evolved_success(recommendation))
    }
}

impl Default for NodeUtilization {
    fn default() -> Self {
        Self {
            requests_routed: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time: Duration::from_millis(0),
            current_load: 0.0,
        }
    }
} 