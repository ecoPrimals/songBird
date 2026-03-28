// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Pattern recognition, sovereignty assessment, and network-effects detection for federation-aware discovery.

use crate::traits::ServiceInfo;
use songbird_types::SongbirdResult;
use std::collections::HashMap;

use super::{
    FederationAwareServiceInfo, FederationCapabilities, HierarchyPosition, NetworkEffectType,
    OverrideCapabilities, PotentialNetworkEffect, PrimalCategory, PrimalPattern,
    SovereigntyAssessment, SovereigntyLevel,
};

/// Federation pattern recognizer (NOT hardcoded primal knowledge)
#[derive(Debug)]
pub struct FederationPatternRecognizer {
    /// Known patterns (learned, not hardcoded)
    known_patterns: HashMap<String, PrimalPattern>,
}

impl Default for FederationPatternRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

impl FederationPatternRecognizer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            known_patterns: Self::initialize_learned_patterns(),
        }
    }

    fn initialize_learned_patterns() -> HashMap<String, PrimalPattern> {
        let mut patterns = HashMap::new();

        patterns.insert(
            "security-genetic".to_string(),
            PrimalPattern {
                pattern_signature: "security-genetic".to_string(),
                characteristic_capabilities: vec![
                    "genetic-spawning".to_string(),
                    "hsm-integration".to_string(),
                    "entropy-assessment".to_string(),
                    "quantum-security".to_string(),
                ],
                behavioral_indicators: vec![
                    "high-entropy-responses".to_string(),
                    "security-first-design".to_string(),
                    "adaptive-threat-response".to_string(),
                ],
                likely_category: PrimalCategory::SecurityFocused,
            },
        );

        patterns.insert(
            "federation-sovereign".to_string(),
            PrimalPattern {
                pattern_signature: "federation-sovereign".to_string(),
                characteristic_capabilities: vec![
                    "sovereign-federation".to_string(),
                    "quorum-sensing".to_string(),
                    "fractal-networking".to_string(),
                    "zero-leaders".to_string(),
                ],
                behavioral_indicators: vec![
                    "sovereignty-preservation".to_string(),
                    "decentralized-coordination".to_string(),
                    "human-dignity-protection".to_string(),
                ],
                likely_category: PrimalCategory::FederationFocused,
            },
        );

        patterns
    }

    /// Detect federation capabilities through pattern matching
    pub async fn detect_federation_capabilities(
        &self,
        service: &ServiceInfo,
    ) -> SongbirdResult<Option<FederationCapabilities>> {
        let mut service_capabilities = service.tags.clone();

        if let Some(serde_json::Value::Array(arr)) = service.metadata.get("capabilities") {
            for v in arr {
                if let Some(s) = v.as_str() {
                    let t = s.to_string();
                    if !service_capabilities.contains(&t) {
                        service_capabilities.push(t);
                    }
                }
            }
        }

        let mut best_match: Option<(&str, &PrimalPattern, f64)> = None;

        for (pattern_name, pattern) in &self.known_patterns {
            let confidence = self.calculate_pattern_confidence(&service_capabilities, pattern);

            if confidence > 0.6 && best_match.as_ref().is_none_or(|(_, _, c)| *c < confidence) {
                best_match = Some((pattern_name.as_str(), pattern, confidence));
            }
        }

        if let Some((_, pattern, confidence)) = best_match {
            Ok(Some(FederationCapabilities {
                supports_sovereign_federation: pattern
                    .characteristic_capabilities
                    .contains(&"sovereign-federation".to_string()),
                supports_entropy_hierarchy: pattern
                    .characteristic_capabilities
                    .contains(&"entropy-assessment".to_string()),
                supports_quorum_sensing: pattern
                    .characteristic_capabilities
                    .contains(&"quorum-sensing".to_string()),
                detected_pattern: pattern.clone(),
                pattern_confidence: confidence,
            }))
        } else {
            Ok(None)
        }
    }

    fn calculate_pattern_confidence(
        &self,
        service_capabilities: &[String],
        pattern: &PrimalPattern,
    ) -> f64 {
        let matching_capabilities = pattern
            .characteristic_capabilities
            .iter()
            .filter(|cap| service_capabilities.contains(cap))
            .count();

        if pattern.characteristic_capabilities.is_empty() {
            return 0.0;
        }

        #[expect(clippy::cast_precision_loss, reason = "intentional for scoring calculation")]
        let num = matching_capabilities as f64;
        #[expect(clippy::cast_precision_loss, reason = "intentional for scoring calculation")]
        let den = pattern.characteristic_capabilities.len() as f64;
        num / den
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
    pub const fn new() -> Self {
        Self
    }

    /// Assess sovereignty level of a service
    pub async fn assess_sovereignty(
        &self,
        _service: &ServiceInfo,
        federation_caps: &Option<FederationCapabilities>,
    ) -> SongbirdResult<SovereigntyAssessment> {
        let mut assessment = SovereigntyAssessment {
            sovereignty_level: SovereigntyLevel::Unknown,
            entropy_level: None,
            hierarchy_position: None,
            override_capabilities: OverrideCapabilities::Unknown,
            confidence: 0.5,
        };

        if let Some(caps) = federation_caps.as_ref().filter(|c| c.supports_entropy_hierarchy) {
            match caps.detected_pattern.likely_category {
                PrimalCategory::SecurityFocused => {
                    assessment.hierarchy_position = Some(HierarchyPosition::HumanSupervised);
                    assessment.sovereignty_level = SovereigntyLevel::High;
                    assessment.override_capabilities = OverrideCapabilities::MachineOnly;
                }
                PrimalCategory::FederationFocused => {
                    assessment.hierarchy_position = Some(HierarchyPosition::Human);
                    assessment.sovereignty_level = SovereigntyLevel::Complete;
                    assessment.override_capabilities = OverrideCapabilities::Universal;
                }
                _ => {
                    assessment.sovereignty_level = SovereigntyLevel::Moderate;
                    assessment.override_capabilities = OverrideCapabilities::None;
                }
            }

            assessment.confidence = caps.pattern_confidence;
        }

        Ok(assessment)
    }
}

impl Default for SovereigntyAssessment {
    fn default() -> Self {
        Self {
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
    pub const fn new() -> Self {
        Self
    }

    /// Detect potential network effects for a single service
    pub async fn detect_potential_effects(
        &self,
        _service: &ServiceInfo,
        federation_caps: &Option<FederationCapabilities>,
    ) -> SongbirdResult<Vec<PotentialNetworkEffect>> {
        let mut effects = Vec::new();

        if let Some(caps) = federation_caps {
            if caps.supports_entropy_hierarchy && caps.supports_sovereign_federation {
                effects.push(PotentialNetworkEffect {
                    effect_type: NetworkEffectType::Multiplicative {
                        enhanced_capabilities: vec![
                            "entropy-aware-federation".to_string(),
                            "hierarchical-sovereignty".to_string(),
                        ],
                    },
                    benefit_multiplier: 2.5,
                    required_conditions: vec![
                        "compatible-security-primal".to_string(),
                        "compatible-federation-primal".to_string(),
                    ],
                    confidence: caps.pattern_confidence,
                });
            }

            if caps.supports_quorum_sensing {
                effects.push(PotentialNetworkEffect {
                    effect_type: NetworkEffectType::Emergent {
                        novel_capabilities: vec![
                            "collective-intelligence".to_string(),
                            "swarm-coordination".to_string(),
                        ],
                    },
                    benefit_multiplier: 1.8,
                    required_conditions: vec!["multiple-quorum-sensing-nodes".to_string()],
                    confidence: caps.pattern_confidence * 0.8,
                });
            }
        }

        Ok(effects)
    }

    /// Detect network effects between two services
    pub async fn detect_pairwise_effects(
        &self,
        service1: &FederationAwareServiceInfo,
        service2: &FederationAwareServiceInfo,
    ) -> SongbirdResult<Vec<PotentialNetworkEffect>> {
        let mut effects = Vec::new();

        let service1_security = service1.federation_capabilities.as_ref().is_some_and(|caps| {
            caps.detected_pattern.likely_category == PrimalCategory::SecurityFocused
        });

        let service2_federation = service2.federation_capabilities.as_ref().is_some_and(|caps| {
            caps.detected_pattern.likely_category == PrimalCategory::FederationFocused
        });

        if service1_security && service2_federation {
            effects.push(PotentialNetworkEffect {
                effect_type: NetworkEffectType::Multiplicative {
                    enhanced_capabilities: vec![
                        "entropy-aware-sovereign-federation".to_string(),
                        "genetic-quorum-sensing".to_string(),
                        "hierarchical-sovereignty-enforcement".to_string(),
                    ],
                },
                benefit_multiplier: 3.0,
                required_conditions: vec![
                    "compatible-protocols".to_string(),
                    "mutual-trust-establishment".to_string(),
                ],
                confidence: 0.9,
            });
        }

        Ok(effects)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::federation_aware_discovery::{
        DiscoveryMetadata, FederationAwareServiceInfo, FederationCapabilities, PrimalPattern,
    };
    use crate::traits::service::{ServiceInfo, ServiceStatus};
    use chrono::Utc;
    use std::collections::HashMap;
    use std::time::SystemTime;

    fn sample_service(
        tags: Vec<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> ServiceInfo {
        let now = Utc::now();
        ServiceInfo {
            service_id: "fed-1".to_string(),
            name: "n".to_string(),
            version: "1.0.0".to_string(),
            service_type: "t".to_string(),
            description: None,
            endpoints: vec![],
            health_check_endpoint: None,
            metadata,
            tags,
            dependencies: vec![],
            status: ServiceStatus::Running,
            created_at: now,
            updated_at: now,
            instance_id: "i".to_string(),
            host: "127.0.0.1".to_string(),
            port: 9000,
        }
    }

    fn fed_aware_shell(
        base: ServiceInfo,
        caps: Option<FederationCapabilities>,
    ) -> FederationAwareServiceInfo {
        FederationAwareServiceInfo {
            base_info: base,
            federation_capabilities: caps,
            sovereignty_assessment: SovereigntyAssessment::default(),
            network_effects: vec![],
            discovery_metadata: DiscoveryMetadata {
                discovered_at: SystemTime::UNIX_EPOCH,
                discovery_method: "unit".to_string(),
                last_health_check: None,
                discovery_confidence: 1.0,
            },
        }
    }

    #[tokio::test]
    async fn pattern_recognizer_empty_service_no_match() {
        let r = FederationPatternRecognizer::new();
        let s = sample_service(vec![], HashMap::new());
        let out = r.detect_federation_capabilities(&s).await.unwrap();
        assert!(out.is_none());
    }

    #[tokio::test]
    async fn pattern_recognizer_security_genetic_high_confidence() {
        let r = FederationPatternRecognizer::new();
        let s = sample_service(
            vec![
                "genetic-spawning".to_string(),
                "hsm-integration".to_string(),
                "entropy-assessment".to_string(),
            ],
            HashMap::new(),
        );
        let out = r.detect_federation_capabilities(&s).await.unwrap().unwrap();
        assert!(out.supports_entropy_hierarchy);
        assert_eq!(out.detected_pattern.likely_category, PrimalCategory::SecurityFocused);
        assert!(out.pattern_confidence > 0.6);
    }

    #[tokio::test]
    async fn pattern_recognizer_capabilities_from_metadata_array() {
        let r = FederationPatternRecognizer::new();
        let mut meta = HashMap::new();
        meta.insert(
            "capabilities".to_string(),
            serde_json::json!([
                "genetic-spawning",
                "hsm-integration",
                "entropy-assessment",
                "quantum-security"
            ]),
        );
        let s = sample_service(vec![], meta);
        let out = r.detect_federation_capabilities(&s).await.unwrap().unwrap();
        assert_eq!(out.pattern_confidence, 1.0);
    }

    #[tokio::test]
    async fn sovereignty_security_focused_with_entropy_hierarchy() {
        let assessor = SovereigntyAssessor::new();
        let caps = FederationCapabilities {
            supports_sovereign_federation: false,
            supports_entropy_hierarchy: true,
            supports_quorum_sensing: false,
            detected_pattern: PrimalPattern {
                pattern_signature: "security-genetic".to_string(),
                characteristic_capabilities: vec![],
                behavioral_indicators: vec![],
                likely_category: PrimalCategory::SecurityFocused,
            },
            pattern_confidence: 0.88,
        };
        let s = sample_service(vec![], HashMap::new());
        let a = assessor.assess_sovereignty(&s, &Some(caps)).await.unwrap();
        assert!(matches!(a.sovereignty_level, SovereigntyLevel::High));
        assert!(matches!(a.hierarchy_position, Some(HierarchyPosition::HumanSupervised)));
        assert!(matches!(a.override_capabilities, OverrideCapabilities::MachineOnly));
        assert!((a.confidence - 0.88).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn sovereignty_no_caps_stays_unknown() {
        let assessor = SovereigntyAssessor::new();
        let s = sample_service(vec![], HashMap::new());
        let a = assessor.assess_sovereignty(&s, &None).await.unwrap();
        assert!(matches!(a.sovereignty_level, SovereigntyLevel::Unknown));
    }

    #[tokio::test]
    async fn network_effects_multiplicative_and_emergent() {
        let d = NetworkEffectsDetector::new();
        let caps = FederationCapabilities {
            supports_sovereign_federation: true,
            supports_entropy_hierarchy: true,
            supports_quorum_sensing: true,
            detected_pattern: PrimalPattern {
                pattern_signature: "federation-sovereign".to_string(),
                characteristic_capabilities: vec![],
                behavioral_indicators: vec![],
                likely_category: PrimalCategory::FederationFocused,
            },
            pattern_confidence: 0.8,
        };
        let s = sample_service(vec![], HashMap::new());
        let effects = d.detect_potential_effects(&s, &Some(caps)).await.unwrap();
        assert_eq!(effects.len(), 2);
        assert!((effects[0].benefit_multiplier - 2.5).abs() < f64::EPSILON);
        assert!((effects[1].benefit_multiplier - 1.8).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn network_effects_pairwise_security_and_federation() {
        let d = NetworkEffectsDetector::new();
        let sec = FederationCapabilities {
            supports_sovereign_federation: false,
            supports_entropy_hierarchy: true,
            supports_quorum_sensing: false,
            detected_pattern: PrimalPattern {
                pattern_signature: "security-genetic".to_string(),
                characteristic_capabilities: vec![],
                behavioral_indicators: vec![],
                likely_category: PrimalCategory::SecurityFocused,
            },
            pattern_confidence: 0.7,
        };
        let fed = FederationCapabilities {
            supports_sovereign_federation: true,
            supports_entropy_hierarchy: true,
            supports_quorum_sensing: true,
            detected_pattern: PrimalPattern {
                pattern_signature: "federation-sovereign".to_string(),
                characteristic_capabilities: vec![],
                behavioral_indicators: vec![],
                likely_category: PrimalCategory::FederationFocused,
            },
            pattern_confidence: 0.8,
        };
        let s1 = sample_service(vec![], HashMap::new());
        let s2 = sample_service(vec![], HashMap::new());
        let fa1 = fed_aware_shell(s1, Some(sec));
        let fa2 = fed_aware_shell(s2, Some(fed));
        let pair = d.detect_pairwise_effects(&fa1, &fa2).await.unwrap();
        assert_eq!(pair.len(), 1);
        assert!((pair[0].benefit_multiplier - 3.0).abs() < f64::EPSILON);
    }
}
