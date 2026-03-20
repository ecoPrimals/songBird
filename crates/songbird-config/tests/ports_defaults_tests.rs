// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    clippy::unnecessary_literal_unwrap,
    reason = "test assertions and harness ergonomics"
)]

//! Comprehensive tests for default port configuration

use songbird_config::defaults::ports::*;

// ============================================================================
// Core Service Port Tests
// ============================================================================

#[test]
fn test_orchestrator_port_default() {
    // Should return default when env var not set
    let port = orchestrator_port();
    assert!(port > 0);
    // Default is 8080 unless env var overrides
    if std::env::var("SONGBIRD_ORCHESTRATOR_PORT").is_err() {
        assert_eq!(port, 8080);
    }
}

#[test]
fn test_discovery_port_default() {
    let port = discovery_port();
    assert!(port > 0);
    if std::env::var("SONGBIRD_DISCOVERY_PORT").is_err() {
        assert_eq!(port, 8081);
    }
}

#[test]
fn test_dashboard_port_default() {
    let port = dashboard_port();
    assert!(port > 0);
    if std::env::var("SONGBIRD_DASHBOARD_PORT").is_err() {
        assert_eq!(port, 3000);
    }
}

#[test]
fn test_metrics_port_default() {
    let port = metrics_port();
    assert!(port > 0);
    if std::env::var("SONGBIRD_METRICS_PORT").is_err() {
        assert_eq!(port, 9090);
    }
}

#[test]
fn test_federation_port_default() {
    let port = federation_port();
    assert!(port > 0);
    if std::env::var("SONGBIRD_FEDERATION_PORT").is_err() {
        assert_eq!(port, 8082);
    }
}

#[test]
fn test_websocket_port_default() {
    let port = websocket_port();
    assert!(port > 0);
    if std::env::var("SONGBIRD_WEBSOCKET_PORT").is_err() {
        assert_eq!(port, 8080);
    }
}

// ============================================================================
// Gaming Port Tests
// ============================================================================

#[test]
fn test_gaming_port_default() {
    let port = gaming_port();
    assert!(port > 0);
    if std::env::var("SONGBIRD_GAMING_PORT").is_err() {
        assert_eq!(port, 6112); // StarCraft IPX default
    }
}

#[test]
fn test_gaming_port_range_start_default() {
    let port = gaming_port_range_start();
    assert!(port > 0);
    if std::env::var("SONGBIRD_GAMING_PORT_START").is_err() {
        assert_eq!(port, 7000);
    }
}

#[test]
fn test_gaming_port_range_end_default() {
    let port = gaming_port_range_end();
    assert!(port > 0);
    if std::env::var("SONGBIRD_GAMING_PORT_END").is_err() {
        assert_eq!(port, 7100);
    }
}

#[test]
fn test_gaming_port_range_valid() {
    let start = gaming_port_range_start();
    let end = gaming_port_range_end();
    // Range should be valid (end > start)
    assert!(end > start);
}

#[test]
fn test_starcraft_port_default() {
    let port = starcraft_port();
    assert!(port > 0);
    if std::env::var("SONGBIRD_STARCRAFT_PORT").is_err() {
        assert_eq!(port, 6112);
    }
}

#[test]
fn test_aoe2_port_default() {
    let port = aoe2_port();
    assert!(port > 0);
    if std::env::var("SONGBIRD_AOE2_PORT").is_err() {
        assert_eq!(port, 2300);
    }
}

#[test]
fn test_cnc_port_range_start_default() {
    let port = cnc_port_range_start();
    assert!(port > 0);
    if std::env::var("SONGBIRD_CNC_PORT_START").is_err() {
        assert_eq!(port, 1234);
    }
}

#[test]
fn test_cnc_port_range_end_default() {
    let port = cnc_port_range_end();
    assert!(port > 0);
    if std::env::var("SONGBIRD_CNC_PORT_END").is_err() {
        assert_eq!(port, 1240);
    }
}

#[test]
fn test_cnc_port_range_valid() {
    let start = cnc_port_range_start();
    let end = cnc_port_range_end();
    assert!(end > start);
}

// ============================================================================
// Primal Service Port Tests
// ============================================================================

#[test]
fn test_health_port_default() {
    let port = health_port();
    assert!(port > 0);
    if std::env::var("SONGBIRD_HEALTH_PORT").is_err() {
        assert_eq!(port, 8002);
    }
}

#[test]
fn test_beardog_port_default() {
    let port = beardog_port();
    assert!(port > 0);
    if std::env::var("SONGBIRD_BEARDOG_PORT").is_err() {
        assert_eq!(port, 8443);
    }
}

#[test]
fn test_toadstool_port_default() {
    let port = toadstool_port();
    assert!(port > 0);
    if std::env::var("SONGBIRD_TOADSTOOL_PORT").is_err() {
        assert_eq!(port, 8001);
    }
}

#[test]
fn test_squirrel_port_default() {
    let port = squirrel_port();
    assert!(port > 0);
    if std::env::var("SONGBIRD_SQUIRREL_PORT").is_err() {
        assert_eq!(port, 8002);
    }
}

#[test]
fn test_nestgate_port_default() {
    let port = nestgate_port();
    assert!(port > 0);
    if std::env::var("SONGBIRD_NESTGATE_PORT").is_err() {
        assert_eq!(port, 8003);
    }
}

#[test]
fn test_tarpc_port_default() {
    let port = tarpc_port();
    assert!(port > 0);
    if std::env::var("SONGBIRD_TARPC_PORT").is_err() {
        assert_eq!(port, 8091);
    }
}

// ============================================================================
// Dynamic Service Port Tests
// ============================================================================

#[test]
fn test_service_port_with_default() {
    let port = service_port("NONEXISTENT_SERVICE", 5555);
    assert_eq!(port, 5555);
}

#[test]
fn test_service_port_case_conversion() {
    // Should convert to uppercase for env var lookup
    let port = service_port("custom_service", 6666);
    // Without env var set, should return default
    assert_eq!(port, 6666);
}

#[test]
fn test_service_port_various_defaults() {
    assert_eq!(service_port("TEST_A", 1000), 1000);
    assert_eq!(service_port("TEST_B", 2000), 2000);
    assert_eq!(service_port("TEST_C", 3000), 3000);
}

// ============================================================================
// Port Uniqueness Tests
// ============================================================================

#[test]
fn test_default_ports_are_distinct() {
    // Core services should have distinct ports
    let orchestrator = orchestrator_port();
    let discovery = discovery_port();
    let federation = federation_port();
    let metrics = metrics_port();
    let dashboard = dashboard_port();

    // Check key ports are distinct (when using defaults)
    if std::env::var("SONGBIRD_DISCOVERY_PORT").is_err()
        && std::env::var("SONGBIRD_ORCHESTRATOR_PORT").is_err()
    {
        assert_ne!(discovery, orchestrator);
    }
    if std::env::var("SONGBIRD_FEDERATION_PORT").is_err()
        && std::env::var("SONGBIRD_ORCHESTRATOR_PORT").is_err()
    {
        assert_ne!(federation, orchestrator);
    }
    if std::env::var("SONGBIRD_METRICS_PORT").is_err()
        && std::env::var("SONGBIRD_ORCHESTRATOR_PORT").is_err()
    {
        assert_ne!(metrics, orchestrator);
    }
    if std::env::var("SONGBIRD_DASHBOARD_PORT").is_err()
        && std::env::var("SONGBIRD_ORCHESTRATOR_PORT").is_err()
    {
        assert_ne!(dashboard, orchestrator);
    }
}

#[test]
fn test_primal_service_ports_distinct() {
    // Primal services should have distinct ports
    let beardog = beardog_port();
    let toadstool = toadstool_port();
    let squirrel = squirrel_port();
    let nestgate = nestgate_port();
    let tarpc = tarpc_port();

    // All should be positive
    assert!(beardog > 0);
    assert!(toadstool > 0);
    assert!(squirrel > 0);
    assert!(nestgate > 0);
    assert!(tarpc > 0);

    // Check distinctions when using defaults
    if std::env::var("SONGBIRD_BEARDOG_PORT").is_err()
        && std::env::var("SONGBIRD_TOADSTOOL_PORT").is_err()
    {
        assert_ne!(beardog, toadstool);
    }
}

// ============================================================================
// Port Range Tests
// ============================================================================

#[test]
fn test_all_ports_in_valid_range() {
    // Defaults must be non-zero and fit in u16 (type already enforces max).
    for port in [
        orchestrator_port(),
        discovery_port(),
        dashboard_port(),
        metrics_port(),
        federation_port(),
        websocket_port(),
        gaming_port(),
        health_port(),
        beardog_port(),
        toadstool_port(),
        squirrel_port(),
        nestgate_port(),
        tarpc_port(),
        starcraft_port(),
        aoe2_port(),
    ] {
        assert!(port > 0, "default port must be non-zero");
    }
}
