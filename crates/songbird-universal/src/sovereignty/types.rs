// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecurityLevel {
    /// Maximum security - all protections enabled
    Maximum,
    /// High security - most protections enabled
    High,
    /// Medium security - balanced approach
    Medium,
    /// Low security - minimal protections
    Low,
    /// Minimal security - basic protections only
    Minimal,
}

/// Sovereignty level classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SovereigntyLevel {
    /// Fully sovereign - complete autonomy and control
    FullySovereign,
    /// Highly sovereign - most autonomy preserved
    HighlySovereign,
    /// Moderately sovereign - balanced autonomy
    ModeratelySovereign,
    /// Limited sovereignty - reduced autonomy
    LimitedSovereignty,
    /// Non-sovereign - no autonomy guarantees
    NonSovereign,
}

impl SovereigntyLevel {
    /// Returns a numerical score for this sovereignty level (0.0 to 1.0)
    #[must_use]
    pub const fn score(&self) -> f64 {
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityCapability {
    /// Encryption capability
    Encryption,
    /// Authentication capability
    Authentication,
    /// Authorization capability
    Authorization,
    /// Federation-aware routing
    FederationAware,
    /// Network-optimized paths
    NetworkOptimized,
    /// Sovereignty-compliant operations
    SovereigntyCompliant,
}

/// Sovereignty compliance level
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(
    clippy::enum_variant_names,
    reason = "variant names match policy vocabulary (Fully/Mostly/Partially)"
)]
pub enum SovereigntyComplianceLevel {
    /// Fully compliant with all sovereignty requirements
    FullyCompliant,
    /// Mostly compliant with sovereignty requirements
    MostlyCompliant,
    /// Partially compliant with sovereignty requirements
    PartiallyCompliant,
    /// Non-compliant with sovereignty requirements
    NonCompliant,
}

/// Sovereignty risk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereigntyRisk {
    /// Unique identifier for this risk
    pub risk_id: String,
    /// Type of sovereignty risk
    pub risk_type: SovereigntyRiskType,
    /// Severity level of the risk
    pub severity: RiskSeverity,
    /// Strategies to mitigate this risk
    pub mitigation_strategies: Vec<String>,
}

/// Security assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAssessment {
    /// Overall security score (0.0 to 1.0)
    pub security_score: f64,
    /// Assessed security level
    pub security_level: SecurityLevel,
    /// List of identified security vulnerabilities
    pub identified_vulnerabilities: Vec<String>,
}

/// Federation capability type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FederationCapabilityType {
    /// Cross-node communication capability
    CrossNodeCommunication,
    /// Consensus participation capability
    ConsensusParticipation,
    /// Data replication capability
    DataReplication,
    /// Load distribution capability
    LoadDistribution,
    /// Health monitoring capability
    HealthMonitoring,
    /// Route optimization capability
    RouteOptimization,
}

/// Performance characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceCharacteristics {
    /// Average latency in milliseconds
    pub latency_ms: f64,
    /// Throughput in operations per second
    pub throughput_ops_per_sec: f64,
    /// Reliability score (0.0 to 1.0)
    pub reliability_score: f64,
}

impl Default for PerformanceCharacteristics {
    fn default() -> Self {
        Self {
            latency_ms: 0.0,
            throughput_ops_per_sec: 0.0,
            reliability_score: 1.0,
        }
    }
}

/// Network effect type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkEffectType {
    /// Performance improvement from network participation
    PerformanceImprovement,
    /// Security enhancement through federation
    SecurityEnhancement,
    /// Cost reduction via resource sharing
    CostReduction,
    /// Capability expansion through collaboration
    CapabilityExpansion,
}

/// Decision factor
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(
    clippy::struct_field_names,
    reason = "factor_name/weight/value mirror decision engine schema"
)]
pub struct DecisionFactor {
    /// Name of the decision factor
    pub factor_name: String,
    /// Weight of this factor in the decision (0.0 to 1.0)
    pub factor_weight: f64,
    /// Current value of this factor
    pub factor_value: f64,
}

/// Sovereignty risk type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SovereigntyRiskType {
    /// Data sovereignty and control risks
    DataSovereignty,
    /// Jurisdictional compliance risks
    JurisdictionalCompliance,
    /// Network dependency risks
    NetworkDependency,
    /// Third-party service risks
    ThirdPartyRisk,
}

/// Risk severity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskSeverity {
    /// Critical severity - immediate action required
    Critical,
    /// High severity - requires attention
    High,
    /// Medium severity - monitor closely
    Medium,
    /// Low severity - acceptable risk
    Low,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
    #![allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
    #![allow(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]

    use super::*;

    #[test]
    fn test_sovereignty_adapter_config_default() {
        let config = SovereigntyAdapterConfig::default();
        assert!(config.enable_sovereignty_routing);
        assert!(config.enable_federation_routing);
        assert!(config.enable_network_optimization);
        assert_eq!(config.sovereignty_timeout, Duration::from_secs(3));
        assert_eq!(config.sovereignty_preference_weight, 0.8);
    }

    #[test]
    fn test_sovereignty_level_scores() {
        assert_eq!(SovereigntyLevel::FullySovereign.score(), 1.0);
        assert_eq!(SovereigntyLevel::HighlySovereign.score(), 0.8);
        assert_eq!(SovereigntyLevel::ModeratelySovereign.score(), 0.6);
        assert_eq!(SovereigntyLevel::LimitedSovereignty.score(), 0.4);
        assert_eq!(SovereigntyLevel::NonSovereign.score(), 0.0);
    }

    #[test]
    fn test_security_level_equality() {
        assert_eq!(SecurityLevel::Maximum, SecurityLevel::Maximum);
        assert_eq!(SecurityLevel::High, SecurityLevel::High);
        assert_ne!(SecurityLevel::Maximum, SecurityLevel::High);
    }

    #[test]
    fn test_security_capability_equality() {
        assert_eq!(SecurityCapability::Encryption, SecurityCapability::Encryption);
        assert_eq!(
            SecurityCapability::SovereigntyCompliant,
            SecurityCapability::SovereigntyCompliant
        );
        assert_ne!(SecurityCapability::Encryption, SecurityCapability::Authentication);
    }

    #[test]
    fn test_sovereignty_adapter_config_custom() {
        let config = SovereigntyAdapterConfig {
            enable_sovereignty_routing: false,
            enable_federation_routing: true,
            enable_network_optimization: false,
            sovereignty_timeout: Duration::from_secs(5),
            sovereignty_preference_weight: 0.5,
        };

        assert!(!config.enable_sovereignty_routing);
        assert!(config.enable_federation_routing);
        assert!(!config.enable_network_optimization);
        assert_eq!(config.sovereignty_timeout, Duration::from_secs(5));
        assert_eq!(config.sovereignty_preference_weight, 0.5);
    }

    #[test]
    fn test_routing_path_creation() {
        let path = RoutingPath {
            segments: vec![],
            sovereignty_score: 0.9,
            efficiency_score: 0.8,
            combined_score: 0.85,
            security_level: SecurityLevel::High,
        };

        assert_eq!(path.sovereignty_score, 0.9);
        assert_eq!(path.efficiency_score, 0.8);
        assert_eq!(path.combined_score, 0.85);
        assert_eq!(path.security_level, SecurityLevel::High);
    }

    #[test]
    fn test_sovereignty_assessment_creation() {
        let assessment = PathSovereigntyAssessment {
            overall_score: 0.95,
            segment_assessments: vec![],
            compliance_level: SovereigntyComplianceLevel::FullyCompliant,
            sovereignty_risks: vec![],
        };

        assert_eq!(assessment.overall_score, 0.95);
        assert!(matches!(assessment.compliance_level, SovereigntyComplianceLevel::FullyCompliant));
        assert!(assessment.sovereignty_risks.is_empty());
    }

    #[test]
    fn test_federation_capability_creation() {
        let capability = FederationCapability {
            capability_id: "test-capability".to_string(),
            capability_type: FederationCapabilityType::CrossNodeCommunication,
            availability_score: 0.99,
            performance_characteristics: PerformanceCharacteristics {
                latency_ms: 50.0,
                throughput_ops_per_sec: 1000.0,
                reliability_score: 0.95,
            },
        };

        assert_eq!(capability.capability_id, "test-capability");
        assert_eq!(capability.availability_score, 0.99);
        assert_eq!(capability.performance_characteristics.latency_ms, 50.0);
    }

    #[test]
    fn test_expected_network_effect_creation() {
        let effect = ExpectedNetworkEffect {
            effect_id: "performance-boost".to_string(),
            effect_type: NetworkEffectType::PerformanceImprovement,
            impact_magnitude: 0.75,
            confidence_level: 0.85,
        };

        assert_eq!(effect.effect_id, "performance-boost");
        assert!(matches!(effect.effect_type, NetworkEffectType::PerformanceImprovement));
        assert_eq!(effect.impact_magnitude, 0.75);
        assert_eq!(effect.confidence_level, 0.85);
    }

    #[test]
    fn test_sovereignty_risk_creation() {
        let risk = SovereigntyRisk {
            risk_id: "data-sovereignty-001".to_string(),
            risk_type: SovereigntyRiskType::DataSovereignty,
            severity: RiskSeverity::High,
            mitigation_strategies: vec!["encryption".to_string(), "local-storage".to_string()],
        };

        assert_eq!(risk.risk_id, "data-sovereignty-001");
        assert!(matches!(risk.risk_type, SovereigntyRiskType::DataSovereignty));
        assert!(matches!(risk.severity, RiskSeverity::High));
        assert_eq!(risk.mitigation_strategies.len(), 2);
    }

    #[test]
    fn test_routing_decision_metadata_creation() {
        use std::time::SystemTime;

        let metadata = RoutingDecisionMetadata {
            decision_timestamp: SystemTime::now(),
            algorithm_version: "v1.0.0".to_string(),
            decision_factors: vec![],
            alternative_paths_count: 3,
        };

        assert_eq!(metadata.algorithm_version, "v1.0.0");
        assert_eq!(metadata.alternative_paths_count, 3);
    }

    #[test]
    fn test_decision_factor_creation() {
        let factor = DecisionFactor {
            factor_name: "sovereignty_score".to_string(),
            factor_weight: 0.8,
            factor_value: 0.95,
        };

        assert_eq!(factor.factor_name, "sovereignty_score");
        assert_eq!(factor.factor_weight, 0.8);
        assert_eq!(factor.factor_value, 0.95);
    }

    #[test]
    fn test_security_assessment_creation() {
        let assessment = SecurityAssessment {
            security_score: 0.92,
            security_level: SecurityLevel::High,
            identified_vulnerabilities: vec!["minor-timing-leak".to_string()],
        };

        assert_eq!(assessment.security_score, 0.92);
        assert_eq!(assessment.security_level, SecurityLevel::High);
        assert_eq!(assessment.identified_vulnerabilities.len(), 1);
    }

    #[test]
    fn test_performance_characteristics_creation() {
        let perf = PerformanceCharacteristics {
            latency_ms: 25.5,
            throughput_ops_per_sec: 5000.0,
            reliability_score: 0.999,
        };

        assert_eq!(perf.latency_ms, 25.5);
        assert_eq!(perf.throughput_ops_per_sec, 5000.0);
        assert_eq!(perf.reliability_score, 0.999);
    }

    #[test]
    fn test_sovereignty_aware_routing_decision_creation() {
        use std::time::SystemTime;

        let decision = SovereigntyAwareRoutingDecision {
            selected_path: RoutingPath {
                segments: vec![],
                sovereignty_score: 0.9,
                efficiency_score: 0.85,
                combined_score: 0.875,
                security_level: SecurityLevel::High,
            },
            sovereignty_assessment: PathSovereigntyAssessment {
                overall_score: 0.9,
                segment_assessments: vec![],
                compliance_level: SovereigntyComplianceLevel::FullyCompliant,
                sovereignty_risks: vec![],
            },
            federation_capabilities: vec![],
            expected_network_effects: vec![],
            decision_metadata: RoutingDecisionMetadata {
                decision_timestamp: SystemTime::now(),
                algorithm_version: "v1.0.0".to_string(),
                decision_factors: vec![],
                alternative_paths_count: 2,
            },
        };

        assert_eq!(decision.selected_path.sovereignty_score, 0.9);
        assert_eq!(decision.sovereignty_assessment.overall_score, 0.9);
    }

    #[test]
    fn risk_severity_json_roundtrip() {
        for s in
            [RiskSeverity::Critical, RiskSeverity::High, RiskSeverity::Medium, RiskSeverity::Low]
        {
            let json = serde_json::to_string(&s).expect("serialize");
            let back: RiskSeverity = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, s);
        }
    }

    #[test]
    fn sovereignty_risk_type_json_roundtrip() {
        let t = SovereigntyRiskType::JurisdictionalCompliance;
        let json = serde_json::to_string(&t).expect("serialize");
        let back: SovereigntyRiskType = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(back, SovereigntyRiskType::JurisdictionalCompliance));
    }

    #[test]
    fn network_effect_type_json_roundtrip() {
        let t = NetworkEffectType::CapabilityExpansion;
        let json = serde_json::to_string(&t).expect("serialize");
        let back: NetworkEffectType = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(back, NetworkEffectType::CapabilityExpansion));
    }

    #[test]
    fn federation_capability_type_json_roundtrip() {
        let t = FederationCapabilityType::ConsensusParticipation;
        let json = serde_json::to_string(&t).expect("serialize");
        let back: FederationCapabilityType = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(back, FederationCapabilityType::ConsensusParticipation));
    }

    #[test]
    fn performance_characteristics_default_impl() {
        let p = PerformanceCharacteristics::default();
        assert_eq!(p.latency_ms, 0.0);
        assert_eq!(p.reliability_score, 1.0);
    }
}
