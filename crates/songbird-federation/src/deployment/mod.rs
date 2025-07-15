//! Federated deployment management
//!
//! Handles deployment of services across federation nodes

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::types::*;
use songbird_errors::Result;

/// Deployment manager for federated services
pub struct DeploymentManager {
    config: FederationConfig,
    /// Active deployments
    deployments: Arc<RwLock<HashMap<Uuid, FederatedDeploymentResult>>>,
    /// Available nodes for deployment
    available_nodes: Arc<RwLock<Vec<FederationNode>>>,
}

impl DeploymentManager {
    /// Create new deployment manager
    pub async fn new(config: FederationConfig) -> Result<Self> {
        Ok(Self {
            config,
            deployments: Arc::new(RwLock::new(HashMap::new())),
            available_nodes: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Start deployment manager
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting federated deployment manager");

        // Start deployment monitoring task
        let deployments = self.deployments.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            loop {
                interval.tick().await;

                // Monitor deployment health
                let mut deployments_guard = deployments.write().await;
                for (deployment_id, deployment) in deployments_guard.iter_mut() {
                    if deployment.deployment_status == "pending" {
                        // Check if deployment should be marked as running
                        let elapsed = chrono::Utc::now()
                            .signed_duration_since(deployment.created_at)
                            .num_seconds() as u64;
                        if elapsed > 60 {
                            deployment.deployment_status = "running".to_string();
                            tracing::info!("Deployment {} transitioned to running", deployment_id);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Deploy services across federation
    pub async fn deploy_federated(
        &self,
        team_id: String,
        requirements: FederatedDeploymentRequirements,
    ) -> Result<FederatedDeploymentResult> {
        let deployment_id = Uuid::new_v4();

        tracing::info!(
            "Starting federated deployment {} for team {}",
            deployment_id,
            team_id
        );

        // Select nodes for deployment based on requirements
        let selected_nodes = self.select_nodes_for_deployment(&requirements).await?;

        if selected_nodes.is_empty() {
            return Err(songbird_errors::SongbirdError::service_error(
                "deployment",
                "No suitable nodes available for deployment".to_string(),
            ));
        }

        // Deploy to selected nodes
        let mut deployed_nodes = Vec::new();
        let mut endpoints = HashMap::new();

        for node in selected_nodes {
            match self.deploy_to_node(&node, &requirements).await {
                Ok(endpoint) => {
                    deployed_nodes.push(node.node_id);
                    endpoints.insert(node.node_id.to_string(), endpoint);
                    tracing::info!("Successfully deployed to node: {}", node.name);
                }
                Err(e) => {
                    tracing::warn!("Failed to deploy to node {}: {}", node.name, e);
                }
            }
        }

        let deployment_result = FederatedDeploymentResult {
            deployment_id,
            team_id,
            deployed_nodes,
            deployment_status: "pending".to_string(),
            endpoints,
            created_at: chrono::Utc::now(),
        };

        // Store deployment
        {
            let mut deployments = self.deployments.write().await;
            deployments.insert(deployment_id, deployment_result.clone());
        }

        tracing::info!(
            "Federated deployment {} created successfully",
            deployment_id
        );
        Ok(deployment_result)
    }

    /// Check deployment status
    pub async fn get_deployment_status(&self, deployment_id: Uuid) -> Result<String> {
        let deployments = self.deployments.read().await;

        if let Some(deployment) = deployments.get(&deployment_id) {
            Ok(deployment.deployment_status.clone())
        } else {
            Err(songbird_errors::SongbirdError::service_error(
                "deployment",
                format!("Deployment {deployment_id} not found"),
            ))
        }
    }

    /// Stop deployment
    pub async fn stop_deployment(&self, deployment_id: Uuid) -> Result<()> {
        let mut deployments = self.deployments.write().await;

        if let Some(deployment) = deployments.get_mut(&deployment_id) {
            // Stop deployment on all nodes - we need to get the node info from our available nodes
            for &node_id in &deployment.deployed_nodes {
                let nodes = self.available_nodes.read().await;
                if let Some(node) = nodes.iter().find(|n| n.node_id == node_id) {
                    match self.stop_deployment_on_node(node, deployment_id).await {
                        Ok(_) => {
                            tracing::info!("Stopped deployment on node: {}", node.name);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to stop deployment on node {}: {e}", node.name);
                        }
                    }
                }
            }

            deployment.deployment_status = "stopped".to_string();
            tracing::info!("Deployment {deployment_id} stopped");
        } else {
            return Err(songbird_errors::SongbirdError::service_error(
                "deployment",
                format!("Deployment {deployment_id} not found"),
            ));
        }

        Ok(())
    }

    /// Select nodes for deployment based on requirements
    async fn select_nodes_for_deployment(
        &self,
        requirements: &FederatedDeploymentRequirements,
    ) -> Result<Vec<FederationNode>> {
        let nodes = self.available_nodes.read().await;
        let mut selected_nodes = Vec::new();

        for node in nodes.iter() {
            if self.node_meets_requirements(node, requirements).await {
                selected_nodes.push(node.clone());

                // Stop when we have enough nodes
                if selected_nodes.len() >= requirements.node_count as usize {
                    break;
                }
            }
        }

        Ok(selected_nodes)
    }

    /// Check if a node meets deployment requirements
    async fn node_meets_requirements(
        &self,
        node: &FederationNode,
        requirements: &FederatedDeploymentRequirements,
    ) -> bool {
        // Check resource requirements
        if node.metrics.cpu_usage > 80.0 {
            // Node is too busy
            return false;
        }

        if node.metrics.memory_usage > 80.0 {
            // Node is low on memory
            return false;
        }

        // Check required capabilities
        if !requirements.required_capabilities.is_empty() {
            // For now, just check if it's a Tower node with capabilities
            match &node.node_type {
                NodeType::Tower { capabilities, .. } => {
                    for required_cap in &requirements.required_capabilities {
                        if !capabilities.specializations.contains(required_cap) {
                            return false;
                        }
                    }
                }
                _ => {
                    // Non-tower nodes don't have specializations
                    return false;
                }
            }
        }

        // Check node status
        if node.status != NodeStatus::Online {
            return false;
        }

        true
    }

    /// Deploy to a specific node
    async fn deploy_to_node(
        &self,
        node: &FederationNode,
        requirements: &FederatedDeploymentRequirements,
    ) -> Result<String> {
        // Create deployment request
        let deployment_request = serde_json::json!({
            "node_count": requirements.node_count,
            "required_capabilities": requirements.required_capabilities,
            "performance_requirements": {
                "min_cpu_cores": requirements.performance_requirements.min_cpu_cores,
                "min_memory_gb": requirements.performance_requirements.min_memory_gb,
                "min_bandwidth_mbps": requirements.performance_requirements.min_bandwidth_mbps,
            },
            "security_requirements": {
                "security_level": requirements.security_requirements.security_level,
                "require_beardog": requirements.security_requirements.require_beardog,
            }
        });

        // Send deployment request to node
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| {
                songbird_errors::SongbirdError::service_error(
                    "deployment",
                    format!("Failed to create HTTP client: {e}"),
                )
            })?;

        // Try each address until we find one that works
        for address in &node.addresses {
            let deployment_url = format!("http://{address}/deployment/deploy");

            match client
                .post(&deployment_url)
                .json(&deployment_request)
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    // Parse response to get service endpoint
                    match response.json::<serde_json::Value>().await {
                        Ok(result) => {
                            if let Some(endpoint) = result.get("endpoint").and_then(|v| v.as_str())
                            {
                                return Ok(endpoint.to_string());
                            } else {
                                return Ok(format!("http://{address}/service"));
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse deployment response: {e}");
                            return Ok(format!("http://{address}/service"));
                        }
                    }
                }
                Ok(response) => {
                    tracing::warn!(
                        "Deployment request to {address} failed with status: {}",
                        response.status()
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to send deployment request to {address}: {e}");
                }
            }
        }

        Err(songbird_errors::SongbirdError::service_error(
            "deployment",
            format!("Failed to deploy to node: {}", node.name),
        ))
    }

    /// Stop deployment on a specific node
    async fn stop_deployment_on_node(
        &self,
        node: &FederationNode,
        deployment_id: Uuid,
    ) -> Result<()> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| {
                songbird_errors::SongbirdError::service_error(
                    "deployment",
                    format!("Failed to create HTTP client: {e}"),
                )
            })?;

        let stop_request = serde_json::json!({
            "deployment_id": deployment_id
        });

        // Try each address until we find one that works
        for address in &node.addresses {
            let stop_url = format!("http://{address}/deployment/stop");

            match client.post(&stop_url).json(&stop_request).send().await {
                Ok(response) if response.status().is_success() => {
                    return Ok(());
                }
                Ok(response) => {
                    tracing::warn!(
                        "Stop request to {} failed with status: {}",
                        address,
                        response.status()
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to send stop request to {}: {}", address, e);
                }
            }
        }

        Err(songbird_errors::SongbirdError::service_error(
            "deployment",
            format!("Failed to stop deployment on node: {}", node.name),
        ))
    }

    /// Update available nodes
    pub async fn update_available_nodes(&self, nodes: Vec<FederationNode>) -> Result<()> {
        let mut available_nodes = self.available_nodes.write().await;
        *available_nodes = nodes;
        tracing::info!("Updated available nodes: {} nodes", available_nodes.len());
        Ok(())
    }

    /// Get deployment statistics
    pub async fn get_deployment_stats(&self) -> DeploymentStats {
        let deployments = self.deployments.read().await;
        let nodes = self.available_nodes.read().await;

        let mut running_count = 0;
        let mut pending_count = 0;
        let mut stopped_count = 0;

        for deployment in deployments.values() {
            match deployment.deployment_status.as_str() {
                "running" => running_count += 1,
                "pending" => pending_count += 1,
                "stopped" => stopped_count += 1,
                _ => {}
            }
        }

        DeploymentStats {
            total_deployments: deployments.len(),
            running_deployments: running_count,
            pending_deployments: pending_count,
            stopped_deployments: stopped_count,
            available_nodes: nodes.len(),
        }
    }
}

/// Deployment statistics
#[derive(Debug, Clone)]
pub struct DeploymentStats {
    pub total_deployments: usize,
    pub running_deployments: usize,
    pub pending_deployments: usize,
    pub stopped_deployments: usize,
    pub available_nodes: usize,
}
