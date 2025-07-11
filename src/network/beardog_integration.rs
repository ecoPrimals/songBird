//! BearDog Integration Module - FRAGO Implementation
//! 
//! Implements the exact NetworkEvent/SecurityEvent interfaces specified in the BearDog FRAGO
//! for BSTP network orchestration layer integration


use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

use crate::errors::{Result, SongbirdError};

// ============================================================================
// FRAGO-SPECIFIED NETWORK EVENT TYPES
// ============================================================================

/// NetworkEvent - Exact FRAGO specification for BearDog integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkEvent {
    PeerDiscovered { 
        peer_id: String, 
        capabilities: PeerCapabilities 
    },
    PeerDisconnected { 
        peer_id: String, 
        reason: DisconnectReason 
    },
    RouteOptimized { 
        old_latency: u64, 
        new_latency: u64 
    },
    NetworkCongestion { 
        severity: CongestionLevel, 
        affected_peers: Vec<String> 
    },
    ThreatIndicator { 
        suspicious_activity: ThreatIndicator, 
        source_peer: String 
    },
}

/// SecurityEvent - Exact FRAGO specification for BearDog integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEvent {
    SessionEstablished { 
        session_id: String, 
        peer_id: String 
    },
    SecurityUpgrade { 
        session_id: String, 
        new_security_level: SecurityLevel 
    },
    ThreatMitigation { 
        action: SecurityAction, 
        affected_routes: Vec<String> 
    },
    ComplianceRequirement { 
        requirement: ComplianceRule, 
        enforcement_level: EnforcementLevel 
    },
}

// Supporting types for NetworkEvent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerCapabilities {
    pub protocol_support: Vec<String>,
    pub bandwidth_mbps: u32,
    pub latency_ms: u16,
    pub gaming_optimized: bool,
    pub security_level: SecurityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisconnectReason {
    NetworkError,
    SecurityViolation,
    PerformanceDegradation,
    UserInitiated,
    ServerShutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CongestionLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIndicator {
    pub threat_type: String,
    pub severity: u8, // 1-10 scale
    pub patterns: Vec<String>,
    pub timestamps: Vec<SystemTime>,
}

// Supporting types for SecurityEvent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Basic,
    Enhanced,
    Maximum,
    Gaming, // Optimized for gaming performance
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityAction {
    Isolate,
    Throttle,
    Monitor,
    Block,
    Redirect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRule {
    pub rule_id: String,
    pub description: String,
    pub category: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnforcementLevel {
    Alert,
    Notify,
    Block,
    Strict,
    Moderate,
    Lenient,
}

// ============================================================================
// FRAGO-SPECIFIED INTEGRATION COMPONENTS
// ============================================================================

/// BearDog Integration Manager - Central coordination point
pub struct BearDogIntegration {
    network_event_publisher: NetworkEventPublisher,
    security_event_consumer: SecurityEventConsumer,
    shared_metrics: SharedMetrics,
    config: BearDogConfig,
}

/// NetworkEvent Publisher for sending events to BearDog
pub struct NetworkEventPublisher {
    sender: mpsc::UnboundedSender<NetworkEvent>,
    published_count: Arc<RwLock<u64>>,
}

/// SecurityEvent Consumer for receiving events from BearDog  
pub struct SecurityEventConsumer {
    _receiver: mpsc::UnboundedReceiver<SecurityEvent>,
    processed_count: Arc<RwLock<u64>>,
}

/// Shared performance metrics between Songbird and BearDog
#[derive(Debug, Clone)]
pub struct SharedMetrics {
    pub network_latency: Arc<RwLock<HashMap<String, u64>>>,
    pub security_events_per_minute: Arc<RwLock<u64>>,
    pub active_peers: Arc<RwLock<u32>>,
    pub threat_level: Arc<RwLock<u8>>,
}

/// BearDog integration configuration
#[derive(Debug, Clone)]
pub struct BearDogConfig {
    pub endpoint: String,
    pub timeout: Duration,
    pub retry_attempts: u32,
    pub enable_batching: bool,
    pub batch_size: usize,
    pub performance_mode: PerformanceMode,
}

#[derive(Debug, Clone)]
pub enum PerformanceMode {
    Gaming,    // <1ms target
    Standard,  // <10ms target  
    Bulk,      // Best effort
}

impl BearDogIntegration {
    /// Create new BearDog integration instance
    pub fn new(config: BearDogConfig) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let (_security_sender, security_receiver) = mpsc::unbounded_channel();
        
        // Keep receivers alive to prevent channel closure
        // In a real implementation, these would be used by background tasks
        tokio::spawn(async move {
            let mut receiver = event_receiver;
            while let Some(_event) = receiver.recv().await {
                // In real implementation, forward to BearDog
                // For testing, just consume events
            }
        });
        
        Self {
            network_event_publisher: NetworkEventPublisher {
                sender: event_sender,
                published_count: Arc::new(RwLock::new(0)),
            },
            security_event_consumer: SecurityEventConsumer {
                _receiver: security_receiver,
                processed_count: Arc::new(RwLock::new(0)),
            },
            shared_metrics: SharedMetrics::new(),
            config,
        }
    }

    /// FRAGO: Exact interface implementation - Publish NetworkEvent to BearDog
    pub async fn publish_network_event(&self, event: NetworkEvent) -> Result<()> {
        let start = Instant::now();
        
        match self.network_event_publisher.publish(event.clone()).await {
            Ok(_) => {
                let latency = start.elapsed();
                
                if matches!(self.config.performance_mode, PerformanceMode::Gaming) && latency > Duration::from_micros(500) {
                    warn!("Gaming mode latency exceeded: {}μs", latency.as_micros());
                }
                
                debug!("Published NetworkEvent: {:?} in {}μs", event, latency.as_micros());
                Ok(())
            }
            Err(e) => {
                error!("Failed to publish NetworkEvent: {}", e);
                Err(SongbirdError::Network {
                    service: "BearDog".to_string(),
                    message: format!("Event publish failed: {}", e),
                    details: None,
                })
            }
        }
    }

    /// FRAGO: Exact interface implementation - Consume SecurityEvent from BearDog
    pub async fn consume_security_event(&self, event: SecurityEvent) -> Result<()> {
        let start = Instant::now();
        
        match event {
            SecurityEvent::SessionEstablished { session_id, peer_id } => {
                info!("🔐 New secure session: {} for peer {}", session_id, peer_id);
                self.configure_secure_routes(&session_id, &peer_id).await?;
            }
            SecurityEvent::SecurityUpgrade { session_id, new_security_level } => {
                info!("🔐 Security upgrade: {} to {:?}", session_id, new_security_level);
                self.apply_security_upgrade(&session_id, &new_security_level).await?;
            }
            SecurityEvent::ThreatMitigation { action, affected_routes } => {
                warn!("🚨 Threat mitigation: {:?} on routes {:?}", action, affected_routes);
                self.apply_threat_mitigation(&action, &affected_routes).await?;
            }
            SecurityEvent::ComplianceRequirement { requirement, enforcement_level } => {
                info!("📋 Compliance requirement: {} ({:?})", requirement.rule_id, enforcement_level);
                self.apply_compliance_requirement(&requirement, &enforcement_level).await?;
            }
        }

        let processing_time = start.elapsed();
        debug!("Processed SecurityEvent in {}μs", processing_time.as_micros());
        
        *self.security_event_consumer.processed_count.write().await += 1;
        
        Ok(())
    }

    /// FRAGO: Sync performance metrics with BearDog
    pub async fn sync_performance_metrics(&self) -> Result<PerformanceMetrics> {
        let metrics = PerformanceMetrics {
            avg_latency_ms: self.calculate_avg_latency().await,
            active_connections: *self.shared_metrics.active_peers.read().await,
            events_per_minute: *self.shared_metrics.security_events_per_minute.read().await,
            threat_level: *self.shared_metrics.threat_level.read().await,
            uptime_seconds: self.get_uptime_seconds(),
        };
        
        debug!("Synced performance metrics: {:?}", metrics);
        Ok(metrics)
    }

    async fn configure_secure_routes(&self, session_id: &str, peer_id: &str) -> Result<()> {
        info!("Configuring secure routes for session {} peer {}", session_id, peer_id);
        Ok(())
    }

    async fn apply_security_upgrade(&self, session_id: &str, security_level: &SecurityLevel) -> Result<()> {
        info!("Applying security upgrade for session {} to {:?}", session_id, security_level);
        Ok(())
    }

    async fn apply_threat_mitigation(&self, action: &SecurityAction, affected_routes: &[String]) -> Result<()> {
        warn!("Applying threat mitigation {:?} to routes: {:?}", action, affected_routes);
        Ok(())
    }

    async fn apply_compliance_requirement(&self, requirement: &ComplianceRule, enforcement_level: &EnforcementLevel) -> Result<()> {
        info!("Applying compliance rule: {} ({})", requirement.rule_id, requirement.description);
        
        // Apply the rule based on enforcement level
        match enforcement_level {
            EnforcementLevel::Strict => {
                // Apply strict enforcement
                warn!("Strict enforcement for rule: {}", requirement.rule_id);
            }
            EnforcementLevel::Moderate => {
                // Apply moderate enforcement
                info!("Moderate enforcement for rule: {}", requirement.rule_id);
            }
            EnforcementLevel::Lenient => {
                // Apply lenient enforcement
                debug!("Lenient enforcement for rule: {}", requirement.rule_id);
            }
            EnforcementLevel::Alert => {
                // Apply alert level enforcement
                warn!("Alert enforcement for rule: {}", requirement.rule_id);
            }
            EnforcementLevel::Notify => {
                // Apply notify level enforcement
                info!("Notify enforcement for rule: {}", requirement.rule_id);
            }
            EnforcementLevel::Block => {
                // Apply block enforcement
                error!("Block enforcement for rule: {}", requirement.rule_id);
            }
        }
        
        Ok(())
    }

    async fn calculate_avg_latency(&self) -> f64 {
        let latencies = self.shared_metrics.network_latency.read().await;
        if latencies.is_empty() {
            return 0.0;
        }
        
        let sum: u64 = latencies.values().sum();
        sum as f64 / latencies.len() as f64
    }

    fn get_uptime_seconds(&self) -> u64 {
        60 // Placeholder
    }
}

impl NetworkEventPublisher {
    pub async fn publish(&self, event: NetworkEvent) -> Result<()> {
        self.sender.send(event).map_err(|e| {
            SongbirdError::Network {
                service: "BearDog".to_string(),
                message: format!("Failed to send event: {}", e),
                details: None,
            }
        })?;
        
        *self.published_count.write().await += 1;
        Ok(())
    }

    pub async fn get_published_count(&self) -> u64 {
        *self.published_count.read().await
    }
}

impl Default for SharedMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedMetrics {
    pub fn new() -> Self {
        Self {
            network_latency: Arc::new(RwLock::new(HashMap::new())),
            security_events_per_minute: Arc::new(RwLock::new(0)),
            active_peers: Arc::new(RwLock::new(0)),
            threat_level: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn update_latency(&self, peer_id: String, latency_ms: u64) {
        self.network_latency.write().await.insert(peer_id, latency_ms);
    }

    pub async fn set_threat_level(&self, level: u8) {
        *self.threat_level.write().await = level;
    }
}

/// Performance metrics structure for BearDog sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub avg_latency_ms: f64,
    pub active_connections: u32,
    pub events_per_minute: u64,
    pub threat_level: u8,
    pub uptime_seconds: u64,
}

impl Default for BearDogConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://beardog.internal:8443".to_string(),
            timeout: Duration::from_millis(50), // Gaming-optimized
            retry_attempts: 3,
            enable_batching: true,
            batch_size: 100,
            performance_mode: PerformanceMode::Gaming,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_beardog_integration_creation() {
        let config = BearDogConfig::default();
        let integration = BearDogIntegration::new(config);
        
        assert_eq!(integration.network_event_publisher.get_published_count().await, 0);
    }

    #[tokio::test]
    async fn test_network_event_publishing() {
        let config = BearDogConfig::default();
        let integration = BearDogIntegration::new(config);
        
        let event = NetworkEvent::PeerDiscovered {
            peer_id: "test-peer".to_string(),
            capabilities: PeerCapabilities {
                protocol_support: vec!["BSTP".to_string()],
                bandwidth_mbps: 1000,
                latency_ms: 1,
                gaming_optimized: true,
                security_level: SecurityLevel::Gaming,
            },
        };
        
        let result = integration.publish_network_event(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_performance_metrics_sync() {
        let config = BearDogConfig::default();
        let integration = BearDogIntegration::new(config);
        
        let metrics = integration.sync_performance_metrics().await;
        assert!(metrics.is_ok());
        
        let metrics = metrics.unwrap();
        assert_eq!(metrics.avg_latency_ms, 0.0);
    }
}
