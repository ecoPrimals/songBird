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
