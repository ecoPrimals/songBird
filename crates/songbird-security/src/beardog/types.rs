//! BearDog Security Types
//!
//! Core data structures and enums for the BearDog security integration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct BearDogSecurityIntegration {
    pub client: BearDogClient,
    pub sessions: Vec<BearDogSecuritySession>,
    pub threat_detector: BearDogThreatDetector,
    pub zero_trust_engine: BearDogZeroTrustEngine,
    pub encryption_engine: BearDogEncryptionEngine,
    pub audit_logger: BearDogAuditLogger,
    pub compliance_checker: BearDogComplianceChecker,
}

#[derive(Debug, Clone)]
pub struct BearDogClient {
    pub config: BearDogClientConfig,
    pub sessions: Vec<BearDogSecuritySession>,
    pub active_tunnels: Vec<BSTPTunnel>,
    pub genetics: SecurityGenetics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogClientConfig {
    pub beardog_path: String,
    pub timeout: Duration,
    pub max_retries: u32,
    pub enable_genetics: bool,
    pub gaming_optimization: GamingOptimizationLevel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GamingOptimizationLevel {
    /// No gaming optimizations - standard security
    None,
    /// Basic gaming optimizations - reduced latency
    Basic,
    /// Full gaming optimizations - maximum performance  
    Gaming,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogSecuritySession {
    pub session_id: String,
    pub user_id: String,
    pub start_time: SystemTime,
    pub last_activity: SystemTime,
    pub genetics: SecurityGenetics,
    pub trust_score: TrustScore,
    pub tunnel: Option<BSTPTunnel>,
    pub threat_indicators: Vec<ThreatIndicator>,
    pub access_decisions: Vec<AccessDecision>,
    pub state: SecuritySessionState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SecuritySessionState {
    /// Session is being established
    Initializing,
    /// Session is active and secure
    Active,
    /// Session is suspended due to threats
    Suspended,
    /// Session is being terminated
    Terminating,
    /// Session has ended
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityGenetics {
    pub crypto_genes: CryptoGenes,
    pub auth_genes: AuthGenes,
    pub threat_genes: ThreatGenes,
    pub performance_genes: PerformanceGenes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoGenes {
    pub algorithm_preference: String,
    pub key_strength: u32,
    pub rotation_frequency: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthGenes {
    pub multi_factor_required: bool,
    pub biometric_preference: String,
    pub session_timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatGenes {
    pub detection_sensitivity: f32,
    pub response_aggressiveness: f32,
    pub learning_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceGenes {
    pub latency_tolerance: Duration,
    pub throughput_priority: f32,
    pub resource_usage_limit: f32,
}

#[derive(Debug, Clone)]
pub struct BSTPTunnel {
    pub tunnel_id: String,
    pub local_endpoint: String,
    pub remote_endpoint: String,
    pub keys: BSTPKeys,
    pub state: BSTPTunnelState,
    pub created_at: SystemTime,
    pub last_used: SystemTime,
    pub bytes_transferred: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BSTPTunnelState {
    /// Tunnel is being established
    Connecting,
    /// Tunnel is active and ready
    Connected,
    /// Tunnel is temporarily suspended
    Suspended,
    /// Tunnel is being torn down
    Disconnecting,
    /// Tunnel is closed
    Closed,
    /// Tunnel failed to establish or maintain
    Failed,
}

#[derive(Debug, Clone)]
pub struct BSTPKeys {
    pub encryption_key: Vec<u8>,
    pub authentication_key: Vec<u8>,
    pub rotation_schedule: Duration,
}

#[derive(Debug, Clone)]
pub struct BearDogThreatDetector {
    pub patterns: Vec<ThreatPattern>,
    pub genetics: ThreatGenetics,
    pub evolution: ThreatEvolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedThreat {
    pub threat_id: String,
    pub threat_type: String,
    pub confidence: f32,
    pub indicators: Vec<ThreatIndicator>,
    pub detected_at: SystemTime,
    pub source_ip: Option<String>,
    pub target_resource: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIndicator {
    pub indicator_type: String,
    pub value: String,
    pub confidence: f32,
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
}

#[derive(Debug, Clone)]
pub struct ThreatGenetics {
    pub detection_genes: Vec<String>,
    pub evolution_rate: f32,
}

#[derive(Debug, Clone)]
pub struct ThreatPattern {
    pub pattern_id: String,
    pub signature: Vec<u8>,
    pub confidence_threshold: f32,
}

#[derive(Debug, Clone)]
pub struct ThreatEvolution {
    pub generation: u32,
    pub fitness_score: f32,
    pub mutations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BearDogZeroTrustEngine {
    pub trust_factors: Vec<TrustFactor>,
    pub access_policies: Vec<String>,
    pub decision_cache: HashMap<String, AccessDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustScore {
    pub overall_score: f32,
    pub factors: HashMap<String, f32>,
    pub last_updated: SystemTime,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub struct TrustFactor {
    pub factor_type: String,
    pub weight: f32,
    pub current_value: f32,
    pub history: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessDecision {
    pub decision_id: String,
    pub user_id: String,
    pub resource: String,
    pub action: String,
    pub decision: AccessResult,
    pub trust_score: f32,
    pub factors_considered: Vec<String>,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AccessResult {
    Allow,
    Deny,
    Challenge,
    Monitor,
}

#[derive(Debug, Clone)]
pub struct BearDogEncryptionEngine {
    pub active_keys: HashMap<String, EncryptionKey>,
    pub key_rotation_schedule: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionKey {
    pub key_id: String,
    pub key_data: Vec<u8>,
    pub algorithm: String,
    pub created_at: SystemTime,
    pub expires_at: Option<SystemTime>,
    pub usage_count: u64,
}

#[derive(Debug, Clone)]
pub struct BearDogAuditLogger {
    pub events: Vec<AuditEvent>,
    pub encryption_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub timestamp: SystemTime,
    pub event_type: String,
    pub user_id: Option<String>,
    pub resource: Option<String>,
    pub action: String,
    pub result: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct BearDogComplianceChecker {
    pub standards: Vec<ComplianceStandard>,
    pub last_check: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStandard {
    pub standard_name: String,
    pub version: String,
    pub requirements: Vec<String>,
    pub compliance_level: f32,
}

impl Default for BearDogClientConfig {
    fn default() -> Self {
        Self {
            beardog_path: "../beardog".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            enable_genetics: true,
            gaming_optimization: GamingOptimizationLevel::Gaming,
        }
    }
} 