// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::all, reason = "test assertions and harness ergonomics")]
#![allow(unused, reason = "unused bindings/imports in this compilation unit")]

use super::*;

#[test]
fn test_security_error_creation() {
    let error = SecurityError {
        message: "Authentication failed".to_string(),
        operation: Some("login".to_string()),
        required_permission: Some("user".to_string()),
        context: Some("authentication".to_string()),
        remediation: None,
    };

    assert_eq!(error.operation, Some("login".to_string()));
    assert!(error.to_string().contains("Security error"));
}

#[test]
fn test_songbird_error_variants() {
    let network_error = SongbirdError::Network {
        message: "Connection timeout".to_string(),
        interface: Some("eth0".to_string()),
        suggestion: None,
    };

    let security_error = SongbirdError::Security(SecurityError {
        message: "Unauthorized access".to_string(),
        operation: Some("read".to_string()),
        required_permission: Some("admin".to_string()),
        context: None,
        remediation: None,
    });

    assert!(network_error.to_string().contains("Network error"));
    assert!(security_error.to_string().contains("Unauthorized access"));
}

#[test]
fn test_error_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let error = SongbirdError::Security(SecurityError {
        message: "Invalid token".to_string(),
        operation: Some("authenticate".to_string()),
        required_permission: Some("test_permission".to_string()),
        context: None,
        remediation: None,
    });

    let serialized = serde_json::to_string(&error).map_err(|e| {
        SongbirdError::configuration(format!("Test: serialization should succeed: {e}"))
    })?;
    let deserialized: SongbirdError = serde_json::from_str(&serialized).map_err(|e| {
        SongbirdError::configuration(format!("Test: deserialization should succeed: {e}"))
    })?;

    assert_eq!(deserialized.to_string(), error.to_string());
    Ok(())
}

#[test]
fn test_service_error_with_recovery() {
    let error = SongbirdError::Service {
        service: "database".to_string(),
        message: "Connection failed".to_string(),
        suggested_alternatives: vec!["backup-db".to_string(), "cache".to_string()],
        recovery_actions: vec!["retry".to_string(), "fallback".to_string()],
    };

    match error {
        SongbirdError::Service {
            suggested_alternatives,
            recovery_actions,
            ..
        } => {
            assert_eq!(suggested_alternatives.len(), 2);
            assert_eq!(recovery_actions.len(), 2);
            assert!(suggested_alternatives.contains(&"backup-db".to_string()));
        }
        _ => panic!("Expected Service error"),
    }
}

#[test]
fn test_error_context_and_suggestions() {
    let mut security_error = SongbirdError::security("Invalid token");
    security_error.with_context("authentication");
    security_error.with_suggestion("Check network connectivity");

    let mut network_error = SongbirdError::network("Connection failed");
    network_error.with_suggestion("Check network connectivity");

    assert!(security_error.to_string().contains("Invalid token"));
    assert!(network_error.to_string().contains("Connection failed"));
}

#[test]
fn test_from_str_and_string_configuration() {
    let e: SongbirdError = "env missing".into();
    assert!(e.to_string().contains("env missing"));

    let e2: SongbirdError = String::from("boxed").into();
    assert!(e2.to_string().contains("boxed"));
}

#[test]
fn test_from_addr_parse_error_display() {
    let parse_result = ":::".parse::<std::net::IpAddr>();
    let Err(addr_err) = parse_result else {
        panic!("invalid IP should not parse");
    };
    let err: SongbirdError = addr_err.into();
    assert!(err.to_string().contains("Address parse") || err.to_string().contains("parse"));
}

#[test]
fn test_songbird_error_timeout_has_suggestion() {
    let e = SongbirdError::timeout("deadline exceeded");
    assert!(e.to_string().contains("deadline"));
    match e {
        SongbirdError::Network {
            suggestion: Some(s),
            ..
        } => {
            assert!(
                s.contains("timeout") || s.contains("network"),
                "timeout() should attach remediation text, got: {s}"
            );
        }
        SongbirdError::Network {
            suggestion: None,
            ..
        } => panic!("timeout() should attach remediation text"),
        _ => panic!("expected Network variant for timeout()"),
    }
}

#[test]
fn test_metrics_and_registry_constructors_display() {
    let m = SongbirdError::metrics("counter stuck", "flush");
    assert!(m.to_string().contains("Metrics"));

    let r = SongbirdError::registry("not found", "lookup");
    assert!(r.to_string().contains("Registry"));
}

#[test]
fn test_from_serde_json_error_maps_to_serialization_variant() {
    let bad = "not json";
    let Err(json_err) = serde_json::from_str::<serde_json::Value>(bad) else {
        panic!("invalid json should not parse");
    };
    let e: SongbirdError = json_err.into();
    let s = e.to_string();
    assert!(s.contains("Serialization") || s.contains("JSON"));
}

#[test]
fn test_from_io_error_maps_to_network() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
    let e: SongbirdError = io_err.into();
    assert!(e.to_string().contains("IO error") || e.to_string().contains("Network"));
}

#[test]
fn test_not_implemented_constructors_display() {
    let e = SongbirdError::not_implemented("federation_mesh");
    assert!(e.to_string().contains("Not implemented") || e.to_string().contains("federation_mesh"));

    let e2 = SongbirdError::not_implemented_with_detail("btsp_bidirectional", "wire ATT first");
    assert!(e2.to_string().contains("btsp_bidirectional"));
    match e2 {
        SongbirdError::NotImplemented {
            detail,
            ..
        } => assert_eq!(detail.as_deref(), Some("wire ATT first")),
        _ => panic!("expected NotImplemented"),
    }
}

#[test]
fn test_rpc_event_discovery_protocol_runtime_constructors() {
    let r = SongbirdError::rpc("method failed");
    assert!(r.to_string().contains("RPC"));

    let ev = SongbirdError::event("handler panicked");
    assert!(ev.to_string().contains("Event"));

    let d = SongbirdError::discovery("mdns timeout");
    assert!(d.to_string().contains("Discovery"));

    let p = SongbirdError::protocol("version skew");
    assert!(p.to_string().contains("Protocol"));
}

#[test]
fn test_load_balancing_constructor_display() {
    let e = SongbirdError::load_balancing("no backends", "round_robin");
    assert!(e.to_string().contains("Load balancing") || e.to_string().contains("backends"));
}

#[test]
fn test_validation_with_suggestion_updates_field() {
    let mut e = SongbirdError::validation("bad port");
    e.with_suggestion("use 1024-65535");
    match e {
        SongbirdError::Validation {
            suggestion,
            ..
        } => {
            assert_eq!(suggestion.as_deref(), Some("use 1024-65535"));
        }
        _ => panic!("expected Validation"),
    }
}

#[test]
fn test_with_context_is_no_op_for_service_variant() {
    let mut e = SongbirdError::service("db", "down");
    e.with_context("extra");
    assert!(e.to_string().contains("db") || e.to_string().contains("down"));
}

#[test]
fn test_automation_hint_and_urgency_json_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let hint = AutomationHint::RetryExponential {
        max_attempts: 3,
        base_delay_ms: 100,
    };
    let js = serde_json::to_string(&hint)?;
    let back: AutomationHint = serde_json::from_str(&js)?;
    match back {
        AutomationHint::RetryExponential {
            max_attempts,
            base_delay_ms,
        } => {
            assert_eq!(max_attempts, 3);
            assert_eq!(base_delay_ms, 100);
        }
        _ => panic!("expected RetryExponential variant"),
    }

    // `Ord` follows enum declaration order (Critical < … < Low), not severity semantics.
    assert!(Urgency::Critical < Urgency::Low);
    Ok(())
}

#[test]
fn test_security_error_display_impl_path() {
    let s = SecurityError {
        message: "denied".into(),
        operation: None,
        required_permission: None,
        context: None,
        remediation: None,
    };
    assert_eq!(s.to_string(), "Security error: denied");
    let err: &(dyn std::error::Error) = &s;
    assert!(err.to_string().contains("denied"));
}

#[test]
fn test_runtime_and_response_extraction_variants_display() {
    let runtime = SongbirdError::Runtime {
        message: "task panicked".into(),
        component: Some("worker".into()),
        debug_info: Some("stack omitted".into()),
    };
    assert!(
        runtime.to_string().contains("Async runtime")
            || runtime.to_string().contains("task panicked")
    );

    let extract = SongbirdError::ResponseExtraction {
        message: "empty data field".into(),
    };
    assert!(extract.to_string().contains("Response error"));
    assert!(extract.to_string().contains("empty data"));
}

#[test]
fn test_configuration_with_context_updates_suggestion() {
    let mut e = SongbirdError::configuration("missing bind");
    e.with_context("set SONGBIRD_BIND_ADDRESS");
    match e {
        SongbirdError::Configuration {
            suggestion,
            ..
        } => {
            assert_eq!(suggestion.as_deref(), Some("set SONGBIRD_BIND_ADDRESS"));
        }
        _ => panic!("expected Configuration"),
    }
}

#[test]
fn test_network_with_context_updates_suggestion() {
    let mut e = SongbirdError::network("timeout");
    e.with_context("retry after 5s");
    match e {
        SongbirdError::Network {
            suggestion,
            ..
        } => {
            assert_eq!(suggestion.as_deref(), Some("retry after 5s"));
        }
        _ => panic!("expected Network"),
    }
}

#[test]
fn test_automation_hint_all_variants_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let hints = [
        AutomationHint::RetryExponential {
            max_attempts: 5,
            base_delay_ms: 200,
        },
        AutomationHint::RetryFixed {
            max_attempts: 2,
            interval_ms: 50,
        },
        AutomationHint::FallbackService {
            alternatives: vec!["backup".into()],
        },
        AutomationHint::EscalateHuman {
            urgency: Urgency::High,
        },
        AutomationHint::Ignore,
        AutomationHint::CircuitOpen {
            retry_after_secs: 30,
        },
    ];
    for hint in hints {
        let js = serde_json::to_string(&hint)?;
        let _: AutomationHint = serde_json::from_str(&js)?;
    }
    Ok(())
}

#[test]
fn test_service_error_display_includes_service_name() {
    let e = SongbirdError::service("orchestrator", "not ready");
    let s = e.to_string();
    assert!(s.contains("orchestrator"));
    assert!(s.contains("not ready"));
}

#[test]
fn test_validation_error_display() {
    let e = SongbirdError::Validation {
        message: "port out of range".into(),
        field: Some("port".into()),
        suggestion: None,
    };
    assert!(e.to_string().contains("Validation"));
    assert!(e.to_string().contains("port"));
}

#[test]
fn test_serialization_error_with_format_field() {
    let e = SongbirdError::Serialization {
        format: Some("TOML".into()),
        message: "unexpected key".into(),
        debug_info: Some("line 4".into()),
    };
    assert!(e.to_string().contains("Serialization"));
    assert!(e.to_string().contains("unexpected key"));
}

#[test]
fn test_rpc_error_with_method_and_code() {
    let e = SongbirdError::Rpc {
        message: "internal".into(),
        method: Some("health.check".into()),
        code: Some(-32000),
    };
    let s = e.to_string();
    assert!(s.contains("RPC"));
    assert!(s.contains("internal"));
}

#[test]
fn test_discovery_error_with_backend() {
    let e = SongbirdError::Discovery {
        message: "timeout".into(),
        backend: Some("consul".into()),
        retry_strategy: Some("linear".into()),
    };
    assert!(e.to_string().contains("Discovery"));
    assert!(e.to_string().contains("timeout"));
}
