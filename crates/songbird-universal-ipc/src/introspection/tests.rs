// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::expect_used, reason = "test assertions")]

use super::{
    SONGBIRD_CAPABILITY_STRINGS, canonical_family_id, capabilities_list, discover_capabilities,
    health, health_check, health_liveness, health_readiness, identity, normalize_method,
    primal_capabilities, primal_info, rpc_discover_standard, rpc_methods,
};

use std::collections::HashMap;
use std::env::VarError;

#[test]
fn primal_info_has_expected_keys() {
    let v = primal_info();
    assert_eq!(v["name"], "songbird");
    assert_eq!(v["role"], "network_orchestrator");
    assert!(v.get("capabilities").is_some());
    assert!(v.get("version").is_some());
}

#[test]
fn primal_capabilities_is_array_of_objects() {
    let v = primal_capabilities();
    let caps = v["capabilities"].as_array().unwrap();
    assert!(!caps.is_empty());
    assert!(caps[0].get("name").is_some());
}

#[test]
fn health_includes_uptime_and_services() {
    let v = health(42, 7);
    assert_eq!(v["uptime_seconds"], 42);
    assert_eq!(v["services"], 7);
    assert_eq!(v["status"], "healthy");
}

#[test]
fn health_liveness_is_minimal() {
    let v = health_liveness();
    assert_eq!(v, serde_json::json!({ "status": "healthy" }));
    assert!(v.get("uptime_seconds").is_none());
}

#[test]
fn capabilities_list_matches_const_table() {
    let v = capabilities_list();
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), SONGBIRD_CAPABILITY_STRINGS.len());
    for (i, s) in SONGBIRD_CAPABILITY_STRINGS.iter().enumerate() {
        assert_eq!(arr[i].as_str().unwrap(), *s);
    }
}

#[test]
fn identity_includes_family_id() {
    let v = identity("fam-test");
    assert_eq!(v["family_id"], "fam-test");
    let caps = v["capabilities"].as_array().unwrap();
    assert!(caps.iter().any(|c| c == "ipc.register"));
}

#[test]
fn rpc_methods_non_empty() {
    let v = rpc_methods();
    let methods = v["methods"].as_array().unwrap();
    assert!(!methods.is_empty());
}

#[test]
fn discover_capabilities_lists_http_and_ipc() {
    let v = discover_capabilities();
    assert_eq!(v["primal"], "songbird");
    let caps = v["capabilities"].as_array().unwrap();
    assert!(caps.iter().any(|c| c == "http.request"));
}

#[test]
fn canonical_family_id_prefers_orchestrator() {
    let m = HashMap::from([
        ("SONGBIRD_ORCHESTRATOR_FAMILY_ID", "orch"),
        ("BIOMEOS_FAMILY_ID", "biome"),
    ]);
    assert_eq!(
        canonical_family_id(|k| m.get(k).copied().map(String::from).ok_or(VarError::NotPresent)),
        "orch"
    );
}

#[test]
fn canonical_family_id_falls_back_to_biomeos() {
    let m = HashMap::from([("BIOMEOS_FAMILY_ID", "biome-only")]);
    assert_eq!(
        canonical_family_id(|k| m.get(k).copied().map(String::from).ok_or(VarError::NotPresent)),
        "biome-only"
    );
}

#[test]
fn canonical_family_id_falls_back_to_songbird_family_id() {
    let m = HashMap::from([("SONGBIRD_FAMILY_ID", "sb")]);
    assert_eq!(
        canonical_family_id(|k| m.get(k).copied().map(String::from).ok_or(VarError::NotPresent)),
        "sb"
    );
}

#[test]
fn canonical_family_id_falls_back_to_family_id() {
    let m = HashMap::from([("FAMILY_ID", "fam")]);
    assert_eq!(
        canonical_family_id(|k| m.get(k).copied().map(String::from).ok_or(VarError::NotPresent)),
        "fam"
    );
}

#[test]
fn canonical_family_id_falls_back_to_node_family_id() {
    let m = HashMap::from([("NODE_FAMILY_ID", "node")]);
    assert_eq!(
        canonical_family_id(|k| m.get(k).copied().map(String::from).ok_or(VarError::NotPresent)),
        "node"
    );
}

#[test]
fn canonical_family_id_default_when_missing() {
    let m: HashMap<&str, &str> = HashMap::new();
    assert_eq!(
        canonical_family_id(|k| m.get(k).copied().map(String::from).ok_or(VarError::NotPresent)),
        "default"
    );
}

#[test]
fn rpc_discover_standard_includes_core_methods() {
    let v = rpc_discover_standard();
    let methods = v["methods"].as_array().unwrap();
    let names: Vec<&str> = methods.iter().filter_map(|x| x.as_str()).collect();
    assert!(names.contains(&"health.liveness"));
    assert!(names.contains(&"health.readiness"));
    assert!(names.contains(&"health.check"));
    assert!(names.contains(&"identity"));
    assert!(names.contains(&"peer.connect"));
    assert!(names.contains(&"tor.circuit.build"));
}

#[test]
fn rpc_methods_includes_igd_and_tor_entries() {
    let v = rpc_methods();
    let methods = v["methods"].as_array().unwrap();
    let has_igd = methods.iter().any(|m| m["name"] == "igd.discover");
    let has_tor = methods.iter().any(|m| m["name"] == "tor.status");
    assert!(has_igd);
    assert!(has_tor);
}

#[test]
fn primal_info_lists_discovery_methods() {
    let v = primal_info();
    let dm = v["discovery_methods"].as_array().unwrap();
    assert!(dm.iter().any(|x| x == "mdns"));
}

#[test]
fn identity_lists_ipc_methods_in_capabilities() {
    let v = identity("fam");
    let caps = v["capabilities"].as_array().unwrap();
    assert!(caps.iter().any(|c| c == "ipc.register"));
}

#[test]
fn health_zero_uptime_and_zero_services() {
    let v = health(0, 0);
    assert_eq!(v["uptime_seconds"], 0);
    assert_eq!(v["services"], 0);
    assert_eq!(v["primal"], "songbird");
}

#[test]
fn rpc_methods_has_jsonrpc_and_non_empty_methods() {
    let v = rpc_methods();
    assert_eq!(v["jsonrpc"], "2.0");
    let m = v["methods"].as_array().unwrap();
    assert!(m.len() > 5);
    assert!(m.iter().any(|x| x["name"] == "primal.info"));
}

#[test]
fn discover_capabilities_includes_tor_and_mesh() {
    let v = discover_capabilities();
    let caps = v["capabilities"].as_array().unwrap();
    let s: Vec<&str> = caps.iter().filter_map(|x| x.as_str()).collect();
    assert!(s.contains(&"tor.connect"));
    assert!(s.contains(&"mesh.status"));
}

#[test]
fn songbird_capability_strings_count_matches_network_ipc_crypto() {
    assert!(SONGBIRD_CAPABILITY_STRINGS.contains(&"ipc.jsonrpc"));
    assert!(SONGBIRD_CAPABILITY_STRINGS.contains(&"network.tls"));
    assert_eq!(SONGBIRD_CAPABILITY_STRINGS.len(), 14);
}

#[test]
fn rpc_discover_standard_contains_capabilities_listing() {
    let v = rpc_discover_standard();
    let methods = v["methods"].as_array().unwrap();
    assert!(methods.iter().any(|m| m == "primal.capabilities"));
    assert!(methods.iter().any(|m| m == "health.liveness"));
    assert!(methods.iter().any(|m| m == "health.readiness"));
    assert!(methods.iter().any(|m| m == "health.check"));
    assert!(methods.iter().any(|m| m == "capabilities.list"));
}

#[test]
fn health_readiness_reports_ready_status() {
    let v = health_readiness();
    assert_eq!(v["status"], "ready");
    assert!(v.get("subsystems").is_some());
    assert_eq!(v["subsystems"]["ipc"], "up");
}

#[test]
fn health_check_includes_primal_and_version() {
    let v = health_check();
    assert_eq!(v["status"], "healthy");
    assert_eq!(v["primal"], "songbird");
    assert!(v.get("version").is_some());
    assert!(v.get("subsystems").is_some());
    assert!(v["uptime_seconds"].is_null());
}

#[test]
fn normalize_method_canonicalizes_capability_list_aliases() {
    assert_eq!(normalize_method("capabilities.list"), "capabilities.list");
    assert_eq!(normalize_method("capability.list"), "capabilities.list");
    assert_eq!(normalize_method("primal.capabilities"), "primal.capabilities");
}

#[test]
fn normalize_method_canonicalizes_health_aliases() {
    assert_eq!(normalize_method("health.liveness"), "health.liveness");
    assert_eq!(normalize_method("ping"), "health.liveness");
    assert_eq!(normalize_method("health"), "health.check");
    assert_eq!(normalize_method("status"), "health.check");
    assert_eq!(normalize_method("check"), "health.check");
}

#[test]
fn normalize_method_passes_through_unknown() {
    assert_eq!(normalize_method("compute.route"), "compute.route");
    assert_eq!(normalize_method("ipc.register"), "ipc.register");
}

#[test]
fn primal_capabilities_includes_mesh_and_onion() {
    let v = primal_capabilities();
    let caps = v["capabilities"].as_array().unwrap();
    let names: Vec<&str> = caps.iter().filter_map(|c| c["name"].as_str()).collect();
    assert!(names.contains(&"mesh"));
    assert!(names.contains(&"onion"));
}
