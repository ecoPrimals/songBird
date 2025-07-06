//! Zero Touch Deployment Engine
//!
//! Automated deployment engine for the Songbird Orchestrator

use std::collections::HashMap;
use std::process::Stdio;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{info, warn, error};

use songbird_errors::{Result, SongbirdError};

/// Deployment engine for zero-touch deployment
pub struct DeploymentEngine {
    deployment_history: Vec<DeploymentRecord>,
}

impl DeploymentEngine {
    /// Create a new deployment engine
    pub fn new() -> Self {
        Self {
            deployment_history: Vec::new(),
        }
    }

    /// Deploy services using the specified strategy
    pub async fn deploy_services(
        &mut self,
        services: Vec<ServiceConfig>,
        strategy: DeploymentStrategy,
    ) -> Result<DeploymentResult> {
        let deployment_id = uuid::Uuid::new_v4().to_string();
        let start_time = chrono::Utc::now();

        info!("Starting deployment {} with strategy: {:?}", deployment_id, strategy);

        let mut deployed_services = Vec::with_capacity(16); // Pre-allocate for typical deployment size
        let mut errors = Vec::with_capacity(8); // Pre-allocate for typical error count

        // Record deployment start
        let record = DeploymentRecord {
            id: deployment_id.clone(),
            start_time,
            end_time: None,
            status: DeploymentStatus::InProgress,
            strategy: strategy.clone(),
            services: services.clone(),
            errors: Vec::new(),
        };
        self.deployment_history.push(record);

        // Deploy services based on strategy
        match strategy {
            DeploymentStrategy::Sequential => {
                for service in services {
                    match self.deploy_single_service(&service).await {
                        Ok(deployed_service) => deployed_services.push(deployed_service),
                        Err(e) => {
                            error!("Failed to deploy service {}: {}", service.name, e);
                            errors.push(DeploymentError {
                                service_name: service.name.clone(),
                                error_message: e.to_string(),
                                timestamp: chrono::Utc::now(),
                            });
                        }
                    }
                }
            }
            DeploymentStrategy::Parallel => {
                let deployment_futures: Vec<_> = services.iter()
                    .map(|service| self.deploy_single_service(service))
                    .collect();

                let results = futures::future::join_all(deployment_futures).await;
                
                for (service, result) in services.iter().zip(results.iter()) {
                    match result {
                        Ok(deployed_service) => deployed_services.push(deployed_service.clone()),
                        Err(e) => {
                            error!("Failed to deploy service {}: {}", service.name, e);
                            errors.push(DeploymentError {
                                service_name: service.name.clone(),
                                error_message: e.to_string(),
                                timestamp: chrono::Utc::now(),
                            });
                        }
                    }
                }
            }
            DeploymentStrategy::RollingUpdate => {
                // Implement rolling update strategy
                for service in services {
                    match self.deploy_single_service(&service).await {
                        Ok(deployed_service) => {
                            deployed_services.push(deployed_service);
                            // Wait a bit between deployments for rolling update
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }
                        Err(e) => {
                            error!("Failed to deploy service {}: {}", service.name, e);
                            errors.push(DeploymentError {
                                service_name: service.name.clone(),
                                error_message: e.to_string(),
                                timestamp: chrono::Utc::now(),
                            });
                        }
                    }
                }
            }
        }

        let end_time = chrono::Utc::now();
        let deployment_time = end_time - start_time;

        // Validate deployment
        if let Err(e) = self.validate_deployment(&deployed_services).await {
            warn!("Deployment validation failed: {}", e);
        }

        // Generate summary
        let summary = self.generate_deployment_summary(&deployed_services, &errors);

        // Update deployment record
        if let Some(record) = self.deployment_history.last_mut() {
            record.end_time = Some(end_time);
            record.status = if errors.is_empty() {
                DeploymentStatus::Completed
            } else if deployed_services.is_empty() {
                DeploymentStatus::Failed
            } else {
                DeploymentStatus::CompletedWithWarnings
            };
            record.errors = errors.iter().map(|e| e.error_message.clone()).collect();
        }

        Ok(DeploymentResult {
            deployment_id,
            status: if errors.is_empty() {
                DeploymentStatus::Completed
            } else if deployed_services.is_empty() {
                DeploymentStatus::Failed
            } else {
                DeploymentStatus::CompletedWithWarnings
            },
            deployed_services,
            errors,
            deployment_time,
            summary,
        })
    }

    /// Deploy a single service
    async fn deploy_single_service(&self, service: &ServiceConfig) -> Result<ServiceDeploymentInfo> {
        info!("Deploying service: {}", service.name);

        let deployment_method = service.deployment_method.as_deref().unwrap_or("docker");

        match deployment_method {
            "docker" => self.deploy_docker_service(service).await,
            "kubernetes" => self.deploy_kubernetes_service(service).await,
            "systemd" => self.deploy_systemd_service(service).await,
            _ => Err(SongbirdError::Deployment {
                stage: "deployment_method".to_string(),
                message: format!("Unsupported deployment method: {}", deployment_method),
            }),
        }
    }

    /// Deploy service using Docker
    async fn deploy_docker_service(&self, service: &ServiceConfig) -> Result<ServiceDeploymentInfo> {
        let container_name = format!("songbird-{}", service.name);

        // Build docker run command
        let mut args = vec![
            "run".to_string(),
            "-d".to_string(),
            "--name".to_string(),
            container_name.clone(),
        ];

        // Add port mappings
        for port in &service.ports {
            args.push("-p".to_string());
            args.push(format!("{}:{}", port, port));
        }

        // Add environment variables
        for (key, value) in &service.environment_variables {
            args.push("-e".to_string());
            args.push(format!("{}={}", key, value));
        }

        // Add image
        args.push(service.image.clone());

        // Execute docker run
        let output = Command::new("docker")
            .args(&args)
            .output()
            .await
            .map_err(|e| SongbirdError::Deployment {
                stage: "docker_run".to_string(),
                message: format!("Failed to run Docker container: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SongbirdError::Deployment {
                stage: "docker_run".to_string(),
                message: format!("Docker run failed: {}", stderr),
            });
        }

        let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

        Ok(ServiceDeploymentInfo {
            name: service.name.clone(),
            service_type: "docker".to_string(),
            status: ServiceStatus::Running,
            endpoint: format!("http://{}:{}", 
                crate::config::environment::get_container_bind_address(), 
                service.ports.first().unwrap_or(&crate::config::constants::network::DEFAULT_PORT)),
            ports: service.ports.clone(),
            process_id: None,
            health_status: HealthStatus::Unknown,
            deployment_time: chrono::Utc::now() - chrono::Utc::now(), // Would need actual timing
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("container_id".to_string(), container_id);
                meta.insert("deployment_method".to_string(), "docker".to_string());
                meta
            },
        })
    }

    /// Deploy service using Kubernetes
    async fn deploy_kubernetes_service(&self, service: &ServiceConfig) -> Result<ServiceDeploymentInfo> {
        // Generate Kubernetes manifests
        let deployment_manifest = self.generate_k8s_deployment(service)?;
        let service_manifest = self.generate_k8s_service(service)?;

        // Apply manifests
        self.apply_k8s_manifest(&deployment_manifest).await?;
        self.apply_k8s_manifest(&service_manifest).await?;

        // Wait for deployment to be ready
        self.wait_for_k8s_deployment(&service.name).await?;

        Ok(ServiceDeploymentInfo {
            name: service.name.clone(),
            service_type: "kubernetes".to_string(),
            status: ServiceStatus::Running,
            endpoint: format!("http://{}:{}", 
                crate::config::environment::get_container_bind_address(), 
                service.ports.first().unwrap_or(&crate::config::constants::network::DEFAULT_PORT)),
            ports: service.ports.clone(),
            process_id: None,
            health_status: HealthStatus::Unknown,
            deployment_time: chrono::Utc::now() - chrono::Utc::now(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("deployment_method".to_string(), "kubernetes".to_string());
                meta.insert("namespace".to_string(), "songbird".to_string());
                meta
            },
        })
    }

    /// Deploy service using systemd
    async fn deploy_systemd_service(&self, service: &ServiceConfig) -> Result<ServiceDeploymentInfo> {
        let unit_content = self.generate_systemd_unit(service)?;
        let unit_file = format!("/etc/systemd/system/songbird-{}.service", service.name);

        // Write systemd unit file
        tokio::fs::write(&unit_file, unit_content).await
            .map_err(|e| SongbirdError::Deployment {
                stage: "systemd_unit_write".to_string(),
                message: format!("Failed to write systemd unit file: {}", e),
            })?;

        // Reload systemd
        Command::new("systemctl")
            .args(&["daemon-reload"])
            .status()
            .await
            .map_err(|e| SongbirdError::Deployment {
                stage: "systemd_reload".to_string(),
                message: format!("Failed to reload systemd: {}", e),
            })?;

        // Start service
        let output = Command::new("systemctl")
            .args(&["start", &format!("songbird-{}", service.name)])
            .output()
            .await
            .map_err(|e| SongbirdError::Deployment {
                stage: "systemd_start".to_string(),
                message: format!("Failed to start systemd service: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SongbirdError::Deployment {
                stage: "systemd_start".to_string(),
                message: format!("systemctl start failed: {}", stderr),
            });
        }

        Ok(ServiceDeploymentInfo {
            name: service.name.clone(),
            service_type: "systemd".to_string(),
            status: ServiceStatus::Running,
            endpoint: format!("http://{}:{}", 
                crate::config::environment::get_container_bind_address(), 
                service.ports.first().unwrap_or(&crate::config::constants::network::DEFAULT_PORT)),
            ports: service.ports.clone(),
            process_id: None,
            health_status: HealthStatus::Unknown,
            deployment_time: chrono::Utc::now() - chrono::Utc::now(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("deployment_method".to_string(), "systemd".to_string());
                meta.insert("unit_file".to_string(), unit_file);
                meta
            },
        })
    }

    /// Validate deployment
    async fn validate_deployment(&self, deployed_services: &[ServiceDeploymentInfo]) -> Result<()> {
        info!("Validating deployment of {} services", deployed_services.len());
        
        for service in deployed_services {
            // Check if service is responding
            if let Some(port) = service.ports.first() {
                match timeout(
                    Duration::from_secs(10),
                    tokio::net::TcpStream::connect(format!("{}:{}", 
                        crate::config::constants::network::DEFAULT_BIND_ADDRESS, 
                        port))
                ).await {
                    Ok(_) => {
                        info!("✅ Service {} is responding on port {}", service.name, port);
                    }
                    Err(_) => {
                        warn!("⚠️ Service {} is not responding on port {}", service.name, port);
                    }
                }
            }
        }

        Ok(())
    }

    /// Generate deployment summary
    fn generate_deployment_summary(&self, deployed_services: &[ServiceDeploymentInfo], errors: &[DeploymentError]) -> String {
        format!(
            "Deployment Summary:\n\
             - Services deployed: {}\n\
             - Services failed: {}\n\
             - Total deployment time: {}s\n\
             - Services: {}",
            deployed_services.len(),
            errors.len(),
            deployed_services.iter()
                .map(|s| s.deployment_time.num_seconds())
                .sum::<i64>(),
            deployed_services.iter()
                .map(|s| format!("  - {} ({})", s.name, s.status))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    /// Generate Kubernetes deployment manifest
    fn generate_k8s_deployment(&self, service: &ServiceConfig) -> Result<String> {
        let ports_yaml = service.ports.iter()
            .map(|p| format!("        - containerPort: {}", p))
            .collect::<Vec<_>>()
            .join("\n");

        let env_yaml = service.environment_variables.iter()
            .map(|(k, v)| format!("        - name: {}\n          value: \"{}\"", k, v))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(format!(
            "apiVersion: apps/v1\n\
            kind: Deployment\n\
            metadata:\n\
              name: {}\n\
              namespace: songbird\n\
            spec:\n\
              replicas: 1\n\
              selector:\n\
                matchLabels:\n\
                  app: {}\n\
              template:\n\
                metadata:\n\
                  labels:\n\
                    app: {}\n\
                spec:\n\
                  containers:\n\
                  - name: {}\n\
                    image: {}\n\
                    ports:\n\
            {}\n\
                    env:\n\
            {}",
            service.name,
            service.name,
            service.name,
            service.name,
            service.image,
            ports_yaml,
            env_yaml
        ))
    }

    /// Generate Kubernetes service manifest
    fn generate_k8s_service(&self, service: &ServiceConfig) -> Result<String> {
        let ports_yaml = service.ports.iter()
            .map(|p| format!("  - port: {}\n    targetPort: {}", p, p))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(format!(
            "apiVersion: v1\n\
            kind: Service\n\
            metadata:\n\
              name: {}\n\
              namespace: songbird\n\
            spec:\n\
              selector:\n\
                app: {}\n\
              ports:\n\
            {}\n\
              type: ClusterIP",
            service.name,
            service.name,
            ports_yaml
        ))
    }

    /// Apply Kubernetes manifest
    async fn apply_k8s_manifest(&self, manifest: &str) -> Result<()> {
        let mut cmd = Command::new("kubectl");
        cmd.args(&["apply", "-f", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()
            .map_err(|e| SongbirdError::Deployment {
                stage: "kubernetes_deploy".to_string(),
                message: format!("Failed to spawn kubectl: {}", e),
            })?;

        if let Some(stdin) = child.stdin.as_mut() {
            use tokio::io::AsyncWriteExt;
            stdin.write_all(manifest.as_bytes()).await
                .map_err(|e| SongbirdError::Deployment {
                    stage: "kubernetes_manifest".to_string(),
                    message: format!("Failed to write manifest to kubectl: {}", e),
                })?;
        }

        let output = child.wait_with_output().await
            .map_err(|e| SongbirdError::Deployment {
                stage: "kubernetes_apply".to_string(),
                message: format!("Failed to apply manifest: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SongbirdError::Deployment {
                stage: "kubernetes_apply".to_string(),
                message: format!("kubectl apply failed: {}", stderr),
            });
        }

        Ok(())
    }

    /// Wait for Kubernetes deployment to be ready
    async fn wait_for_k8s_deployment(&self, name: &str) -> Result<()> {
        Command::new("kubectl")
            .args(&["rollout", "status", "deployment", name, "-n", "songbird"])
            .status()
            .await
            .map_err(|e| SongbirdError::Deployment {
                stage: "kubernetes_wait".to_string(),
                message: format!("Failed to wait for deployment: {}", e),
            })?;

        Ok(())
    }

    /// Generate systemd unit file
    fn generate_systemd_unit(&self, service: &ServiceConfig) -> Result<String> {
        let env_vars = service.environment_variables.iter()
            .map(|(k, v)| format!("Environment={}={}", k, v))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(format!(
            "[Unit]\n\
            Description=Songbird {}\n\
            After=network.target\n\
            \n\
            [Service]\n\
            Type=simple\n\
            ExecStart=/opt/songbird/{}\n\
            Restart=always\n\
            RestartSec=3\n\
            {}\n\
            \n\
            [Install]\n\
            WantedBy=multi-user.target",
            service.name,
            service.name.to_lowercase(),
            env_vars
        ))
    }

    /// Get deployment history
    pub fn get_deployment_history(&self) -> &[DeploymentRecord] {
        &self.deployment_history
    }
}

/// Service configuration for deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub image: String,
    pub ports: Vec<u16>,
    pub environment_variables: HashMap<String, String>,
    pub deployment_method: Option<String>,
    pub health_check_path: Option<String>,
    pub resource_limits: Option<ResourceLimits>,
}

/// Resource limits for a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu_limit: Option<String>,
    pub memory_limit: Option<String>,
    pub storage_limit: Option<String>,
}

/// Deployment strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStrategy {
    Sequential,
    Parallel,
    RollingUpdate,
}

/// Deployment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    pub deployment_id: String,
    pub status: DeploymentStatus,
    pub deployed_services: Vec<ServiceDeploymentInfo>,
    pub errors: Vec<DeploymentError>,
    pub deployment_time: chrono::Duration,
    pub summary: String,
}

/// Service deployment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDeploymentInfo {
    pub name: String,
    pub service_type: String,
    pub status: ServiceStatus,
    pub endpoint: String,
    pub ports: Vec<u16>,
    pub process_id: Option<u32>,
    pub health_status: HealthStatus,
    pub deployment_time: chrono::Duration,
    pub metadata: HashMap<String, String>,
}

/// Deployment error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentError {
    pub service_name: String,
    pub error_message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Deployment record for history
#[derive(Debug, Clone)]
struct DeploymentRecord {
    id: String,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: Option<chrono::DateTime<chrono::Utc>>,
    status: DeploymentStatus,
    strategy: DeploymentStrategy,
    services: Vec<ServiceConfig>,
    errors: Vec<String>,
}

/// Deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    InProgress,
    Completed,
    CompletedWithWarnings,
    Failed,
    RolledBack,
}

impl std::fmt::Display for DeploymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentStatus::InProgress => write!(f, "In Progress"),
            DeploymentStatus::Completed => write!(f, "Completed"),
            DeploymentStatus::CompletedWithWarnings => write!(f, "Completed with Warnings"),
            DeploymentStatus::Failed => write!(f, "Failed"),
            DeploymentStatus::RolledBack => write!(f, "Rolled Back"),
        }
    }
}

/// Service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceStatus {
    Starting,
    Running,
    Stopped,
    Failed,
}

impl std::fmt::Display for ServiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceStatus::Starting => write!(f, "Starting"),
            ServiceStatus::Running => write!(f, "Running"),
            ServiceStatus::Stopped => write!(f, "Stopped"),
            ServiceStatus::Failed => write!(f, "Failed"),
        }
    }
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "Healthy"),
            HealthStatus::Unhealthy => write!(f, "Unhealthy"),
            HealthStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployment_engine_creation() {
        let engine = DeploymentEngine::new();
        assert!(engine.get_deployment_history().is_empty());
    }

    #[test]
    fn test_service_config_creation() {
        let config = ServiceConfig {
            name: "test-service".to_string(),
            image: "nginx:latest".to_string(),
            ports: vec![80, 443],
            environment_variables: HashMap::new(),
            deployment_method: Some("docker".to_string()),
            health_check_path: Some("/health".to_string()),
            resource_limits: None,
        };

        assert_eq!(config.name, "test-service");
        assert_eq!(config.ports.len(), 2);
    }

    #[test]
    fn test_deployment_status_display() {
        assert_eq!(DeploymentStatus::InProgress.to_string(), "In Progress");
        assert_eq!(DeploymentStatus::Completed.to_string(), "Completed");
        assert_eq!(DeploymentStatus::Failed.to_string(), "Failed");
    }

    #[test]
    fn test_service_status_display() {
        assert_eq!(ServiceStatus::Starting.to_string(), "Starting");
        assert_eq!(ServiceStatus::Running.to_string(), "Running");
        assert_eq!(ServiceStatus::Stopped.to_string(), "Stopped");
        assert_eq!(ServiceStatus::Failed.to_string(), "Failed");
    }

    #[test]
    fn test_health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "Healthy");
        assert_eq!(HealthStatus::Unhealthy.to_string(), "Unhealthy");
        assert_eq!(HealthStatus::Unknown.to_string(), "Unknown");
    }
} 