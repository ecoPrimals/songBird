// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use std::time::Duration;
use tracing::{error, info, warn};

// Server Management Module
//
// Provides server management functionality for the Songbird Orchestrator application)
// including health monitoring, status tracking, and server lifecycle management.

use crate::app::{OrchestratorStatus, SongbirdOrchestrator};
use anyhow::Result;
use songbird_types::SafeEnv;
use tokio::time::interval;

// Server API modules
pub mod chunked_upload;
pub mod compute_api;
pub mod consent_api; // ✅ NEW: Consent Management API (Dec 18, 2025 - Week 5 MVP)
pub mod deployment_api;
pub mod events; // ✅ NEW: Real-Time Event Broadcasting (Nov 11, 2025 - Phase 4)
pub mod execution_api;
pub mod federation; // ✅ REFACTORED: Domain-driven module (Jan 21, 2026 - 971 → 4 files)
pub mod intelligent_protocol_router; // ✅ NEW: Intelligent Protocol Selection (Dec 18, 2025)
pub mod jsonrpc_api; // ✅ NEW: JSON-RPC 2.0 Universal Gateway (Nov 11, 2025)
pub mod protocol_api; // ✅ NEW: Progressive Protocol Enhancement API (Nov 11, 2025)
pub mod service_registry_api; // ✅ NEW: Universal Port Authority API (Dec 20, 2025) - Inter-primal registration
pub mod tarpc_server; // ✅ NEW: tarpc High-Performance Native RPC (Nov 11, 2025 - Phase 3)
pub mod task_api; // ✅ NEW: Task Lifecycle API (Dec 18, 2025 - Week 1 MVP)
pub mod websocket_api; // ✅ NEW: WebSocket Real-Time API (Nov 11, 2025 - Phase 4)
/// Server management and monitoring functionality
pub struct ServerManager {
    health_check_interval: Duration,
    start_time: std::time::Instant,
}

impl ServerManager {
    /// Create new server manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            health_check_interval:
                songbird_types::defaults::timeouts::DEFAULT_HEALTH_CHECK_INTERVAL,
            start_time: std::time::Instant::now(),
        }
    }

    /// Set health check interval
    #[must_use]
    pub const fn with_health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = interval;
        self
    }

    /// Start server monitoring
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn start_monitoring(&self, orchestrator: &SongbirdOrchestrator) -> Result<()> {
        info!("🔍 Server monitoring initialized");

        // Perform initial health check
        let health_results = self.run_comprehensive_health_check(orchestrator).await;

        // Log health check results
        for (component, healthy) in health_results {
            if healthy {
                info!("✅ {component} health check: HEALTHY");
            } else {
                warn!("❌ {component} health check: UNHEALTHY");
            }
        }

        info!("Server monitoring started successfully");
        Ok(())
    }

    /// Get server status
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn get_server_status(
        &self,
        orchestrator: &SongbirdOrchestrator,
    ) -> Result<ServerStatus> {
        let status = orchestrator.get_status().await?;
        Ok(ServerStatus {
            orchestrator: status,
            uptime: self.start_time.elapsed().as_secs(),
            health_check_interval: self.health_check_interval,
        })
    }

    /// Run comprehensive health check for all orchestrator components
    async fn run_comprehensive_health_check(
        &self,
        orchestrator: &SongbirdOrchestrator,
    ) -> Vec<(String, bool)> {
        let mut health_results = Vec::new();

        // Check service registry health
        let service_registry_healthy = self.check_service_registry_health(orchestrator).await;
        health_results.push((String::from("Service Registry"), service_registry_healthy));

        // Check gaming manager health
        let gaming_healthy = self.check_gaming_manager_health(orchestrator).await;
        health_results.push((String::from("Gaming Manager"), gaming_healthy));

        // Check federation manager health
        let federation_healthy = self.check_federation_manager_health(orchestrator).await;
        health_results.push((String::from("Federation Manager"), federation_healthy));

        // Check observability manager health
        let observability_healthy = self.check_observability_manager_health(orchestrator).await;
        health_results.push((String::from("Observability Manager"), observability_healthy));

        // Check security integration health
        let security_healthy = self.check_security_integration_health(orchestrator).await;
        health_results.push((String::from("Security Integration"), security_healthy));

        health_results
    }

    /// Check service registry health
    async fn check_service_registry_health(&self, orchestrator: &SongbirdOrchestrator) -> bool {
        // Validate service registry is operational
        let service_registry = orchestrator.service_registry();

        // Check if we can retrieve service count (basic functionality test)
        let service_count = service_registry.get_all_services().await.len();
        if service_count > 0 {
            tracing::debug!(
                "Service registry responding to health check, {} services",
                service_count
            );
        } else {
            tracing::debug!("Service registry health check - no services registered");
        }
        true // Still healthy regardless of service count
    }

    /// Check gaming manager health
    async fn check_gaming_manager_health(&self, orchestrator: &SongbirdOrchestrator) -> bool {
        // Same signal as HTTP health: peers advertising gaming-related capabilities.
        match orchestrator.get_status().await {
            Ok(s) => {
                tracing::debug!(
                    gaming_active = s.gaming_active,
                    "Gaming manager health check (orchestrator status)"
                );
                true
            }
            Err(e) => {
                tracing::warn!("Gaming health check: get_status failed: {e}");
                false
            }
        }
    }

    /// Check federation manager health
    async fn check_federation_manager_health(&self, orchestrator: &SongbirdOrchestrator) -> bool {
        // Aligns with `app::health::get_status`: mesh coordinator + `FederationState` active nodes.
        match orchestrator.get_status().await {
            Ok(s) => {
                tracing::debug!(
                    federation_connected = s.federation_connected,
                    "Federation manager health check (orchestrator status)"
                );
                s.federation_connected
            }
            Err(e) => {
                tracing::warn!("Federation health check: get_status failed: {e}");
                false
            }
        }
    }

    /// Check observability manager health
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    async fn check_observability_manager_health(
        &self,
        _orchestrator: &SongbirdOrchestrator,
    ) -> bool {
        // Observability manager health validation
        // In a real implementation, this would check metrics collection
        tracing::debug!("Observability manager health check completed");
        true
    }

    /// Check security integration health via crypto-provider discovery
    async fn check_security_integration_health(&self, orchestrator: &SongbirdOrchestrator) -> bool {
        // Probe the orchestrator status first — if the core is unhealthy the
        // security layer cannot be considered healthy either.
        match orchestrator.get_status().await {
            Ok(status) => {
                tracing::debug!(
                    gaming = status.gaming_active,
                    federation = status.federation_connected,
                    "Security: orchestrator status retrieved"
                );
                // Additionally verify the crypto provider is discoverable.
                match crate::primal_discovery::discover_crypto_provider().await {
                    Ok(_socket) => {
                        tracing::debug!("Security: crypto provider discovered");
                        true
                    }
                    Err(e) => {
                        tracing::warn!("Security: crypto provider not available: {e}");
                        // Degraded but not fatal — report unhealthy so the
                        // health dashboard surfaces the gap.
                        false
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Security health check: get_status failed: {e}");
                false
            }
        }
    }
}

impl Default for ServerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive server status information
#[derive(Debug, Clone)]
pub struct ServerStatus {
    pub orchestrator: OrchestratorStatus,
    pub uptime: u64,
    pub health_check_interval: Duration,
}

/// Health check service
pub struct HealthCheckService {
    check_interval: Duration,
}

impl HealthCheckService {
    /// Create new health check service
    #[must_use]
    pub const fn new(check_interval: Duration) -> Self {
        Self {
            check_interval,
        }
    }

    /// Run health check on orchestrator
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn run_health_check(
        &self,
        orchestrator: &SongbirdOrchestrator,
    ) -> Result<HealthCheckResult> {
        let status = orchestrator.get_status().await?;

        // Determine overall health based on status
        let health = if status.gaming_active || status.federation_connected {
            HealthStatus::Healthy
        } else {
            HealthStatus::Warning
        };

        Ok(HealthCheckResult {
            status: health,
            gaming_active: status.gaming_active,
            federation_connected: status.federation_connected,
            active_sessions: status.active_sessions,
            total_players: status.total_players,
            timestamp: std::time::SystemTime::now(),
        })
    }

    /// Start continuous health checking
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    pub async fn start_continuous_health_check(
        &self,
        _orchestrator: &SongbirdOrchestrator,
    ) -> Result<()> {
        let mut health_interval = interval(self.check_interval);

        tokio::spawn(async move {
            loop {
                health_interval.tick().await;

                // Perform basic health check without orchestrator reference
                // In production, this would use a shared health state or service registry
                let health_status = Self::check_system_health().await;

                match health_status {
                    HealthStatus::Healthy => {
                        info!("🔍 Continuous health check completed - Status: Healthy");
                    }
                    HealthStatus::Warning => {
                        warn!("🔍 Continuous health check completed - Status: Warning");
                    }
                    HealthStatus::Critical => {
                        error!("🔍 Continuous health check completed - Status: Critical");
                    }
                }
            }
        });

        Ok(())
    }

    /// Check basic system health
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    async fn check_system_health() -> HealthStatus {
        // Basic system health check without orchestrator dependency
        // In production, this would check system resources, services, etc.
        HealthStatus::Healthy
    }
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub gaming_active: bool,
    pub federation_connected: bool,
    pub active_sessions: u32,
    pub total_players: u32,
    pub timestamp: std::time::SystemTime,
}

/// Health status enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

/// Service monitoring functionality
pub struct ServiceMonitor {
    check_interval: Duration,
}

impl ServiceMonitor {
    /// Create new service monitor
    #[must_use]
    pub const fn new(check_interval: Duration) -> Self {
        Self {
            check_interval,
        }
    }

    /// Start service monitoring
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    pub async fn start_monitoring(&self) -> Result<()> {
        let mut monitor_interval = interval(self.check_interval);

        tokio::spawn(async move {
            loop {
                monitor_interval.tick().await;

                // Monitor services
                let mut healthy_count = 0;
                let mut warning_count = 0;
                let mut critical_count = 0;
                let _total_services = 4; // registry, gaming, federation, security

                // Monitor service registry
                if Self::check_service_registry_health().await {
                    healthy_count += 1;
                } else {
                    critical_count += 1;
                }

                // Monitor gaming bridges
                if Self::check_gaming_bridges_health().await {
                    healthy_count += 1;
                } else {
                    warning_count += 1;
                }

                // Monitor federation connections
                if Self::check_federation_connections_health().await {
                    healthy_count += 1;
                } else {
                    warning_count += 1;
                }

                // Monitor security services
                if Self::check_security_services_health().await {
                    healthy_count += 1;
                } else {
                    critical_count += 1;
                }

                info!(
                    "📊 Service monitoring check completed - Healthy: {}, Warning: {}, Critical: {}",
                    healthy_count, warning_count, critical_count
                );
            }
        });

        Ok(())
    }

    /// Get service monitoring report
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn get_monitoring_report(&self) -> Result<ServiceMonitoringReport> {
        // Implement comprehensive monitoring report generation
        let mut healthy_services = 0;
        let mut warning_services = 0;
        let mut critical_services = 0;
        let total_services = 4; // registry, gaming, federation, security

        // Check service registry health
        if Self::check_service_registry_health().await {
            healthy_services += 1;
        } else {
            critical_services += 1;
        }

        // Check gaming bridges health
        if Self::check_gaming_bridges_health().await {
            healthy_services += 1;
        } else {
            warning_services += 1;
        }

        // Check federation connections health
        if Self::check_federation_connections_health().await {
            healthy_services += 1;
        } else {
            warning_services += 1;
        }

        // Check security services health
        if Self::check_security_services_health().await {
            healthy_services += 1;
        } else {
            critical_services += 1;
        }

        Ok(ServiceMonitoringReport {
            services_monitored: total_services,
            healthy_services,
            warning_services,
            critical_services,
            timestamp: std::time::SystemTime::now(),
        })
    }

    /// Check service registry health
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    async fn check_service_registry_health() -> bool {
        // In production, this would check actual service registry endpoints
        // For now, simulate with environment variable check
        SafeEnv::get_or_default("SONGBIRD_SERVICE_REGISTRY_ENABLED", "true") != "false"
    }

    /// Check gaming bridges health
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    async fn check_gaming_bridges_health() -> bool {
        // In production, this would check actual gaming bridge endpoints
        // For now, simulate with environment variable check
        SafeEnv::get_or_default("SONGBIRD_GAMING_ENABLED", "true") != "false"
    }

    /// Check federation connections health
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    async fn check_federation_connections_health() -> bool {
        let explicitly_disabled = songbird_process_env::var("SONGBIRD_FEDERATION_ENABLED")
            .or_else(|_| songbird_process_env::var("FEDERATION_ENABLED"))
            .is_ok_and(|v| matches!(v.to_lowercase().as_str(), "false" | "0" | "no" | "off"));
        !explicitly_disabled
    }

    /// Check security services health
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    async fn check_security_services_health() -> bool {
        // In production, this would check actual security service endpoints
        // For now, simulate with environment variable check
        SafeEnv::get_or_default("SONGBIRD_SECURITY_ENABLED", "true") != "false"
    }
}

/// Service monitoring report
#[derive(Debug, Clone)]
pub struct ServiceMonitoringReport {
    pub services_monitored: u32,
    pub healthy_services: u32,
    pub warning_services: u32,
    pub critical_services: u32,
    pub timestamp: std::time::SystemTime,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn server_manager_default_interval() {
        let mgr = ServerManager::new();
        assert_eq!(
            mgr.health_check_interval,
            songbird_types::defaults::timeouts::DEFAULT_HEALTH_CHECK_INTERVAL
        );
    }

    #[test]
    fn server_manager_custom_interval() {
        let mgr = ServerManager::new().with_health_check_interval(Duration::from_secs(60));
        assert_eq!(mgr.health_check_interval, Duration::from_secs(60));
    }

    #[test]
    fn server_manager_default_trait() {
        let mgr = ServerManager::default();
        assert_eq!(
            mgr.health_check_interval,
            songbird_types::defaults::timeouts::DEFAULT_HEALTH_CHECK_INTERVAL
        );
    }

    #[test]
    fn health_status_equality() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_eq!(HealthStatus::Warning, HealthStatus::Warning);
        assert_eq!(HealthStatus::Critical, HealthStatus::Critical);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Warning);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Critical);
    }

    #[test]
    fn health_check_service_creation() {
        let svc = HealthCheckService::new(Duration::from_secs(15));
        assert_eq!(svc.check_interval, Duration::from_secs(15));
    }

    #[test]
    fn service_monitor_creation() {
        let monitor = ServiceMonitor::new(Duration::from_secs(10));
        assert_eq!(monitor.check_interval, Duration::from_secs(10));
    }

    #[tokio::test]
    async fn service_monitor_report_all_healthy() {
        let _env_lock = crate::test_sync_env::env_lock();
        let monitor = ServiceMonitor::new(Duration::from_secs(5));
        let report = monitor.get_monitoring_report().await.unwrap();
        assert_eq!(report.services_monitored, 4);
        assert_eq!(report.healthy_services, 4);
        assert_eq!(report.warning_services, 0);
        assert_eq!(report.critical_services, 0);
    }

    #[tokio::test]
    async fn service_monitor_env_disables_service() {
        let _env_lock = crate::test_sync_env::env_lock();
        let _guard =
            songbird_process_env::ScopedEnv::new("SONGBIRD_SERVICE_REGISTRY_ENABLED", "false");
        let monitor = ServiceMonitor::new(Duration::from_secs(5));
        let report = monitor.get_monitoring_report().await.unwrap();
        assert_eq!(report.critical_services, 1);
        assert_eq!(report.healthy_services, 3);
    }

    #[tokio::test]
    async fn service_monitor_gaming_disabled() {
        let _env_lock = crate::test_sync_env::env_lock();
        let _guard = songbird_process_env::ScopedEnv::new("SONGBIRD_GAMING_ENABLED", "false");
        let monitor = ServiceMonitor::new(Duration::from_secs(5));
        let report = monitor.get_monitoring_report().await.unwrap();
        assert_eq!(report.warning_services, 1);
    }

    #[tokio::test]
    async fn service_monitor_federation_disabled() {
        let _env_lock = crate::test_sync_env::env_lock();
        let _guard = songbird_process_env::ScopedEnv::new("SONGBIRD_FEDERATION_ENABLED", "false");
        let monitor = ServiceMonitor::new(Duration::from_secs(5));
        let report = monitor.get_monitoring_report().await.unwrap();
        assert_eq!(report.warning_services, 1);
    }

    #[tokio::test]
    async fn service_monitor_security_disabled() {
        let _env_lock = crate::test_sync_env::env_lock();
        let _guard = songbird_process_env::ScopedEnv::new("SONGBIRD_SECURITY_ENABLED", "false");
        let monitor = ServiceMonitor::new(Duration::from_secs(5));
        let report = monitor.get_monitoring_report().await.unwrap();
        assert_eq!(report.critical_services, 1);
    }

    #[tokio::test]
    async fn check_system_health_returns_healthy() {
        let status = HealthCheckService::check_system_health().await;
        assert_eq!(status, HealthStatus::Healthy);
    }
}
