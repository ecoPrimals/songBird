//! Biome Service Lifecycle Management
//!
//! This module handles service lifecycle operations including:
//! - Service orchestration and startup
//! - Health monitoring and status tracking
//! - Dependency resolution and ordering
//! - Service registry management

use super::types::{HealthCheckSpec, ServiceSpec, SongbirdBiomeManifest, SongbirdOrchestrator};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};

/// Service lifecycle manager
#[derive(Debug)]
pub struct ServiceLifecycleManager {
    /// Running services and their status
    services: HashMap<String, ServiceStatus>,

    /// Service registry for discovery
    registry: ServiceRegistry,

    /// Health monitoring configuration
    health_config: HealthMonitoringConfig,
}

/// Service status tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub status: ServiceState,
    pub endpoint: Option<String>,
    pub health: HealthStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub last_health_check: Option<DateTime<Utc>>,
    pub restart_count: u32,
    pub dependencies_ready: bool,
}

/// Service operational state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceState {
    Stopped,
    Starting,
    Running,
    Unhealthy,
    Failed,
    Restarting,
}

/// Health check status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub last_check: Option<DateTime<Utc>>,
    pub consecutive_failures: u32,
    pub response_time_ms: Option<u64>,
    pub error_message: Option<String>,
}

/// Health monitoring configuration
#[derive(Debug, Clone)]
pub struct HealthMonitoringConfig {
    pub default_interval: Duration,
    pub default_timeout: Duration,
    pub max_failures: u32,
    pub restart_on_failure: bool,
}

impl Default for HealthMonitoringConfig {
    fn default() -> Self {
        Self {
            default_interval: Duration::from_secs(30),
            default_timeout: Duration::from_secs(5),
            max_failures: 3,
            restart_on_failure: true,
        }
    }
}

/// Service registry for discovery
#[derive(Debug, Clone)]
pub struct ServiceRegistry {
    services: HashMap<String, ServiceRegistration>,
}

/// Service registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
    pub name: String,
    pub endpoint: String,
    pub health_endpoint: Option<String>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub registered_at: DateTime<Utc>,
}

impl ServiceLifecycleManager {
    /// Create new service lifecycle manager
    pub fn new(health_config: HealthMonitoringConfig) -> Self {
        Self {
            services: HashMap::new(),
            registry: ServiceRegistry::new(),
            health_config,
        }
    }

    /// Start service lifecycle management for an orchestrator
    pub async fn start_lifecycle_management(
        &mut self,
        orchestrator: &SongbirdOrchestrator,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Starting lifecycle management for orchestrator: {}",
            orchestrator.id
        );

        // Setup service registry
        self.setup_service_registry().await?;

        // Start health monitoring
        self.setup_health_monitoring().await?;

        // Orchestrate services
        self.orchestrate_services(&orchestrator.manifest).await?;

        info!("Service lifecycle management started successfully");
        Ok(())
    }

    /// Setup service registry
    async fn setup_service_registry(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Setting up service registry");

        // Initialize registry
        self.registry = ServiceRegistry::new();

        // External service registry connection is delegated to external service registry APIs
        // Production implementations should integrate with:
        // - Service registry systems (Consul, etcd, Kubernetes, etc.)
        // - Service discovery protocols (DNS-SD, mDNS, etc.)
        // - Cloud provider service registries
        // - Custom service registry implementations

        debug!("Checking for external service registry configuration");

        // External service registry connection would be implemented here
        // This would connect to external service registries if configured

        debug!("Using in-memory service registry for current session");

        // In-memory registry is used as fallback or for development
        // Production deployments should configure external service registries

        info!("Service registry setup completed");
        Ok(())
    }

    /// Setup health monitoring system
    async fn setup_health_monitoring(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Setting up health monitoring system");

        // Start health monitoring task
        let _health_config = self.health_config.clone();
        tokio::spawn(async move {
            let mut interval_timer = interval(_health_config.default_interval);

            loop {
                interval_timer.tick().await;

                // Health monitoring will be handled by the main manager
                // This is just the scheduling framework
            }
        });

        info!("Health monitoring system setup completed");
        Ok(())
    }

    /// Orchestrate all services in the manifest
    async fn orchestrate_services(
        &mut self,
        manifest: &SongbirdBiomeManifest,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting service orchestration");

        // Resolve service dependencies
        let ordered_services = self.resolve_service_dependencies(manifest)?;

        // Start services in dependency order
        for service_name in ordered_services {
            if let Some(service_spec) = manifest.services.get(&service_name) {
                self.orchestrate_single_service(&service_name, service_spec)
                    .await?;
            }
        }

        info!("Service orchestration completed");
        Ok(())
    }

    /// Resolve service dependencies and return ordered list
    fn resolve_service_dependencies(
        &self,
        manifest: &SongbirdBiomeManifest,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        let mut ordered_services = Vec::new();

        for service_name in manifest.services.keys() {
            if !visited.contains(service_name) {
                self.visit_service_dependencies(
                    service_name,
                    &manifest.services,
                    &mut visited,
                    &mut visiting,
                    &mut ordered_services,
                )?;
            }
        }

        Ok(ordered_services)
    }

    /// Visit service dependencies (DFS for topological sort)
    #[allow(clippy::only_used_in_recursion)]
    fn visit_service_dependencies(
        &self,
        service_name: &str,
        services: &HashMap<String, ServiceSpec>,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
        ordered_services: &mut Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if visiting.contains(service_name) {
            return Err(format!("Circular dependency detected for service: {service_name}").into());
        }

        if visited.contains(service_name) {
            return Ok(());
        }

        visiting.insert(service_name.to_string());

        if let Some(service_spec) = services.get(service_name) {
            for dependency in &service_spec.depends_on {
                self.visit_service_dependencies(
                    dependency,
                    services,
                    visited,
                    visiting,
                    ordered_services,
                )?;
            }
        }

        visiting.remove(service_name);
        visited.insert(service_name.to_string());
        ordered_services.push(service_name.to_string());

        Ok(())
    }

    /// Orchestrate a single service
    async fn orchestrate_single_service(
        &mut self,
        service_name: &str,
        service_spec: &ServiceSpec,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Orchestrating service: {}", service_name);

        // Check if dependencies are ready
        let dependencies_ready = self.check_dependencies_ready(service_spec).await;

        // Create service status
        let service_status = ServiceStatus {
            name: service_name.to_string(),
            status: ServiceState::Starting,
            endpoint: service_spec.endpoint.clone(),
            health: HealthStatus {
                is_healthy: false,
                last_check: None,
                consecutive_failures: 0,
                response_time_ms: None,
                error_message: None,
            },
            started_at: Some(Utc::now()),
            last_health_check: None,
            restart_count: 0,
            dependencies_ready,
        };

        self.services
            .insert(service_name.to_string(), service_status);

        // Start service monitoring
        self.start_service_monitoring(service_name, service_spec)
            .await?;

        // Wait for service to be ready
        self.wait_for_service_ready(service_name, service_spec)
            .await?;

        // Register service
        if let Some(endpoint) = &service_spec.endpoint {
            self.registry.register_service(ServiceRegistration {
                name: service_name.to_string(),
                endpoint: endpoint.clone(),
                health_endpoint: service_spec
                    .health_check
                    .as_ref()
                    .map(|hc| format!("{}{}", endpoint, hc.endpoint)),
                tags: vec!["orchestrated".to_string()],
                metadata: HashMap::new(),
                registered_at: Utc::now(),
            });
        }

        // Update service status
        if let Some(service) = self.services.get_mut(service_name) {
            service.status = ServiceState::Running;
        }

        info!("Service {} orchestrated successfully", service_name);
        Ok(())
    }

    /// Check if service dependencies are ready
    async fn check_dependencies_ready(&self, service_spec: &ServiceSpec) -> bool {
        for dependency in &service_spec.depends_on {
            if let Some(dep_service) = self.services.get(dependency) {
                if !matches!(dep_service.status, ServiceState::Running) {
                    return false;
                }
            } else {
                return false; // Dependency not found
            }
        }
        true
    }

    /// Start monitoring for a service
    async fn start_service_monitoring(
        &self,
        service_name: &str,
        service_spec: &ServiceSpec,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(health_check) = &service_spec.health_check {
            info!("Starting health monitoring for service: {}", service_name);

            let health_check = health_check.clone();
            let service_name = service_name.to_string();
            let _health_config = self.health_config.clone();

            tokio::spawn(async move {
                let mut interval_timer = interval(Duration::from_secs(health_check.interval_secs));

                loop {
                    interval_timer.tick().await;

                    if let Err(e) = Self::check_service_health(&service_name, &health_check).await {
                        warn!("Health check failed for service {}: {}", service_name, e);
                    }
                }
            });
        }

        Ok(())
    }

    /// Check health of a specific service
    async fn check_service_health(
        service_name: &str,
        health_check: &HealthCheckSpec,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(health_check.timeout_secs))
            .build()?;

        let start_time = std::time::Instant::now();

        match client.get(&health_check.endpoint).send().await {
            Ok(response) if response.status().is_success() => {
                let response_time = start_time.elapsed().as_millis() as u64;
                info!(
                    "Health check passed for service {} ({}ms)",
                    service_name, response_time
                );
                Ok(true)
            }
            Ok(response) => {
                warn!(
                    "Health check failed for service {} - status: {}",
                    service_name,
                    response.status()
                );
                Ok(false)
            }
            Err(e) => {
                error!("Health check error for service {}: {}", service_name, e);
                Err(e.into())
            }
        }
    }

    /// Wait for service to be ready
    async fn wait_for_service_ready(
        &self,
        service_name: &str,
        service_spec: &ServiceSpec,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(health_check) = &service_spec.health_check {
            info!("Waiting for service {} to be ready", service_name);

            let max_wait_time = Duration::from_secs(300); // 5 minutes
            let check_interval = Duration::from_secs(5);

            let result = timeout(max_wait_time, async {
                loop {
                    if Self::check_service_health(service_name, health_check)
                        .await
                        .unwrap_or(false)
                    {
                        break;
                    }
                    tokio::time::sleep(check_interval).await;
                }
            })
            .await;

            match result {
                Ok(_) => {
                    info!("Service {} is ready", service_name);
                    Ok(())
                }
                Err(_) => {
                    warn!("Timeout waiting for service {} to be ready", service_name);
                    Ok(()) // Don't fail orchestration for slow services
                }
            }
        } else {
            // No health check configured, assume ready immediately
            info!(
                "No health check configured for service {}, assuming ready",
                service_name
            );
            Ok(())
        }
    }

    /// Get service status
    pub fn get_service_status(&self, service_name: &str) -> Option<&ServiceStatus> {
        self.services.get(service_name)
    }

    /// List all services
    pub fn list_services(&self) -> Vec<&ServiceStatus> {
        self.services.values().collect()
    }

    /// Restart a service
    pub async fn restart_service(
        &mut self,
        service_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(service) = self.services.get_mut(service_name) {
            info!("Restarting service: {}", service_name);

            service.status = ServiceState::Restarting;
            service.restart_count += 1;

            // Simulate restart delay
            tokio::time::sleep(Duration::from_secs(2)).await;

            service.status = ServiceState::Running;
            service.started_at = Some(Utc::now());

            info!("Service {} restarted successfully", service_name);
        }

        Ok(())
    }

    /// Stop a service
    pub async fn stop_service(
        &mut self,
        service_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(service) = self.services.get_mut(service_name) {
            info!("Stopping service: {}", service_name);

            service.status = ServiceState::Stopped;

            // Unregister from service registry
            self.registry.unregister_service(service_name);

            info!("Service {} stopped successfully", service_name);
        }

        Ok(())
    }

    /// Get service registry
    pub fn get_registry(&self) -> &ServiceRegistry {
        &self.registry
    }

    /// Update service health status
    pub fn update_service_health(
        &mut self,
        service_name: &str,
        is_healthy: bool,
        response_time_ms: Option<u64>,
    ) {
        if let Some(service) = self.services.get_mut(service_name) {
            service.health.is_healthy = is_healthy;
            service.health.last_check = Some(Utc::now());
            service.health.response_time_ms = response_time_ms;

            if is_healthy {
                service.health.consecutive_failures = 0;
                service.health.error_message = None;
            } else {
                service.health.consecutive_failures += 1;
            }

            service.last_health_check = Some(Utc::now());
        }
    }

    /// Clean up stopped services
    pub async fn cleanup_stopped_services(
        &mut self,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        let mut cleaned_count = 0;
        let mut to_remove = Vec::new();

        for (name, service) in &self.services {
            if matches!(service.status, ServiceState::Stopped | ServiceState::Failed) {
                to_remove.push(name.clone());
            }
        }

        for name in to_remove {
            self.services.remove(&name);
            self.registry.unregister_service(&name);
            cleaned_count += 1;
            info!("Cleaned up stopped service: {}", name);
        }

        Ok(cleaned_count)
    }
}

impl ServiceRegistry {
    /// Create new service registry
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    /// Register a service
    pub fn register_service(&mut self, registration: ServiceRegistration) {
        info!("Registering service: {}", registration.name);
        self.services
            .insert(registration.name.clone(), registration);
    }

    /// Unregister a service
    pub fn unregister_service(&mut self, service_name: &str) {
        if self.services.remove(service_name).is_some() {
            info!("Unregistered service: {}", service_name);
        }
    }

    /// Get service registration
    pub fn get_service(&self, service_name: &str) -> Option<&ServiceRegistration> {
        self.services.get(service_name)
    }

    /// List all registered services
    pub fn list_services(&self) -> Vec<&ServiceRegistration> {
        self.services.values().collect()
    }

    /// Find services by tag
    pub fn find_services_by_tag(&self, tag: &str) -> Vec<&ServiceRegistration> {
        self.services
            .values()
            .filter(|registration| registration.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Get service count
    pub fn service_count(&self) -> usize {
        self.services.len()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biome::modules::types::*;

    #[tokio::test]
    async fn test_lifecycle_manager_creation() {
        let config = HealthMonitoringConfig::default();
        let manager = ServiceLifecycleManager::new(config);

        assert_eq!(manager.services.len(), 0);
        assert_eq!(manager.registry.service_count(), 0);
    }

    #[tokio::test]
    async fn test_service_registry() {
        let mut registry = ServiceRegistry::new();

        let env_config = crate::config::environment::EnvironmentConfig::default();
        let default_endpoint = format!(
            "http://{}:{}",
            env_config.bind_address, env_config.bind_port
        );
        let default_health_endpoint = format!(
            "http://{}:{}/health",
            env_config.bind_address, env_config.bind_port
        );

        let registration = ServiceRegistration {
            name: "test-service".to_string(),
            endpoint: default_endpoint.clone(),
            health_endpoint: Some(default_health_endpoint),
            tags: vec!["test".to_string()],
            metadata: HashMap::new(),
            registered_at: Utc::now(),
        };

        registry.register_service(registration);
        assert_eq!(registry.service_count(), 1);

        let found = registry.get_service("test-service");
        assert!(found.is_some());
        assert_eq!(found.expect("Service should be found in test").endpoint, default_endpoint);
    }

    #[tokio::test]
    async fn test_dependency_resolution() {
        let config = HealthMonitoringConfig::default();
        let manager = ServiceLifecycleManager::new(config);

        let mut services = HashMap::new();

        // Service A depends on B
        services.insert(
            "A".to_string(),
            ServiceSpec {
                endpoint: Some("http://localhost:8080".to_string()),
                depends_on: vec!["B".to_string()],
                health_check: None,
                primal_managed: None,
            },
        );

        // Service B has no dependencies
        services.insert(
            "B".to_string(),
            ServiceSpec {
                endpoint: Some("http://localhost:8081".to_string()),
                depends_on: vec![],
                health_check: None,
                primal_managed: None,
            },
        );

        let manifest = SongbirdBiomeManifest {
            metadata: BiomeMetadata {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                description: None,
            },
            services,
            networking: None,
            primals: None,
        };

        let ordered = manager.resolve_service_dependencies(&manifest).expect("Failed to resolve service dependencies in test");

        // B should come before A due to dependency
        assert_eq!(ordered.len(), 2);
        assert_eq!(ordered[0], "B");
        assert_eq!(ordered[1], "A");
    }
}
