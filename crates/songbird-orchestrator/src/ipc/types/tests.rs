// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use std::time::SystemTime;

use super::*;

// P2P Discovery Tests (v3.19.3)
#[test]
fn test_discover_request_deserialization() {
    let json = r#"{
            "family_tags": ["nat0", "lan0"],
            "timeout_ms": 3000
        }"#;

    let req: DiscoverByFamilyRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.family_tags, vec!["nat0", "lan0"]);
    assert_eq!(req.timeout_ms, 3000);
}

#[test]
fn test_discover_request_default_timeout() {
    let json = r#"{"family_tags": ["nat0"]}"#;

    let req: DiscoverByFamilyRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.timeout_ms, 5000); // Default
}

#[test]
fn test_genetic_proof_serialization() {
    let proof = GeneticProof {
        family_id: "nat0".to_string(),
        parent_seed_hash: "abc123".to_string(),
        relationship: "sibling".to_string(),
    };

    let json = serde_json::to_string(&proof).unwrap();
    assert!(json.contains("nat0"));
    assert!(json.contains("abc123"));
    assert!(json.contains("sibling"));
}

// Service Registry Tests (v3.20.0)
#[test]
fn test_register_service_request_deserialization() {
    let json = r#"{
            "primal_name": "security provider",
            "capabilities": ["encryption", "identity"],
            "endpoint": "/run/user/1000/security-provider-nat0.sock",
            "protocol": "json-rpc",
            "health_check_interval": 60
        }"#;

    let req: RegisterServiceRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.primal_name, "security provider");
    assert_eq!(req.capabilities, vec!["encryption", "identity"]);
    assert_eq!(req.endpoint, "/run/user/1000/security-provider-nat0.sock");
    assert_eq!(req.protocol, "json-rpc");
    assert_eq!(req.health_check_interval, 60);
}

#[test]
fn test_register_service_request_default_health_interval() {
    let json = r#"{
            "primal_name": "compute-provider",
            "capabilities": ["compute"],
            "endpoint": "/tmp/biomeos/compute.sock",
            "protocol": "json-rpc"
        }"#;

    let req: RegisterServiceRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.health_check_interval, 30); // Default
}

#[test]
fn test_discover_by_capability_request_deserialization() {
    let json = r#"{
            "capability": "encryption",
            "protocol": "json-rpc"
        }"#;

    let req: DiscoverByCapabilityRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.capability, "encryption");
    assert_eq!(req.protocol, Some("json-rpc".to_string()));
}

#[test]
fn test_discover_by_capability_wildcard() {
    let json = r#"{"capability": "*"}"#;

    let req: DiscoverByCapabilityRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.capability, "*");
    assert!(req.protocol.is_none());
}

#[test]
fn test_primal_endpoint_serialization() {
    let endpoint = PrimalEndpoint {
        service_id: "security-provider-12345".to_string(),
        primal_name: "security provider".to_string(),
        capabilities: vec!["encryption".to_string()],
        endpoint: "/run/user/1000/security-provider-nat0.sock".to_string(),
        protocol: "json-rpc".to_string(),
        last_health_check: "2026-01-10T12:00:00Z".to_string(),
        health_status: "healthy".to_string(),
    };

    let json = serde_json::to_string(&endpoint).unwrap();
    assert!(json.contains("security-provider-12345"));
    assert!(json.contains("security provider"));
    assert!(json.contains("encryption"));
    assert!(json.contains("healthy"));
}

#[test]
fn test_health_status_serialization() {
    let health = HealthStatus {
        service_id: "songbird".to_string(),
        status: "healthy".to_string(),
        message: None,
        timestamp: "2026-01-10T12:00:00Z".to_string(),
    };

    let json = serde_json::to_string(&health).unwrap();
    assert!(json.contains("songbird"));
    assert!(json.contains("healthy"));
    // Message should not be in JSON when None
    assert!(!json.contains("message"));
}

#[test]
fn system_time_epoch_iso8601() {
    let s = system_time_to_iso8601(SystemTime::UNIX_EPOCH);
    assert!(s.starts_with("1970-"));
}

#[test]
fn health_check_request_response_roundtrip() {
    let req = HealthCheckRequest {};
    let j = serde_json::to_string(&req).unwrap();
    let _: HealthCheckRequest = serde_json::from_str(&j).unwrap();
    let resp = HealthCheckResponse {
        health: HealthStatus {
            service_id: "s".to_string(),
            status: "ok".to_string(),
            message: None,
            timestamp: "t".to_string(),
        },
    };
    let j2 = serde_json::to_string(&resp).unwrap();
    let back: HealthCheckResponse = serde_json::from_str(&j2).unwrap();
    assert_eq!(back.health.service_id, "s");
}

#[test]
fn discover_by_family_request_serde_roundtrip() {
    let r = DiscoverByFamilyRequest {
        family_tags: vec!["a".to_string()],
        timeout_ms: 1234,
    };
    let j = serde_json::to_string(&r).unwrap();
    let back: DiscoverByFamilyRequest = serde_json::from_str(&j).unwrap();
    assert_eq!(r.family_tags, back.family_tags);
    assert_eq!(r.timeout_ms, back.timeout_ms);
}

#[test]
fn genetic_proof_roundtrip() {
    let p = GeneticProof {
        family_id: "f".to_string(),
        parent_seed_hash: "h".to_string(),
        relationship: "r".to_string(),
    };
    let j = serde_json::to_string(&p).unwrap();
    let back: GeneticProof = serde_json::from_str(&j).unwrap();
    assert_eq!(p.family_id, back.family_id);
    assert_eq!(p.relationship, back.relationship);
}

#[test]
fn health_status_with_message_json() {
    let h = HealthStatus {
        service_id: "x".to_string(),
        status: "degraded".to_string(),
        message: Some("m".to_string()),
        timestamp: "t".to_string(),
    };
    let j = serde_json::to_string(&h).unwrap();
    assert!(j.contains("message"));
    let back: HealthStatus = serde_json::from_str(&j).unwrap();
    assert_eq!(h.message, back.message);
    assert_eq!(h.service_id, back.service_id);
}

#[test]
fn discover_by_family_response_roundtrip() {
    let r = DiscoverByFamilyResponse {
        nodes: vec![DiscoveredNode {
            node_id: "n1".to_string(),
            node_name: Some("name".to_string()),
            genetic_families: vec!["f1".to_string()],
            sub_federations: vec![],
            capabilities: vec!["c".to_string()],
            btsp_endpoint: None,
            https_endpoint: "https://h".to_string(),
            last_seen: "2026-01-01T00:00:00Z".to_string(),
        }],
    };
    let j = serde_json::to_string(&r).unwrap();
    let back: DiscoverByFamilyResponse = serde_json::from_str(&j).unwrap();
    assert_eq!(back.nodes.len(), 1);
    assert_eq!(back.nodes[0].node_id, "n1");
}

#[test]
fn create_genetic_tunnel_request_optional_fields_omit() {
    let r = CreateGeneticTunnelRequest {
        peer_node_id: "peer".to_string(),
        peer_endpoint: None,
        genetic_proof: None,
    };
    let j = serde_json::to_string(&r).unwrap();
    assert!(!j.contains("peer_endpoint"));
    let back: CreateGeneticTunnelRequest = serde_json::from_str(&j).unwrap();
    assert_eq!(back.peer_node_id, "peer");
    assert!(back.genetic_proof.is_none());
}

#[test]
fn create_genetic_tunnel_response_roundtrip() {
    let r = CreateGeneticTunnelResponse {
        tunnel_id: "t1".to_string(),
        status: "established".to_string(),
        local_endpoint: Some("127.0.0.1:1".to_string()),
        peer_endpoint: Some("r".to_string()),
        encryption: Some("aes-gcm".to_string()),
        created_at: "2026-01-01T00:00:00Z".to_string(),
    };
    let j = serde_json::to_string(&r).unwrap();
    let back: CreateGeneticTunnelResponse = serde_json::from_str(&j).unwrap();
    assert_eq!(back.tunnel_id, r.tunnel_id);
    assert_eq!(back.status, "established");
}

#[test]
fn announce_capabilities_request_empty_defaults() {
    let j = r#"{"capabilities":["x"]}"#;
    let r: AnnounceCapabilitiesRequest = serde_json::from_str(j).unwrap();
    assert!(r.sub_federations.is_empty());
    assert!(r.genetic_families.is_empty());
}

#[test]
fn announce_capabilities_response_roundtrip() {
    let r = AnnounceCapabilitiesResponse {
        status: "updated".to_string(),
        broadcasting: true,
        updated_at: "t".to_string(),
    };
    let j = serde_json::to_string(&r).unwrap();
    let back: AnnounceCapabilitiesResponse = serde_json::from_str(&j).unwrap();
    assert!(back.broadcasting);
}

#[test]
fn register_service_response_roundtrip() {
    let r = RegisterServiceResponse {
        service_id: "svc-1".to_string(),
        status: "registered".to_string(),
        registered_at: "t".to_string(),
    };
    let j = serde_json::to_string(&r).unwrap();
    let back: RegisterServiceResponse = serde_json::from_str(&j).unwrap();
    assert_eq!(back.service_id, "svc-1");
}

#[test]
fn discover_by_capability_response_roundtrip() {
    let r = DiscoverByCapabilityResponse {
        primals: vec![PrimalEndpoint {
            service_id: "s".to_string(),
            primal_name: "p".to_string(),
            capabilities: vec![],
            endpoint: "/sock".to_string(),
            protocol: "json-rpc".to_string(),
            last_health_check: "t".to_string(),
            health_status: "unknown".to_string(),
        }],
    };
    let j = serde_json::to_string(&r).unwrap();
    let back: DiscoverByCapabilityResponse = serde_json::from_str(&j).unwrap();
    assert_eq!(back.primals.len(), 1);
}

#[test]
fn get_service_health_request_response_roundtrip() {
    let req = GetServiceHealthRequest {
        service_id: "abc".to_string(),
    };
    let j = serde_json::to_string(&req).unwrap();
    let back: GetServiceHealthRequest = serde_json::from_str(&j).unwrap();
    assert_eq!(back.service_id, "abc");

    let resp = GetServiceHealthResponse {
        health: HealthStatus {
            service_id: "abc".to_string(),
            status: "healthy".to_string(),
            message: None,
            timestamp: "ts".to_string(),
        },
    };
    let j2 = serde_json::to_string(&resp).unwrap();
    let back2: GetServiceHealthResponse = serde_json::from_str(&j2).unwrap();
    assert_eq!(back2.health.status, "healthy");
}

#[test]
fn create_genetic_tunnel_request_full_roundtrip() {
    let r = CreateGeneticTunnelRequest {
        peer_node_id: "n-beta".to_string(),
        peer_endpoint: Some("udp://192.168.0.2:4433".to_string()),
        genetic_proof: Some(GeneticProof {
            family_id: "fam-x".to_string(),
            parent_seed_hash: "seed".to_string(),
            relationship: "parent".to_string(),
        }),
    };
    let j = serde_json::to_string(&r).unwrap();
    let back: CreateGeneticTunnelRequest = serde_json::from_str(&j).unwrap();
    assert_eq!(back.peer_node_id, r.peer_node_id);
    assert_eq!(back.peer_endpoint, r.peer_endpoint);
    assert_eq!(back.genetic_proof.as_ref().unwrap().family_id, "fam-x");
}

#[test]
fn discovered_node_serde_roundtrip_optional_and_sub_feds() {
    let n = DiscoveredNode {
        node_id: "nid".to_string(),
        node_name: None,
        genetic_families: vec!["g1".to_string()],
        sub_federations: vec!["sf1".to_string()],
        capabilities: vec!["cap".to_string()],
        btsp_endpoint: Some("btsp://h".to_string()),
        https_endpoint: "https://x".to_string(),
        last_seen: "2026-02-01T00:00:00Z".to_string(),
    };
    let j = serde_json::to_string(&n).unwrap();
    let back: DiscoveredNode = serde_json::from_str(&j).unwrap();
    assert_eq!(back.node_id, n.node_id);
    assert!(back.node_name.is_none());
    assert_eq!(back.sub_federations, vec!["sf1"]);
    assert_eq!(back.btsp_endpoint, n.btsp_endpoint);
}

#[test]
fn announce_capabilities_request_full_roundtrip() {
    let r = AnnounceCapabilitiesRequest {
        capabilities: vec!["a".to_string(), "b".to_string()],
        sub_federations: vec!["sub".to_string()],
        genetic_families: vec!["gf".to_string()],
    };
    let j = serde_json::to_string(&r).unwrap();
    let back: AnnounceCapabilitiesRequest = serde_json::from_str(&j).unwrap();
    assert_eq!(r.capabilities, back.capabilities);
    assert_eq!(r.sub_federations, back.sub_federations);
    assert_eq!(r.genetic_families, back.genetic_families);
}

#[test]
fn discovered_node_empty_sub_federations_omitted_in_json() {
    let n = DiscoveredNode {
        node_id: "n".to_string(),
        node_name: Some("nm".to_string()),
        genetic_families: vec![],
        sub_federations: vec![],
        capabilities: vec![],
        btsp_endpoint: None,
        https_endpoint: "https://h".to_string(),
        last_seen: "t".to_string(),
    };
    let j = serde_json::to_string(&n).unwrap();
    assert!(!j.contains("sub_federations"));
    let back: DiscoveredNode = serde_json::from_str(&j).unwrap();
    assert!(back.sub_federations.is_empty());
}

#[test]
fn capability_resolve_request_roundtrip() {
    let req = CapabilityResolveRequest {
        capability: "crypto".to_string(),
    };
    let j = serde_json::to_string(&req).unwrap();
    let back: CapabilityResolveRequest = serde_json::from_str(&j).unwrap();
    assert_eq!(back.capability, "crypto");
}

#[test]
fn capability_resolve_response_roundtrip() {
    let resp = CapabilityResolveResponse {
        service_id: "svc-sec-001".to_string(),
        primal_name: "bearDog".to_string(),
        endpoint: "/run/user/1000/biomeos/security.sock".to_string(),
        protocol: "json-rpc".to_string(),
        socket: Some("/run/user/1000/biomeos/security.sock".to_string()),
        native_endpoint: "unix:///run/user/1000/biomeos/security.sock".to_string(),
        virtual_endpoint: "capability://crypto@bearDog".to_string(),
        capabilities: vec!["crypto".to_string(), "identity".to_string()],
    };
    let j = serde_json::to_string(&resp).unwrap();
    let back: CapabilityResolveResponse = serde_json::from_str(&j).unwrap();
    assert_eq!(back.service_id, "svc-sec-001");
    assert_eq!(back.capabilities.len(), 2);
    assert_eq!(back.native_endpoint, "unix:///run/user/1000/biomeos/security.sock");
    assert_eq!(back.virtual_endpoint, "capability://crypto@bearDog");
}
