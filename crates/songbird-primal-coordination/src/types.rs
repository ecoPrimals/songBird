// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Common types for primal coordination
//!
//! **ZERO HARDCODING**: No primal names, only capabilities

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Capability type - WHAT is needed, not WHO provides it
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CapabilityType {
    /// Security operations (key management, signing, encryption)
    Security,
    /// Compute operations (ML inference, task execution)
    Compute,
    /// Storage operations (data persistence, caching)
    Storage,
    /// AI operations (model serving, training)
    Ai,
    /// Discovery operations (service registration, lookup)
    Discovery,
    /// Orchestration operations (coordination, scheduling)
    Orchestration,
    /// Networking operations (P2P, federation)
    Networking,
    /// Custom capability
    #[serde(untagged)]
    Custom(String),
}

impl CapabilityType {
    /// Convert to string representation
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Security => "security",
            Self::Compute => "compute",
            Self::Storage => "storage",
            Self::Ai => "ai",
            Self::Discovery => "discovery",
            Self::Orchestration => "orchestration",
            Self::Networking => "networking",
            Self::Custom(s) => s,
        }
    }

    /// Parse from string
    #[must_use]
    #[expect(
        clippy::should_implement_trait,
        reason = "intentional pattern; clippy false positive for this API"
    )] // infallible parse (returns Self, not Result)
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "security" => Self::Security,
            "compute" => Self::Compute,
            "storage" => Self::Storage,
            "ai" => Self::Ai,
            "discovery" => Self::Discovery,
            "orchestration" => Self::Orchestration,
            "networking" => Self::Networking,
            _ => Self::Custom(s.to_string()),
        }
    }
}

impl std::fmt::Display for CapabilityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Capabilities offered by a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalCapabilities {
    /// Services provided
    pub services: Vec<String>,

    /// Resources available
    pub resources: HashMap<String, String>,

    /// Service metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Service quality metrics
    pub quality: ServiceQuality,
}

impl PrimalCapabilities {
    /// Check if this primal supports a specific capability
    #[must_use]
    pub fn supports_capability(&self, capability: &CapabilityType) -> bool {
        self.services.iter().any(|s| s == capability.as_str())
    }

    /// Check if this primal can handle a workload
    #[must_use]
    pub fn supports_workload(&self, workload: &Workload) -> bool {
        self.services.contains(&workload.service_type)
    }
}

/// Service quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceQuality {
    /// Average response time in milliseconds
    pub avg_response_time_ms: Option<u64>,

    /// Availability (0.0 - 1.0)
    pub availability: Option<f64>,

    /// Throughput (requests per second)
    pub throughput_rps: Option<f64>,

    /// Current load (0.0 - 1.0)
    pub current_load: Option<f64>,
}

impl Default for ServiceQuality {
    fn default() -> Self {
        Self {
            avg_response_time_ms: None,
            availability: Some(1.0),
            throughput_rps: None,
            current_load: Some(0.0),
        }
    }
}

/// Request sent from Songbird to a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum PrimalRequest {
    /// Discover primal capabilities
    DiscoverCapabilities,

    /// Generate cryptographic keys
    GenerateKeys,

    /// Sign lineage proof
    SignLineage {
        /// Keys produced during genesis key generation.
        keys: GeneratedKeys,
        /// Witness data to bind into the lineage signature.
        proof: WitnessProof,
        /// Node receiving the signed lineage.
        node_id: NodeId,
    },

    /// Deploy compute workload
    DeployWorkload(Workload),

    /// Query status
    Status,

    /// Custom request
    Custom {
        /// Operation name understood by the target primal.
        operation: String,
        /// JSON parameters for the operation.
        params: serde_json::Value,
    },
}

/// Response received by Songbird from a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum PrimalResponse {
    /// Capabilities discovered
    Capabilities(PrimalCapabilities),

    /// Keys generated successfully
    KeysGenerated(GeneratedKeys),

    /// Lineage signed successfully
    LineageSigned(Lineage),

    /// Workload deployed successfully
    WorkloadDeployed(DeploymentId),

    /// Status response
    StatusResponse(ServiceStatus),

    /// Error occurred
    Error(String),

    /// Custom response
    Custom(serde_json::Value),
}

/// Opaque node identifier used in genesis and coordination messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NodeId(
    /// Stable string form (often a UUID).
    pub String,
);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl NodeId {
    /// Create a new random node ID
    #[must_use]
    pub fn random() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

/// Identity bundle produced when genesis completes successfully.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    /// Participant node id.
    pub node_id: NodeId,
    /// Public key bytes advertised to the federation.
    pub public_key: Vec<u8>,
    /// Cryptographic lineage binding the node to its ancestors.
    pub lineage: Lineage,
    /// Proximity or witness data collected during the ceremony.
    pub witness_proof: WitnessProof,
}

/// Key material references returned by a security primal (private material stays remote).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedKeys {
    /// Public key bytes for verification.
    pub public_key: Vec<u8>,
    /// Opaque handle to private key material; not the raw secret.
    pub private_key_handle: String,
}

/// Opaque witness attestation blob built during genesis coordination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessProof {
    /// Serialized proof bytes (format defined by the security primal).
    pub data: Vec<u8>,
}

/// Signed lineage record for a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lineage {
    /// Opaque lineage payload bytes.
    pub data: Vec<u8>,
}

/// Workload description for compute primals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workload {
    /// Workload identifier.
    pub id: String,
    /// Service or workload type key used for routing.
    pub service_type: String,
    /// Resource or scheduling hints.
    pub requirements: HashMap<String, String>,
    /// Arbitrary JSON parameters for the target primal.
    pub payload: serde_json::Value,
}

/// Opaque deployment id returned after scheduling a workload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DeploymentId(
    /// Server-assigned deployment identifier string.
    pub String,
);

impl std::fmt::Display for DeploymentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DeploymentId {
    /// Create a new random deployment ID
    #[must_use]
    pub fn random() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

/// Health and capability snapshot reported by a primal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    /// Whether the primal considers itself healthy.
    pub healthy: bool,
    /// Reported software version string.
    pub version: String,
    /// Capability tags or service names.
    pub capabilities: Vec<String>,
    /// Optional diagnostic metrics as JSON values.
    pub metrics: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn capability_type_as_str_and_from_str() {
        assert_eq!(CapabilityType::Security.as_str(), "security");
        assert_eq!(CapabilityType::from_str("COMPUTE"), CapabilityType::Compute);
        assert_eq!(
            CapabilityType::from_str("custom_cap"),
            CapabilityType::Custom("custom_cap".into())
        );
        assert_eq!(CapabilityType::Storage.as_str(), "storage");
        assert_eq!(CapabilityType::Ai.as_str(), "ai");
        assert_eq!(CapabilityType::Discovery.as_str(), "discovery");
        assert_eq!(CapabilityType::Orchestration.as_str(), "orchestration");
        assert_eq!(CapabilityType::Networking.as_str(), "networking");
    }

    #[test]
    fn capability_type_display_matches_as_str() {
        assert_eq!(
            format!("{}", CapabilityType::Security),
            CapabilityType::Security.as_str(),
            "Display should match canonical capability string"
        );
        assert_eq!(
            format!("{}", CapabilityType::Custom("foo".into())),
            "foo",
            "Custom capability displays raw string"
        );
    }

    #[test]
    fn capability_type_hash_and_eq() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        assert!(set.insert(CapabilityType::Security));
        assert!(!set.insert(CapabilityType::Security));
        assert!(set.insert(CapabilityType::Custom("x".into())));
        assert_eq!(
            CapabilityType::from_str("SECURITY"),
            CapabilityType::Security,
            "from_str is case-insensitive for known names"
        );
    }

    #[test]
    fn capability_type_serde_known_and_custom() {
        let sec: CapabilityType = serde_json::from_str("\"security\"").expect("serde security");
        assert_eq!(sec, CapabilityType::Security);
        let custom: CapabilityType = serde_json::from_str("\"my_cap\"").expect("serde custom");
        assert_eq!(custom, CapabilityType::Custom("my_cap".into()));
        let round = serde_json::to_string(&CapabilityType::Compute).expect("to_string");
        let back: CapabilityType = serde_json::from_str(&round).expect("roundtrip");
        assert_eq!(back, CapabilityType::Compute);
    }

    #[test]
    fn primal_capabilities_supports_capability() {
        let caps = PrimalCapabilities {
            services: vec!["security".into(), "compute".into()],
            resources: HashMap::new(),
            metadata: HashMap::new(),
            quality: ServiceQuality::default(),
        };
        assert!(caps.supports_capability(&CapabilityType::Security));
        assert!(!caps.supports_capability(&CapabilityType::Ai));
    }

    #[test]
    fn primal_capabilities_supports_workload_negative() {
        let caps = PrimalCapabilities {
            services: vec!["batch".into()],
            resources: HashMap::new(),
            metadata: HashMap::new(),
            quality: ServiceQuality::default(),
        };
        let w = Workload {
            id: "1".into(),
            service_type: "other".into(),
            requirements: HashMap::new(),
            payload: serde_json::json!({}),
        };
        assert!(!caps.supports_workload(&w), "workload service_type must match a listed service");
    }

    #[test]
    fn service_quality_default_has_availability_and_zero_load() {
        let q = ServiceQuality::default();
        assert_eq!(q.availability, Some(1.0));
        assert_eq!(q.current_load, Some(0.0));
    }

    #[test]
    fn primal_request_response_roundtrip_json() {
        let req = PrimalRequest::DiscoverCapabilities;
        let v = serde_json::to_value(&req).unwrap();
        let back: PrimalRequest = serde_json::from_value(v).unwrap();
        assert!(matches!(back, PrimalRequest::DiscoverCapabilities));

        let resp = PrimalResponse::Error("oops".into());
        let v2 = serde_json::to_value(&resp).unwrap();
        let back2: PrimalResponse = serde_json::from_value(v2).unwrap();
        assert!(matches!(back2, PrimalResponse::Error(s) if s == "oops"));
    }

    #[test]
    fn node_id_and_deployment_id_random_are_non_empty() {
        assert!(!NodeId::random().0.is_empty());
        assert!(!DeploymentId::random().0.is_empty());
    }

    #[test]
    fn workload_roundtrip() {
        let w = Workload {
            id: "w1".into(),
            service_type: "svc".into(),
            requirements: HashMap::from([("cpu".into(), "1".into())]),
            payload: serde_json::json!({"k": 1}),
        };
        let json = serde_json::to_string(&w).unwrap();
        let w2: Workload = serde_json::from_str(&json).unwrap();
        assert_eq!(w2.id, w.id);
        assert_eq!(w2.service_type, w.service_type);
    }

    #[test]
    fn primal_request_custom_and_status_roundtrip() {
        let req = PrimalRequest::Custom {
            operation: "op".into(),
            params: serde_json::json!({"a": 1}),
        };
        let v = serde_json::to_value(&req).expect("serialize custom request");
        let back: PrimalRequest = serde_json::from_value(v).expect("deserialize custom request");
        assert!(
            matches!(back, PrimalRequest::Custom { ref operation, .. } if operation == "op"),
            "Custom request roundtrips through JSON"
        );

        let status_req = PrimalRequest::Status;
        let v2 = serde_json::to_value(&status_req).expect("status");
        let back2: PrimalRequest = serde_json::from_value(v2).expect("status back");
        assert!(matches!(back2, PrimalRequest::Status));
    }

    #[test]
    fn primal_response_variants_roundtrip() {
        let caps = PrimalCapabilities {
            services: vec!["s".into()],
            resources: HashMap::new(),
            metadata: HashMap::new(),
            quality: ServiceQuality::default(),
        };
        let r = PrimalResponse::Capabilities(caps);
        let v = serde_json::to_value(&r).expect("caps response");
        let back: PrimalResponse = serde_json::from_value(v).expect("caps back");
        assert!(matches!(back, PrimalResponse::Capabilities(_)));

        let keys = PrimalResponse::KeysGenerated(GeneratedKeys {
            public_key: vec![1],
            private_key_handle: "h".into(),
        });
        let v2 = serde_json::to_value(&keys).expect("keys");
        let back2: PrimalResponse = serde_json::from_value(v2).expect("keys back");
        assert!(matches!(back2, PrimalResponse::KeysGenerated(_)));

        let lineage = PrimalResponse::LineageSigned(Lineage {
            data: vec![2, 3],
        });
        let v3 = serde_json::to_value(&lineage).expect("lineage");
        let back3: PrimalResponse = serde_json::from_value(v3).expect("lineage back");
        assert!(matches!(back3, PrimalResponse::LineageSigned(_)));

        let dep = PrimalResponse::WorkloadDeployed(DeploymentId("d1".into()));
        let v4 = serde_json::to_value(&dep).expect("deploy");
        let back4: PrimalResponse = serde_json::from_value(v4).expect("deploy back");
        assert!(matches!(back4, PrimalResponse::WorkloadDeployed(_)));

        let st = PrimalResponse::StatusResponse(ServiceStatus {
            healthy: true,
            version: "1".into(),
            capabilities: vec![],
            metrics: HashMap::new(),
        });
        let v5 = serde_json::to_value(&st).expect("status resp");
        let back5: PrimalResponse = serde_json::from_value(v5).expect("status resp back");
        assert!(matches!(back5, PrimalResponse::StatusResponse(_)));

        let custom = PrimalResponse::Custom(serde_json::json!([1]));
        let v6 = serde_json::to_value(&custom).expect("custom resp");
        let back6: PrimalResponse = serde_json::from_value(v6).expect("custom resp back");
        assert!(matches!(back6, PrimalResponse::Custom(_)));
    }

    #[test]
    fn identity_roundtrip() {
        let id = Identity {
            node_id: NodeId("n".into()),
            public_key: vec![9],
            lineage: Lineage {
                data: vec![1],
            },
            witness_proof: WitnessProof {
                data: vec![2],
            },
        };
        let json = serde_json::to_string(&id).expect("identity json");
        let back: Identity = serde_json::from_str(&json).expect("identity back");
        assert_eq!(back.node_id.0, "n");
        assert_eq!(back.public_key, vec![9]);
    }

    #[test]
    fn deployment_id_and_node_id_equality() {
        assert_eq!(NodeId("a".into()), NodeId("a".into()));
        assert_ne!(NodeId("a".into()), NodeId("b".into()));
        assert_eq!(DeploymentId("x".into()), DeploymentId("x".into()));
    }

    #[test]
    fn primal_capabilities_supports_workload_positive_when_service_matches() {
        let caps = PrimalCapabilities {
            services: vec!["batch".into(), "streaming".into()],
            resources: HashMap::new(),
            metadata: HashMap::new(),
            quality: ServiceQuality::default(),
        };
        let w = Workload {
            id: "1".into(),
            service_type: "streaming".into(),
            requirements: HashMap::new(),
            payload: serde_json::json!({}),
        };
        assert!(caps.supports_workload(&w));
    }

    #[test]
    fn primal_capabilities_supports_capability_rejects_substring_match() {
        let caps = PrimalCapabilities {
            services: vec!["security-v2".into()],
            resources: HashMap::new(),
            metadata: HashMap::new(),
            quality: ServiceQuality::default(),
        };
        assert!(
            !caps.supports_capability(&CapabilityType::Security),
            "services list uses exact string match, not prefix/substring"
        );
    }

    #[test]
    fn capability_type_from_str_unknown_becomes_custom_preserved_case() {
        let c = CapabilityType::from_str("MyCustomCap");
        assert_eq!(c, CapabilityType::Custom("MyCustomCap".into()));
        assert_eq!(c.as_str(), "MyCustomCap");
    }

    #[test]
    fn service_status_roundtrip() {
        let s = ServiceStatus {
            healthy: false,
            version: "0.0.0".into(),
            capabilities: vec!["a".into()],
            metrics: HashMap::from([("k".into(), serde_json::json!(1))]),
        };
        let json = serde_json::to_string(&s).expect("serialize");
        let back: ServiceStatus = serde_json::from_str(&json).expect("deserialize");
        assert!(!back.healthy);
        assert_eq!(back.version, "0.0.0");
        assert_eq!(back.capabilities, vec!["a"]);
    }

    #[test]
    fn node_id_display_matches_inner_string() {
        let n = NodeId("nid-1".into());
        assert_eq!(format!("{n}"), "nid-1");
    }

    #[test]
    fn deployment_id_display_matches_inner_string() {
        let d = DeploymentId("dep-9".into());
        assert_eq!(format!("{d}"), "dep-9");
    }
}
