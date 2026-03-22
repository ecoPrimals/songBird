// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::*;
use crate::traits::discovery::{ServiceEvent, ServiceHealthStatus};
use crate::traits::service::{ServiceEndpoint, ServiceInfo, ServiceStatus};
use chrono::Utc;
use futures_util::stream;
use std::collections::HashMap;
use std::pin::Pin;

fn sample_service(
    id: &str,
    tags: Vec<String>,
    metadata_capabilities: Option<serde_json::Value>,
) -> ServiceInfo {
    let mut metadata = HashMap::new();
    if let Some(c) = metadata_capabilities {
        metadata.insert("capabilities".to_string(), c);
    }

    ServiceInfo {
        service_id: id.to_string(),
        name: id.to_string(),
        version: "1.0.0".to_string(),
        service_type: "test".to_string(),
        description: None,
        endpoints: vec![ServiceEndpoint {
            path: "/".into(),
            method: "GET".into(),
            description: None,
            parameters: vec![],
            response_schema: None,
            auth_required: false,
            rate_limit: None,
        }],
        health_check_endpoint: None,
        metadata,
        tags,
        dependencies: vec![],
        status: ServiceStatus::Running,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        instance_id: id.to_string(),
        host: "127.0.0.1".into(),
        port: 8080,
    }
}

/// Minimal stub for driving federation-aware discovery without network I/O.
struct StubDiscovery {
    services: Vec<ServiceInfo>,
}

impl ServiceDiscovery for StubDiscovery {
    async fn discover(&self, _query: ServiceQuery) -> SongbirdResult<Vec<ServiceInfo>> {
        Ok(self.services.clone())
    }

    async fn register(&self, _service: ServiceInfo) -> SongbirdResult<()> {
        Ok(())
    }

    async fn unregister(&self, _service_id: &str) -> SongbirdResult<()> {
        Ok(())
    }

    async fn watch(
        &self,
        _query: ServiceQuery,
    ) -> SongbirdResult<Pin<Box<dyn futures_util::Stream<Item = ServiceEvent> + Send>>> {
        Ok(Box::pin(stream::empty()))
    }

    async fn update_health(
        &self,
        _service_id: &str,
        _health: ServiceHealthStatus,
    ) -> SongbirdResult<()> {
        Ok(())
    }

    async fn list_all(&self) -> SongbirdResult<Vec<ServiceInfo>> {
        Ok(self.services.clone())
    }

    async fn exists(&self, _service_id: &str) -> SongbirdResult<bool> {
        Ok(false)
    }

    async fn is_registered(&self, _service_id: &str) -> SongbirdResult<bool> {
        Ok(false)
    }

    async fn update_metadata(
        &self,
        _service_id: &str,
        _metadata: HashMap<String, String>,
    ) -> SongbirdResult<()> {
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[test]
fn federation_discovery_config_default() {
    let c = FederationDiscoveryConfig::default();
    assert!(c.enable_federation_patterns);
    assert_eq!(c.federation_timeout, Duration::from_secs(5));
}

#[test]
fn get_sovereign_federation_services_filters() {
    let fed = FederationAwareDiscovery::new(
        StubDiscovery {
            services: vec![],
        },
        FederationDiscoveryConfig::default(),
    );

    let with_caps = FederationAwareServiceInfo {
        base_info: sample_service("a", vec![], None),
        federation_capabilities: Some(FederationCapabilities {
            supports_sovereign_federation: true,
            supports_entropy_hierarchy: false,
            supports_quorum_sensing: false,
            detected_pattern: PrimalPattern {
                pattern_signature: "p".into(),
                characteristic_capabilities: vec![],
                behavioral_indicators: vec![],
                likely_category: PrimalCategory::FederationFocused,
            },
            pattern_confidence: 0.9,
        }),
        sovereignty_assessment: SovereigntyAssessment::default(),
        network_effects: vec![],
        discovery_metadata: DiscoveryMetadata {
            discovered_at: SystemTime::UNIX_EPOCH,
            discovery_method: "test".into(),
            last_health_check: None,
            discovery_confidence: 1.0,
        },
    };

    let no_caps = FederationAwareServiceInfo {
        base_info: sample_service("b", vec![], None),
        federation_capabilities: None,
        sovereignty_assessment: SovereigntyAssessment::default(),
        network_effects: vec![],
        discovery_metadata: DiscoveryMetadata {
            discovered_at: SystemTime::UNIX_EPOCH,
            discovery_method: "test".into(),
            last_health_check: None,
            discovery_confidence: 1.0,
        },
    };

    let slice = [with_caps, no_caps];
    let out = fed.get_sovereign_federation_services(&slice);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].base_info.service_id, "a");
}

#[test]
fn get_services_by_sovereignty_level_matches_discriminant() {
    let fed = FederationAwareDiscovery::new(
        StubDiscovery {
            services: vec![],
        },
        FederationDiscoveryConfig::default(),
    );

    let s = FederationAwareServiceInfo {
        base_info: sample_service("x", vec![], None),
        federation_capabilities: None,
        sovereignty_assessment: SovereigntyAssessment {
            sovereignty_level: SovereigntyLevel::High,
            entropy_level: None,
            hierarchy_position: None,
            override_capabilities: OverrideCapabilities::MachineOnly,
            confidence: 0.8,
        },
        network_effects: vec![],
        discovery_metadata: DiscoveryMetadata {
            discovered_at: SystemTime::UNIX_EPOCH,
            discovery_method: "test".into(),
            last_health_check: None,
            discovery_confidence: 1.0,
        },
    };

    let matched =
        fed.get_services_by_sovereignty_level(std::slice::from_ref(&s), &SovereigntyLevel::High);
    assert_eq!(matched.len(), 1);
}

#[test]
fn calculate_network_effect_potential_scales_with_size() {
    let fed = FederationAwareDiscovery::new(
        StubDiscovery {
            services: vec![],
        },
        FederationDiscoveryConfig::default(),
    );

    let effect = PotentialNetworkEffect {
        effect_type: NetworkEffectType::SecurityEnhancement {
            security_boost: 1.0,
        },
        benefit_multiplier: 2.0,
        required_conditions: vec![],
        confidence: 0.5,
    };

    let one = FederationAwareServiceInfo {
        base_info: sample_service("n1", vec![], None),
        federation_capabilities: None,
        sovereignty_assessment: SovereigntyAssessment::default(),
        network_effects: vec![effect.clone()],
        discovery_metadata: DiscoveryMetadata {
            discovered_at: SystemTime::UNIX_EPOCH,
            discovery_method: "test".into(),
            last_health_check: None,
            discovery_confidence: 1.0,
        },
    };

    let two = FederationAwareServiceInfo {
        base_info: sample_service("n2", vec![], None),
        federation_capabilities: None,
        sovereignty_assessment: SovereigntyAssessment::default(),
        network_effects: vec![effect],
        discovery_metadata: DiscoveryMetadata {
            discovered_at: SystemTime::UNIX_EPOCH,
            discovery_method: "test".into(),
            last_health_check: None,
            discovery_confidence: 1.0,
        },
    };

    let p1 = fed.calculate_network_effect_potential(std::slice::from_ref(&one));
    let p2 = fed.calculate_network_effect_potential(&[one, two]);
    assert!(p2 > p1);
}

#[tokio::test]
async fn discover_federation_aware_runs_end_to_end() {
    let tags = vec![
        "sovereign-federation".to_string(),
        "entropy-assessment".to_string(),
        "quorum-sensing".to_string(),
        "fractal-networking".to_string(),
    ];
    let svc = sample_service("fed-svc", tags, None);

    let fed = FederationAwareDiscovery::new(
        StubDiscovery {
            services: vec![svc],
        },
        FederationDiscoveryConfig::default(),
    );

    let out = fed.discover_federation_aware_services().await.unwrap();
    assert_eq!(out.len(), 1);
    assert!(out[0].federation_capabilities.is_some());
    assert!(!out[0].network_effects.is_empty());
}

#[tokio::test]
async fn pattern_detect_uses_metadata_capabilities_array() {
    let recognizer = FederationPatternRecognizer::new();
    let svc = sample_service(
        "meta",
        vec![],
        Some(serde_json::json!([
            "sovereign-federation",
            "entropy-assessment",
            "quorum-sensing",
            "fractal-networking"
        ])),
    );

    let caps = recognizer.detect_federation_capabilities(&svc).await.unwrap();
    assert!(caps.is_some());
    let c = caps.unwrap();
    assert!(c.supports_sovereign_federation);
    assert!(c.supports_quorum_sensing);
}

#[tokio::test]
async fn pairwise_security_and_federation_synergy() {
    let detector = NetworkEffectsDetector::new();

    let sec = FederationAwareServiceInfo {
        base_info: sample_service("s", vec![], None),
        federation_capabilities: Some(FederationCapabilities {
            supports_sovereign_federation: false,
            supports_entropy_hierarchy: true,
            supports_quorum_sensing: false,
            detected_pattern: PrimalPattern {
                pattern_signature: "security-genetic".into(),
                characteristic_capabilities: vec![],
                behavioral_indicators: vec![],
                likely_category: PrimalCategory::SecurityFocused,
            },
            pattern_confidence: 0.9,
        }),
        sovereignty_assessment: SovereigntyAssessment::default(),
        network_effects: vec![],
        discovery_metadata: DiscoveryMetadata {
            discovered_at: SystemTime::UNIX_EPOCH,
            discovery_method: "test".into(),
            last_health_check: None,
            discovery_confidence: 1.0,
        },
    };

    let fed = FederationAwareServiceInfo {
        base_info: sample_service("f", vec![], None),
        federation_capabilities: Some(FederationCapabilities {
            supports_sovereign_federation: true,
            supports_entropy_hierarchy: true,
            supports_quorum_sensing: false,
            detected_pattern: PrimalPattern {
                pattern_signature: "federation-sovereign".into(),
                characteristic_capabilities: vec![],
                behavioral_indicators: vec![],
                likely_category: PrimalCategory::FederationFocused,
            },
            pattern_confidence: 0.9,
        }),
        sovereignty_assessment: SovereigntyAssessment::default(),
        network_effects: vec![],
        discovery_metadata: DiscoveryMetadata {
            discovered_at: SystemTime::UNIX_EPOCH,
            discovery_method: "test".into(),
            last_health_check: None,
            discovery_confidence: 1.0,
        },
    };

    let pe = detector.detect_pairwise_effects(&sec, &fed).await.unwrap();
    assert_eq!(pe.len(), 1);
}

#[test]
fn federation_aware_service_info_roundtrips_json() {
    let info = FederationAwareServiceInfo {
        base_info: sample_service("id", vec!["t".into()], None),
        federation_capabilities: None,
        sovereignty_assessment: SovereigntyAssessment::default(),
        network_effects: vec![],
        discovery_metadata: DiscoveryMetadata {
            discovered_at: SystemTime::UNIX_EPOCH,
            discovery_method: "m".into(),
            last_health_check: None,
            discovery_confidence: 0.5,
        },
    };

    let json = serde_json::to_string(&info).unwrap();
    let back: FederationAwareServiceInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(back.base_info.service_id, "id");
}
