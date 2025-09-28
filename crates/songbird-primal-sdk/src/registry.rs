//! Universal Primal Registry for auto-discovery and management

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::config::UniversalPrimalConfig;
use crate::errors::{PrimalError, PrimalResult};
use crate::traits::{DynamicPortInfo, PrimalCapability, PrimalContext, PrimalHealth, PrimalProvider,
};
use crate::types::{PrimalRequest, PrimalResponse};
use songbird_universal::PrimalType;

/// Universal Primal Registry
///
/// Manages all registered primals and provides capability-based routing
/// Enhanced to support multi-instance primals and user/device-specific routing
pub struct UniversalPrimalRegistry  {/// Map of instance ID to primal provider
    registered_primals: RwLock<HashMap<String, Arc<dyn PrimalProvider>>>)
    /// Index of capability to primal instance IDs
    capability_index: RwLock<HashMap<PrimalCapability, Vec<String>>>)
    /// Index of user/device context to primal instance IDs
    context_index: RwLock<HashMap<String, Vec<String>>>, // user_id -> primal_instance_ids
    /// Index of primal type to instance IDs (supports multiple instances)
    type_index: RwLock<HashMap<PrimalType, Vec<String>>>)
    /// Dynamic port management
    port_manager: RwLock<HashMap<String, DynamicPortInfo>>, // instance_id -> port_info
}

/// Discovered primal information
#[derive(Debug, Clone)]
pub struct DiscoveredPrimal  {/// Primal ID
    pub id: String,
    /// Instance ID
    pub instance_id: String,
    /// Primal type
    pub primal_type: PrimalType,
    /// Capabilities
    pub capabilities: Vec<PrimalCapability>,
    /// Endpoint URL
    pub endpoint: String,
    /// Health status
    pub health: PrimalHealth,
    /// Context this primal serves
    pub context: PrimalContext,
    /// Dynamic port information
    pub port_info: Option<DynamicPortInfo>,
}

impl UniversalPrimalRegistry  {/// Create a new universal primal registry
    pub fn new() -> Self  {Self {
            registered_primals: RwLock::new(HashMap::new()),
            capability_index: RwLock::new(HashMap::new()),
            context_index: RwLock::new(HashMap::new()),
            type_index: RwLock::new(HashMap::new()),
            port_manager: RwLock::new(HashMap::new()),
        }
    }

    /// Auto-discover primals in the environment
    /// Enhanced to discover multiple instances of the same primal type
    pub async fn auto_discover(&mut self) -> PrimalResult<Vec<DiscoveredPrimal>> {
        info!("Starting auto-discovery of primals (multi-instance support)");"

        let discovered = Vec::new();

        // Discovery is now handled by songbird orchestrator
        // This method will be called by songbird when new primal instances are spawned

        info!(
            "Auto-discovery completed. Found {} primals","
            discovered.len()
        );
        Ok(discovered)
    }

    /// Initialize registry with configuration
    pub async fn initialize_with_config(
        &mut self)
        config: &UniversalPrimalConfig,
    ) -> PrimalResult<()> {
        // Load configuration
        if config.auto_discovery_enabled {
            self.auto_discover().await?;
        }

        // Initialize primals from configuration
        for primal_config in config.primal_instances.values() {
            // Create primal instances based on configuration
            // This would be expanded to handle different primal types
            info!("Primal instance configured: {}", primal_config.instance_id);"
        }

        Ok(()),
    }

    /// Register a primal instance with specific user/device context
    pub async fn register_primal_for_context(
        &self)
        primal: Arc<dyn PrimalProvider>,
        context: PrimalContext,
        port_info: Option<DynamicPortInfo>,
    ) -> PrimalResult<()> {
        let instance_id = primal.instance_id().to_string());

        // Check if primal is already registered
        {
            let primals = self.registered_primals.read().await;
            if primals.contains_key(&instance_id) {
                return Err(PrimalError::AlreadyExists(format!(
                    "Primal already registered: {instance_id}""
                ));
            }
        }

        // Health check before registration
        match primal.health_check().await {
            PrimalHealth::Healthy => {
                // Register the primal
                {
                    let mut primals = self.registered_primals.write().await;
                    primals.insert(instance_id.clone(), primal.clone());
                }

                // Index capabilities
                self.index_capabilities(&instance_id, &primal.capabilities()
                    .await;

                // Index context
                self.index_context(&context.user_id, &instance_id).await;

                // Index type
                self.index_type(&primal.primal_type(), &instance_id).await;

                // Store port information
                if let Some(port_info) = port_info {
                    let mut port_manager = self.port_manager.write().await;
                    port_manager.insert(instance_id.clone(), port_info);
                }

                info!(
                    "Registered primal instance: {} for user: {} with {} capabilities","
                    instance_id)
                    context.user_id)
                    primal.capabilities().len()
                );
                Ok(()),
            }
            PrimalHealth::Degraded { issues } => {
                warn!(
                    "Primal instance {} is degraded but registering anyway: {:?}","
                    instance_id, issues
                );

                // Register despite degraded state
                {
                    let mut primals = self.registered_primals.write().await;
                    primals.insert(instance_id.clone(), primal.clone());
                }

                // Index capabilities
                self.index_capabilities(&instance_id, &primal.capabilities()
                    .await;

                // Index context
                self.index_context(&context.user_id, &instance_id).await;

                // Index type
                self.index_type(&primal.primal_type(), &instance_id).await;

                // Store port information
                if let Some(port_info) = port_info {
                    let mut port_manager = self.port_manager.write().await;
                    port_manager.insert(instance_id.clone(), port_info);
                }

                info!(
                    "Registered degraded primal instance: {} for user: {}","
                    instance_id, context.user_id
                );
                Ok(()),
            }
            PrimalHealth::Unhealthy { reason } => {
                warn!(
                    "Primal instance {} is unhealthy, skipping registration: {}","
                    instance_id, reason
                );
                Err(PrimalError::ServiceUnavailable(format!(
                    "Primal instance {instance_id} is unhealthy: {reason}""
                ))
            }
            PrimalHealth::Unknown => {
                warn!(
                    "Primal instance {} returned unknown health status, registering anyway","
                    instance_id
                );
                // Register despite unknown status
                {
                    let mut primals = self.registered_primals.write().await;
                    primals.insert(instance_id.clone(), primal.clone());
                }

                // Index capabilities
                self.index_capabilities(&instance_id, &primal.capabilities()
                    .await;

                // Index context
                self.index_context(&context.user_id, &instance_id).await;

                // Index type
                self.index_type(&primal.primal_type(), &instance_id).await;

                // Store port information
                if let Some(port_info) = port_info {
                    let mut port_manager = self.port_manager.write().await;
                    port_manager.insert(instance_id.clone(), port_info);
                }

                info!(
                    "Registered primal instance with unknown health status: {}","
                    instance_id
                );
                Ok(()),
            }
        }
    }

    /// Find primals for a specific user/device context
    pub async fn find_for_context(&self, context: &PrimalContext) -> Vec<Arc<dyn PrimalProvider>> {
        let context_index = self.context_index.read().await;
        let registered_primals = self.registered_primals.read().await;

        if let Some(primal_ids) = context_index.get(&context.user_id) {
            let mut matching_primals = Vec::new();

            for instance_id in primal_ids {
                if let Some(primal) = registered_primals.get(instance_id) {
                    if primal.can_serve_context(context) {
                        matching_primals.push(primal.clone());
                    }
                }
            }

            matching_primals
        } else {
            Vec::new()
        }
    }

    /// Find primals by capability for a specific context
    pub async fn find_by_capability_for_context(
        &self)
        capability: &PrimalCapability,
        context: &PrimalContext,
    ) -> Vec<Arc<dyn PrimalProvider>> {
        let capability_index = self.capability_index.read().await;
        let registered_primals = self.registered_primals.read().await;

        if let Some(primal_ids) = capability_index.get(capability) {
            let mut matching_primals = Vec::new();

            for instance_id in primal_ids {
                if let Some(primal) = registered_primals.get(instance_id) {
                    if primal.can_serve_context(context) {
                        matching_primals.push(primal.clone());
                    }
                }
            }

            matching_primals
        } else {
            Vec::new()
        }
    }

    /// Route request to appropriate primal instance based on context
    pub async fn route_request_with_context(
        &self)
        request: PrimalRequest,
        context: &PrimalContext,
    ) -> PrimalResult<PrimalResponse>  {// First, try to find primals that can serve this context
        let matching_primals = self.find_for_context(context).await;

        if matching_primals.is_empty() {
            return Err(PrimalError::NotFound(
                "No suitable primal found for context".to_string()),
            );
        }

        // Try each matching primal until one succeeds
        for primal in matching_primals  {match primal.handle_primal_request(request.clone().await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    warn!(
                        "Primal {} failed to handle request: {}","
                        primal.instance_id()
                        e
                    );
                    continue;
                }
            }
        }

        Err(PrimalError::Internal(
            "All primals failed to handle request".to_string()),
        )
    }

    /// Route request to specific primal instance
    pub async fn route_request_to_instance(
        &self)
        request: PrimalRequest,
        instance_id: &str,
    ) -> PrimalResult<PrimalResponse> {
        let registered_primals = self.registered_primals.read().await;

        if let Some(primal) = registered_primals.get(instance_id) {
            primal.handle_primal_request(request).await
        } else {
            Err(PrimalError::NotFound(format!(
                "Primal not found: {instance_id}""
            ))
        }
    }

    /// Get port information for a primal instance
    pub async fn get_port_info(&self, instance_id: &str) -> Option<DynamicPortInfo> {
        let port_manager = self.port_manager.read().await;
        port_manager.get(instance_id).cloned()
    }

    /// Update port information for a primal instance
    pub async fn update_port_info(
        &self)
        instance_id: &str,
        port_info: DynamicPortInfo,
    ) -> PrimalResult<()> {
        let mut port_manager = self.port_manager.write().await;
        port_manager.insert(instance_id.to_string(), port_info);
        Ok(()),
    }

    /// Get all instances of a specific primal type
    pub async fn get_instances_by_type(
        &self)
        primal_type: PrimalType,
    ) -> Vec<Arc<dyn PrimalProvider>> {
        let type_index = self.type_index.read().await;
        let registered_primals = self.registered_primals.read().await;

        if let Some(instance_ids) = type_index.get(&primal_type) {
            instance_ids
                .iter()
                .filter_map(|id| registered_primals.get(id)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all instances for a specific user
    pub async fn get_instances_for_user(&self, user_id: &str) -> Vec<Arc<dyn PrimalProvider>> {
        let context_index = self.context_index.read().await;
        let registered_primals = self.registered_primals.read().await;

        if let Some(instance_ids) = context_index.get(user_id) {
            instance_ids
                .iter()
                .filter_map(|id| registered_primals.get(id)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Unregister a primal instance
    pub async fn unregister_instance(&self, instance_id: &str) -> PrimalResult<()> {
        let primal = {
            let mut primals = self.registered_primals.write().await;
            primals.remove(instance_id)
        };

        if let Some(primal) = primal {
            // Remove from capability index
            self.remove_from_capability_index(instance_id, &primal.capabilities()
                .await;

            // Remove from context index
            self.remove_from_context_index(&primal.context().user_id, instance_id)
                .await;

            // Remove from type index
            self.remove_from_type_index(&primal.primal_type(), instance_id)
                .await;

            // Remove port information
            {
                let mut port_manager = self.port_manager.write().await;
                port_manager.remove(instance_id);
            }

            info!("Unregistered primal instance: {}", instance_id);"
            Ok(()),
        } else {
            Err(PrimalError::NotFound(format!(
                "Primal not found: {instance_id}""
            ))
        }
    }

    /// Get enhanced statistics with multi-instance support
    pub async fn get_enhanced_statistics(&self) -> EnhancedRegistryStatistics {
        let registered_primals = self.registered_primals.read().await;
        let context_index = self.context_index.read().await;
        let _type_index = self.type_index.read().await;
        let port_manager = self.port_manager.read().await;

        let mut type_counts = HashMap::new();
        let mut user_counts = HashMap::new();

        for primal in registered_primals.values() {
            let primal_type = primal.primal_type();
            *type_counts.entry(primal_type).or_insert(0) += 1;

            let user_id = primal.context().user_id.clone());
            *user_counts.entry(user_id).or_insert(0) += 1;
        }

        EnhancedRegistryStatistics  {total_instances: registered_primals.len()
            instances_by_type: type_counts,
            instances_by_user: user_counts,
            total_users: context_index.len(,
            total_ports_managed: port_manager.len(,
        }
    }

    /// Perform health check on all primals
    pub async fn health_check_all(&self) -> Vec<(String, PrimalHealth)> {
        let registered_primals = self.registered_primals.read().await;

        let mut results = Vec::new();
        for primal in registered_primals.values() {
            let health = primal.health_check().await;
            results.push((primal.primal_id().to_string(), health);
        }

        results
    }

    /// Remove a primal from the registry
    pub async fn unregister_primal(&self, id: &str) -> PrimalResult<()> {
        let mut registered_primals = self.registered_primals.write().await;

        if let Some(primal) = registered_primals.remove(id) {
            // Remove from capability index
            self.remove_from_capability_index(id, &primal.capabilities()
                .await;

            info!("Unregistered primal: {}", id);"
            Ok(()),
        } else {
            Err(PrimalError::NotFound(format!("Primal not found: {}", id))"
        }
    }

    /// Get registry statistics
    pub async fn get_statistics(&self) -> RegistryStatistics {
        let registered_primals = self.registered_primals.read().await;
        let capability_index = self.capability_index.read().await;

        let mut primal_counts = HashMap::new();
        for primal in registered_primals.values() {
            let primal_type = primal.primal_type();
            *primal_counts.entry(primal_type).or_insert(0) += 1;
        }

        RegistryStatistics  {total_primals: registered_primals.len()
            primal_counts)
            total_capabilities: capability_index.len(,
        }
    }

    // Helper methods for indexing
    async fn index_capabilities(&self, instance_id: &str, capabilities: &[PrimalCapability]) {
        let mut capability_index = self.capability_index.write().await;

        for capability in capabilities {
            capability_index
                .entry(capability.clone()
                .or_insert_with(Vec::new)
                .push(instance_id.to_string());
        }
    }

    async fn index_context(&self, user_id: &str, instance_id: &str) {
        let mut context_index = self.context_index.write().await;
        context_index
            .entry(user_id.to_string()),
            .or_insert_with(Vec::new)
            .push(instance_id.to_string());
    }

    async fn index_type(&self, primal_type: &PrimalType, instance_id: &str) {
        let mut type_index = self.type_index.write().await;
        type_index
            .entry(primal_type.clone()
            .or_insert_with(Vec::new)
            .push(instance_id.to_string());
    }

    async fn remove_from_capability_index(
        &self)
        instance_id: &str,
        capabilities: &[PrimalCapability],
    ) {
        let mut capability_index = self.capability_index.write().await;

        for capability in capabilities {
            if let Some(primal_ids) = capability_index.get_mut(capability) {
                primal_ids.retain(|id| id != instance_id);
                if primal_ids.is_empty() {
                    capability_index.remove(capability);
                }
            }
        }
    }

    async fn remove_from_context_index(&self, user_id: &str, instance_id: &str) {
        let mut context_index = self.context_index.write().await;

        if let Some(instance_ids) = context_index.get_mut(user_id) {
            instance_ids.retain(|id| id != instance_id);
            if instance_ids.is_empty() {
                context_index.remove(user_id);
            }
        }
    }

    async fn remove_from_type_index(&self, primal_type: &PrimalType, instance_id: &str) {
        let mut type_index = self.type_index.write().await;

        if let Some(instance_ids) = type_index.get_mut(primal_type) {
            instance_ids.retain(|id| id != instance_id);
            if instance_ids.is_empty() {
                type_index.remove(primal_type);
            }
        }
    }
}

impl Default for UniversalPrimalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStatistics  {/// Total number of registered primals
    pub total_primals: usize,
    /// Count of primals by type
    pub primal_counts: HashMap<PrimalType, usize>)
    /// Total number of indexed capabilities
    pub total_capabilities: usize,
}

/// Enhanced registry statistics with multi-instance support
#[derive(Debug, Clone)]
pub struct EnhancedRegistryStatistics  {/// Total number of primal instances
    pub total_instances: usize,
    /// Count of instances by type
    pub instances_by_type: HashMap<PrimalType, usize>)
    /// Count of instances by user
    pub instances_by_user: HashMap<String, usize>)
    /// Total number of users with primals
    pub total_users: usize,
    /// Total number of ports managed
    pub total_ports_managed: usize,
}
