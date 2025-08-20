/// Federation Load Monitoring and Capacity Management
///
/// This module implements comprehensive load monitoring, capacity calculation,
/// and connection counting for federation nodes. It provides real-time metrics
/// and intelligent load balancing decisions.
use crate::messaging::{NodeHealthStatus, NodeLoadMetrics};
use crate::types::FederationNode;
use serde::{Deserialize, Serialize};
use songbird_errors::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Comprehensive load monitoring system for federation nodes
pub struct LoadMonitor {
    /// Node load history
    load_history: Arc<RwLock<HashMap<String, Vec<LoadSnapshot>>>>,

    /// Capacity calculations cache
    capacity_cache: Arc<RwLock<HashMap<String, NodeCapacity>>>,

    /// Connection tracking
    connection_tracker: Arc<RwLock<ConnectionTracker>>,

    /// Load monitoring configuration
    config: LoadMonitorConfig,

    /// Performance metrics
    metrics: Arc<RwLock<LoadMonitorMetrics>>,
}

/// Configuration for load monitoring
#[derive(Debug, Clone)]
pub struct LoadMonitorConfig {
    /// How often to collect load metrics (seconds)
    pub collection_interval_secs: u64,

    /// How many historical snapshots to keep per node
    pub history_retention_count: usize,

    /// Threshold for high load warning (percentage)
    pub high_load_threshold: f64,

    /// Threshold for critical load alert (percentage)
    pub critical_load_threshold: f64,

    /// Maximum connections per node before load balancing
    pub max_connections_per_node: u64,

    /// Capacity calculation update interval (seconds)
    pub capacity_update_interval_secs: u64,
}

impl Default for LoadMonitorConfig {
    fn default() -> Self {
        Self {
            collection_interval_secs: 30,
            history_retention_count: 100, // Keep last 100 snapshots (~50 minutes at 30s interval)
            high_load_threshold: 70.0,
            critical_load_threshold: 90.0,
            max_connections_per_node: 1000,
            capacity_update_interval_secs: 60,
        }
    }
}

/// Load snapshot at a specific point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadSnapshot {
    pub timestamp: SystemTime,
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub disk_usage_percent: f64,
    pub network_throughput_mbps: f64,
    pub active_connections: u64,
    pub request_rate_per_second: f64,
    pub response_time_ms: f64,
    pub load_score: f64, // Calculated composite load score (0-100)
}

/// Node capacity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapacity {
    pub node_id: String,
    pub max_cpu_capacity: f64,
    pub max_memory_capacity: f64,
    pub max_network_capacity: f64,
    pub max_connections: u64,
    pub current_utilization: f64,
    pub available_capacity: f64,
    pub capacity_trend: CapacityTrend,
    pub last_updated: SystemTime,
}

/// Capacity trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapacityTrend {
    Increasing,
    Stable,
    Decreasing,
    Volatile,
}

/// Connection tracking system
#[derive(Debug, Clone, Default)]
pub struct ConnectionTracker {
    /// Active connections per node
    pub connections_per_node: HashMap<String, u64>,

    /// Connection history for trend analysis
    pub connection_history: HashMap<String, Vec<ConnectionSnapshot>>,

    /// Total connections across all nodes
    pub total_connections: u64,

    /// Peak connections recorded
    pub peak_connections: u64,

    /// Connection statistics
    pub stats: ConnectionStats,
}

/// Connection snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionSnapshot {
    pub timestamp: SystemTime,
    pub active_connections: u64,
    pub new_connections_per_second: f64,
    pub closed_connections_per_second: f64,
}

/// Connection statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConnectionStats {
    pub total_connections_opened: u64,
    pub total_connections_closed: u64,
    pub average_connection_duration_ms: f64,
    pub connection_success_rate: f64,
}

/// Load monitor performance metrics
#[derive(Debug, Clone, Default)]
pub struct LoadMonitorMetrics {
    pub nodes_monitored: usize,
    pub snapshots_collected: u64,
    pub capacity_calculations: u64,
    pub load_alerts_triggered: u64,
    pub monitoring_uptime_secs: u64,
    pub last_collection_time: Option<SystemTime>,
}

impl LoadMonitor {
    /// Create new load monitor
    pub fn new(config: LoadMonitorConfig) -> Self {
        Self {
            load_history: Arc::new(RwLock::new(HashMap::new())),
            capacity_cache: Arc::new(RwLock::new(HashMap::new())),
            connection_tracker: Arc::new(RwLock::new(ConnectionTracker::default())),
            config,
            metrics: Arc::new(RwLock::new(LoadMonitorMetrics::default())),
        }
    }

    /// Start load monitoring for a node
    pub async fn start_monitoring(&self) -> SongbirdResult<()> {
        let node_id = node.cluster_name.to_string();
        info!("🔍 Starting load monitoring for node: {}", node_id);

        // Initialize load history
        {
            let mut history = self.load_history.write().await;
            history.insert(node_id.clone(), Vec::new());
        }

        // Initialize capacity tracking
        {
            let mut capacity_cache = self.capacity_cache.write().await;
            capacity_cache.insert(
                node_id.clone(),
                NodeCapacity {
                    node_id: node_id.clone(),
                    max_cpu_capacity: 100.0,
                    max_memory_capacity: 100.0,
                    max_network_capacity: 1000.0, // Mbps
                    max_connections: self.config.max_connections_per_node,
                    current_utilization: 0.0,
                    available_capacity: 100.0,
                    capacity_trend: CapacityTrend::Stable,
                    last_updated: SystemTime::now(),
                },
            );
        }

        // Initialize connection tracking
        {
            let mut tracker = self.connection_tracker.write().await;
            tracker.connections_per_node.insert(node_id.clone(), 0);
            tracker.connection_history.insert(node_id, Vec::new());
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.nodes_monitored += 1;
        }

        Ok(())
    }

    /// Stop monitoring a node
    pub async fn stop_monitoring(&self) -> SongbirdResult<()> {
        info!("🛑 Stopping load monitoring for node: {}", node_id);

        // Remove from all tracking structures
        {
            let mut history = self.load_history.write().await;
            history.remove(node_id);
        }

        {
            let mut capacity_cache = self.capacity_cache.write().await;
            capacity_cache.remove(node_id);
        }

        {
            let mut tracker = self.connection_tracker.write().await;
            if let Some(connections) = tracker.connections_per_node.remove(node_id) {
                tracker.total_connections = tracker.total_connections.saturating_sub(connections);
            }
            tracker.connection_history.remove(node_id);
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.nodes_monitored = metrics.nodes_monitored.saturating_sub(1);
        }

        Ok(())
    }

    /// Record load metrics for a node
    pub async fn record_load_metrics(&self) -> SongbirdResult<()> {
        let timestamp = SystemTime::now();

        // Calculate composite load score
        let load_score = self.calculate_load_score(&metrics);

        let snapshot = LoadSnapshot {
            timestamp,
            cpu_usage_percent: metrics.cpu_usage_percent,
            memory_usage_percent: metrics.memory_usage_percent,
            disk_usage_percent: 0.0,      // Not provided in NodeLoadMetrics
            network_throughput_mbps: 0.0, // Not provided in NodeLoadMetrics
            active_connections: metrics.active_connections,
            request_rate_per_second: metrics.request_rate_per_second,
            response_time_ms: metrics.response_time_ms,
            load_score,
        };

        // Store in history
        {
            let mut history = self.load_history.write().await;
            if let Some(node_history) = history.get_mut(node_id) {
                node_history.push(snapshot.clone());

                // Maintain history size limit
                if node_history.len() > self.config.history_retention_count {
                    node_history.remove(0);
                }
            }
        }

        // Update capacity calculations
        self.update_node_capacity(node_id, &snapshot).await?;

        // Check for load alerts
        self.check_load_alerts(node_id, &snapshot).await?;

        // Update metrics
        {
            let mut monitor_metrics = self.metrics.write().await;
            monitor_metrics.snapshots_collected += 1;
            monitor_metrics.last_collection_time = Some(timestamp);
        }

        debug!(
            "📊 Recorded load metrics for {}: load_score={:.1}",
            node_id, load_score
        );
        Ok(())
    }

    /// Calculate composite load score (0-100)
    fn calculate_load_score(&self, metrics: &NodeLoadMetrics) -> f64 {
        // Weighted average of different metrics
        let cpu_weight = 0.35;
        let memory_weight = 0.30;
        let connections_weight = 0.20;
        let response_time_weight = 0.15;

        // Normalize connection count (assume max 1000 connections = 100% load)
        let connection_load = (metrics.active_connections as f64
            / self.config.max_connections_per_node as f64
            * 100.0)
            .min(100.0);

        // Normalize response time (assume 1000ms = 100% load)
        let response_time_load = (metrics.response_time_ms / 10.0).min(100.0);

        let composite_score = (metrics.cpu_usage_percent * cpu_weight)
            + (metrics.memory_usage_percent * memory_weight)
            + (connection_load * connections_weight)
            + (response_time_load * response_time_weight);

        composite_score.min(100.0)
    }

    /// Update node capacity calculations
    async fn update_node_capacity(&self) -> SongbirdResult<()> {
        let mut capacity_cache = self.capacity_cache.write().await;

        if let Some(capacity) = capacity_cache.get_mut(node_id) {
            // Update current utilization
            capacity.current_utilization = snapshot.load_score;
            capacity.available_capacity = (100.0 - snapshot.load_score).max(0.0);
            capacity.last_updated = snapshot.timestamp;

            // Calculate capacity trend
            capacity.capacity_trend = self.calculate_capacity_trend(node_id).await;

            debug!(
                "🧮 Updated capacity for {}: utilization={:.1}%, available={:.1}%",
                node_id, capacity.current_utilization, capacity.available_capacity
            );
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.capacity_calculations += 1;
        }

        Ok(())
    }

    /// Calculate capacity trend based on historical data
    async fn calculate_capacity_trend(&self) -> CapacityTrend {
        let history = self.load_history.read().await;

        if let Some(node_history) = history.get(node_id) {
            if node_history.len() < 5 {
                return CapacityTrend::Stable; // Not enough data
            }

            // Look at last 5 snapshots to determine trend
            let recent = &node_history[node_history.len().saturating_sub(5)..];
            let load_scores: Vec<f64> = recent.iter().map(|s| s.load_score).collect();

            // Calculate trend direction
            let first_half_avg = load_scores[..2].iter().sum::<f64>() / 2.0;
            let second_half_avg = load_scores[3..].iter().sum::<f64>() / 2.0;
            let trend_difference = second_half_avg - first_half_avg;

            // Calculate volatility
            let avg_load = load_scores.iter().sum::<f64>() / load_scores.len() as f64;
            let variance = load_scores
                .data
                .iter()
                .map(|score| (score - avg_load).powi(2))
                .sum::<f64>()
                / load_scores.len() as f64;
            let volatility = variance.sqrt();

            // Determine trend
            if volatility > 15.0 {
                CapacityTrend::Volatile
            } else if trend_difference > 5.0 {
                CapacityTrend::Increasing
            } else if trend_difference < -5.0 {
                CapacityTrend::Decreasing
            } else {
                CapacityTrend::Stable
            }
        } else {
            CapacityTrend::Stable
        }
    }

    /// Check for load alerts and trigger warnings
    async fn check_load_alerts(&self) -> SongbirdResult<()> {
        if snapshot.load_score >= self.config.critical_load_threshold {
            warn!(
                "🚨 CRITICAL LOAD ALERT: Node {} load at {:.1}% (threshold: {:.1}%)",
                node_id, snapshot.load_score, self.config.critical_load_threshold
            );

            // Update metrics
            let mut metrics = self.metrics.write().await;
            metrics.load_alerts_triggered += 1;
        } else if snapshot.load_score >= self.config.high_load_threshold {
            warn!(
                "⚠️ HIGH LOAD WARNING: Node {} load at {:.1}% (threshold: {:.1}%)",
                node_id, snapshot.load_score, self.config.high_load_threshold
            );
        }

        Ok(())
    }

    /// Record connection count for a node
    pub async fn record_connection_count(&self) -> SongbirdResult<()> {
        let timestamp = SystemTime::now();

        {
            let mut tracker = self.connection_tracker.write().await;

            // Update current connection count
            let old_count = tracker
                .connections_per_node
                .insert(node_id.to_string(), connection_count)
                .unwrap_or(0);

            // Update total connections
            tracker.total_connections = tracker.total_connections + connection_count - old_count;

            // Update peak connections
            if connection_count > tracker.peak_connections {
                tracker.peak_connections = connection_count;
            }

            // Add to connection history
            if let Some(history) = tracker.connection_history.get_mut(node_id) {
                history.push(ConnectionSnapshot {
                    timestamp,
                    active_connections: connection_count,
                    new_connections_per_second: 0.0, // Would need rate calculation
                    closed_connections_per_second: 0.0, // Would need rate calculation
                });

                // Maintain history size
                if history.len() > self.config.history_retention_count {
                    history.remove(0);
                }
            }
        }

        debug!(
            "🔗 Recorded {} connections for node {}",
            connection_count, node_id
        );
        Ok(())
    }

    /// Get current load status for a node
    pub async fn get_node_load_status(&self) -> SongbirdResult<()> {let history = self.load_history.read().await;

        if let Some(node_history) = history.get(node_id) {
            Ok(songbird_errors::node_history.last()).cloned()))
        } else {
            Ok(songbird_errors::None)))
        }
    }

    /// Get node capacity information
    pub async fn get_node_capacity(&self) -> SongbirdResult<()> {let capacity_cache = self.capacity_cache.read().await;
        Ok(songbird_errors::capacity_cache.get(node_id)).cloned()))
    }

    /// Get connection count for a node
    pub async fn get_connection_count(&self) -> SongbirdResult<u64> {
        let tracker = self.connection_tracker.read().await;
        Ok(songbird_errors::
            tracker
                .connections_per_node
                .get(node_id))
                .copied()
                .unwrap_or(0),
        ))
    }

    /// Get total connection count across all nodes
    pub async fn get_total_connections(&self) -> u64 {
        let tracker = self.connection_tracker.read().await;
        tracker.total_connections
    }

    /// Get load monitoring metrics
    pub async fn get_metrics(&self) -> LoadMonitorMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Get nodes that are currently overloaded
    pub async fn get_overloaded_nodes(&self) -> SongbirdResult<()> {let capacity_cache = self.capacity_cache.read().await;
        let overloaded: Vec<String> = capacity_cache
            .data
            .iter()
            .filter(|(_, capacity)| capacity.current_utilization >= self.config.high_load_threshold)
            .map(|(node_id, _)| node_id.clone())
            .collect();

        Ok(songbird_errors::overloaded)))
    }

    /// Get available nodes for load balancing
    ///
    /// # Errors
    /// Returns error if node discovery fails
    pub async fn get_available_nodes(&self) -> SongbirdResult<Vec<(String, f64)>> {
        let capacity_cache = self.capacity_cache.read().await;
        let mut available: Vec<(String, f64)> = capacity_cache
            .data
            .iter()
            .filter(|(_, capacity)| capacity.available_capacity > 20.0) // At least 20% available
            .map(|(node_id, capacity)| (node_id.clone(), capacity.available_capacity))
            .collect();

        // Sort by available capacity (highest first)
        available.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(songbird_errors::available)))
    }
}
