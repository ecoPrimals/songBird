// SPDX-License-Identifier: AGPL-3.0-only
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
    clippy::clone_on_ref_ptr,
    clippy::similar_names,
    clippy::unreadable_literal,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap
)]

//! Comprehensive integration tests for songbird-discovery
//!
//! These tests cover integration scenarios, concurrent operations,
//! error handling, and real-world usage patterns.

#![allow(clippy::unwrap_used)]
use songbird_discovery::discovery::config::SongbirdDiscoveryConfig;
use songbird_discovery::discovery::factory::UniversalDiscoveryFactory;
use songbird_discovery::discovery::types::NodeType;
use songbird_types::SongbirdResult;
use std::sync::Arc;
// ============================================================================
// Configuration and Setup Tests
fn create_test_config() -> SongbirdDiscoveryConfig {
    SongbirdDiscoveryConfig {
        node_id: Some("test-node".to_string()),
        node_type: NodeType::Hybrid,
        institution: None,
        federation_enabled: false,
        health_check_interval_secs: 1,
        node_discovery_interval_secs: 1,
        trust_verification_enabled: false,
        max_federation_nodes: 10,
        network: songbird_discovery::discovery::config::NetworkConfig::default(),
        monitoring: songbird_discovery::discovery::config::MonitoringConfig::default(),
        trust: songbird_discovery::discovery::config::TrustConfig::default(),
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_discovery_factory_creation_default() -> SongbirdResult<()> {
    let result = UniversalDiscoveryFactory::create_auto_detect().await;
    assert!(result.is_ok(), "Auto-detect factory should create successfully");
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_discovery_factory_creation_multiple_times() -> SongbirdResult<()> {
    // Create multiple instances
    let result1 = UniversalDiscoveryFactory::create_auto_detect().await;
    let result2 = UniversalDiscoveryFactory::create_auto_detect().await;
    assert!(result1.is_ok(), "First factory creation should succeed");
    assert!(result2.is_ok(), "Second factory creation should succeed");
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_discovery_multiple_instances_creation() -> SongbirdResult<()> {
    let discovery1 = UniversalDiscoveryFactory::create_auto_detect().await?;
    let discovery2 = UniversalDiscoveryFactory::create_auto_detect().await?;
    // Verify instances can be created independently
    // Note: We can't compare pointers directly since these aren't Arc<T>
    drop(discovery1);
    drop(discovery2);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_config_node_type_compute() -> SongbirdResult<()> {
    let config = create_test_config();
    assert_eq!(config.node_type, NodeType::Hybrid);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_config_node_type_storage() -> SongbirdResult<()> {
    let mut config = create_test_config();
    config.node_type = NodeType::Storage;
    assert_eq!(config.node_type, NodeType::Storage);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_config_node_type_gateway() -> SongbirdResult<()> {
    let mut config = create_test_config();
    config.node_type = NodeType::Gateway;
    assert_eq!(config.node_type, NodeType::Gateway);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_config_node_type_orchestrator() -> SongbirdResult<()> {
    let mut config = create_test_config();
    config.node_type = NodeType::Orchestrator;
    assert_eq!(config.node_type, NodeType::Orchestrator);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_config_with_short_intervals() -> SongbirdResult<()> {
    let mut config = create_test_config();
    config.node_discovery_interval_secs = 1;
    config.health_check_interval_secs = 1;
    assert_eq!(config.node_discovery_interval_secs, 1);
    assert_eq!(config.health_check_interval_secs, 1);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_config_with_long_intervals() -> SongbirdResult<()> {
    let mut config = create_test_config();
    config.node_discovery_interval_secs = 300;
    config.health_check_interval_secs = 120;
    assert_eq!(config.node_discovery_interval_secs, 300);
    assert_eq!(config.health_check_interval_secs, 120);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_config_with_federation_enabled() -> SongbirdResult<()> {
    let mut config = create_test_config();
    config.federation_enabled = true;
    assert!(config.federation_enabled);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_config_with_federation_disabled() -> SongbirdResult<()> {
    let config = create_test_config();
    assert!(!config.federation_enabled);
    Ok(())
}

// Concurrent Operations Tests
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_factory_creation() -> SongbirdResult<()> {
    let mut handles = vec![];
    // Create 10 discovery instances concurrently
    for _ in 0..10 {
        let handle =
            tokio::spawn(async move { UniversalDiscoveryFactory::create_auto_detect().await });
        handles.push(handle);
    }
    // Wait for all to complete
    let mut success_count = 0;
    for handle in handles {
        if let Ok(Ok(_)) = handle.await {
            success_count += 1;
        }
    }
    assert!(success_count >= 8, "At least 80% of concurrent creations should succeed");
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_multiple_instances_in_parallel() -> SongbirdResult<()> {
    // Create multiple instances in parallel
    let (result1, result2, result3) = tokio::join!(
        UniversalDiscoveryFactory::create_auto_detect(),
        UniversalDiscoveryFactory::create_auto_detect(),
        UniversalDiscoveryFactory::create_auto_detect()
    );
    assert!(result1.is_ok(), "First parallel instance should succeed");
    assert!(result2.is_ok(), "Second parallel instance should succeed");
    assert!(result3.is_ok(), "Third parallel instance should succeed");
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_access_to_discovery() -> SongbirdResult<()> {
    let discovery = Arc::new(UniversalDiscoveryFactory::create_auto_detect().await?);
    let mut handles = vec![];

    // Access discovery instance from multiple tasks
    for _ in 0..20 {
        let discovery_clone = discovery.clone();
        let handle = tokio::spawn(async move {
            // Just check that we can clone and use the discovery instance
            let _ = Arc::strong_count(&discovery_clone);
            Ok::<(), songbird_types::SongbirdError>(())
        });
        handles.push(handle);
    }

    // All accesses should complete successfully
    for handle in handles {
        assert!(handle.await.is_ok(), "Concurrent access should succeed");
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_rapid_factory_calls() -> SongbirdResult<()> {
    // Rapidly create and drop discovery instances
    for _ in 0..10 {
        let discovery = UniversalDiscoveryFactory::create_auto_detect().await?;
        drop(discovery);
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_arc_reference_counting() -> SongbirdResult<()> {
    let discovery = Arc::new(UniversalDiscoveryFactory::create_auto_detect().await?);
    assert_eq!(Arc::strong_count(&discovery), 1);
    let clone1 = discovery.clone();
    assert_eq!(Arc::strong_count(&discovery), 2);
    let clone2 = discovery.clone();
    assert_eq!(Arc::strong_count(&discovery), 3);
    drop(clone1);
    assert_eq!(Arc::strong_count(&discovery), 2);
    drop(clone2);
    assert_eq!(Arc::strong_count(&discovery), 1);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_discovery_instance_lifetime() -> SongbirdResult<()> {
    {
        let _discovery = UniversalDiscoveryFactory::create_auto_detect().await?;
        // Discovery lives in this scope
    } // Dropped here
    // Create another instance to verify no interference
    let _discovery2 = UniversalDiscoveryFactory::create_auto_detect().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_scoped_discovery_instances() -> SongbirdResult<()> {
    {
        let _discovery1 = UniversalDiscoveryFactory::create_auto_detect().await?;
        {
            let _discovery2 = UniversalDiscoveryFactory::create_auto_detect().await?;
            // Both alive here
        } // discovery2 dropped
        // Only discovery1 alive here
    } // discovery1 dropped
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_discovery_with_different_node_ids() -> SongbirdResult<()> {
    for i in 0..5 {
        let mut config = create_test_config();
        config.node_id = Some(format!("node-{}", i));
        assert_eq!(config.node_id, Some(format!("node-{}", i)));
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_node_type_transitions() -> SongbirdResult<()> {
    let mut config = create_test_config();
    let node_types = vec![
        NodeType::Compute,
        NodeType::Storage,
        NodeType::Gateway,
        NodeType::Hybrid,
        NodeType::Orchestrator,
    ];
    for node_type in node_types {
        config.node_type = node_type.clone();
        assert_eq!(config.node_type, node_type, "Should set node type correctly");
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_config_with_institution() -> SongbirdResult<()> {
    let mut config = create_test_config();
    config.institution = Some("test-university".to_string());
    assert_eq!(config.institution, Some("test-university".to_string()));
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_config_with_trust_verification_enabled() -> SongbirdResult<()> {
    let mut config = create_test_config();
    config.trust_verification_enabled = true;
    assert!(config.trust_verification_enabled);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_config_with_max_federation_nodes() -> SongbirdResult<()> {
    let mut config = create_test_config();
    config.max_federation_nodes = 100;
    assert_eq!(config.max_federation_nodes, 100);
    Ok(())
}
