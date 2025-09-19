use crate::load_monitoring::{LoadMonitor, LoadMonitorConfig};
/// Federation Background Heartbeat System
///
/// This module implements a comprehensive background heartbeat system that maintains
/// federation node connectivity, monitors health status, and provides automatic
/// failover detection and recovery.
use crate::messaging::{FederationMessage, FederationMessenger, NodeLoadMetrics};
use crate::types::FederationNode;
use serde::{Deserialize, Serialize};
use songbird_errors::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{RwLock, broadcast};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

/// Heartbeat system for federation health monitoring
pub struct HeartbeatSystem {
    /// System configuration
    config: HeartbeatConfig,

    /// Federation messenger for sending heartbeats
    messenger: Arc<FederationMessenger>,

    /// Load monitor for collecting metrics
    load_monitor: Arc<LoadMonitor>,

    /// Node health status tracking
    node_health: Arc<RwLock<HashMap<String, NodeHealthInfo>>>,

    /// Background task handles
    task_handles: Arc<RwLock<Vec<JoinHandle<()>>>>,

    /// Heartbeat statistics
    stats: Arc<RwLock<HeartbeatStats>>,

    /// Health event broadcaster
    health_events_tx: broadcast::Sender<HealthEvent>,

    /// System running state
    running: Arc<RwLock<bool>>,
}

/// Heartbeat system configuration
#[derive(Debug, Clone)]
pub struct HeartbeatConfig {
    /// Heartbeat interval in seconds
    pub heartbeat_interval_secs: u64,

    /// Timeout for considering a node offline (seconds)
    pub node_timeout_secs: u64,

    /// Number of missed heartbeats before marking node as degraded
    pub degraded_threshold: u32,

    /// Number of missed heartbeats before marking node as offline
    pub offline_threshold: u32,

    /// Health check interval in seconds
    pub health_check_interval_secs: u64,

    /// Maximum concurrent health checks
    pub max_concurrent_health_checks: usize,

    /// Enable automatic failover
    pub enable_automatic_failover: bool,

    /// Failover detection threshold (seconds)
    pub failover_detection_threshold_secs: u64,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval_secs: 15,            // Send heartbeat every 15 seconds
            node_timeout_secs: 60,                  // Consider node offline after 60 seconds
            degraded_threshold: 2,                  // 2 missed heartbeats = degraded
            offline_threshold: 4,                   // 4 missed heartbeats = offline
            health_check_interval_secs: 30,         // Health check every 30 seconds
            max_concurrent_health_checks: 10,       // Max 10 concurrent checks
            enable_automatic_failover: true,        // Enable automatic failover
            failover_detection_threshold_secs: 120, // Failover after 2 minutes
        }
    }
}

/// Node health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHealthInfo {
    pub node_id: String,
    pub status: NodeStatus,
    pub last_heartbeat: SystemTime,
    pub last_seen: SystemTime,
    pub consecutive_failures: u32,
    pub total_heartbeats_received: u64,
    pub total_heartbeats_missed: u64,
    pub average_response_time_ms: f64,
    pub load_metrics: Option<NodeLoadMetrics>,
    pub health_score: f64, // 0-100, calculated from various factors
}

/// Node status in the federation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeStatus {
    /// Node is healthy and responsive
    Healthy,
    /// Node is responding but with degraded performance
    Degraded { reason: String },
    /// Node is not responding but recently seen
    Unresponsive,
    /// Node is considered offline
    Offline,
    /// Node has been removed from federation
    Removed,
}

/// Health event notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthEvent {
    /// Node came online
    NodeOnline {
        node_id: String,
        timestamp: SystemTime,
    },
    /// Node became degraded
    NodeDegraded {
        node_id: String,
        reason: String,
        timestamp: SystemTime,
    },
    /// Node became unresponsive
    NodeUnresponsive {
        node_id: String,
        timestamp: SystemTime,
    },
    /// Node went offline
    NodeOffline {
        node_id: String,
        timestamp: SystemTime,
    },
    /// Node was removed from federation
    NodeRemoved {
        node_id: String,
        timestamp: SystemTime,
    },
    /// Automatic failover triggered
    FailoverTriggered {
        failed_node: String,
        backup_nodes: Vec<String>,
        timestamp: SystemTime,
    },
}

/// Heartbeat system statistics
#[derive(Debug, Clone, Default)]
pub struct HeartbeatStats {
    pub total_heartbeats_sent: u64,
    pub total_heartbeats_received: u64,
    pub nodes_currently_healthy: usize,
    pub nodes_currently_degraded: usize,
    pub nodes_currently_offline: usize,
    pub health_events_triggered: u64,
    pub failovers_triggered: u64,
    pub uptime_seconds: u64,
    pub last_heartbeat_sent: Option<SystemTime>,
    pub last_health_check: Option<SystemTime>,
}

impl HeartbeatSystem {
    /// Create new heartbeat system
    pub fn new(
        config: HeartbeatConfig,
        messenger: Arc<FederationMessenger>,
        load_monitor: Arc<LoadMonitor>,
    ) -> Self {
        let (health_events_tx, _) = broadcast::channel(1000);

        Self {
            config,
            messenger,
            load_monitor,
            node_health: Arc::new(RwLock::new(HashMap::new())),
            task_handles: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(HeartbeatStats::default())),
            health_events_tx,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the heartbeat system
    pub async fn start(&self) -> SongbirdResult<()> {
        info!("💓 Starting federation heartbeat system");

        {
            let mut running = self.running.write().await;
            if *running {
                return Err(songbird_errors::SongbirdError::Federation { 
                    service: "federation".to_string(), 
                    message: "Heartbeat system already running".to_string(), 
                    peer: None, 
                    recovery_actions: vec!["retry_operation".to_string(), "check_network".to_string()] 
                });
            }
            *running = true;
        }

        // Start heartbeat sender task
        self.start_heartbeat_sender().await?;

        // Start health monitoring task
        self.start_health_monitor().await?;

        // Start failover detection task
        if self.config.enable_automatic_failover {
            self.start_failover_monitor().await?;
        }

        // Start statistics updater
        self.start_stats_updater().await?;

        info!("✅ Federation heartbeat system started successfully");
        Ok(())
    }

    /// Stop the heartbeat system
    pub async fn stop(&self) -> SongbirdResult<()> {
        info!("🛑 Stopping federation heartbeat system");

        {
            let mut running = self.running.write().await;
            *running = false;
        }

        // Cancel all background tasks
        {
            let mut handles = self.task_handles.write().await;
            for handle in handles.drain(..) {
                handle.abort();
            }
        }

        info!("✅ Federation heartbeat system stopped");
        Ok(())
    }

    /// Register a node for heartbeat monitoring
    pub async fn register_node(&self) -> SongbirdResult<()> {
        let node_id = node.cluster_name.to_string();

        let health_info = NodeHealthInfo {
            node_id: node_id.clone(),
            status: NodeStatus::Healthy,
            last_heartbeat: SystemTime::now(),
            last_seen: SystemTime::now(),
            consecutive_failures: 0,
            total_heartbeats_received: 0,
            total_heartbeats_missed: 0,
            average_response_time_ms: 0.0,
            load_metrics: None,
            health_score: 100.0,
        };

        {
            let mut node_health = self.node_health.write().await;
            node_health.insert(node_id.clone(), health_info);
        }

        // Trigger health event
        let event = HealthEvent::NodeOnline {
            node_id: node_id.clone(),
            timestamp: SystemTime::now(),
        };
        let _ = self.health_events_tx.send(event);

        info!("📝 Registered node for heartbeat monitoring: {}", node_id);
        Ok(())
    }

    /// Unregister a node from heartbeat monitoring
    pub async fn unregister_node(&self) -> SongbirdResult<()> {
        {
            let mut node_health = self.node_health.write().await;
            node_health.remove(node_id);
        }

        // Trigger health event
        let event = HealthEvent::NodeRemoved {
            node_id: node_id.to_string(),
            timestamp: SystemTime::now(),
        };
        let _ = self.health_events_tx.send(event);

        info!(
            "🗑️ Unregistered node from heartbeat monitoring: {}",
            node_id
        );
        Ok(())
    }

    /// Process received heartbeat from a node
    pub async fn process_heartbeat(&self) -> SongbirdResult<()> {
        let now = SystemTime::now();

        {
            let mut node_health = self.node_health.write().await;
            if let Some(health_info) = node_health.get_mut(node_id) {
                // Update heartbeat information
                health_info.last_heartbeat = now;
                health_info.last_seen = now;
                health_info.total_heartbeats_received += 1;
                health_info.consecutive_failures = 0;
                health_info.load_metrics = Some(load_metrics.clone());

                // Calculate health score based on load metrics
                health_info.health_score = self.calculate_health_score(&load_metrics);

                // Update status based on health score
                let new_status = if health_info.health_score >= 80.0 {
                    NodeStatus::Healthy
                } else if health_info.health_score >= 50.0 {
                    NodeStatus::Degraded {
                        reason: format!("Load score: {:.1}", health_info.health_score),
                    }
                } else {
                    NodeStatus::Degraded {
                        reason: format!("High load: {:.1}", health_info.health_score),
                    }
                };

                // Check for status change
                if health_info.status != new_status {
                    let event = match &new_status {
                        NodeStatus::Healthy => HealthEvent::NodeOnline {
                            node_id: node_id.to_string(),
                            timestamp: now,
                        },
                        NodeStatus::Degraded { reason } => HealthEvent::NodeDegraded {
                            node_id: node_id.to_string(),
                            reason: reason.clone(),
                            timestamp: now,
                        },
                        _ => HealthEvent::NodeUnresponsive {
                            node_id: node_id.to_string(),
                            timestamp: now,
                        },
                    };

                    let _ = self.health_events_tx.send(event);
                }

                health_info.status = new_status;
            }
        }

        // Record load metrics in load monitor
        self.load_monitor
            .record_load_metrics(node_id, load_metrics)
            .await?;

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_heartbeats_received += 1;
        }

        debug!("💓 Processed heartbeat from node: {}", node_id);
        Ok(())
    }

    /// Calculate health score from load metrics
    fn calculate_health_score(&self, metrics: &NodeLoadMetrics) -> f64 {
        // Health score is inverse of load - lower load = higher health
        let cpu_health = 100.0 - metrics.cpu_usage_percent;
        let memory_health = 100.0 - metrics.memory_usage_percent;
        let response_health = (1000.0 - metrics.response_time_ms.min(1000.0)) / 10.0;

        // Weighted average
        (cpu_health * 0.4 + memory_health * 0.4 + response_health * 0.2)
            .max(0.0)
            .min(100.0)
    }

    /// Start heartbeat sender background task
    async fn start_heartbeat_sender(&self) -> SongbirdResult<()> {
        let messenger = self.messenger.clone();
        let load_monitor = self.load_monitor.clone();
        let stats = self.stats.clone();
        let running = self.running.clone();
        let interval = Duration::from_secs(self.config.heartbeat_interval_secs);

        let handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            while *running.read().await {
                interval_timer.tick().await;

                // Get current load metrics for this node
                let load_metrics = NodeLoadMetrics {
                    cpu_usage_percent: 0.0,       // Would get from system
                    memory_usage_percent: 0.0,    // Would get from system
                    active_connections: 0,        // Would get from connection tracker
                    request_rate_per_second: 0.0, // Would calculate from metrics
                    response_time_ms: 0.0,        // Would get from recent requests
                };

                // Send heartbeat to all nodes
                match messenger.send_heartbeat(load_metrics).await {
                    Ok(sent_count) => {
                        debug!("💓 Sent heartbeat to {} nodes", sent_count.data);

                        // Update stats
                        let mut stats_guard = stats.write().await;
                        stats_guard.total_heartbeats_sent += sent_count.data;
                        stats_guard.last_heartbeat_sent = Some(SystemTime::now());
                    }
                    Err(e) => {
                        warn!("Failed to send heartbeat: {}", e);
                    }
                }
            }

            debug!("Heartbeat sender task stopped");
        });

        {
            let mut handles = self.task_handles.write().await;
            handles.push(handle);
        }

        Ok(())
    }

    /// Start health monitoring background task
    async fn start_health_monitor(&self) -> SongbirdResult<()> {
        let node_health = self.node_health.clone();
        let health_events_tx = self.health_events_tx.clone();
        let running = self.running.clone();
        let config = self.config.clone();
        let interval = Duration::from_secs(self.config.health_check_interval_secs);

        let handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            while *running.read().await {
                interval_timer.tick().await;

                let now = SystemTime::now();
                let timeout_duration = Duration::from_secs(config.node_timeout_secs);

                // Check each node's health
                {
                    let mut health_guard = node_health.write().await;
                    for (node_id, health_info) in health_guard.iter_mut() {
                        let time_since_heartbeat = now
                            .duration_since(health_info.last_heartbeat)
                            .unwrap_or(Duration::from_secs(u64::MAX));

                        if time_since_heartbeat > timeout_duration {
                            health_info.consecutive_failures += 1;
                            health_info.total_heartbeats_missed += 1;

                            let new_status = if health_info.consecutive_failures
                                >= config.offline_threshold
                            {
                                NodeStatus::Offline
                            } else if health_info.consecutive_failures >= config.degraded_threshold
                            {
                                NodeStatus::Unresponsive
                            } else {
                                continue;
                            };

                            // Check for status change
                            if health_info.status != new_status {
                                let event = match new_status {
                                    NodeStatus::Offline => HealthEvent::NodeOffline {
                                        node_id: node_id.clone(),
                                        timestamp: now,
                                    },
                                    NodeStatus::Unresponsive => HealthEvent::NodeUnresponsive {
                                        node_id: node_id.clone(),
                                        timestamp: now,
                                    },
                                    _ => continue,
                                };

                                let _ = health_events_tx.send(event);
                                health_info.status = new_status;

                                warn!(
                                    "⚠️ Node {} status changed to {:?}",
                                    node_id, health_info.status
                                );
                            }
                        }
                    }
                }
            }

            debug!("Health monitor task stopped");
        });

        {
            let mut handles = self.task_handles.write().await;
            handles.push(handle);
        }

        Ok(())
    }

    /// Start automatic failover monitoring
    async fn start_failover_monitor(&self) -> SongbirdResult<()> {
        let node_health = self.node_health.clone();
        let health_events_tx = self.health_events_tx.clone();
        let running = self.running.clone();
        let config = self.config.clone();
        let stats = self.stats.clone();
        let interval = Duration::from_secs(config.failover_detection_threshold_secs / 2);

        let handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            while *running.read().await {
                interval_timer.tick().await;

                // Check for nodes that need failover
                let health_guard = node_health.read().await;
                for (node_id, health_info) in health_guard.iter() {
                    if health_info.status == NodeStatus::Offline {
                        let time_offline = SystemTime::now()
                            .duration_since(health_info.last_seen)
                            .unwrap_or(Duration::ZERO);

                        if time_offline.as_secs() >= config.failover_detection_threshold_secs {
                            // Find backup nodes
                            let backup_nodes: Vec<String> = health_guard
                                .data
                                .iter()
                                .filter(|(_, h)| h.status == NodeStatus::Healthy)
                                .map(|(id, _)| id.clone())
                                .collect();

                            if !backup_nodes.is_empty() {
                                let event = HealthEvent::FailoverTriggered {
                                    failed_node: node_id.clone(),
                                    backup_nodes: backup_nodes.clone(),
                                    timestamp: SystemTime::now(),
                                };

                                let _ = health_events_tx.send(event);

                                // Update stats
                                let mut stats_guard = stats.write().await;
                                stats_guard.failovers_triggered += 1;

                                warn!("🔄 Automatic failover triggered for node {} to {} backup nodes", 
                                      node_id, backup_nodes.len());
                            }
                        }
                    }
                }
            }

            debug!("Failover monitor task stopped");
        });

        {
            let mut handles = self.task_handles.write().await;
            handles.push(handle);
        }

        Ok(())
    }

    /// Start statistics updater task
    async fn start_stats_updater(&self) -> SongbirdResult<()> {
        let node_health = self.node_health.clone();
        let stats = self.stats.clone();
        let running = self.running.clone();
        let start_time = Instant::now();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            while *running.read().await {
                interval.tick().await;

                // Update node counts
                let health_guard = node_health.read().await;
                let mut healthy_count = 0;
                let mut degraded_count = 0;
                let mut offline_count = 0;

                for health_info in health_guard.values() {
                    match health_info.status {
                        NodeStatus::Healthy => healthy_count += 1,
                        NodeStatus::Degraded { .. } | NodeStatus::Unresponsive => {
                            degraded_count += 1
                        }
                        NodeStatus::Offline | NodeStatus::Removed => offline_count += 1,
                    }
                }

                // Update stats
                {
                    let mut stats_guard = stats.write().await;
                    stats_guard.nodes_currently_healthy = healthy_count;
                    stats_guard.nodes_currently_degraded = degraded_count;
                    stats_guard.nodes_currently_offline = offline_count;
                    stats_guard.uptime_seconds = start_time.elapsed().as_secs();
                }
            }

            debug!("Stats updater task stopped");
        });

        {
            let mut handles = self.task_handles.write().await;
            handles.push(handle);
        }

        Ok(())
    }

    /// Subscribe to health events
    pub fn subscribe_to_health_events(&self) -> broadcast::Receiver<HealthEvent> {
        self.health_events_tx.subscribe()
    }

    /// Get current heartbeat statistics
    pub async fn get_stats(&self) -> HeartbeatStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Get health information for all nodes
    pub async fn get_all_node_health(&self) -> HashMap<String, NodeHealthInfo> {
        let node_health = self.node_health.read().await;
        node_health.clone()
    }

    /// Get health information for a specific node
    pub async fn get_node_health(&self) -> Option<NodeHealthInfo> {
        let node_health = self.node_health.read().await;
        node_health.get(node_id).cloned()
    }

    /// Get list of healthy nodes
    pub async fn get_healthy_nodes(&self) -> Vec<String> {
        let node_health = self.node_health.read().await;
        node_health
            .data
            .iter()
            .filter(|(_, health)| health.status == NodeStatus::Healthy)
            .map(|(node_id, _)| node_id.clone())
            .collect()
    }

    /// Check if the heartbeat system is running
    pub async fn is_running(&self) -> bool {
        let running = self.running.read().await;
        *running
    }
}
