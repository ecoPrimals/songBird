// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

// Chaos Engineering Manager
//
// Core chaos engineering management following canonical patterns.
// Extracted from monolithic fault_injection.rs for maintainability.

use crate::chaos_engineering::config::{
    ByzantineFailureConfig, ChaosExperiment, ExperimentStatus, ExperimentType, MetricSnapshot,
    NetworkFaultConfig, PerformanceDegradationConfig, ResourceConstraintConfig,
    ServiceFailureConfig,
};
use songbird_types::{SongbirdError, errors::SongbirdResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime};
use tokio::time::sleep;

/// Global chaos engineering manager
static CHAOS_MANAGER: std::sync::LazyLock<ChaosEngineeringManager> =
    std::sync::LazyLock::new(ChaosEngineeringManager::new);

/// Chaos engineering manager for coordinating fault injection experiments
#[derive(Debug)]
pub struct ChaosEngineeringManager {
    /// Active experiments
    experiments: Arc<RwLock<HashMap<String, ChaosExperiment>>>,
    /// Fault injection state
    #[allow(dead_code)]
    faults: Arc<RwLock<HashMap<String, FaultInjection>>>,
    /// Metrics collection
    #[allow(dead_code)]
    metrics: Arc<RwLock<Vec<MetricSnapshot>>>,
}

/// Fault injection state
#[derive(Debug, Clone)]
pub struct FaultInjection {
    /// Fault type
    pub fault_type: String,
    /// Target component
    pub target: String,
    /// Fault configuration
    pub config: serde_json::Value,
    /// Start time
    pub start_time: Instant,
    /// Duration
    pub duration: Duration,
    /// Active status
    pub active: bool,
}

impl ChaosEngineeringManager {
    /// Create a new chaos engineering manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            experiments: Arc::new(RwLock::new(HashMap::new())),
            faults: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get the global chaos manager instance
    #[must_use]
    pub fn global() -> &'static Self {
        &CHAOS_MANAGER
    }

    /// Start a chaos experiment
    ///
    /// # Errors
    /// Returns an error if the experiment cannot be started or if there are configuration issues.
    pub async fn start_experiment(
        &self,
        mut experiment: ChaosExperiment,
    ) -> SongbirdResult<String> {
        // Set experiment start time
        experiment.start_time = Some(SystemTime::now());
        experiment.status = ExperimentStatus::Running;

        let experiment_id = experiment.id.clone();

        // Store experiment
        {
            let mut experiments = self.experiments.write().map_err(|e| {
                SongbirdError::service("test-utils", format!("Failed to acquire write lock: {e}"))
            })?;
            experiments.insert(experiment_id.clone(), experiment.clone());
        }

        // Start fault injection based on experiment type
        match experiment.experiment_type {
            ExperimentType::NetworkFault => {
                if let Some(config) = &experiment.config.network_fault {
                    self.inject_network_fault(&experiment_id, config).await?;
                }
            }
            ExperimentType::ServiceFailure => {
                if let Some(config) = &experiment.config.service_failure {
                    self.inject_service_failure(&experiment_id, config).await?;
                }
            }
            ExperimentType::ResourceConstraint => {
                if let Some(config) = &experiment.config.resource_constraint {
                    self.inject_resource_constraint(&experiment_id, config).await?;
                }
            }
            ExperimentType::ByzantineFailure => {
                if let Some(config) = &experiment.config.byzantine_failure {
                    self.inject_byzantine_failure(&experiment_id, config).await?;
                }
            }
            ExperimentType::PerformanceDegradation => {
                if let Some(config) = &experiment.config.performance_degradation {
                    self.inject_performance_degradation(&experiment_id, config).await?;
                }
            }
            _ => {
                return Err(SongbirdError::service(
                    "test-utils",
                    format!("Experiment type {:?} not yet implemented", experiment.experiment_type),
                ));
            }
        }

        Ok(experiment_id)
    }

    /// Stop a chaos experiment
    ///
    /// # Errors
    /// Returns an error if the experiment cannot be stopped or is not found.
    pub async fn stop_experiment(&self, experiment_id: &str) -> SongbirdResult<()> {
        // Update experiment status
        {
            let mut experiments = self.experiments.write().map_err(|e| {
                SongbirdError::service("test-utils", format!("Failed to acquire write lock: {e}"))
            })?;

            if let Some(experiment) = experiments.get_mut(experiment_id) {
                experiment.status = ExperimentStatus::Stopped;
                experiment.end_time = Some(SystemTime::now());
            } else {
                return Err(SongbirdError::service(
                    "test-utils",
                    format!("Experiment {experiment_id} not found"),
                ));
            }
        }

        // Stop fault injection
        self.stop_fault_injection(experiment_id).await?;

        Ok(())
    }

    /// Get experiment status
    ///
    /// # Errors
    /// Returns an error if the experiment is not found.
    #[allow(clippy::unused_async)]
    pub async fn get_experiment_status(
        &self,
        experiment_id: &str,
    ) -> SongbirdResult<ChaosExperiment> {
        let experiments = self.experiments.read().map_err(|e| {
            SongbirdError::service("test-utils", format!("Failed to acquire read lock: {e}"))
        })?;

        experiments.get(experiment_id).cloned().ok_or_else(|| {
            SongbirdError::service("test-utils", format!("Experiment {experiment_id} not found"))
        })
    }

    /// List all active experiments
    ///
    /// # Errors
    ///
    /// Returns an error if the read lock cannot be acquired.
    #[allow(clippy::unused_async)]
    pub async fn list_experiments(&self) -> SongbirdResult<Vec<ChaosExperiment>> {
        let experiments = self.experiments.read().map_err(|e| {
            SongbirdError::service("test-utils", format!("Failed to acquire read lock: {e}"))
        })?;

        Ok(experiments.values().cloned().collect())
    }

    // Private fault injection methods...
    #[allow(clippy::unused_async)]
    async fn inject_network_fault(
        &self,
        experiment_id: &str,
        config: &NetworkFaultConfig,
    ) -> SongbirdResult<()> {
        tracing::info!("Injecting network fault for experiment: {}", experiment_id);

        // Simulate network latency if configured
        if let Some(latency_ms) = config.latency_ms {
            tokio::spawn(async move {
                loop {
                    sleep(Duration::from_millis(latency_ms)).await;
                    // In production, this would inject actual network delays
                }
            });
        }

        Ok(())
    }

    #[allow(clippy::unused_async)]
    async fn inject_service_failure(
        &self,
        _experiment_id: &str,
        config: &ServiceFailureConfig,
    ) -> SongbirdResult<()> {
        if config.failure_rate > 0.0 {
            tracing::info!("Injecting service errors with rate: {}", config.failure_rate);
        }
        Ok(())
    }

    #[allow(clippy::unused_async)]
    async fn inject_resource_constraint(
        &self,
        experiment_id: &str,
        _config: &ResourceConstraintConfig,
    ) -> SongbirdResult<()> {
        tracing::info!("Injecting resource constraints for experiment: {}", experiment_id);
        Ok(())
    }

    #[allow(clippy::unused_async)]
    async fn inject_byzantine_failure(
        &self,
        experiment_id: &str,
        _config: &ByzantineFailureConfig,
    ) -> SongbirdResult<()> {
        tracing::info!("Injecting byzantine failures for experiment: {}", experiment_id);
        Ok(())
    }

    #[allow(clippy::unused_async)]
    async fn inject_performance_degradation(
        &self,
        experiment_id: &str,
        _config: &PerformanceDegradationConfig,
    ) -> SongbirdResult<()> {
        tracing::info!("Injecting performance degradation for experiment: {}", experiment_id);
        Ok(())
    }

    #[allow(clippy::unused_async)]
    async fn stop_fault_injection(&self, experiment_id: &str) -> SongbirdResult<()> {
        tracing::info!("Stopped fault injection for experiment: {}", experiment_id);
        Ok(())
    }
}

impl Default for ChaosEngineeringManager {
    fn default() -> Self {
        Self::new()
    }
}
