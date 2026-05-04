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
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

#[cfg(test)]
#[allow(clippy::all, reason = "test assertions and harness ergonomics")]
#[allow(unused, reason = "test assertions and harness ergonomics")]
mod chaos_activation_tests {
    use songbird_test_utils::chaos_engineering::{
        ChaosEngineeringManager, ExperimentConfig, ExperimentStatus, ExperimentType,
        NetworkFaultConfig,
    };
    use songbird_types::SongbirdError;

    #[tokio::test]
    async fn test_chaos_manager_creation() {
        let _manager = ChaosEngineeringManager::new();
        // Basic validation that the manager can be created
        // Chaos engineering manager created successfully - no assertion needed
    }

    #[tokio::test]
    async fn test_network_fault_configuration() {
        let config = NetworkFaultConfig {
            latency_ms: Some(100),
            packet_loss_percent: Some(5.0),
            bandwidth_limit_bps: Some(1_000_000),
            partition_enabled: false,
        };

        assert_eq!(config.latency_ms, Some(100));
        assert_eq!(config.packet_loss_percent, Some(5.0));
        assert_eq!(config.bandwidth_limit_bps, Some(1_000_000));
        assert!(!config.partition_enabled);
    }
    #[tokio::test]
    async fn test_experiment_configuration() {
        let network_config = NetworkFaultConfig {
            latency_ms: Some(50),
            packet_loss_percent: Some(2.0),
            bandwidth_limit_bps: None,
            partition_enabled: false,
        };

        let experiment_config = ExperimentConfig {
            network_fault: Some(network_config),
            service_failure: None,
            resource_constraint: None,
            byzantine_failure: None,
            performance_degradation: None,
        };
        assert!(experiment_config.network_fault.is_some());
        assert!(experiment_config.service_failure.is_none());
    }
    #[test]
    fn test_experiment_types() -> anyhow::Result<()> {
        // Test that all experiment types can be created
        let experiment_types = vec![
            ExperimentType::NetworkFault,
            ExperimentType::ServiceFailure,
            ExperimentType::ResourceConstraint,
            ExperimentType::ByzantineFailure,
            ExperimentType::PerformanceDegradation,
            ExperimentType::ConfigurationError,
            ExperimentType::SecurityAttack,
            ExperimentType::DependencyFailure,
        ];

        assert_eq!(experiment_types.len(), 8);

        // Verify we can serialize/deserialize experiment types
        for exp_type in experiment_types {
            let serialized = serde_json::to_string(&exp_type)
                .map_err(|e| SongbirdError::configuration("Should serialize".to_string()))?;
            let _deserialized: ExperimentType = serde_json::from_str(&serialized)
                .map_err(|e| SongbirdError::configuration("Error occurred".to_string()))?;
        }
        Ok(())
    }
    #[test]
    fn test_experiment_status() -> anyhow::Result<()> {
        let statuses = vec![
            ExperimentStatus::Preparing,
            ExperimentStatus::Running,
            ExperimentStatus::Completed,
            ExperimentStatus::Failed,
            ExperimentStatus::Stopped,
        ];

        assert_eq!(statuses.len(), 5);

        // Verify serialization works for all statuses
        for status in statuses {
            let serialized = serde_json::to_string(&status)
                .map_err(|e| SongbirdError::configuration("Should serialize".to_string()))?;
            let _deserialized: ExperimentStatus = serde_json::from_str(&serialized)
                .map_err(|e| SongbirdError::configuration("Error occurred".to_string()))?;
        }
        Ok(())
    }
}
