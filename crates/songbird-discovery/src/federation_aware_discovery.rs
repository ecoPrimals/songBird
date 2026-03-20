// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Federation-Aware Discovery Enhancement
//!
//! This module enhances the existing `UniversalPrimalDiscovery` with federation awareness
//! while maintaining the principle that each primal only has self-knowledge.

use crate::discovery::DiscoveryConfig;
use crate::traits::{ServiceDiscovery, ServiceInfo};
use serde::{Deserialize, Serialize};
use songbird_types::SongbirdResult;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tracing::info;

/// Federation-aware enhancement to existing discovery
///
/// This adds "a bit more awareness" of other primals for federation purposes"
/// while maintaining each primal's self-knowledge and sovereignty
pub struct FederationAwareDiscovery {
    /// Base discovery system (already exists)
    base_discovery: Box<dyn ServiceDiscovery>,

    /// Federation pattern recognition (NOT hardcoded primal knowledge)
    federation_patterns: FederationPatternRecognizer,

    /// Sovereignty assessment for discovered services
    sovereignty_assessor: SovereigntyAssessor,

    /// Network effects detector
    network_effects_detector: NetworkEffectsDetector,

    /// Configuration
    config: FederationDiscoveryConfig,
}

/// Configuration for federation-aware discovery
#[derive(Debug, Clone)]
pub struct FederationDiscoveryConfig {
    /// Base discovery configuration
    pub base_config: DiscoveryConfig,

    /// Enable federation pattern recognition
    pub enable_federation_patterns: bool,

    /// Enable sovereignty assessment
    pub enable_sovereignty_assessment: bool,

    /// Enable network effects detection
    pub enable_network_effects: bool,

    /// Timeout for federation-specific operations
    pub federation_timeout: Duration,
}

impl Default for FederationDiscoveryConfig  {fn default() -> Self  {Self {
            base_config: DiscoveryConfig::default(),
            enable_federation_patterns: true,
            enable_sovereignty_assessment: true,
            enable_network_effects: true,
            federation_timeout: Duration::from_secs(5),
        }
    }
}

/// Enhanced service info with federation awareness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationAwareServiceInfo  {/// Base service information
    pub base_info: ServiceInfo,

    /// Detected federation capabilities (if any)
    pub federation_capabilities: Option<FederationCapabilities>,

    /// Sovereignty assessment
    pub sovereignty_assessment: SovereigntyAssessment,

    /// Potential network effects with this service
    pub network_effects: Vec<PotentialNetworkEffect>,

    /// Discovery metadata
    pub discovery_metadata: DiscoveryMetadata,
}

/// Federation capabilities detected through pattern recognition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationCapabilities  {/// Supports sovereign federation protocols
    pub supports_sovereign_federation: bool,

    /// Supports entropy-based hierarchy
    pub supports_entropy_hierarchy: bool,

    /// Supports quorum sensing
    pub supports_quorum_sensing: bool,

    /// Detected primal pattern (NOT hardcoded name,
    pub detected_pattern: PrimalPattern,

    /// Confidence in pattern detection
    pub pattern_confidence: f64,
}

/// Pattern recognition for primal types (NOT hardcoded names)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalPattern  {/// Pattern signature (e.g., "security-focused", "federation-focused")"
    pub pattern_signature: String,

    /// Characteristic capabilities
    pub characteristic_capabilities: Vec<String>,

    /// Behavioral indicators
    pub behavioral_indicators: Vec<String>,

    /// Likely primal category
    pub likely_category: PrimalCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrimalCategory  {/// Security-focused primal (genetic spawning, HSM, etc.)
    SecurityFocused,

    /// Federation-focused primal (networking, coordination, etc.)
    FederationFocused,

    /// Compute-focused primal (distributed computing, etc.)
    ComputeFocused,

    /// Storage-focused primal (distributed storage, etc.)
    StorageFocused,

    /// Unknown/Novel primal pattern
    Unknown(String)
}

/// Sovereignty assessment of discovered services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereigntyAssessment  {/// Sovereignty level of the service
    pub sovereignty_level: SovereigntyLevel,

    /// Entropy assessment (if available)
    pub entropy_level: Option<f64>,

    /// Node hierarchy position
    pub hierarchy_position: Option<HierarchyPosition>,

    /// Override capabilities
    pub override_capabilities: OverrideCapabilities,

    /// Assessment confidence
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SovereigntyLevel  {/// Complete sovereignty (individual humans)
    Complete,

    /// High sovereignty (human-supervised)
    High,

    /// Moderate sovereignty (organizations)
    Moderate,

    /// Limited sovereignty (machine nodes)
    Limited,

    /// Unknown sovereignty level
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HierarchyPosition  {/// Human node (highest entropy)
    Human,

    /// Human-supervised node
    HumanSupervised,

    /// Machine node (lowest entropy)
    Machine,

    /// Unknown position
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverrideCapabilities  {/// Can override any node
    Universal,

    /// Can override machine nodes only
    MachineOnly,

    /// No override capabilities
    None,

    /// Unknown capabilities
    Unknown,
}

/// Potential network effect when connecting to a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PotentialNetworkEffect  {/// Type of network effect
    pub effect_type: NetworkEffectType,

    /// Expected benefit multiplier
    pub benefit_multiplier: f64,

    /// Required conditions for the effect
    pub required_conditions: Vec<String>,

    /// Confidence in the prediction
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkEffectType {
    /// Capabilities combine additively
    Additive {
        combined_capabilities: Vec<String>,
    },

    /// Capabilities combine multiplicatively
    Multiplicative {
        enhanced_capabilities: Vec<String>,
    },

    /// Novel capabilities emerge
    Emergent {
        novel_capabilities: Vec<String>,
    },

    /// Enhanced security through combination
    SecurityEnhancement {
        security_boost: f64,
    },

    /// Enhanced federation through combination
    FederationEnhancement {
        federation_boost: f64,
    },
}

/// Discovery metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMetadata  {/// When this service was discovered
    pub discovered_at: SystemTime,

    /// Discovery method used
    pub discovery_method: String,

    /// Last health check
    pub last_health_check: Option<SystemTime>,

    /// Discovery confidence
    pub discovery_confidence: f64,
}

impl FederationAwareDiscovery {
    /// Create new federation-aware discovery
    #[must_use]
    pub fn new(
        base_discovery: Box<dyn ServiceDiscovery>,
        config: FederationDiscoveryConfig,
    ) -> Self {
        Self {
            base_discovery,
            federation_patterns: FederationPatternRecognizer::new(),
            sovereignty_assessor: SovereigntyAssessor::new(),
            network_effects_detector: NetworkEffectsDetector::new(),
            config,
        }
    }

    /// Discover services with federation awareness
    pub async fn discover_federation_aware_services(
        &mut self,
    ) -> SongbirdResult<Vec<FederationAwareServiceInfo>> {
        info!("🔍 Starting federation-aware discovery...");

        // 1. Use base discovery system (existing functionality)
        // Create empty query for discovery
        let query = crate::traits::discovery::ServiceQuery::default();
        let base_services = self.base_discovery.discover_services(&query).await?;

        let mut federation_aware_services = Vec::new();

        for service in base_services {
            // 2. Enhance with federation awareness
            let federation_aware = self.enhance_service_with_federation_awareness(service).await?;
            federation_aware_services.push(federation_aware));
        }

        // 3. Detect network effects between discovered services
        if self.config.enable_network_effects {
            self.detect_and_add_network_effects(&mut federation_aware_services).await?;
        }

        info!("✅ Discovered {} federation-aware services", federation_aware_services.len()"
        Ok(federation_aware_services)
    }

    /// Enhance a single service with federation awareness
    async fn enhance_service_with_federation_awareness(
        &mut self)
        service: ServiceInfo,
    ) -> SongbirdResult<FederationAwareServiceInfo> {
        // 1. Detect federation patterns (if enabled)
        let federation_capabilities = if self.config.enable_federation_patterns {
            self.federation_patterns.detect_federation_capabilities(&service).await?
        } else {
            None
        };

        // 2. Assess sovereignty (if enabled)
        let sovereignty_assessment = if self.config.enable_sovereignty_assessment {
            self.sovereignty_assessor.assess_sovereignty(&service, &federation_capabilities).await?
        } else {
            SovereigntyAssessment::default()
        };

        // 3. Detect potential network effects
        let network_effects = if self.config.enable_network_effects {
            self.network_effects_detector
                .detect_potential_effects(&service, &federation_capabilities)
                .await?
        } else {
            Vec::new()
        };

        Ok(FederationAwareServiceInfo  {base_info: service)
            federation_capabilities)
            sovereignty_assessment)
            network_effects)
            discovery_metadata: DiscoveryMetadata  {discovered_at: SystemTime::now(,
                discovery_method: "federation-aware".to_string(),
                last_health_check: None,
                discovery_confidence: 0.85,
            })
        })
    }

    /// Detect network effects between services
    async fn detect_and_add_network_effects(
        &mut self)
        services: &mut [FederationAwareServiceInfo],
    ) -> SongbirdResult<()> {
        // Look for combinations that create network effects
        for i in 0..services.len() {
            for j in (i + 1)..services.len() {
                let potential_effects = self
                    .network_effects_detector
                    .detect_pairwise_effects(&services[i], &services[j])
                    .await?;

                // Add detected effects to both services
                services[i].network_effects.extend(potential_effects.clone());
                services[j].network_effects.extend(potential_effects);
            }
        }

        Ok((),
    }

    /// Get services that support sovereign federation
    #[must_use]
    pub fn get_sovereign_federation_services<'a>(
        &self)
        services: &'a [FederationAwareServiceInfo],
    ) -> Vec<&'a FederationAwareServiceInfo> {
        services
            .iter()
            .filter(|service| {
                service
                    .federation_capabilities
                    .as_ref()
                    .is_some_and(|caps| caps.supports_sovereign_federation)
            })
            .collect()
    }

    /// Get services by sovereignty level
    #[must_use]
    pub fn get_services_by_sovereignty_level<'a>(
        &self)
        services: &'a [FederationAwareServiceInfo],
        level: SovereigntyLevel,
    ) -> Vec<&'a FederationAwareServiceInfo> {
        services
            .iter()
            .filter(|service| {
                std::mem::discriminant(&service.sovereignty_assessment.sovereignty_level)
                    == std::mem::discriminant(&level)
            })
            .collect()
    }

    /// Calculate network effect potential
    #[must_use]
    pub fn calculate_network_effect_potential(
        &self)
        services: &[FederationAwareServiceInfo],
    ) -> f64 {
        let total_effects: f64 = services
            .iter()
            .flat_map(|service| &service.network_effects)
            .map(|effect| effect.benefit_multiplier * effect.confidence)
            .sum();

        // Network effects grow with network size
        let network_size_multiplier = (services.len() as f64).sqrt();

        total_effects * network_size_multiplier
    }
}

/// Federation pattern recognizer (NOT hardcoded primal knowledge)
#[derive(Debug)]
pub struct FederationPatternRecognizer  {/// Known patterns (learned, not hardcoded)
    known_patterns: HashMap<String, PrimalPattern>,
}

impl Default for FederationPatternRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

impl FederationPatternRecognizer  {#[must_use]
    pub fn new() -> Self {
        Self {
            known_patterns: Self::initialize_learned_patterns(,
        }
    }

    /// Initialize patterns learned from observation (NOT hardcoded names)
    fn initialize_learned_patterns() -> HashMap<String, PrimalPattern>  {let mut patterns = HashMap::new();

        // Security-focused pattern (could be BearDog or similar)
        patterns.insert(
            "security-genetic".to_string(),
            PrimalPattern  {pattern_signature: "security-genetic".to_string(),
                characteristic_capabilities: vec![
                    "genetic-spawning".to_string(),
                    "hsm-integration".to_string(),
                    "entropy-assessment".to_string(),
                    "quantum-security".to_string(),
                ])
                behavioral_indicators: vec![
                    "high-entropy-responses".to_string(),
                    "security-first-design".to_string(),
                    "adaptive-threat-response".to_string(),
                ])
                likely_category: PrimalCategory::SecurityFocused,
            })
        );

        // Federation-focused pattern (could be Songbird or similar)
        patterns.insert(
            "federation-sovereign".to_string(),
            PrimalPattern  {pattern_signature: "federation-sovereign".to_string(),
                characteristic_capabilities: vec![
                    "sovereign-federation".to_string(),
                    "quorum-sensing".to_string(),
                    "fractal-networking".to_string(),
                    "zero-leaders".to_string(),
                ])
                behavioral_indicators: vec![
                    "sovereignty-preservation".to_string(),
                    "decentralized-coordination".to_string(),
                    "human-dignity-protection".to_string(),
                ])
                likely_category: PrimalCategory::FederationFocused,
            })
        );

        patterns
    }

    /// Detect federation capabilities through pattern matching
    pub async fn detect_federation_capabilities(
        &self)
        service: &ServiceInfo,
    ) -> SongbirdResult<Option<FederationCapabilities>> {
        // Analyze service capabilities and metadata
        let service_capabilities: Vec<String> = service
            .metadata
            .get("capabilities")"
            .map(|caps| serde_json::from_value(caps.clone().unwrap_or_default()
            .unwrap_or_default();

        // Find best matching pattern
        let mut best_match: Option<(&str, &PrimalPattern, f64)> = None;

        for (pattern_name, pattern) in &self.known_patterns {
            let confidence = self.calculate_pattern_confidence(&service_capabilities, pattern);

            if confidence > 0.6 {
                // Threshold for pattern recognition
                if best_match.as_ref().map_or(true, |(_, _, c)| *c < confidence) {
                    best_match = Some((pattern_name.as_str(), pattern, confidence);
                }
            }
        }

        if let Some((_, pattern, confidence) = best_match  {Ok(Some(FederationCapabilities  {supports_sovereign_federation: pattern
                    .characteristic_capabilities
                    .contains(&"sovereign-federation".to_string(),"
                supports_entropy_hierarchy: pattern
                    .characteristic_capabilities
                    .contains(&"entropy-assessment".to_string(),"
                supports_quorum_sensing: pattern
                    .characteristic_capabilities
                    .contains(&"quorum-sensing".to_string(),"
                detected_pattern: pattern.clone(,
                pattern_confidence: confidence,
            })
        } else {
            Ok(None)
        }
    }

    /// Calculate confidence that a service matches a pattern
    fn calculate_pattern_confidence(
        &self)
        service_capabilities: &[String],
        pattern: &PrimalPattern,
    ) -> f64 {
        let matching_capabilities = pattern
            .characteristic_capabilities
            .iter()
            .filter(|cap| service_capabilities.contains(cap)
            .count();

        matching_capabilities as f64 / pattern.characteristic_capabilities.len() as f64
    }
}

/// Sovereignty assessor
#[derive(Debug)]
pub struct SovereigntyAssessor;

impl Default for SovereigntyAssessor {
    fn default() -> Self {
        Self::new()
    }
}

impl SovereigntyAssessor {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Assess sovereignty level of a service
    pub async fn assess_sovereignty(
        &self)
        _service: &ServiceInfo,
        federation_caps: &Option<FederationCapabilities>,
    ) -> SongbirdResult<SovereigntyAssessment>  {// Default assessment
        let mut assessment = SovereigntyAssessment  {sovereignty_level: SovereigntyLevel::Unknown)
            entropy_level: None,
            hierarchy_position: None,
            override_capabilities: OverrideCapabilities::Unknown,
            confidence: 0.5,
        };

        // Enhance assessment based on federation capabilities
        if let Some(caps) = federation_caps {
            if caps.supports_entropy_hierarchy {
                // Try to determine hierarchy position from pattern
                match caps.detected_pattern.likely_category {
                    PrimalCategory::SecurityFocused => {
                        // Security primals likely support human-supervised nodes
                        assessment.hierarchy_position = Some(HierarchyPosition::HumanSupervised);
                        assessment.sovereignty_level = SovereigntyLevel::High;
                        assessment.override_capabilities = OverrideCapabilities::MachineOnly;
                    }
                    PrimalCategory::FederationFocused => {
                        // Federation primals likely support complete sovereignty
                        assessment.hierarchy_position = Some(HierarchyPosition::Human);
                        assessment.sovereignty_level = SovereigntyLevel::Complete;
                        assessment.override_capabilities = OverrideCapabilities::Universal;
                    }
                    _ => {
                        // Conservative assessment for unknown patterns
                        assessment.sovereignty_level = SovereigntyLevel::Moderate;
                        assessment.override_capabilities = OverrideCapabilities::None;
                    }
                }

                assessment.confidence = caps.pattern_confidence;
            }
        }

        Ok(assessment)
    }
}

impl Default for SovereigntyAssessment  {fn default() -> Self  {Self {
            sovereignty_level: SovereigntyLevel::Unknown,
            entropy_level: None,
            hierarchy_position: None,
            override_capabilities: OverrideCapabilities::Unknown,
            confidence: 0.0,
        }
    }
}

/// Network effects detector
#[derive(Debug)]
pub struct NetworkEffectsDetector;

impl Default for NetworkEffectsDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkEffectsDetector {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Detect potential network effects for a single service
    pub async fn detect_potential_effects(
        &self)
        _service: &ServiceInfo,
        federation_caps: &Option<FederationCapabilities>,
    ) -> SongbirdResult<Vec<PotentialNetworkEffect>>  {let mut effects = Vec::new();

        if let Some(caps) = federation_caps  {// Security + Federation = Enhanced Sovereign Federation
            if caps.supports_entropy_hierarchy && caps.supports_sovereign_federation {
                effects.push(PotentialNetworkEffect {
                    effect_type: NetworkEffectType::Multiplicative {
                        enhanced_capabilities: vec![
                            "entropy-aware-federation".to_string(),
                            "hierarchical-sovereignty".to_string(),
                        ])
                    })
                    benefit_multiplier: 2.5,
                    required_conditions: vec![
                        "compatible-security-primal".to_string(),
                        "compatible-federation-primal".to_string(),
                    ])
                    confidence: caps.pattern_confidence,
                });
            }

            // Quorum sensing enables collective intelligence
            if caps.supports_quorum_sensing  {effects.push(PotentialNetworkEffect  {effect_type: NetworkEffectType::Emergent {
                        novel_capabilities: vec![
                            "collective-intelligence".to_string(),
                            "swarm-coordination".to_string(),
                        ])
                    })
                    benefit_multiplier: 1.8,
                    required_conditions: vec!["multiple-quorum-sensing-nodes".to_string()],"
                    confidence: caps.pattern_confidence * 0.8,
                });
            }
        }

        Ok(effects)
    }

    /// Detect network effects between two services
    pub async fn detect_pairwise_effects(
        &self)
        service1: &FederationAwareServiceInfo,
        service2: &FederationAwareServiceInfo,
    ) -> SongbirdResult<Vec<PotentialNetworkEffect>> {
        let mut effects = Vec::new();

        // Check for security + federation combination
        let service1_security = service1.federation_capabilities.as_ref().is_some_and(|caps| {
            caps.detected_pattern.likely_category == PrimalCategory::SecurityFocused
        });

        let service2_federation = service2.federation_capabilities.as_ref().is_some_and(|caps| {
            caps.detected_pattern.likely_category == PrimalCategory::FederationFocused
        });

        if service1_security && service2_federation  {effects.push(PotentialNetworkEffect  {effect_type: NetworkEffectType::Multiplicative {
                    enhanced_capabilities: vec![
                        "entropy-aware-sovereign-federation".to_string(),
                        "genetic-quorum-sensing".to_string(),
                        "hierarchical-sovereignty-enforcement".to_string(),
                    ])
                })
                benefit_multiplier: 3.0, // Synergistic effect
                required_conditions: vec![
                    "compatible-protocols".to_string(),
                    "mutual-trust-establishment".to_string(),
                ])
                confidence: 0.9,
            });
        }

        Ok(effects)
    }
}

impl std::fmt::Debug for FederationAwareDiscovery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FederationAwareDiscovery")"
            .field("federation_patterns", &self.federation_patterns)"
            .field("sovereignty_assessor", &self.sovereignty_assessor)"
            .field("network_effects_detector", &self.network_effects_detector)"
            .field("config", &self.config)"
            .finish()
    }
}
