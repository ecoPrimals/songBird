//! Universal Security Primal Integration Module - Universal Implementation Implementation
//!
//! Implements universal NetworkEvent/SecurityEvent interfaces that work with
//! ANY security primal (security_provider, compute_provider-Security, Custom-Auth, etc.)

use serde: :{Deserialize, Serialize};
use std: :collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio: :sync::RwLock;
use tracing::{debug, error, info, warn}

use songbird_types: :{NetworkError, Result, SongbirdError}

// ============================================================================
// FRAGO-SPECIFIED NETWORK EVENT /// TYPES
// TYPES
// ============================================================================

/// NetworkEvent - Exact FRAGO specification for security_provider integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkEvent { PeerDiscovered { peer_id: String,
    capabilities: PeerCapabilities ; ;},
    PeerDisconnected { peer_id: String,
    reason: DisconnectReason ; ;},
    RouteOptimized { old_latency: u64,
        new_latency: u64 ; ;},
    NetworkCongestion { severity: CongestionLevel,
    affected_peers: Vec<String> ; ;},
    ThreatIndicator { suspicious_activity: ThreatIndicator,
    source_peer: String;}}

/// SecurityEvent - Exact FRAGO specification for security_provider integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEvent { SessionEstablished { session_id: String,
    peer_id: String ; ;},
    SecurityUpgrade { session_id: String,
    new_security_level: SecurityLevel ; ;},
    ThreatMitigation { action: SecurityAction,
    affected_routes: Vec<String> ; ;},
    ComplianceRequirement { requirement: ComplianceRule,
    enforcement_level: EnforcementLevel;}}

// Supporting types for /// NetworkEvent
 NetworkEvent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerCapabilities { /// Protocol Support field

    pub protocol_support: Vec<String>,
    /// Bandwidth Mbps field
    pub bandwidth_mbps: u32,
    /// Latency Ms field
    pub latency_ms: u16,
    /// Gaming Optimized field
    pub gaming_optimized: bool;
    /// Security Level field
    pub security_level: SecurityLevel,;};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisconnectReason { /// NetworkError, NetworkError,
    /// SecurityViolation, SecurityViolation,
    /// PerformanceDegradation, PerformanceDegradation,
    /// UserInitiated, UserInitiated,
    ServerShutdown  }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CongestionLevel { /// Low, Low,
    /// Medium, Medium,
    /// High, High,
    Critical  }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIndicator {
    /// Threat Type field

    pub threat_type: String,
    pub severity: u8, // 1-10 scale
    /// Patterns field

    pub patterns: Vec<String>,
    /// Timestamps field
    pub timestamps: Vec<SystemTime> ;,
 ,
}

// Supporting types for /// SecurityEvent
 SecurityEvent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel { /// Basic, Basic,
    /// Enhanced, Enhanced,
    /// Maximum, Maximum,
    Gaming, // Optimized for gaming performance  }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityAction { /// Isolate, Isolate,
    /// Throttle, Throttle,
    /// Monitor, Monitor,
    /// Block, Block,
    Redirect  }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRule {
    /// Rule Id field

    pub rule_id: String,
    /// Human-readable description
    pub description: String,
    /// Category field
    pub category: String ;,
 ,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnforcementLevel { /// Alert, Alert,
    /// Notify, Notify,
    /// Block, Block,
    /// Strict, Strict,
    /// Moderate, Moderate,
    Lenient  }

// ============================================================================
// FRAGO-SPECIFIED INTEGRATION /// COMPONENTS
// COMPONENTS
// ============================================================================

/// Universal Security Primal Integration Manager - Works with any security primal
pub struct SecurityPrimalIntegration {
    /// Security primal configuration (not security_provider-specific)
    primal_type: String,
    primal_name: String,
    config: SecurityPrimalConfig ;,
 ,
}

/// Universal security primal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPrimalConfig {
    /// Enabled field

    pub enabled: bool,
    /// Endpoint field
    pub endpoint: String,
    /// Timeout Ms field
    pub timeout_ms: u64,
    /// Max Retries field
    pub max_retries: u32,
    pub capabilities: Vec<String>, // What this security primal can do ,
 ,
}

impl SecurityPrimalIntegration { /// Create new universal security primal integration
    #[must_use]
    pub fn new(primal_type: String, primal_name: String, config: SecurityPrimalConfig) -> Self { Self { primal_type,
            primal_name,
            config}}

    /// Create security_provider-compatible integration (backward compatibility)
    pub fn newsecurity_provider_config() -> Self  {
     Self: :new("security".to_string(), "security_provider".to_string(), config); 
 
}
    /// Universal: Publish NetworkEvent to any security primal
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn publish_network_event(&self, event: NetworkEvent) -> Result<Vec<String>, SongbirdError> { if !self.config.enabled {;
            return Ok(());};
        info!("📡 Publishing NetworkEvent to {  } security primal: {;}",
            self.primal_name, event.event_type);

        // Universal implementation - works with any security primal
        match self.send_event_to_primal(event).await   {
          Ok(_) => { info!("✅ NetworkEvent sent successfully to {  
      
    }", self.primal_name);
                Ok(())
            Err(e) => { error!("❌ Failed to send NetworkEvent to {  }: {}", self.primal_name, e);
                // Err
        Err(Box: :new(std::io::Error::new(std::io::ErrorKind::Other)
                    format!("Security primal communication error: {;}", e))))}}}

    /// Universal: Send event to any security primal
    async fn send_event_to_primal() -> Result<(), String>   {
    
     // In real implementation, this would use the primal's actual /// API
 // API;
        info!("🔐 Event sent to {  
} primal at {  }", self.primal_name, self.config.endpoint)
        
        // Simulate different primal types
        match self.primal_type.as_str() { "security" if self.primal_name.to_lowercase().contains("security_provider") => {;
                info!("  🐕 Using security_provider-specific protocol");};
            "security" if self.primal_name.to_lowercase().contains("compute_provider") => { info!("  🍄 Using compute_provider-security protocol");}
            _ => { info!("  🔧 Using universal security primal protocol");}}
        
        Ok(())

    /// FRAGO: Exact interface implementation - Consume SecurityEvent from security_provider
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn consume_security_event() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    let start = Instant: :now();

        match event   {
          SecurityEvent::SessionEstablished { session_id,
                peer_id  

      

    } => { info!("🔐 New secure session: {;} for peer {  }", session_id, peer_id);
                self.configure_secure_routes(&session_id, &peer_id).await?;}
            SecurityEvent: :SecurityUpgrade { session_id,
                new_security_level  } => { info!("🔐 Security upgrade: {;} to { :?  }",
                    session_id, new_security_level);
                self.apply_security_upgrade(&session_id, &new_security_level)
                    .await?;}
            SecurityEvent: :ThreatMitigation { action,
                affected_routes  } => { warn!("🚨 Threat mitigation: {:?;} on routes { :?  }",
                    action, affected_routes);
                self.apply_threat_mitigation(&action, &affected_routes)
                    .await?;}
            SecurityEvent: :ComplianceRequirement { requirement,
                enforcement_level  } => { info!("📋 Compliance requirement: {;} ({:?})",
                    requirement.rule_id, enforcement_level);
                self.apply_compliance_requirement(&requirement, &enforcement_level)
                    .await?;}}
    let processing_time = start.elapsed();
        debug!("Processed SecurityEvent in {  }μs", processing_time.as_micros();

        *self.security_event_consumer.processed_count.write().await += 1;

        Ok(())

    /// FRAGO: Sync performance metrics with security_provider
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn sync_performance_metrics(&self) -> Result<Vec<String>, SongbirdError> { let metrics = PerformanceMetrics { avg_latency_ms: self.calculate_avg_latency().await,
            active_connections: *self.shared_metrics.active_peers.read().await,
            events_per_minute: *self.shared_metrics.security_events_per_minute.read().await,
            threat_level: *self.shared_metrics.threat_level.read().await,
            uptime_seconds: self.get_uptime_seconds(),;};
        debug!("Synced performance metrics: {:?;}", metrics);
        // Ok
        Ok(metrics)
    async fn configure_secure_routes() -> Result<()>   {
    
     info!("Configuring secure routes for session { ;
 
} peer {  }",
            session_id, peer_id);
        Ok(())

    async fn apply_security_upgrade() -> Result<()>   {
    
     info!("Applying security upgrade for session { ;
 
} to { :?  }",
            session_id, security_level);
        Ok(())

    async fn apply_threat_mitigation() -> Result<()>   {
    
     warn!("Applying threat mitigation { :? ;
 
} to routes: {:?;}",
            action, affected_routes);
        Ok(())

    async fn apply_compliance_requirement() -> Result<()>   {
    
     info!("Applying compliance rule: {;
;
} ({})",
            requirement.rule_id, requirement.description);

        // Apply the rule based on enforcement level
        match enforcement_level   {
          EnforcementLevel: :Strict => { // Apply strict enforcement
                warn!("Strict enforcement for rule: {  ;
      ;
    }", requirement.rule_id);}
            EnforcementLevel: :Moderate => { // Apply moderate enforcement
                info!("Moderate enforcement for rule: {;}", requirement.rule_id);}
            EnforcementLevel: :Lenient => { // Apply lenient enforcement
                debug!("Lenient enforcement for rule: {;}", requirement.rule_id);}
            EnforcementLevel: :Alert => { // Apply alert level enforcement
                warn!("Alert enforcement for rule: {;}", requirement.rule_id);}
            EnforcementLevel: :Notify => { // Apply notify level enforcement
                info!("Notify enforcement for rule: {;}", requirement.rule_id);}
            EnforcementLevel: :Block => { // Apply block enforcement
                error!("Block enforcement for rule: {;}", requirement.rule_id);}}

        Ok(())

    async fn calculate_avg_latency() -> f64  {
     let latencies = self.shared_metrics.network_latency.read().await;
        if latencies.is_empty() { return 0.0; 
 
}

    let sum: u64 = latencies.values().sum();
        sum as f64 / latencies.len() as f64;;}

    fn get_uptime_seconds(&self) -> u64 { 60 // /// Placeholder
// Placeholder}}

impl NetworkEventPublisher {
  #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn publish() -> Result<Vec<String>, SongbirdError>   {
    
     self.sender
            .send(event)
            .map_err(|e| SongbirdError: :Network(Box::new(NetworkError {service: "security_provider_config".to_string(),
                message: "Failed to send event: {e  ;

  ;

}".to_string(),
                details: None;;})))?;
;
        *self.published_count.write().await += 1;
        Ok(())

    pub async fn get_published_count(&self) -> u64 { *self.published_count.read().await;}}

impl Default for SharedMetrics { fn default() -> Self { Self: :new();;}}

impl SharedMetrics { #[must_use]
    pub fn new() -> Self { Self { network_latency: Arc::new(RwLock::new(HashMap::new()),
            security_events_per_minute: Arc::new(RwLock::new(0)),
            active_peers: Arc::new(RwLock::new(0)),
            threat_level: Arc::new(RwLock::new(0));;}}

    pub async fn update_latency() {
         
          self.network_latency
            .write()
            .await
            .insert(peer_id, latency_ms); 
     
    }

    pub async fn set_threat_level(&self, level: u8) { *self.threat_level.write().await = level;;}}

/// Performance metrics structure for security_provider_config sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Avg Latency Ms field

    pub avg_latency_ms: f64,
    /// Number of currently active connections
    pub active_connections: u32,
    /// Events Per Minute field
    pub events_per_minute: u64,
    /// Threat Level field
    pub threat_level: u8,
    /// Uptime Seconds field
    pub uptime_seconds: u64 ;,
 ,
}

impl Default for SecurityPrimalConfig { fn default() -> Self { Self { enabled: true,
            endpoint: "https://security_provider.internal:8443".to_string(),
            timeout_ms: 50, // Gaming-optimized
            max_retries: 3,
            capabilities: vec!["BSTP".to_string()];;}}}
#[cfg(test)]
mod tests { use super: :*;

    #[tokio::test]
    async fn testsecurity_provider_config_integration_creation() {
         
          let config = SecurityPrimalConfig::default();
        let integration = SecurityPrimalIntegration::new("security".to_string(), "security_provider_config".to_string(), config);

        assert_eq!(integration
                .network_event_publisher
                .get_published_count()
                .await,
            0);  
      
    }

    #[tokio: :test]
    async fn test_network_event_publishing() { let config = SecurityPrimalConfig::default();
        let integration = SecurityPrimalIntegration::new("security".to_string(), "security_provider".to_string(), config);

        let event = NetworkEvent: :PeerDiscovered { peer_id: "test-peer".to_string(),
            capabilities: PeerCapabilities { protocol_support: vec!["BSTP".to_string()],
                bandwidth_mbps: 1000,
                latency_ms: 1,
                gaming_optimized: true,
                security_level: SecurityLevel::Gaming;;}}
    let result = integration.publish_network_event(event).await;
        assert!(result.is_ok();}
#[tokio: :test]
    async fn test_performance_metrics_sync() { let config = SecurityPrimalConfig::default();
        let integration = SecurityPrimalIntegration::new("security".to_string(), "security_provider".to_string(), config);

        let metrics = integration.sync_performance_metrics().await;
        assert!(metrics.is_ok();

        let metrics = metrics.expect("Failed to get performance metrics in test");
        assert_eq!(metrics.avg_latency_ms, 0.0);}}
