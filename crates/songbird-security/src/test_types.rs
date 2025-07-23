use chrono::Utc;
use std::collections::HashMap;
use std::time::SystemTime;

// Advanced threat detection testing structures
#[derive(Debug, Clone)]
pub struct TestThreatScenario {
    pub scenario_id: String,
    pub threat_type: ThreatType,
    pub severity: ThreatSeverity,
    pub source: ThreatSource,
    pub target: String,
    pub description: String,
    pub indicators: Vec<ThreatIndicator>,
    pub expected_response: ThreatResponse,
}

#[derive(Debug, Clone)]
pub enum ThreatType {
    Malware,
    Phishing,
    SocialEngineering,
    TechnicalSupportScam,
    DataExfiltration,
    UnauthorizedAccess,
    NetworkIntrusion,
    DenialOfService,
    ManInTheMiddle,
    CredentialStuffing,
    PrivilegeEscalation,
    ZeroDayExploit,
}

#[derive(Debug, Clone)]
pub enum ThreatSeverity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

#[derive(Debug, Clone)]
pub enum ThreatSource {
    External,
    Internal,
    Unknown,
    FamilyMember,
    TrustedFriend,
    Stranger,
}

#[derive(Debug, Clone)]
pub struct ThreatIndicator {
    pub indicator_type: String,
    pub value: String,
    pub confidence: f32,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone)]
pub enum ThreatResponse {
    Block,
    Alert,
    Monitor,
    Quarantine,
    Investigate,
    Allow,
}

// Zero Trust networking test structures
#[derive(Debug, Clone)]
pub struct ZeroTrustTestCase {
    pub test_id: String,
    pub description: String,
    pub context: ZeroTrustContext,
    pub expected_outcome: bool,
    pub verification_steps: Vec<VerificationStep>,
}

#[derive(Debug, Clone)]
pub struct ZeroTrustContext {
    pub user_id: String,
    pub device_id: String,
    pub location: NetworkLocation,
    pub session_state: SessionState,
    pub risk_score: f32,
}

#[derive(Debug, Clone)]
pub enum NetworkLocation {
    Home,
    Office,
    Public,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum SessionState {
    Authenticated,
    Unauthenticated,
    Expired,
    Suspended,
}

#[derive(Debug, Clone)]
pub struct VerificationStep {
    pub step_id: String,
    pub step_type: String,
    pub required: bool,
    pub completed: bool,
}

#[derive(Debug, Clone)]
pub struct GamingSecurityTest {
    pub test_id: String,
    pub game_title: String,
    pub players: Vec<GamingPlayer>,
    pub security_requirements: GamingSecurityRequirements,
    pub threat_scenarios: Vec<GamingThreatScenario>,
}

#[derive(Debug, Clone)]
pub struct GamingPlayer {
    pub player_id: String,
    pub username: String,
    pub connection_quality: ConnectionQuality,
    pub trust_level: f32,
    pub location: String,
}

#[derive(Debug, Clone)]
pub enum ConnectionQuality {
    Excellent,
    Good,
    Fair,
    Poor,
    Unstable,
}

#[derive(Debug, Clone)]
pub struct GamingSecurityRequirements {
    pub anti_cheat: bool,
    pub ddos_protection: bool,
    pub encrypted_communication: bool,
    pub player_verification: bool,
    pub secure_matchmaking: bool,
}

#[derive(Debug, Clone)]
pub enum GamingThreatScenario {
    Cheating,
    DDoSAttack,
    PlayerImpersonation,
    NetworkManipulation,
    DataTheft,
    ServiceDisruption,
}

// SecurityTestReport moved to end of file with comprehensive specialized result types

#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct UserCredentials {
    pub username: String,
    pub password: String,
    pub mfa_token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub device_id: String,
    pub os_version: String,
    pub browser_version: String,
    pub is_trusted: bool,
}

#[derive(Debug, Clone)]
pub struct ApplicationInfo {
    pub app_id: String,
    pub version: String,
    pub permissions: Vec<String>,
    pub signature_valid: bool,
}

#[derive(Debug, Clone)]
pub struct TransactionInfo {
    pub transaction_id: String,
    pub user_id: String,
    pub amount: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub is_authorized: bool,
}

#[derive(Debug, Clone)]
pub struct MonitoringData {
    pub active_sessions: u32,
    pub failed_logins: u32,
    pub suspicious_activities: u32,
    pub system_health: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditLogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub result: String,
    pub details: String,
}

// Specialized result types for different security test categories
#[derive(Debug, Clone)]
pub struct ThreatDetectionResult {
    pub threat_id: String,
    pub threat_type: String,
    pub severity_score: f64,
    pub detected: bool,
    pub passed: bool,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct ZeroTrustResult {
    pub test_id: String,
    pub context: String,
    pub trust_score: f64,
    pub passed: bool,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct EncryptionResult {
    pub test_id: String,
    pub algorithm: String,
    pub key_strength: u32,
    pub strength_score: f64,
    pub passed: bool,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct AuditResult {
    pub test_id: String,
    pub audit_type: String,
    pub completeness_score: f64,
    pub passed: bool,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct ComplianceResult {
    pub test_id: String,
    pub standard: String,
    pub compliance_score: f64,
    pub passed: bool,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct SecurityTestReport {
    pub threat_detection_results: Vec<ThreatDetectionResult>,
    pub zero_trust_results: Vec<ZeroTrustResult>,
    pub encryption_results: Vec<EncryptionResult>,
    pub audit_results: Vec<AuditResult>,
    pub compliance_results: Vec<ComplianceResult>,
    pub overall_score: f64,
    pub recommendations: Vec<String>,
}
