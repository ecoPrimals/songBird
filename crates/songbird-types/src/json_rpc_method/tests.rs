// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::*;

#[test]
fn parse_ipc_normalizes_aliases() {
    assert_eq!(
        JsonRpcMethod::parse_ipc("ping").unwrap(),
        JsonRpcMethod::Health(HealthMethod::Liveness)
    );
    assert_eq!(
        JsonRpcMethod::parse_ipc("health").unwrap(),
        JsonRpcMethod::Health(HealthMethod::Check)
    );
}

#[test]
fn roundtrip_display_from_str() {
    let m = JsonRpcMethod::Discovery(DiscoveryMethod::Peers);
    assert_eq!(m.to_string(), "discovery.peers");
    assert_eq!(JsonRpcMethod::from_str("discovery.peers").unwrap(), m);
    assert_eq!(JsonRpcMethod::from_str("find_primals").unwrap(), m);
}

#[test]
fn serde_json_roundtrip() {
    let m = JsonRpcMethod::Federation(FederationMethod::Peers);
    let v = serde_json::to_string(&m).unwrap();
    assert_eq!(v, "\"songbird.federation.peers\"");
    let back: JsonRpcMethod = serde_json::from_str(&v).unwrap();
    assert_eq!(back, m);
}

#[test]
fn biome_os_health_distinct_from_check() {
    assert_eq!(JsonRpcMethod::from_wire_str("health").unwrap(), JsonRpcMethod::BiomeOsHealth);
    assert_eq!(
        JsonRpcMethod::from_wire_str("health.check").unwrap(),
        JsonRpcMethod::Health(HealthMethod::Check)
    );
}

#[test]
fn ipc_find_capability_roundtrip_wire_and_serde() {
    let wire = "ipc.find_capability";
    let m = JsonRpcMethod::from_wire_str(wire).unwrap();
    assert_eq!(m, JsonRpcMethod::Ipc(IpcMethod::FindCapability));
    assert_eq!(m.as_wire_str(), wire);
    assert_eq!(m.to_string(), wire);
    let v = serde_json::to_string(&m).unwrap();
    assert_eq!(v, "\"ipc.find_capability\"");
    let back: JsonRpcMethod = serde_json::from_str(&v).unwrap();
    assert_eq!(back, m);
}

#[test]
fn storage_methods_roundtrip() {
    for (wire, expected) in [
        ("storage.get", JsonRpcMethod::Storage(StorageMethod::Get)),
        ("storage.put", JsonRpcMethod::Storage(StorageMethod::Put)),
        ("storage.delete", JsonRpcMethod::Storage(StorageMethod::Delete)),
        ("storage.list", JsonRpcMethod::Storage(StorageMethod::List)),
        ("storage.flush", JsonRpcMethod::Storage(StorageMethod::Flush)),
    ] {
        let m = JsonRpcMethod::from_wire_str(wire).unwrap();
        assert_eq!(m, expected);
        assert_eq!(m.as_wire_str(), wire);
        let json = serde_json::to_string(&m).unwrap();
        let back: JsonRpcMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(back, m);
    }
}

#[test]
fn capability_resolve_roundtrip() {
    let m = JsonRpcMethod::from_wire_str("capability.resolve").unwrap();
    assert_eq!(m, JsonRpcMethod::Capabilities(CapabilitiesMethod::Resolve));
    assert_eq!(m.as_wire_str(), "capability.resolve");
    let json = serde_json::to_string(&m).unwrap();
    let back: JsonRpcMethod = serde_json::from_str(&json).unwrap();
    assert_eq!(back, m);
}

#[test]
fn lifecycle_composition_roundtrip() {
    let m = JsonRpcMethod::from_wire_str("lifecycle.composition").unwrap();
    assert_eq!(m, JsonRpcMethod::Lifecycle(LifecycleMethod::Composition));
    assert_eq!(m.as_wire_str(), "lifecycle.composition");
}

#[test]
fn lifecycle_validate_consumed_roundtrip() {
    let m = JsonRpcMethod::from_wire_str("lifecycle.validate_consumed").unwrap();
    assert_eq!(m, JsonRpcMethod::Lifecycle(LifecycleMethod::ValidateConsumed));
    assert_eq!(m.as_wire_str(), "lifecycle.validate_consumed");
}

#[test]
fn inference_namespace_canonical_and_aliases() {
    let m = JsonRpcMethod::from_wire_str("inference.infer").unwrap();
    assert_eq!(m, JsonRpcMethod::Inference(InferenceMethod::Infer));

    assert_eq!(
        JsonRpcMethod::parse_ipc("model.infer").unwrap(),
        JsonRpcMethod::Inference(InferenceMethod::Infer)
    );
    assert_eq!(
        JsonRpcMethod::parse_ipc("ai.infer").unwrap(),
        JsonRpcMethod::Inference(InferenceMethod::Infer)
    );
    assert_eq!(
        JsonRpcMethod::parse_ipc("ai.inference").unwrap(),
        JsonRpcMethod::Inference(InferenceMethod::Infer)
    );
    assert_eq!(
        JsonRpcMethod::parse_ipc("model.status").unwrap(),
        JsonRpcMethod::Inference(InferenceMethod::Status)
    );
    assert_eq!(
        JsonRpcMethod::parse_ipc("ai.list").unwrap(),
        JsonRpcMethod::Inference(InferenceMethod::List)
    );
}

#[test]
fn discovery_find_by_capability_normalizes_to_ipc_discover() {
    assert_eq!(
        JsonRpcMethod::parse_ipc("discovery.find_by_capability").unwrap(),
        JsonRpcMethod::Ipc(IpcMethod::Discover)
    );
    assert_eq!(
        JsonRpcMethod::parse_ipc("net.discovery.find_by_capability").unwrap(),
        JsonRpcMethod::Ipc(IpcMethod::Discover)
    );
    assert_eq!(
        JsonRpcMethod::parse_ipc("capability.discover").unwrap(),
        JsonRpcMethod::Ipc(IpcMethod::Discover)
    );
}

#[test]
fn ipc_heartbeat_roundtrip_wire_and_serde() {
    let wire = "ipc.heartbeat";
    let m = JsonRpcMethod::from_wire_str(wire).unwrap();
    assert_eq!(m, JsonRpcMethod::Ipc(IpcMethod::Heartbeat));
    assert_eq!(m.as_wire_str(), wire);
    assert_eq!(m.to_string(), wire);
    let v = serde_json::to_string(&m).unwrap();
    assert_eq!(v, "\"ipc.heartbeat\"");
    let back: JsonRpcMethod = serde_json::from_str(&v).unwrap();
    assert_eq!(back, m);
}

#[test]
fn tower_methods_roundtrip() {
    let cases = [
        ("tower.health", JsonRpcMethod::Tower(TowerMethod::Health)),
        ("tower.mesh_status", JsonRpcMethod::Tower(TowerMethod::MeshStatus)),
    ];
    for (wire, expected) in cases {
        let parsed = JsonRpcMethod::from_wire_str(wire).unwrap();
        assert_eq!(parsed, expected);
        assert_eq!(parsed.as_wire_str(), wire);
        let json = serde_json::to_string(&parsed).unwrap();
        let back: JsonRpcMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(back, expected);
    }
}

#[test]
fn health_ping_roundtrip() {
    let m = JsonRpcMethod::from_wire_str("health.ping").unwrap();
    assert_eq!(m, JsonRpcMethod::Health(HealthMethod::Ping));
    assert_eq!(m.as_wire_str(), "health.ping");
}

#[test]
fn tower_enroll_normalizes_to_mesh_enroll() {
    let m = JsonRpcMethod::parse_ipc("tower.enroll").unwrap();
    assert_eq!(m, JsonRpcMethod::Mesh(MeshMethod::Enroll));
}

#[test]
fn acme_methods_roundtrip() {
    let cases = [
        ("acme.challenge_ready", JsonRpcMethod::Acme(AcmeMethod::ChallengeReady)),
        ("acme.challenge_cleanup", JsonRpcMethod::Acme(AcmeMethod::ChallengeCleanup)),
    ];
    for (wire, expected) in cases {
        let parsed = JsonRpcMethod::from_wire_str(wire).unwrap();
        assert_eq!(parsed, expected);
        assert_eq!(parsed.as_wire_str(), wire);
        let json = serde_json::to_string(&parsed).unwrap();
        let back: JsonRpcMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(back, expected);
    }
}
