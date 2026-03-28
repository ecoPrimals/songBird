// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
#![allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#![allow(clippy::all, reason = "test assertions and harness ergonomics")]

use super::*;

use super::super::types::{RoutingPath, SovereigntyAdapterConfig};
use crate::types::capability::PrimalType;
use crate::types::{HealthStatus, ServiceInfo};
use songbird_types::SongbirdError;

#[tokio::test]
async fn test_adapter_creation_default() {
    let adapter = SovereigntyAwareAdapter::new().await;
    assert!(adapter.is_ok());
}

#[tokio::test]
async fn test_adapter_creation_with_config() {
    let config = SovereigntyAdapterConfig::default();
    let adapter = SovereigntyAwareAdapter::with_config(config).await;
    assert!(adapter.is_ok());
}

#[tokio::test]
async fn test_adapter_config_custom_settings() -> Result<(), Box<dyn std::error::Error>> {
    let config = SovereigntyAdapterConfig {
        enable_sovereignty_routing: false,
        enable_federation_routing: false,
        enable_network_optimization: true,
        sovereignty_timeout: std::time::Duration::from_secs(10),
        sovereignty_preference_weight: 0.5,
    };

    let adapter = SovereigntyAwareAdapter::with_config(config).await?;
    let retrieved_config = adapter.get_config();
    assert!(!retrieved_config.enable_sovereignty_routing);
    assert!(!retrieved_config.enable_federation_routing);
    assert!(retrieved_config.enable_network_optimization);
    assert_eq!(retrieved_config.sovereignty_timeout, std::time::Duration::from_secs(10));
    assert_eq!(retrieved_config.sovereignty_preference_weight, 0.5);
    Ok(())
}

#[tokio::test]
async fn test_get_config() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SovereigntyAwareAdapter::new().await?;
    let config = adapter.get_config();

    assert!(config.enable_sovereignty_routing);
    assert!(config.enable_federation_routing);
    assert!(config.enable_network_optimization);
    Ok(())
}

#[tokio::test]
async fn test_update_config() -> Result<(), Box<dyn std::error::Error>> {
    let mut adapter = SovereigntyAwareAdapter::new().await?;

    let new_config = SovereigntyAdapterConfig {
        enable_sovereignty_routing: false,
        enable_federation_routing: true,
        enable_network_optimization: false,
        sovereignty_timeout: std::time::Duration::from_secs(5),
        sovereignty_preference_weight: 0.6,
    };

    adapter.update_config(new_config);
    let config = adapter.get_config();

    assert!(!config.enable_sovereignty_routing);
    assert!(config.enable_federation_routing);
    assert!(!config.enable_network_optimization);
    assert_eq!(config.sovereignty_timeout, std::time::Duration::from_secs(5));
    assert_eq!(config.sovereignty_preference_weight, 0.6);
    Ok(())
}

#[tokio::test]
async fn test_get_stats() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SovereigntyAwareAdapter::new()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;
    let stats = adapter.get_stats().await;

    assert!(stats.is_ok());
    let stats = stats.map_err(|e| {
        SongbirdError::configuration(format!("Test: stats should be available: {e}"))
    })?;

    assert!(stats.sovereignty_routing_enabled);
    assert!(stats.federation_routing_enabled);
    assert!(stats.network_optimization_enabled);
    assert!(stats.base_adapter_healthy);
    Ok(())
}

#[tokio::test]
async fn test_get_stats_with_disabled_features() -> Result<(), Box<dyn std::error::Error>> {
    let config = SovereigntyAdapterConfig {
        enable_sovereignty_routing: false,
        enable_federation_routing: false,
        enable_network_optimization: false,
        sovereignty_timeout: std::time::Duration::from_secs(3),
        sovereignty_preference_weight: 0.8,
    };

    let adapter = SovereigntyAwareAdapter::with_config(config)
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;
    let stats = adapter
        .get_stats()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test: stats retrieval: {e}")))?;

    assert!(!stats.sovereignty_routing_enabled);
    assert!(!stats.federation_routing_enabled);
    assert!(!stats.network_optimization_enabled);
    Ok(())
}

#[test]
fn test_determine_compliance_level_fully_compliant() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;

    let level = adapter.determine_compliance_level(0.95);
    assert!(matches!(level, super::super::types::SovereigntyComplianceLevel::FullyCompliant));
    Ok(())
}

#[test]
fn test_determine_compliance_level_mostly_compliant() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;

    let level = adapter.determine_compliance_level(0.75);
    assert!(matches!(level, super::super::types::SovereigntyComplianceLevel::MostlyCompliant));
    Ok(())
}

#[test]
fn test_determine_compliance_level_partially_compliant() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;

    let level = adapter.determine_compliance_level(0.55);
    assert!(matches!(level, super::super::types::SovereigntyComplianceLevel::PartiallyCompliant));
    Ok(())
}

#[test]
fn test_determine_compliance_level_non_compliant() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;

    let level = adapter.determine_compliance_level(0.3);
    assert!(matches!(level, super::super::types::SovereigntyComplianceLevel::NonCompliant));
    Ok(())
}

#[test]
fn test_determine_compliance_level_boundary_90() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;

    let level = adapter.determine_compliance_level(0.9);
    assert!(matches!(level, super::super::types::SovereigntyComplianceLevel::FullyCompliant));
    Ok(())
}

#[test]
fn test_determine_compliance_level_boundary_70() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;

    let level = adapter.determine_compliance_level(0.7);
    assert!(matches!(level, super::super::types::SovereigntyComplianceLevel::MostlyCompliant));
    Ok(())
}

#[test]
fn test_determine_compliance_level_boundary_50() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;

    let level = adapter.determine_compliance_level(0.5);
    assert!(matches!(level, super::super::types::SovereigntyComplianceLevel::PartiallyCompliant));
    Ok(())
}

#[tokio::test]
async fn test_generate_basic_paths_empty_services() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SovereigntyAwareAdapter::new()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;
    let services = vec![];

    let result = adapter.generate_basic_paths(&services).await;
    assert!(result.is_ok());

    let paths =
        result.map_err(|e| SongbirdError::configuration(format!("Test: paths generation: {e}")))?;
    assert!(paths.is_empty());
    Ok(())
}

#[test]
fn test_select_best_path_empty_list() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;

    let paths = vec![];
    let result = adapter.select_best_path(&paths);

    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_select_best_path_single_path() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;

    let path = RoutingPath {
        segments: vec![],
        sovereignty_score: 0.8,
        efficiency_score: 0.7,
        combined_score: 0.75,
        security_level: super::super::types::SecurityLevel::High,
    };

    let paths = vec![path];
    let result = adapter.select_best_path(&paths);

    assert!(result.is_ok());
    let selected =
        result.map_err(|e| SongbirdError::configuration(format!("Test: path selection: {e}")))?;
    assert_eq!(selected.combined_score, 0.75);
    Ok(())
}

#[test]
fn test_select_best_path_multiple_paths() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;

    let path1 = RoutingPath {
        segments: vec![],
        sovereignty_score: 0.8,
        efficiency_score: 0.7,
        combined_score: 0.75,
        security_level: super::super::types::SecurityLevel::High,
    };

    let path2 = RoutingPath {
        segments: vec![],
        sovereignty_score: 0.9,
        efficiency_score: 0.85,
        combined_score: 0.88,
        security_level: super::super::types::SecurityLevel::Maximum,
    };

    let path3 = RoutingPath {
        segments: vec![],
        sovereignty_score: 0.6,
        efficiency_score: 0.9,
        combined_score: 0.70,
        security_level: super::super::types::SecurityLevel::Medium,
    };

    let paths = vec![path1, path2, path3];
    let result = adapter.select_best_path(&paths);

    assert!(result.is_ok());
    let selected =
        result.map_err(|e| SongbirdError::configuration(format!("Test: path selection: {e}")))?;
    assert_eq!(selected.combined_score, 0.88); // Should select path2
    Ok(())
}

#[tokio::test]
async fn test_adapter_stats_structure() {
    let stats = AdapterStats {
        sovereignty_routing_enabled: true,
        federation_routing_enabled: false,
        network_optimization_enabled: true,
        base_adapter_healthy: true,
    };

    assert!(stats.sovereignty_routing_enabled);
    assert!(!stats.federation_routing_enabled);
    assert!(stats.network_optimization_enabled);
    assert!(stats.base_adapter_healthy);
}

#[tokio::test]
async fn test_multiple_adapters_independent() -> Result<(), Box<dyn std::error::Error>> {
    let config1 = SovereigntyAdapterConfig {
        enable_sovereignty_routing: true,
        enable_federation_routing: false,
        enable_network_optimization: true,
        sovereignty_timeout: std::time::Duration::from_secs(3),
        sovereignty_preference_weight: 0.8,
    };

    let config2 = SovereigntyAdapterConfig {
        enable_sovereignty_routing: false,
        enable_federation_routing: true,
        enable_network_optimization: false,
        sovereignty_timeout: std::time::Duration::from_secs(5),
        sovereignty_preference_weight: 0.5,
    };

    let adapter1 = SovereigntyAwareAdapter::with_config(config1)
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter1 creation: {e}")))?;
    let adapter2 = SovereigntyAwareAdapter::with_config(config2)
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter2 creation: {e}")))?;

    // Verify they're independent
    assert!(adapter1.get_config().enable_sovereignty_routing);
    assert!(!adapter2.get_config().enable_sovereignty_routing);

    assert!(!adapter1.get_config().enable_federation_routing);
    assert!(adapter2.get_config().enable_federation_routing);

    Ok(())
}

#[tokio::test]
async fn test_config_timeout_very_short() -> Result<(), Box<dyn std::error::Error>> {
    let config_short = SovereigntyAdapterConfig {
        enable_sovereignty_routing: true,
        enable_federation_routing: true,
        enable_network_optimization: true,
        sovereignty_timeout: std::time::Duration::from_millis(100),
        sovereignty_preference_weight: 0.7,
    };

    let config_long = SovereigntyAdapterConfig {
        enable_sovereignty_routing: true,
        enable_federation_routing: true,
        enable_network_optimization: true,
        sovereignty_timeout: std::time::Duration::from_secs(60),
        sovereignty_preference_weight: 0.7,
    };

    let adapter_short = SovereigntyAwareAdapter::with_config(config_short)
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;
    let adapter_long = SovereigntyAwareAdapter::with_config(config_long)
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;

    assert_eq!(
        adapter_short.get_config().sovereignty_timeout,
        std::time::Duration::from_millis(100)
    );
    assert_eq!(adapter_long.get_config().sovereignty_timeout, std::time::Duration::from_secs(60));
    Ok(())
}

#[tokio::test]
async fn test_config_preference_weight_boundary() -> Result<(), Box<dyn std::error::Error>> {
    let config_min = SovereigntyAdapterConfig {
        enable_sovereignty_routing: true,
        enable_federation_routing: true,
        enable_network_optimization: true,
        sovereignty_timeout: std::time::Duration::from_secs(5),
        sovereignty_preference_weight: 0.0,
    };

    let config_max = SovereigntyAdapterConfig {
        enable_sovereignty_routing: true,
        enable_federation_routing: true,
        enable_network_optimization: true,
        sovereignty_timeout: std::time::Duration::from_secs(5),
        sovereignty_preference_weight: 1.0,
    };

    let adapter_min = SovereigntyAwareAdapter::with_config(config_min)
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;
    let adapter_max = SovereigntyAwareAdapter::with_config(config_max)
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;

    assert!((adapter_min.get_config().sovereignty_preference_weight - 0.0).abs() < 0.001);
    assert!((adapter_max.get_config().sovereignty_preference_weight - 1.0).abs() < 0.001);
    Ok(())
}

#[tokio::test]
async fn test_adapter_debug_trait() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SovereigntyAwareAdapter::new()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;

    let debug_str = format!("{adapter:?}");
    assert!(debug_str.contains("SovereigntyAwareAdapter"));
    Ok(())
}

#[tokio::test]
async fn test_generate_basic_paths_non_empty() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SovereigntyAwareAdapter::new()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;

    let services = vec![ServiceInfo {
        name: "edge-service".to_string(),
        primal_type: PrimalType::new("compute"),
        endpoint: "http://127.0.0.1:9000".to_string(),
        capabilities: vec![],
        health: HealthStatus::Unknown,
        metadata: std::collections::HashMap::new(),
    }];

    let paths = adapter.generate_basic_paths(&services).await?;
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].segments.len(), 1);
    assert_eq!(paths[0].combined_score, 0.65);
    Ok(())
}

#[test]
fn test_select_best_path_tie_prefers_one() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;

    let path_a = RoutingPath {
        segments: vec![],
        sovereignty_score: 0.8,
        efficiency_score: 0.7,
        combined_score: 0.75,
        security_level: super::super::types::SecurityLevel::High,
    };
    let path_b = RoutingPath {
        segments: vec![],
        sovereignty_score: 0.7,
        efficiency_score: 0.8,
        combined_score: 0.75,
        security_level: super::super::types::SecurityLevel::High,
    };

    let selected = adapter.select_best_path(&[path_a, path_b])?;
    assert_eq!(selected.combined_score, 0.75);
    Ok(())
}

// ========== Coverage: routing, execution, and decision creation ==========

fn make_request(id: &str) -> crate::types::UniversalRequest {
    crate::types::UniversalRequest {
        request_id: id.to_string(),
        source: "test-client".to_string(),
        target: "compute-service".to_string(),
        action: "process".to_string(),
        parameters: std::collections::HashMap::new(),
        security_context: None,
    }
}

#[tokio::test]
async fn test_route_request_no_services_returns_no_paths_error()
-> Result<(), Box<dyn std::error::Error>> {
    let adapter = SovereigntyAwareAdapter::new().await?;
    let request = make_request("sov-routing-1");

    let result = adapter.route_request(request).await;
    assert!(result.is_err(), "should error when no services are discoverable");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("No valid routing paths"),
        "error should mention no paths, got: {err_msg}"
    );
    Ok(())
}

#[tokio::test]
async fn test_route_request_sovereignty_disabled_no_services_errors()
-> Result<(), Box<dyn std::error::Error>> {
    let config = SovereigntyAdapterConfig {
        enable_sovereignty_routing: false,
        enable_federation_routing: false,
        enable_network_optimization: false,
        sovereignty_timeout: std::time::Duration::from_secs(5),
        sovereignty_preference_weight: 0.7,
    };
    let adapter = SovereigntyAwareAdapter::with_config(config).await?;
    let request = make_request("basic-routing-1");

    let result = adapter.route_request(request).await;
    assert!(result.is_err(), "basic routing with no services should also error");
    Ok(())
}

#[tokio::test]
async fn test_route_request_borrowed_no_services_errors() -> Result<(), Box<dyn std::error::Error>>
{
    let adapter = SovereigntyAwareAdapter::new().await?;
    let request = make_request("borrowed-1");

    let result = adapter.route_request_borrowed(&request).await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_execute_request_no_services_errors() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SovereigntyAwareAdapter::new().await?;
    let request = make_request("exec-1");

    let result = adapter.execute_request(request).await;
    assert!(result.is_err(), "execute_request should fail when no services available");
    Ok(())
}

#[tokio::test]
async fn test_create_routing_decision_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SovereigntyAwareAdapter::new().await?;

    let path = RoutingPath {
        segments: vec![],
        sovereignty_score: 0.85,
        efficiency_score: 0.9,
        combined_score: 0.87,
        security_level: super::super::types::SecurityLevel::High,
    };

    let decision = adapter.create_routing_decision(path.clone(), &[path]).await?;

    assert_eq!(decision.decision_metadata.algorithm_version, "sovereignty-aware-v1.0");
    assert_eq!(decision.decision_metadata.alternative_paths_count, 0);
    assert!(!decision.decision_metadata.decision_factors.is_empty());
    assert_eq!(decision.decision_metadata.decision_factors[0].factor_name, "sovereignty_score");
    Ok(())
}

#[tokio::test]
async fn test_create_routing_decision_sovereignty_assessment()
-> Result<(), Box<dyn std::error::Error>> {
    let adapter = SovereigntyAwareAdapter::new().await?;

    let path = RoutingPath {
        segments: vec![],
        sovereignty_score: 0.95,
        efficiency_score: 0.9,
        combined_score: 0.92,
        security_level: super::super::types::SecurityLevel::Maximum,
    };

    let decision = adapter.create_routing_decision(path.clone(), &[path]).await?;

    assert_eq!(decision.sovereignty_assessment.overall_score, 0.95);
    assert!(matches!(
        decision.sovereignty_assessment.compliance_level,
        super::super::types::SovereigntyComplianceLevel::FullyCompliant
    ));
    Ok(())
}

#[tokio::test]
async fn test_create_routing_decision_federation_capabilities()
-> Result<(), Box<dyn std::error::Error>> {
    let adapter = SovereigntyAwareAdapter::new().await?;

    let path = RoutingPath {
        segments: vec![],
        sovereignty_score: 0.8,
        efficiency_score: 0.8,
        combined_score: 0.8,
        security_level: super::super::types::SecurityLevel::High,
    };

    let decision = adapter.create_routing_decision(path.clone(), &[path]).await?;

    assert!(!decision.federation_capabilities.is_empty());
    assert_eq!(decision.federation_capabilities[0].capability_id, "cross_node_comm");
    assert!(decision.federation_capabilities[0].availability_score > 0.0);
    Ok(())
}

#[tokio::test]
async fn test_create_routing_decision_alternative_count() -> Result<(), Box<dyn std::error::Error>>
{
    let adapter = SovereigntyAwareAdapter::new().await?;

    let path1 = RoutingPath {
        segments: vec![],
        sovereignty_score: 0.8,
        efficiency_score: 0.8,
        combined_score: 0.8,
        security_level: super::super::types::SecurityLevel::High,
    };
    let path2 = RoutingPath {
        segments: vec![],
        sovereignty_score: 0.6,
        efficiency_score: 0.7,
        combined_score: 0.65,
        security_level: super::super::types::SecurityLevel::Medium,
    };

    let decision = adapter.create_routing_decision(path1.clone(), &[path1, path2]).await?;

    assert_eq!(decision.decision_metadata.alternative_paths_count, 1);
    Ok(())
}

#[tokio::test]
async fn test_execute_through_path_response_shape() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SovereigntyAwareAdapter::new().await?;
    let request = make_request("through-path-1");
    let path = RoutingPath {
        segments: vec![],
        sovereignty_score: 0.8,
        efficiency_score: 0.8,
        combined_score: 0.8,
        security_level: super::super::types::SecurityLevel::High,
    };

    let response = adapter.execute_through_path(request, &path).await?;
    assert_eq!(response.request_id, "through-path-1");
    assert!(response.error.is_none());
    let data = response.data.expect("should have data");
    assert_eq!(data["sovereignty"], "routed");
    Ok(())
}

#[tokio::test]
async fn test_generate_basic_paths_multiple_services() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SovereigntyAwareAdapter::new().await?;

    let services = vec![
        ServiceInfo {
            name: "svc-a".to_string(),
            primal_type: PrimalType::new("compute"),
            endpoint: "http://a:9000".to_string(),
            capabilities: vec![],
            health: HealthStatus::Healthy,
            metadata: std::collections::HashMap::new(),
        },
        ServiceInfo {
            name: "svc-b".to_string(),
            primal_type: PrimalType::new("storage"),
            endpoint: "http://b:9001".to_string(),
            capabilities: vec![],
            health: HealthStatus::Healthy,
            metadata: std::collections::HashMap::new(),
        },
    ];

    let paths = adapter.generate_basic_paths(&services).await?;
    assert_eq!(paths.len(), 2);
    for p in &paths {
        assert_eq!(p.segments.len(), 1);
        assert_eq!(p.sovereignty_score, 0.6);
        assert_eq!(p.efficiency_score, 0.7);
        assert_eq!(p.combined_score, 0.65);
    }
    Ok(())
}

#[tokio::test]
async fn test_adapter_stats_clone_and_debug() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SovereigntyAwareAdapter::new().await?;
    let stats = adapter.get_stats().await?;
    let cloned = stats.clone();

    assert_eq!(stats.sovereignty_routing_enabled, cloned.sovereignty_routing_enabled);
    assert_eq!(stats.base_adapter_healthy, cloned.base_adapter_healthy);

    let dbg = format!("{stats:?}");
    assert!(dbg.contains("AdapterStats"));
    Ok(())
}

#[test]
fn test_determine_compliance_level_negative_score() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;

    let level = adapter.determine_compliance_level(-0.1);
    assert!(matches!(level, super::super::types::SovereigntyComplianceLevel::NonCompliant));
    Ok(())
}

#[test]
fn test_determine_compliance_level_above_one() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
        .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {e}")))?;

    let level = adapter.determine_compliance_level(1.5);
    assert!(matches!(level, super::super::types::SovereigntyComplianceLevel::FullyCompliant));
    Ok(())
}
