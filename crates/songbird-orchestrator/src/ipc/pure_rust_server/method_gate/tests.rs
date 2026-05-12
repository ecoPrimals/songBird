// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::*;

#[test]
fn health_methods_are_public() {
    assert_eq!(classify_method("health.check"), MethodAccessLevel::Public);
    assert_eq!(classify_method("health.liveness"), MethodAccessLevel::Public);
    assert_eq!(classify_method("health.readiness"), MethodAccessLevel::Public);
}

#[test]
fn identity_get_is_public() {
    assert_eq!(classify_method("identity.get"), MethodAccessLevel::Public);
}

#[test]
fn capabilities_list_is_public() {
    assert_eq!(classify_method("capabilities.list"), MethodAccessLevel::Public);
    assert_eq!(classify_method("capability.list"), MethodAccessLevel::Public);
}

#[test]
fn auth_introspection_is_public() {
    assert_eq!(classify_method("auth.check"), MethodAccessLevel::Public);
    assert_eq!(classify_method("auth.mode"), MethodAccessLevel::Public);
    assert_eq!(classify_method("auth.peer_info"), MethodAccessLevel::Public);
}

#[test]
fn lifecycle_status_is_public() {
    assert_eq!(classify_method("lifecycle.status"), MethodAccessLevel::Public);
}

#[test]
fn discovery_methods_are_protected() {
    assert_eq!(classify_method("discovery.peers"), MethodAccessLevel::Protected);
    assert_eq!(classify_method("discovery.content_peers"), MethodAccessLevel::Protected);
}

#[test]
fn birdsong_methods_are_protected() {
    assert_eq!(classify_method("birdsong.announce"), MethodAccessLevel::Protected);
}

#[test]
fn mesh_methods_are_protected() {
    assert_eq!(classify_method("mesh.connect"), MethodAccessLevel::Protected);
}

#[test]
fn ipc_methods_are_protected() {
    assert_eq!(classify_method("ipc.register"), MethodAccessLevel::Protected);
    assert_eq!(classify_method("ipc.resolve"), MethodAccessLevel::Protected);
}

#[test]
fn empty_method_is_protected() {
    assert_eq!(classify_method(""), MethodAccessLevel::Protected);
}

#[test]
fn loopback_context() {
    let ctx = CallerContext::loopback();
    assert!(ctx.peer.is_none());
    assert!(ctx.bearer_token.is_none());
    assert_eq!(ctx.origin, ConnectionOrigin::Loopback);
}

#[test]
fn unix_context() {
    let ctx = CallerContext::from_unix();
    assert_eq!(ctx.origin, ConnectionOrigin::Unix);
}

#[test]
fn remote_context() {
    let ctx = CallerContext::remote();
    assert_eq!(ctx.origin, ConnectionOrigin::Remote);
}

#[test]
fn from_tcp_loopback_ipv4() {
    let addr: std::net::SocketAddr = "127.0.0.1:12345".parse().unwrap();
    let ctx = CallerContext::from_tcp(addr);
    assert_eq!(ctx.origin, ConnectionOrigin::Loopback);
}

#[test]
fn from_tcp_loopback_ipv6() {
    let addr: std::net::SocketAddr = "[::1]:9000".parse().unwrap();
    let ctx = CallerContext::from_tcp(addr);
    assert_eq!(ctx.origin, ConnectionOrigin::Loopback);
}

#[test]
fn from_tcp_remote() {
    let addr: std::net::SocketAddr = "192.168.1.50:8080".parse().unwrap();
    let ctx = CallerContext::from_tcp(addr);
    assert_eq!(ctx.origin, ConnectionOrigin::Remote);
}

#[test]
fn enforcement_mode_as_str() {
    assert_eq!(EnforcementMode::Permissive.as_str(), "permissive");
    assert_eq!(EnforcementMode::Enforced.as_str(), "enforced");
}

#[test]
fn public_method_always_passes() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let caller = CallerContext::loopback();
    assert!(gate.check("health.check", &caller).is_ok());
    assert!(gate.check("identity.get", &caller).is_ok());
    assert!(gate.check("capabilities.list", &caller).is_ok());
    assert!(gate.check("auth.check", &caller).is_ok());
}

#[test]
fn protected_method_passes_in_permissive_mode() {
    let gate = MethodGate::new(EnforcementMode::Permissive);
    let caller = CallerContext::loopback();
    assert!(gate.check("discovery.peers", &caller).is_ok());
}

#[test]
fn protected_method_rejected_in_enforced_mode_without_token() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let caller = CallerContext::loopback();
    let result = gate.check("discovery.peers", &caller);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, error_codes::PERMISSION_DENIED);
    assert!(err.message.contains("discovery.peers"));
}

#[test]
fn protected_method_passes_in_enforced_mode_with_token() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let caller = CallerContext {
        bearer_token: Some("valid-token".to_owned()),
        verified_claims: None,
        peer: None,
        origin: ConnectionOrigin::Unix,
    };
    assert!(gate.check("discovery.peers", &caller).is_ok());
}

#[test]
fn gate_error_includes_method_in_data() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let caller = CallerContext::loopback();
    let err = gate.check("mesh.connect", &caller).unwrap_err();
    let method_in_data =
        err.data.as_ref().and_then(|d| d.get("method")).and_then(serde_json::Value::as_str);
    assert_eq!(method_in_data, Some("mesh.connect"));
}

#[test]
fn auth_check_unauthenticated() {
    let gate = MethodGate::new(EnforcementMode::Permissive);
    let caller = CallerContext::loopback();
    let result = handle_auth_check(&caller, &gate);
    assert_eq!(result["authenticated"], false);
    assert_eq!(result["verified"], false);
    assert_eq!(result["enforcement"], "permissive");
    assert_eq!(result["origin"], "loopback");
}

#[test]
fn auth_check_authenticated() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let caller = CallerContext {
        bearer_token: Some("tok".to_owned()),
        verified_claims: None,
        peer: None,
        origin: ConnectionOrigin::Unix,
    };
    let result = handle_auth_check(&caller, &gate);
    assert_eq!(result["authenticated"], true);
    assert_eq!(result["verified"], false);
    assert_eq!(result["enforcement"], "enforced");
}

#[test]
fn auth_check_verified_with_claims() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let caller = CallerContext {
        bearer_token: Some("verified-tok".to_owned()),
        verified_claims: Some(TokenClaims {
            subject: "songbird-test".to_owned(),
            scopes: vec!["discovery.*".to_owned(), "mesh.connect".to_owned()],
            expires_at: None,
        }),
        peer: None,
        origin: ConnectionOrigin::Unix,
    };
    let result = handle_auth_check(&caller, &gate);
    assert_eq!(result["authenticated"], true);
    assert_eq!(result["verified"], true);
    assert_eq!(result["subject"], "songbird-test");
    assert_eq!(result["scopes"][0], "discovery.*");
    assert_eq!(result["scopes"][1], "mesh.connect");
}

#[test]
fn auth_mode_response() {
    let gate = MethodGate::new(EnforcementMode::Permissive);
    let result = handle_auth_mode(&gate);
    assert_eq!(result["mode"], "permissive");
    assert_eq!(result["env_var"], "SONGBIRD_AUTH_MODE");
}

#[test]
fn auth_peer_info_no_creds() {
    let caller = CallerContext::loopback();
    let result = handle_auth_peer_info(&caller);
    assert_eq!(result["available"], false);
}

#[test]
fn auth_peer_info_with_creds() {
    let caller = CallerContext {
        bearer_token: None,
        verified_claims: None,
        peer: Some(PeerCredentials {
            pid: Some(1234),
            uid: 1000,
        }),
        origin: ConnectionOrigin::Unix,
    };
    let result = handle_auth_peer_info(&caller);
    assert_eq!(result["available"], true);
    assert_eq!(result["uid"], 1000);
    assert_eq!(result["pid"], 1234);
}

#[test]
fn dispatch_routes_auth_methods() {
    let gate = MethodGate::new(EnforcementMode::Permissive);
    let caller = CallerContext::loopback();
    assert!(dispatch_auth_method("auth.check", &gate, &caller).is_some());
    assert!(dispatch_auth_method("auth.mode", &gate, &caller).is_some());
    assert!(dispatch_auth_method("auth.peer_info", &gate, &caller).is_some());
}

#[test]
fn dispatch_returns_none_for_non_auth() {
    let gate = MethodGate::new(EnforcementMode::Permissive);
    let caller = CallerContext::loopback();
    assert!(dispatch_auth_method("discovery.peers", &gate, &caller).is_none());
}

#[test]
fn is_gate_handled_method_correct() {
    assert!(is_gate_handled_method("auth.check"));
    assert!(is_gate_handled_method("auth.mode"));
    assert!(is_gate_handled_method("auth.peer_info"));
    assert!(!is_gate_handled_method("auth.issue_ionic"));
    assert!(!is_gate_handled_method("discovery.peers"));
}

#[test]
fn auth_mode_accessible_via_tcp_context() {
    let gate = MethodGate::new(EnforcementMode::Permissive);
    let tcp_addr: std::net::SocketAddr = "192.168.1.100:5000".parse().unwrap();
    let caller = CallerContext::from_tcp(tcp_addr);
    assert_eq!(caller.origin, ConnectionOrigin::Remote);
    let result = dispatch_auth_method("auth.mode", &gate, &caller);
    assert!(result.is_some(), "auth.mode must be reachable over TCP");
    let value = result.unwrap();
    assert_eq!(value["mode"], "permissive");
}

// ─── scope_permits_method tests ──────────────────────────────────────

#[test]
fn scope_wildcard_permits_all() {
    let scopes = vec!["*".to_owned()];
    assert!(scope_permits_method(&scopes, "discovery.peers"));
    assert!(scope_permits_method(&scopes, "mesh.connect"));
    assert!(scope_permits_method(&scopes, "anything"));
}

#[test]
fn scope_domain_wildcard_permits_domain() {
    let scopes = vec!["discovery.*".to_owned()];
    assert!(scope_permits_method(&scopes, "discovery.peers"));
    assert!(scope_permits_method(&scopes, "discovery.register"));
    assert!(!scope_permits_method(&scopes, "mesh.connect"));
    assert!(!scope_permits_method(&scopes, "discover.peers"));
}

#[test]
fn scope_exact_match() {
    let scopes = vec!["mesh.connect".to_owned()];
    assert!(scope_permits_method(&scopes, "mesh.connect"));
    assert!(!scope_permits_method(&scopes, "mesh.disconnect"));
    assert!(!scope_permits_method(&scopes, "mesh.connect.sub"));
}

#[test]
fn scope_multiple_patterns() {
    let scopes = vec!["discovery.*".to_owned(), "mesh.connect".to_owned()];
    assert!(scope_permits_method(&scopes, "discovery.peers"));
    assert!(scope_permits_method(&scopes, "mesh.connect"));
    assert!(!scope_permits_method(&scopes, "mesh.disconnect"));
}

#[test]
fn scope_empty_denies_all() {
    let scopes: Vec<String> = vec![];
    assert!(!scope_permits_method(&scopes, "anything"));
}

#[test]
fn scope_domain_wildcard_boundary() {
    let scopes = vec!["ipc.*".to_owned()];
    assert!(scope_permits_method(&scopes, "ipc.register"));
    assert!(!scope_permits_method(&scopes, "ipcx.register"));
    assert!(!scope_permits_method(&scopes, "ipc"));
}

// ─── extract_bearer_token tests ──────────────────────────────────────

#[test]
fn extract_token_from_params() {
    let mut params = serde_json::json!({
        "_bearer_token": "ionic-tok-123",
        "capability": "compute"
    });
    let token = extract_bearer_token(&mut params);
    assert_eq!(token, Some("ionic-tok-123".to_owned()));
    assert!(params.get("_bearer_token").is_none(), "token should be removed");
    assert_eq!(params["capability"], "compute");
}

#[test]
fn extract_token_missing() {
    let mut params = serde_json::json!({ "capability": "compute" });
    let token = extract_bearer_token(&mut params);
    assert_eq!(token, None);
}

#[test]
fn extract_token_empty_string() {
    let mut params = serde_json::json!({ "_bearer_token": "" });
    let token = extract_bearer_token(&mut params);
    assert_eq!(token, None);
}

#[test]
fn extract_token_non_object_params() {
    let mut params = serde_json::json!([1, 2, 3]);
    let token = extract_bearer_token(&mut params);
    assert_eq!(token, None);
}

// ─── verified claims in gate check ───────────────────────────────────

#[test]
fn verified_claims_with_matching_scope_passes_enforced() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let caller = CallerContext {
        bearer_token: Some("tok".to_owned()),
        verified_claims: Some(TokenClaims {
            subject: "test-primal".to_owned(),
            scopes: vec!["discovery.*".to_owned()],
            expires_at: None,
        }),
        peer: None,
        origin: ConnectionOrigin::Loopback,
    };
    assert!(gate.check("discovery.peers", &caller).is_ok());
}

#[test]
fn verified_claims_wrong_scope_rejected_enforced() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let caller = CallerContext {
        bearer_token: Some("tok".to_owned()),
        verified_claims: Some(TokenClaims {
            subject: "test-primal".to_owned(),
            scopes: vec!["mesh.*".to_owned()],
            expires_at: None,
        }),
        peer: None,
        origin: ConnectionOrigin::Loopback,
    };
    let err = gate.check("discovery.peers", &caller).unwrap_err();
    assert_eq!(err.code, error_codes::PERMISSION_DENIED);
    assert!(err.message.contains("scope"));
}

#[test]
fn verified_claims_wrong_scope_allowed_permissive() {
    let gate = MethodGate::new(EnforcementMode::Permissive);
    let caller = CallerContext {
        bearer_token: Some("tok".to_owned()),
        verified_claims: Some(TokenClaims {
            subject: "test-primal".to_owned(),
            scopes: vec!["mesh.*".to_owned()],
            expires_at: None,
        }),
        peer: None,
        origin: ConnectionOrigin::Loopback,
    };
    assert!(gate.check("discovery.peers", &caller).is_ok());
}
