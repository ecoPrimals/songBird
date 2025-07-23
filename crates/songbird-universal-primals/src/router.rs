//! Universal Primal Router
//!
//! This module provides intelligent routing and load balancing for Universal Primals
//! with performance optimization, circuit breakers, and failover capabilities.

use crate::discovery::PrimalDiscoveryEngine;
use crate::{PrimalCapability, PrimalContext};
use songbird_config::hardcoded_elimination::PrimalConfig;
use songbird_errors::{Result, ServiceError};
use songbird_universal::PrimalType;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// A primal node available for routing
#[derive(Debug, Clone)]
pub struct PrimalNode {
    /// Unique identifier for the primal node
    pub id: String,
    /// Human-readable name of the primal
    pub name: String,
    /// Network endpoint URL
    pub endpoint: String,
    /// Type classification of the primal
    pub primal_type: PrimalType,
    /// Capabilities offered by this primal
    pub capabilities: Vec<PrimalCapability>,
    /// Current health status
    pub health_status: PrimalHealth,
    /// Timestamp of last successful communication
    pub last_seen: chrono::DateTime<chrono::Utc>,
    /// Version string of the primal software
    pub version: String,
    /// Additional metadata key-value pairs
    pub metadata: HashMap<String, String>,
}

/// Health status of a primal node
#[derive(Debug, Clone, PartialEq)]
pub enum PrimalHealth {
    /// Node is healthy and operating normally
    Healthy,
    /// Node is degraded but still functional
    Degraded,
    /// Node is unhealthy and should not receive traffic
    Unhealthy,
    /// Health status is unknown
    Unknown,
}

/// Metrics for primal nodes
#[derive(Debug, Clone)]
pub struct PrimalMetrics {
    /// Average response time in milliseconds
    pub response_time_ms: f64,
    /// Success rate as a percentage (0.0 to 1.0)
    pub success_rate: f64,
    /// Throughput in requests per second
    pub throughput: f64,
    /// Total count of errors
    pub error_count: u64,
    /// When these metrics were last updated
    pub last_updated: Instant,
}

/// Universal Primal Router with intelligent routing and load balancing
pub struct UniversalPrimalRouter {
    _config: PrimalConfig,
    _discovery_engine: PrimalDiscoveryEngine,
    active_primals: Arc<RwLock<HashMap<String, PrimalNode>>>,
    _metrics: Arc<RwLock<HashMap<String, PrimalMetrics>>>,
    _failover_manager: FailoverManager,
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
    load_balancer: LoadBalancer,
    performance_metrics: Arc<RwLock<HashMap<String, PerformanceMetrics>>>,
}

/// Performance metrics for a primal node
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Average response time in milliseconds
    pub response_time_ms: f64,
    /// Success rate as a percentage (0.0 to 1.0)
    pub success_rate: f64,
    /// Throughput in requests per second
    pub throughput: f64,
    /// Total count of errors
    pub error_count: u64,
    /// When these metrics were last updated
    pub last_updated: Instant,
}

/// Circuit breaker for managing failing primal nodes
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    /// Current state of the circuit breaker
    pub state: CircuitState,
    /// Number of consecutive failures
    pub failure_count: u32,
    /// Timestamp of the last failure
    pub last_failure: Option<Instant>,
    /// When to attempt the next retry
    pub next_retry: Option<Instant>,
    /// Failure threshold before opening circuit
    pub threshold: u32,
    /// Timeout duration for circuit breaker
    pub timeout: Duration,
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed, allowing traffic
    Closed,
    /// Circuit is open, blocking traffic
    Open,
    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

/// Load balancer for distributing requests
#[derive(Debug, Clone)]
pub struct LoadBalancer {
    /// Load balancing strategy to use
    pub strategy: LoadBalancingStrategy,
    /// Weight values for weighted algorithms
    pub weights: HashMap<String, f64>,
}

/// Load balancing strategies
#[derive(Debug, Clone)]
pub enum LoadBalancingStrategy {
    /// Simple round-robin distribution
    RoundRobin,
    /// Weighted round-robin distribution
    WeightedRoundRobin,
    /// Route to node with least connections
    LeastConnections,
    /// Route based on latency metrics
    LatencyBased,
    /// Random selection
    Random,
}

/// Failover manager for handling node failures
#[derive(Debug, Clone)]
pub struct FailoverManager {
    /// Backup nodes for each primary node
    pub backup_nodes: HashMap<String, Vec<String>>,
    /// Timeout for failover operations
    pub failover_timeout: Duration,
}

/// Routing request with context and preferences
#[derive(Debug, Clone)]
pub struct RoutingRequest {
    /// Unique identifier for this request
    pub request_id: String,
    /// Required primal type
    pub primal_type: PrimalType,
    /// Required capabilities for the primal
    pub required_capabilities: Vec<PrimalCapability>,
    /// Preferred primal type if available
    pub preferred_type: Option<PrimalType>,
    /// Additional context for routing decision
    pub context: Option<PrimalContext>,
    /// Request timeout duration
    pub timeout: Duration,
    /// Number of retries allowed
    pub retry_count: u32,
    /// Priority level of this request
    pub priority: Priority,
}

/// Request priority levels
#[derive(Debug, Clone, PartialEq)]
pub enum Priority {
    /// Critical priority request
    Critical,
    /// High priority request
    High,
    /// Normal priority request
    Normal,
    /// Low priority request
    Low,
}

/// Routing response with selected primal node
#[derive(Debug, Clone)]
pub struct RoutingResponse {
    /// The selected primal node
    pub selected_node: PrimalNode,
    /// Details about the routing decision
    pub routing_decision: RoutingDecision,
    /// Estimated latency for this route
    pub estimated_latency: Duration,
    /// Confidence score for this routing decision
    pub confidence_score: f64,
}

/// Routing decision details
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    /// Load balancing strategy that was used
    pub strategy_used: LoadBalancingStrategy,
    /// Alternative nodes that were considered
    pub alternatives_considered: Vec<String>,
    /// Factors that influenced the decision
    pub decision_factors: HashMap<String, f64>,
}

impl UniversalPrimalRouter {
    /// Create a new router instance
    pub fn new(config: PrimalConfig) -> Self {
        Self {
            _config: config,
            _discovery_engine: PrimalDiscoveryEngine::new(PrimalConfig::default()),
            active_primals: Arc::new(RwLock::new(HashMap::new())),
            _metrics: Arc::new(RwLock::new(HashMap::new())),
            _failover_manager: FailoverManager {
                backup_nodes: HashMap::new(),
                failover_timeout: Duration::from_secs(30),
            },
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            load_balancer: LoadBalancer {
                strategy: LoadBalancingStrategy::RoundRobin,
                weights: HashMap::new(),
            },
            performance_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a primal node in the routing table
    pub async fn register_node(&self, node: PrimalNode) -> Result<()> {
        let mut routing_table = self.active_primals.write().await;
        let node_id = node.id.clone();
        let node_name = node.name.clone(); // Store name before moving node

        // Initialize performance metrics
        let metrics = PerformanceMetrics {
            response_time_ms: 0.0,
            success_rate: 1.0,
            throughput: 0.0,
            error_count: 0,
            last_updated: Instant::now(),
        };

        // Initialize circuit breaker
        let circuit_breaker = CircuitBreaker {
            state: CircuitState::Closed,
            failure_count: 0,
            last_failure: None,
            next_retry: None,
            threshold: 5,
            timeout: Duration::from_secs(60),
        };

        routing_table.insert(node_id.clone(), node);

        let mut performance_metrics = self.performance_metrics.write().await;
        performance_metrics.insert(node_id.clone(), metrics);

        let mut circuit_breakers = self.circuit_breakers.write().await;
        circuit_breakers.insert(node_id.clone(), circuit_breaker);

        info!("✅ Registered primal node: {} ({})", node_id, node_name);
        Ok(())
    }

    /// Route a request to the best available primal node
    pub async fn route_request(&self, request: RoutingRequest) -> Result<RoutingResponse> {
        debug!(
            "Routing request {} for primal type: {:?}",
            request.request_id, request.primal_type
        );

        // Get eligible nodes
        let eligible_nodes = self.get_eligible_nodes(&request).await?;

        if eligible_nodes.is_empty() {
            return Err(songbird_errors::SongbirdError::Service(Box::new(
                ServiceError {
                    service: format!("{:?}", request.primal_type),
                    message: "No eligible primal nodes available".to_string(),
                    status: Some("no_nodes_available".to_string()),
                    suggestion: Some("Check primal node health and registration".to_string()),
                },
            )));
        }

        // Select best node using load balancing strategy
        let selected_node = self.select_best_node(&eligible_nodes, &request).await?;

        // Create routing decision
        let routing_decision = RoutingDecision {
            strategy_used: self.load_balancer.strategy.clone(),
            alternatives_considered: eligible_nodes.iter().map(|n| n.id.clone()).collect(),
            decision_factors: self
                .calculate_decision_factors(&eligible_nodes, &selected_node)
                .await,
        };

        // Estimate latency
        let estimated_latency = self
            .estimate_latency(&selected_node)
            .await
            .unwrap_or(Duration::from_millis(100));

        // Calculate confidence score
        let confidence_score = self.calculate_confidence_score(&selected_node).await;

        let response = RoutingResponse {
            selected_node: selected_node.clone(),
            routing_decision,
            estimated_latency,
            confidence_score,
        };

        info!(
            "✅ Routed request {} to node: {}",
            request.request_id, selected_node.id
        );
        Ok(response)
    }

    /// Get eligible nodes for a routing request
    async fn get_eligible_nodes(&self, request: &RoutingRequest) -> Result<Vec<PrimalNode>> {
        let routing_table = self.active_primals.read().await;
        let circuit_breakers = self.circuit_breakers.read().await;

        let mut eligible_nodes = Vec::new();

        for (node_id, node) in routing_table.iter() {
            // Check primal type match
            if node.primal_type != request.primal_type {
                continue;
            }

            // Check required capabilities
            let has_required_capabilities =
                request.required_capabilities.iter().all(|required_cap| {
                    node.capabilities
                        .iter()
                        .any(|node_cap| capabilities_match(required_cap, node_cap))
                });

            if !has_required_capabilities {
                continue;
            }

            // Check circuit breaker state
            if let Some(circuit_breaker) = circuit_breakers.get(node_id) {
                if circuit_breaker.state == CircuitState::Open {
                    // Check if it's time to retry
                    if let Some(next_retry) = circuit_breaker.next_retry {
                        if Instant::now() < next_retry {
                            continue;
                        }
                    }
                }
            }

            // Check health status
            if node.health_status == PrimalHealth::Unhealthy {
                continue;
            }

            eligible_nodes.push(node.clone());
        }

        Ok(eligible_nodes)
    }

    /// Select the best node from eligible nodes
    async fn select_best_node(
        &self,
        eligible_nodes: &[PrimalNode],
        request: &RoutingRequest,
    ) -> Result<PrimalNode> {
        if eligible_nodes.is_empty() {
            return Err(songbird_errors::SongbirdError::Service(Box::new(
                ServiceError {
                    service: format!("{}", request.primal_type),
                    message: "No eligible nodes available".to_string(),
                    status: Some("no_nodes_available".to_string()),
                    suggestion: Some("Check primal node health and registration".to_string()),
                },
            )));
        }

        match self.load_balancer.strategy {
            LoadBalancingStrategy::LatencyBased => self.select_by_latency(eligible_nodes).await,
            LoadBalancingStrategy::RoundRobin => self.select_round_robin(eligible_nodes).await,
            LoadBalancingStrategy::WeightedRoundRobin => {
                self.select_weighted_round_robin(eligible_nodes).await
            }
            LoadBalancingStrategy::LeastConnections => {
                self.select_least_connections(eligible_nodes).await
            }
            LoadBalancingStrategy::Random => self.select_random(eligible_nodes).await,
        }
    }

    /// Select node with lowest latency
    async fn select_by_latency(&self, nodes: &[PrimalNode]) -> Result<PrimalNode> {
        let performance_metrics = self.performance_metrics.read().await;

        let mut best_node = &nodes[0];
        let mut best_latency = f64::MAX;

        for node in nodes {
            if let Some(metrics) = performance_metrics.get(&node.id) {
                if metrics.response_time_ms < best_latency {
                    best_latency = metrics.response_time_ms;
                    best_node = node;
                }
            }
        }

        Ok(best_node.clone())
    }

    /// Select node using round robin
    async fn select_round_robin(&self, nodes: &[PrimalNode]) -> Result<PrimalNode> {
        // Simplified round robin - in real implementation would track counter
        let index = (Instant::now().elapsed().as_secs() % nodes.len() as u64) as usize;
        Ok(nodes[index].clone())
    }

    /// Select node using weighted round robin
    async fn select_weighted_round_robin(&self, nodes: &[PrimalNode]) -> Result<PrimalNode> {
        // For simplicity, fall back to regular round robin
        self.select_round_robin(nodes).await
    }

    /// Select node with least connections
    async fn select_least_connections(&self, nodes: &[PrimalNode]) -> Result<PrimalNode> {
        // For simplicity, use latency-based selection
        self.select_by_latency(nodes).await
    }

    /// Select random node
    async fn select_random(&self, nodes: &[PrimalNode]) -> Result<PrimalNode> {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        let selected = nodes.choose(&mut rng).ok_or_else(|| {
            songbird_errors::SongbirdError::from(ServiceError::new(
                "router",
                "No nodes available for random selection",
            ))
        })?;
        Ok(selected.clone())
    }

    /// Calculate decision factors for routing decision
    async fn calculate_decision_factors(
        &self,
        _nodes: &[PrimalNode],
        selected: &PrimalNode,
    ) -> HashMap<String, f64> {
        let mut factors = HashMap::new();

        // Calculate relative performance metrics
        let performance_metrics = self.performance_metrics.read().await;

        if let Some(metrics) = performance_metrics.get(&selected.id) {
            factors.insert("response_time".to_string(), metrics.response_time_ms);
            factors.insert("success_rate".to_string(), metrics.success_rate);
            factors.insert("throughput".to_string(), metrics.throughput);
        }

        // Add health score
        let health_score = match selected.health_status {
            PrimalHealth::Healthy => 1.0,
            PrimalHealth::Degraded => 0.7,
            PrimalHealth::Unhealthy => 0.3,
            PrimalHealth::Unknown => 0.5,
        };
        factors.insert("health_score".to_string(), health_score);

        // Add capability score
        let capability_score = selected.capabilities.len() as f64 / 10.0;
        factors.insert("capability_score".to_string(), capability_score);

        factors
    }

    /// Estimate latency for a node
    async fn estimate_latency(&self, node: &PrimalNode) -> Option<Duration> {
        let performance_metrics = self.performance_metrics.read().await;

        performance_metrics
            .get(&node.id)
            .map(|metrics| Duration::from_millis(metrics.response_time_ms as u64))
    }

    /// Calculate confidence score for a node
    async fn calculate_confidence_score(&self, node: &PrimalNode) -> f64 {
        let performance_metrics = self.performance_metrics.read().await;

        if let Some(metrics) = performance_metrics.get(&node.id) {
            // Combine multiple factors for confidence
            let latency_score = if metrics.response_time_ms < 100.0 {
                1.0
            } else {
                0.5
            };
            let success_score = metrics.success_rate;
            let health_score = match node.health_status {
                PrimalHealth::Healthy => 1.0,
                PrimalHealth::Degraded => 0.7,
                _ => 0.3,
            };

            (latency_score + success_score + health_score) / 3.0
        } else {
            0.5 // Default medium confidence
        }
    }

    /// Update performance metrics for a node
    pub async fn update_metrics(&self, node_id: &str, response_time: Duration, success: bool) {
        let mut performance_metrics = self.performance_metrics.write().await;

        if let Some(metrics) = performance_metrics.get_mut(node_id) {
            // Update response time (exponential moving average)
            let alpha = 0.1;
            metrics.response_time_ms =
                (1.0 - alpha) * metrics.response_time_ms + alpha * response_time.as_millis() as f64;

            // Update success rate
            let current_success_rate = if success { 1.0 } else { 0.0 };
            metrics.success_rate =
                (1.0 - alpha) * metrics.success_rate + alpha * current_success_rate;

            // Update error count
            if !success {
                metrics.error_count += 1;
            }

            metrics.last_updated = Instant::now();
        }
    }

    /// Handle node failure and update circuit breaker
    pub async fn handle_node_failure(&self, node_id: &str) {
        let mut circuit_breakers = self.circuit_breakers.write().await;

        if let Some(circuit_breaker) = circuit_breakers.get_mut(node_id) {
            circuit_breaker.failure_count += 1;
            circuit_breaker.last_failure = Some(Instant::now());

            // Open circuit breaker if threshold is reached
            if circuit_breaker.failure_count >= circuit_breaker.threshold {
                circuit_breaker.state = CircuitState::Open;
                circuit_breaker.next_retry = Some(Instant::now() + circuit_breaker.timeout);
                warn!("Circuit breaker opened for node: {}", node_id);
            }
        }
    }

    /// Get routing statistics
    pub async fn get_routing_stats(&self) -> HashMap<String, serde_json::Value> {
        let routing_table = self.active_primals.read().await;
        let performance_metrics = self.performance_metrics.read().await;
        let circuit_breakers = self.circuit_breakers.read().await;

        let mut stats = HashMap::new();

        // Basic statistics
        stats.insert(
            "total_nodes".to_string(),
            serde_json::Value::Number(routing_table.len().into()),
        );
        stats.insert(
            "healthy_nodes".to_string(),
            serde_json::Value::Number(
                routing_table
                    .values()
                    .filter(|n| n.health_status == PrimalHealth::Healthy)
                    .count()
                    .into(),
            ),
        );

        // Circuit breaker statistics
        let open_circuits = circuit_breakers
            .values()
            .filter(|cb| cb.state == CircuitState::Open)
            .count();
        stats.insert(
            "open_circuits".to_string(),
            serde_json::Value::Number(open_circuits.into()),
        );

        // Performance statistics
        if !performance_metrics.is_empty() {
            let avg_response_time = performance_metrics
                .values()
                .map(|m| m.response_time_ms)
                .sum::<f64>()
                / performance_metrics.len() as f64;
            stats.insert(
                "avg_response_time_ms".to_string(),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(avg_response_time)
                        .unwrap_or(serde_json::Number::from(0)),
                ),
            );
        }

        stats
    }
}

/// Helper function to check if capabilities match
fn capabilities_match(required: &PrimalCapability, available: &PrimalCapability) -> bool {
    matches!(
        (required, available),
        (
            PrimalCapability::Encryption { .. },
            PrimalCapability::Encryption { .. }
        ) | (
            PrimalCapability::FileSystem { .. },
            PrimalCapability::FileSystem { .. }
        ) | (
            PrimalCapability::ContainerRuntime { .. },
            PrimalCapability::ContainerRuntime { .. }
        ) | (
            PrimalCapability::ModelInference { .. },
            PrimalCapability::ModelInference { .. }
        ) | (
            PrimalCapability::Authentication { .. },
            PrimalCapability::Authentication { .. }
        ) | (
            PrimalCapability::ObjectStorage { .. },
            PrimalCapability::ObjectStorage { .. }
        ) | (
            PrimalCapability::ServerlessExecution { .. },
            PrimalCapability::ServerlessExecution { .. }
        ) | (
            PrimalCapability::MachineLearning { .. },
            PrimalCapability::MachineLearning { .. }
        )
    )
}
