// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
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
    dead_code,
    clippy::unnecessary_literal_unwrap,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate,
    reason = "test assertions and harness ergonomics"
)]
#![allow(clippy::all, reason = "test assertions and harness ergonomics")]
#![allow(unused, reason = "test assertions and harness ergonomics")]

//! Comprehensive Health Types Tests
#![allow(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#![allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#![allow(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#![allow(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#![allow(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_possible_truncation, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_sign_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::needless_pass_by_value, reason = "test assertions and harness ergonomics")]
#![allow(clippy::similar_names, reason = "test assertions and harness ergonomics")]
#![allow(clippy::too_many_lines, reason = "test assertions and harness ergonomics")]
#![allow(clippy::module_name_repetitions, reason = "test assertions and harness ergonomics")]
#![allow(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#![allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#![allow(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#![allow(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#![allow(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_possible_truncation, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_sign_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::needless_pass_by_value, reason = "test assertions and harness ergonomics")]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//!
//! Tests for health monitoring types in songbird-registry.

use songbird_registry::types::health::*;
use std::time::Duration;

// ============================================================================
// HEALTH STATUS TESTS
// ============================================================================

#[test]
fn test_health_status_healthy() {
    let status = HealthStatus::healthy();
    assert!(status.healthy);
    assert_eq!(status.score, 1.0);
    assert!(status.message.is_none());
}

#[test]
fn test_health_status_unhealthy() {
    let status = HealthStatus::unhealthy("Service down");
    assert!(!status.healthy);
    assert_eq!(status.score, 0.0);
    assert_eq!(status.message, Some("Service down".to_string()));
}

#[test]
fn test_health_status_degraded() {
    let status = HealthStatus::degraded(0.7, "High load");
    assert!(status.healthy); // 0.7 > 0.5
    assert_eq!(status.score, 0.7);
    assert_eq!(status.message, Some("High load".to_string()));
}

#[test]
fn test_health_status_degraded_unhealthy_threshold() {
    let status = HealthStatus::degraded(0.4, "Very high load");
    assert!(!status.healthy); // 0.4 < 0.5
    assert_eq!(status.score, 0.4);
}

#[test]
fn test_health_status_degraded_boundary() {
    let status = HealthStatus::degraded(0.5, "Exactly at threshold");
    assert!(!status.healthy); // 0.5 is not > 0.5
    assert_eq!(status.score, 0.5);
}

#[test]
fn test_health_status_with_response_time() {
    let status = HealthStatus::healthy().with_response_time(Duration::from_millis(50));

    assert_eq!(status.response_time, Duration::from_millis(50));
}

#[test]
fn test_health_status_with_metadata() {
    let status = HealthStatus::healthy().with_metadata("cpu", "50%").with_metadata("memory", "70%");

    assert!(status.message.is_some());
    let msg = status.message.expect("test precondition");
    assert!(msg.contains("cpu=50%"));
    assert!(msg.contains("memory=70%"));
}

#[test]
fn test_health_status_clone() {
    let status1 = HealthStatus::healthy();
    let status2 = status1.clone();
    assert_eq!(status1.healthy, status2.healthy);
    assert_eq!(status1.score, status2.score);
}

#[test]
fn test_health_status_debug() {
    let status = HealthStatus::healthy();
    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("HealthStatus"));
}

#[test]
fn test_health_status_serialization() {
    let status = HealthStatus::degraded(0.8, "Moderate load");
    let json = serde_json::to_string(&status).expect("Failed to serialize");
    let deserialized: HealthStatus = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.score, status.score);
}

// ============================================================================
// HEALTH CHECK TYPE TESTS
// ============================================================================

#[test]
fn test_health_check_type_http_endpoint() {
    let check = HealthCheckType::HttpEndpoint {
        url: "http://localhost:8080/health".to_string(),
        expected_status: 200,
    };

    if let HealthCheckType::HttpEndpoint {
        url,
        expected_status,
    } = check
    {
        assert_eq!(url, "http://localhost:8080/health");
        assert_eq!(expected_status, 200);
    } else {
        panic!("Expected HttpEndpoint check type");
    }
}

#[test]
fn test_health_check_type_process_check() {
    let check = HealthCheckType::ProcessCheck {
        process_name: "myapp".to_string(),
    };

    if let HealthCheckType::ProcessCheck {
        process_name,
    } = check
    {
        assert_eq!(process_name, "myapp");
    } else {
        panic!("Expected ProcessCheck type");
    }
}

#[test]
fn test_health_check_type_memory_usage() {
    let check = HealthCheckType::MemoryUsage {
        max_percentage: 80.0,
    };

    if let HealthCheckType::MemoryUsage {
        max_percentage,
    } = check
    {
        assert_eq!(max_percentage, 80.0);
    } else {
        panic!("Expected MemoryUsage type");
    }
}

#[test]
fn test_health_check_type_cpu_usage() {
    let check = HealthCheckType::CpuUsage {
        max_percentage: 90.0,
    };

    if let HealthCheckType::CpuUsage {
        max_percentage,
    } = check
    {
        assert_eq!(max_percentage, 90.0);
    } else {
        panic!("Expected CpuUsage type");
    }
}

#[test]
fn test_health_check_type_custom_script() {
    let check = HealthCheckType::CustomScript {
        script_path: "/usr/local/bin/health-check.sh".to_string(),
    };

    if let HealthCheckType::CustomScript {
        script_path,
    } = check
    {
        assert_eq!(script_path, "/usr/local/bin/health-check.sh");
    } else {
        panic!("Expected CustomScript type");
    }
}

#[test]
fn test_health_check_type_clone() {
    let check1 = HealthCheckType::HttpEndpoint {
        url: "http://test".to_string(),
        expected_status: 200,
    };

    let check2 = check1;
    assert!(matches!(check2, HealthCheckType::HttpEndpoint { .. }));
}

#[test]
fn test_health_check_type_debug() {
    let check = HealthCheckType::ProcessCheck {
        process_name: "test".to_string(),
    };

    let debug_str = format!("{:?}", check);
    assert!(debug_str.contains("ProcessCheck"));
}

#[test]
fn test_health_check_type_serialization() {
    let check = HealthCheckType::MemoryUsage {
        max_percentage: 75.0,
    };

    let json = serde_json::to_string(&check).expect("Failed to serialize");
    let deserialized: HealthCheckType = serde_json::from_str(&json).expect("Failed to deserialize");

    assert!(matches!(deserialized, HealthCheckType::MemoryUsage { .. }));
}

// ============================================================================
// HEALTH CHECK CONFIG TESTS
// ============================================================================

#[test]
fn test_health_check_config_default() {
    let config = HealthCheckConfig::default();

    assert!(matches!(config.check_type, HealthCheckType::HttpEndpoint { .. }));
    assert_eq!(config.interval, Duration::from_secs(30));
    assert_eq!(config.timeout, Duration::from_secs(5));
    assert_eq!(config.failure_threshold, 3);
    assert_eq!(config.success_threshold, 1);
}

#[test]
fn test_health_check_config_custom() {
    let config = HealthCheckConfig {
        check_type: HealthCheckType::ProcessCheck {
            process_name: "myapp".to_string(),
        },
        interval: Duration::from_secs(10),
        timeout: Duration::from_secs(2),
        failure_threshold: 5,
        success_threshold: 2,
    };

    assert_eq!(config.interval, Duration::from_secs(10));
    assert_eq!(config.failure_threshold, 5);
}

#[test]
fn test_health_check_config_with_http() {
    let config = HealthCheckConfig {
        check_type: HealthCheckType::HttpEndpoint {
            url: "https://api.example.com/health".to_string(),
            expected_status: 200,
        },
        interval: Duration::from_secs(60),
        timeout: Duration::from_secs(10),
        failure_threshold: 2,
        success_threshold: 1,
    };

    if let HealthCheckType::HttpEndpoint {
        url,
        ..
    } = &config.check_type
    {
        assert!(url.starts_with("https://"));
    }
}

#[test]
fn test_health_check_config_clone() {
    let config1 = HealthCheckConfig::default();
    let config2 = config1.clone();

    assert_eq!(config1.interval, config2.interval);
    assert_eq!(config1.timeout, config2.timeout);
}

#[test]
fn test_health_check_config_debug() {
    let config = HealthCheckConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("HealthCheckConfig"));
}

#[test]
fn test_health_check_config_serialization() {
    let config = HealthCheckConfig {
        check_type: HealthCheckType::CpuUsage {
            max_percentage: 85.0,
        },
        interval: Duration::from_secs(15),
        timeout: Duration::from_secs(3),
        failure_threshold: 4,
        success_threshold: 2,
    };

    let json = serde_json::to_string(&config).expect("Failed to serialize");
    let deserialized: HealthCheckConfig =
        serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.failure_threshold, config.failure_threshold);
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[test]
fn test_health_status_score_clamping() {
    // Score above 1.0 should be clamped
    let status1 = HealthStatus::degraded(1.5, "Over limit");
    assert_eq!(status1.score, 1.0);

    // Score below 0.0 should be clamped
    let status2 = HealthStatus::degraded(-0.5, "Under limit");
    assert_eq!(status2.score, 0.0);
}

#[test]
fn test_health_status_perfect_score() {
    let status = HealthStatus::degraded(1.0, "Perfect");
    assert!(status.healthy);
    assert_eq!(status.score, 1.0);
}

#[test]
fn test_health_status_zero_score() {
    let status = HealthStatus::degraded(0.0, "Complete failure");
    assert!(!status.healthy);
    assert_eq!(status.score, 0.0);
}

#[test]
fn test_health_check_intervals() {
    let fast = HealthCheckConfig {
        check_type: HealthCheckType::HttpEndpoint {
            url: "http://localhost".to_string(),
            expected_status: 200,
        },
        interval: Duration::from_secs(1),
        timeout: Duration::from_millis(100),
        failure_threshold: 1,
        success_threshold: 1,
    };

    let slow = HealthCheckConfig {
        check_type: HealthCheckType::HttpEndpoint {
            url: "http://localhost".to_string(),
            expected_status: 200,
        },
        interval: Duration::from_secs(300),
        timeout: Duration::from_secs(30),
        failure_threshold: 10,
        success_threshold: 5,
    };

    assert!(fast.interval < slow.interval);
}

#[test]
fn test_http_endpoint_various_status_codes() {
    let codes = vec![200, 201, 204, 301, 302, 400, 401, 403, 404, 500];

    for code in codes {
        let check = HealthCheckType::HttpEndpoint {
            url: format!("http://test/{}", code),
            expected_status: code,
        };

        if let HealthCheckType::HttpEndpoint {
            expected_status,
            ..
        } = check
        {
            assert_eq!(expected_status, code);
        }
    }
}

#[test]
fn test_memory_usage_thresholds() {
    let low = HealthCheckType::MemoryUsage {
        max_percentage: 50.0,
    };
    let medium = HealthCheckType::MemoryUsage {
        max_percentage: 75.0,
    };
    let high = HealthCheckType::MemoryUsage {
        max_percentage: 90.0,
    };

    if let (
        HealthCheckType::MemoryUsage {
            max_percentage: low_pct,
        },
        HealthCheckType::MemoryUsage {
            max_percentage: med_pct,
        },
        HealthCheckType::MemoryUsage {
            max_percentage: high_pct,
        },
    ) = (low, medium, high)
    {
        assert!(low_pct < med_pct);
        assert!(med_pct < high_pct);
    }
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_complete_health_check_workflow() {
    // Create config
    let config = HealthCheckConfig {
        check_type: HealthCheckType::HttpEndpoint {
            url: "http://localhost:8080/health".to_string(),
            expected_status: 200,
        },
        interval: Duration::from_secs(30),
        timeout: Duration::from_secs(5),
        failure_threshold: 3,
        success_threshold: 1,
    };

    // Simulate healthy response
    let healthy = HealthStatus::healthy()
        .with_response_time(Duration::from_millis(25))
        .with_metadata("version", "1.0.0");

    assert!(healthy.healthy);
    assert!(healthy.response_time < config.timeout);

    // Simulate degraded response
    let degraded =
        HealthStatus::degraded(0.6, "High load").with_response_time(Duration::from_millis(4500));

    assert!(degraded.healthy);
    assert!(degraded.response_time < config.timeout);

    // Simulate unhealthy response
    let unhealthy = HealthStatus::unhealthy("Service unresponsive");

    assert!(!unhealthy.healthy);
}

#[test]
fn test_health_status_progression() {
    let mut statuses = vec![];

    // Healthy
    statuses.push(HealthStatus::healthy());

    // Degrading
    for score in &[0.9, 0.8, 0.7, 0.6] {
        statuses.push(HealthStatus::degraded(*score, format!("Load increasing: {}", score)));
    }

    // Unhealthy
    statuses.push(HealthStatus::unhealthy("Service down"));

    assert_eq!(statuses.len(), 6);
    assert!(statuses[0].healthy);
    assert!(!statuses[5].healthy);
}

#[test]
fn test_multiple_check_types() {
    let checks = vec![
        HealthCheckType::HttpEndpoint {
            url: "http://localhost:8080/health".to_string(),
            expected_status: 200,
        },
        HealthCheckType::ProcessCheck {
            process_name: "myapp".to_string(),
        },
        HealthCheckType::MemoryUsage {
            max_percentage: 80.0,
        },
        HealthCheckType::CpuUsage {
            max_percentage: 90.0,
        },
        HealthCheckType::CustomScript {
            script_path: "/usr/local/bin/check.sh".to_string(),
        },
    ];

    assert_eq!(checks.len(), 5);
}

#[test]
fn test_failure_threshold_tracking() {
    let config = HealthCheckConfig {
        check_type: HealthCheckType::HttpEndpoint {
            url: "http://localhost".to_string(),
            expected_status: 200,
        },
        interval: Duration::from_secs(10),
        timeout: Duration::from_secs(2),
        failure_threshold: 3,
        success_threshold: 2,
    };

    // Simulate failures
    let mut failures = 0;
    for _ in 0..config.failure_threshold {
        failures += 1;
    }

    assert_eq!(failures, 3);
}

#[test]
fn test_health_check_timeout_scenarios() {
    let quick_timeout = Duration::from_millis(100);
    let normal_timeout = Duration::from_secs(5);
    let long_timeout = Duration::from_secs(30);

    assert!(quick_timeout < normal_timeout);
    assert!(normal_timeout < long_timeout);
}
