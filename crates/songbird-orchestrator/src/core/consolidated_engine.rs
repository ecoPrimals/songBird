// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `ConsolidatedOrchestrator` — wiring load balancer, performance, registry, and scaling.

use std::sync::Arc;

use songbird_types::SongbirdResult;

use super::biome::ServiceRegistry;
use super::consolidated_config::ConsolidatedOrchestratorConfig;
use super::load_balancer::LoadBalancer;
use super::orchestrator_health::{HealthStatus, OrchestratorHealth};
use super::performance::PerformanceMonitor;
use super::scaling::AutoScaler;

/// Consolidated orchestrator engine
///
/// **ZERO-COPY OPTIMIZATION** (Dec 8, 2025):
/// Config is wrapped in Arc to prevent expensive clones in hot paths.
/// This config is read-only after creation and shared across components.
#[derive(Debug)]
pub struct ConsolidatedOrchestrator {
    #[expect(dead_code, reason = "shared Arc config for future component reads")]
    config: Arc<ConsolidatedOrchestratorConfig>,
    load_balancer: LoadBalancer,
    performance_monitor: PerformanceMonitor,
    service_registry: ServiceRegistry,
    auto_scaler: AutoScaler,
}

impl ConsolidatedOrchestrator {
    /// Create new consolidated orchestrator
    ///
    /// **ZERO-COPY**: Config is wrapped in Arc and shared, not cloned.
    #[must_use]
    pub fn new(config: ConsolidatedOrchestratorConfig) -> Self {
        let config = Arc::new(config);
        Self {
            config: Arc::clone(&config),
            load_balancer: LoadBalancer::new(config.load_balancing.clone()),
            performance_monitor: PerformanceMonitor::new(config.performance.clone()),
            service_registry: ServiceRegistry::new(config.registry.clone()),
            auto_scaler: AutoScaler::new(config.scaling.clone()),
        }
    }

    /// Initialize the orchestrator
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn initialize(&mut self) -> SongbirdResult<()> {
        self.load_balancer.initialize().await?;
        self.performance_monitor.initialize().await?;
        self.service_registry.initialize().await?;
        self.auto_scaler.initialize().await?;
        Ok(())
    }

    /// Start orchestration
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn start(&mut self) -> SongbirdResult<()> {
        self.load_balancer.start().await?;
        self.performance_monitor.start().await?;
        self.service_registry.start().await?;
        self.auto_scaler.start().await?;
        Ok(())
    }

    /// Stop orchestration
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn stop(&mut self) -> SongbirdResult<()> {
        self.auto_scaler.stop().await?;
        self.service_registry.stop().await?;
        self.performance_monitor.stop().await?;
        self.load_balancer.stop().await?;
        Ok(())
    }

    /// Get orchestrator health status
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn health_check(&self) -> SongbirdResult<OrchestratorHealth> {
        Ok(OrchestratorHealth {
            status: HealthStatus::Healthy,
            load_balancer_health: self.load_balancer.health_check().await?,
            performance_health: self.performance_monitor.health_check().await?,
            registry_health: self.service_registry.health_check().await?,
            scaling_health: self.auto_scaler.health_check().await?,
        })
    }
}
