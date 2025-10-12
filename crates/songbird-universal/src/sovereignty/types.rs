//! # 🏛️ Sovereignty-Aware Types
//!
//! **CANONICAL TYPE DEFINITIONS** ✅
//!
//! This module contains all type definitions for the sovereignty-aware
//! universal adapter system. Consolidated from fragmented definitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
// Re-export canonical types
pub use songbird_types::{SongbirdError, SongbirdResult};
/// Configuration for sovereignty-aware adapter
#[derive(Debug, Clone)]
pub struct SovereigntyAdapterConfig {
    /// Enable sovereignty-aware routing
    pub enable_sovereignty_routing: bool,

    /// Enable federation-aware path selection
    pub enable_federation_routing: bool,

    /// Enable network effects optimization
    pub enable_network_optimization: bool,

    /// Timeout for sovereignty-specific operations
    pub sovereignty_timeout: Duration,

    /// Prefer sovereign paths over efficient paths
    pub sovereignty_preference_weight: f64,
}

impl Default for SovereigntyAdapterConfig {
    fn default() -> Self {
        Self {
            enable_sovereignty_routing: true,
            enable_federation_routing: true,
            enable_network_optimization: true,
            sovereignty_timeout: Duration::from_secs(3),
            sovereignty_preference_weight: 0.8, // Heavily prefer sovereign paths
        }
    }
}

/// Enhanced routing decision with sovereignty awareness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereigntyAwareRoutingDecision {
    /// Selected routing path
    pub selected_path: RoutingPath,

    /// Sovereignty assessment of the path
    pub sovereignty_assessment: PathSovereigntyAssessment,

    /// Federation capabilities along the path
    pub federation_capabilities: Vec<FederationCapability>,

    /// Expected network effects from this routing
    pub expected_network_effects: Vec<ExpectedNetworkEffect>,

    /// Routing decision metadata
    pub decision_metadata: RoutingDecisionMetadata,
}

/// Routing path with sovereignty information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingPath {
    /// Path segments (services/nodes along the route)
    pub segments: Vec<PathSegment>,

    /// Total path sovereignty score
    pub sovereignty_score: f64,

    /// Path efficiency score
    pub efficiency_score: f64,

    /// Combined path score (sovereignty + efficiency)
    pub combined_score: f64,

    /// Security level of the path
    pub security_level: SecurityLevel,
}

/// Individual path segment with sovereignty metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSegment {
    /// Service information for this segment
    pub service: crate::types::ServiceInfo,

    /// Sovereignty level of this segment
    pub sovereignty_level: SovereigntyLevel,

    /// Efficiency score for this segment
    pub efficiency_score: f64,

    /// Security capabilities of this segment
    pub security_capabilities: Vec<SecurityCapability>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Assessment of path sovereignty characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSovereigntyAssessment {
    /// Overall sovereignty score (0.0 to 1.0)
    pub overall_score: f64,

    /// Individual segment assessments
    pub segment_assessments: Vec<SegmentSovereigntyAssessment>,

    /// Sovereignty compliance level
    pub compliance_level: SovereigntyComplianceLevel,

    /// Identified sovereignty risks
    pub sovereignty_risks: Vec<SovereigntyRisk>,
}

/// Individual segment sovereignty assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentSovereigntyAssessment {
    /// Segment identifier
    pub segment_id: String,

    /// Sovereignty score for this segment
    pub sovereignty_score: f64,

    /// Sovereignty level classification
    pub sovereignty_level: SovereigntyLevel,

    /// Security assessment
    pub security_assessment: SecurityAssessment,
}

/// Federation capability available along routing path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationCapability {
    /// Capability identifier
    pub capability_id: String,

    /// Capability type
    pub capability_type: FederationCapabilityType,

    /// Availability score (0.0 to 1.0)
    pub availability_score: f64,

    /// Performance characteristics
    pub performance_characteristics: PerformanceCharacteristics,
}

/// Expected network effect from routing decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedNetworkEffect {
    /// Effect identifier
    pub effect_id: String,

    /// Effect type
    pub effect_type: NetworkEffectType,

    /// Expected impact magnitude
    pub impact_magnitude: f64,

    /// Confidence in prediction
    pub confidence_level: f64,
}

/// Metadata about routing decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecisionMetadata {
    /// Decision timestamp
    pub decision_timestamp: std::time::SystemTime,

    /// Decision algorithm version
    pub algorithm_version: String,

    /// Factors considered in decision
    pub decision_factors: Vec<DecisionFactor>,

    /// Alternative paths considered
    pub alternative_paths_count: usize,
}

/// Security level classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityLevel {
    Maximum,
    High,
    Medium,
    Low,
    Minimal,
}

/// Sovereignty level classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SovereigntyLevel {
    FullySovereign,
    HighlySovereign,
    ModeratelySovereign,
    LimitedSovereignty,
    NonSovereign,
}

impl SovereigntyLevel {
    pub fn score(&self) -> f64 {
        match self {
            Self::FullySovereign => 1.0,
            Self::HighlySovereign => 0.8,
            Self::ModeratelySovereign => 0.6,
            Self::LimitedSovereignty => 0.4,
            Self::NonSovereign => 0.0,
        }
    }
}

/// Security capability
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SecurityCapability {
    Encryption,
    Authentication,
    Authorization,
    FederationAware,
    NetworkOptimized,
    SovereigntyCompliant,
}

/// Sovereignty compliance level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SovereigntyComplianceLevel {
    FullyCompliant,
    MostlyCompliant,
    PartiallyCompliant,
    NonCompliant,
}

/// Sovereignty risk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereigntyRisk {
    pub risk_id: String,
    pub risk_type: SovereigntyRiskType,
    pub severity: RiskSeverity,
    pub mitigation_strategies: Vec<String>,
}

/// Security assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAssessment {
    pub security_score: f64,
    pub security_level: SecurityLevel,
    pub identified_vulnerabilities: Vec<String>,
}

/// Federation capability type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FederationCapabilityType {
    CrossNodeCommunication,
    ConsensusParticipation,
    DataReplication,
    LoadDistribution,
}

/// Performance characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceCharacteristics {
    pub latency_ms: f64,
    pub throughput_ops_per_sec: f64,
    pub reliability_score: f64,
}

/// Network effect type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkEffectType {
    PerformanceImprovement,
    SecurityEnhancement,
    CostReduction,
    CapabilityExpansion,
}

/// Decision factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionFactor {
    pub factor_name: String,
    pub factor_weight: f64,
    pub factor_value: f64,
}

/// Sovereignty risk type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SovereigntyRiskType {
    DataSovereignty,
    JurisdictionalCompliance,
    NetworkDependency,
    ThirdPartyRisk,
}

/// Risk severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskSeverity {
    Critical,
    High,
    Medium,
    Low,
}
