---
description: ENFORCE universal service discovery and capability-based service registry
globs: ["songbird/src/**/*.rs", "songbird/crates/**/*.rs"]
---

# Universal Service Registry Specification

## Context
- When implementing dynamic service discovery for universal multi-protocol orchestration
- When managing service capabilities and metadata across all primal types
- When coordinating service lifecycle across the entire ecosystem
- When enabling capability-based service selection and routing

## Requirements

### Universal Service Discovery
- Real-time service registration and deregistration for any primal type
- Capability-based service discovery with flexible filtering
- Service health monitoring and status updates
- Automatic service cleanup for dead instances
- Support for multiple discovery backends (Kubernetes, Consul, DNS, Static, Custom)

### Universal Capability Management
- Rich capability metadata that works with any service type
- Capability validation and enforcement
- Dynamic capability updates during runtime
- Capability-based routing and selection
- Extensible capability system for future primal types

### Universal Cross-Primal Service Coordination
- Standardized service registration across all primals
- Cross-primal service discovery and communication
- Service dependency management
- Primal-specific service policies and constraints

### Universal High Availability and Scalability
- Distributed service registry with consensus
- Service data replication and consistency
- Automatic failover and recovery
- Horizontal scaling support
- Multi-region service discovery

## Architecture

### Core Universal Service Registry
```rust
pub struct UniversalServiceRegistry {
    service_store: Arc<dyn UniversalServiceStore>,
    capability_manager: Arc<UniversalCapabilityManager>,
    health_tracker: Arc<UniversalHealthTracker>,
    event_publisher: Arc<UniversalEventPublisher>,
    discovery_engine: Arc<UniversalDiscoveryEngine>,
    primal_coordinator: Arc<PrimalCoordinator>,
    registry_config: UniversalRegistryConfig,
}

impl UniversalServiceRegistry {
    pub fn new(config: UniversalRegistryConfig) -> Self {
        let service_store = Self::create_service_store(&config);
        let capability_manager = Arc::new(UniversalCapabilityManager::new());
        let health_tracker = Arc::new(UniversalHealthTracker::new());
        let event_publisher = Arc::new(UniversalEventPublisher::new());
        let discovery_engine = Arc::new(UniversalDiscoveryEngine::new(&config));
        let primal_coordinator = Arc::new(PrimalCoordinator::new());

        Self {
            service_store,
            capability_manager,
            health_tracker,
            event_publisher,
            discovery_engine,
            primal_coordinator,
            registry_config: config,
        }
    }

    /// Register any service with universal metadata
    pub async fn register_service(
        &self,
        registration: UniversalServiceRegistration,
    ) -> Result<String, RegistryError> {
        // Generate unique service ID
        let service_id = self.generate_service_id(&registration).await?;
        
        // Validate service registration
        self.validate_registration(&registration).await?;
        
        // Create registered service record
        let registered_service = RegisteredService {
            service_id: service_id.clone(),
            registration,
            status: ServiceStatus::Registered,
            health_status: HealthStatus::Unknown,
            last_heartbeat: Utc::now(),
            metrics: ServiceMetrics::default(),
            capabilities: Vec::new(),
        };
        
        // Store service
        self.service_store.store_service(registered_service.clone()).await?;
        
        // Update capability index
        self.capability_manager
            .index_service_capabilities(&service_id, &registered_service.registration.capabilities)
            .await?;
        
        // Start health monitoring
        self.health_tracker.start_monitoring(&service_id).await?;
        
        // Notify primal coordinator
        self.primal_coordinator
            .notify_service_registered(&registered_service)
            .await?;
        
        // Publish registration event
        self.event_publisher
            .publish_service_registered(registered_service)
            .await?;
        
        Ok(service_id)
    }

    /// Discover services by capability requirements
    pub async fn discover_services(
        &self,
        query: UniversalServiceQuery,
    ) -> Result<Vec<RegisteredService>, RegistryError> {
        // Start with capability-based filtering
        let mut candidates = if query.required_capabilities.is_empty() {
            self.service_store.get_all_services().await?
        } else {
            self.capability_manager
                .find_services_by_capabilities(&query.required_capabilities)
                .await?
        };
        
        // Apply primal type filter
        if let Some(primal_type) = query.primal_type {
            candidates = candidates.into_iter()
                .filter(|service| service.registration.primal_type == primal_type)
                .collect();
        }
        
        // Apply health filter
        if query.healthy_only {
            candidates = candidates.into_iter()
                .filter(|service| service.health_status == HealthStatus::Healthy)
                .collect();
        }
        
        // Apply metadata filters
        if !query.metadata_filters.is_empty() {
            candidates = candidates.into_iter()
                .filter(|service| self.matches_metadata_filters(service, &query.metadata_filters))
                .collect();
        }
        
        // Apply limit
        if let Some(limit) = query.limit {
            candidates.truncate(limit);
        }
        
        // Sort by preference
        self.sort_services_by_preference(&mut candidates, &query.sort_preference).await?;
        
        Ok(candidates)
    }

    /// Update service capabilities dynamically
    pub async fn update_service_capabilities(
        &self,
        service_id: &str,
        capabilities: Vec<ServiceCapability>,
    ) -> Result<(), RegistryError> {
        // Validate capabilities
        self.capability_manager.validate_capabilities(&capabilities).await?;
        
        // Update service record
        self.service_store.update_service_capabilities(service_id, capabilities.clone()).await?;
        
        // Update capability index
        self.capability_manager
            .update_service_capabilities(service_id, &capabilities)
            .await?;
        
        // Publish capability update event
        self.event_publisher
            .publish_capabilities_updated(service_id, capabilities)
            .await?;
        
        Ok(())
    }

    /// Deregister service
    pub async fn deregister_service(&self, service_id: &str) -> Result<(), RegistryError> {
        // Get service record
        let service = self.service_store.get_service(service_id).await?
            .ok_or(RegistryError::ServiceNotFound(service_id.to_string()))?;
        
        // Stop health monitoring
        self.health_tracker.stop_monitoring(service_id).await?;
        
        // Remove from capability index
        self.capability_manager.remove_service_capabilities(service_id).await?;
        
        // Notify primal coordinator
        self.primal_coordinator.notify_service_deregistered(&service).await?;
        
        // Remove from store
        self.service_store.remove_service(service_id).await?;
        
        // Publish deregistration event
        self.event_publisher.publish_service_deregistered(service).await?;
        
        Ok(())
    }

    /// Watch for service changes
    pub async fn watch_services(
        &self,
        query: UniversalServiceQuery,
        callback: ServiceWatchCallback,
    ) -> Result<WatchHandle, RegistryError> {
        self.discovery_engine.watch_services(query, callback).await
    }

    /// Get service health status
    pub async fn get_service_health(&self, service_id: &str) -> Result<ServiceHealth, RegistryError> {
        self.health_tracker.get_service_health(service_id).await
    }

    /// Update service health
    pub async fn update_service_health(
        &self,
        service_id: &str,
        health_status: HealthStatus,
    ) -> Result<(), RegistryError> {
        self.health_tracker.update_service_health(service_id, health_status).await?;
        
        // Update service record
        self.service_store.update_service_health(service_id, health_status).await?;
        
        // Publish health update event
        self.event_publisher.publish_health_updated(service_id, health_status).await?;
        
        Ok(())
    }

    /// Get service by ID
    pub async fn get_service(&self, service_id: &str) -> Result<Option<RegisteredService>, RegistryError> {
        self.service_store.get_service(service_id).await
    }

    /// Get services by primal type
    pub async fn get_services_by_primal(
        &self,
        primal_type: PrimalType,
    ) -> Result<Vec<RegisteredService>, RegistryError> {
        self.service_store.get_services_by_primal(primal_type).await
    }

    /// Get all services
    pub async fn get_all_services(&self) -> Result<Vec<RegisteredService>, RegistryError> {
        self.service_store.get_all_services().await
    }

    /// Get registry statistics
    pub async fn get_statistics(&self) -> Result<RegistryStatistics, RegistryError> {
        Ok(RegistryStatistics {
            total_services: self.service_store.count_services().await?,
            services_by_primal: self.service_store.count_services_by_primal().await?,
            services_by_health: self.service_store.count_services_by_health().await?,
            capability_distribution: self.capability_manager.get_capability_distribution().await?,
        })
    }

    // Helper methods
    async fn generate_service_id(&self, registration: &UniversalServiceRegistration) -> Result<String, RegistryError> {
        let base_id = format!("{}-{}-{}", 
            registration.primal_type.as_str(),
            registration.service.name,
            registration.service.instance_id
        );
        
        // Ensure uniqueness
        let mut counter = 0;
        let mut service_id = base_id.clone();
        
        while self.service_store.service_exists(&service_id).await? {
            counter += 1;
            service_id = format!("{}-{}", base_id, counter);
        }
        
        Ok(service_id)
    }

    async fn validate_registration(&self, registration: &UniversalServiceRegistration) -> Result<(), RegistryError> {
        // Validate service metadata
        if registration.service.name.is_empty() {
            return Err(RegistryError::InvalidRegistration("Service name cannot be empty".to_string()));
        }

        // Validate capabilities
        self.capability_manager.validate_capabilities(&registration.capabilities).await?;

        // Validate endpoints
        for endpoint in &registration.endpoints {
            if endpoint.url.is_empty() {
                return Err(RegistryError::InvalidRegistration("Endpoint URL cannot be empty".to_string()));
            }
        }

        Ok(())
    }

    fn matches_metadata_filters(&self, service: &RegisteredService, filters: &HashMap<String, String>) -> bool {
        for (key, value) in filters {
            if let Some(service_value) = service.registration.metadata.get(key) {
                if service_value != value {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    async fn sort_services_by_preference(
        &self,
        services: &mut Vec<RegisteredService>,
        preference: &SortPreference,
    ) -> Result<(), RegistryError> {
        match preference {
            SortPreference::Health => {
                services.sort_by(|a, b| b.health_status.cmp(&a.health_status));
            }
            SortPreference::Performance => {
                services.sort_by(|a, b| {
                    b.metrics.performance_score.partial_cmp(&a.metrics.performance_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            SortPreference::LastSeen => {
                services.sort_by(|a, b| b.last_heartbeat.cmp(&a.last_heartbeat));
            }
            SortPreference::PrimalType => {
                services.sort_by(|a, b| a.registration.primal_type.cmp(&b.registration.primal_type));
            }
            SortPreference::Capability => {
                services.sort_by(|a, b| b.capabilities.len().cmp(&a.capabilities.len()));
            }
        }
        Ok(())
    }
}
```

### Universal Service Discovery Query
```rust
#[derive(Debug, Clone)]
pub struct UniversalServiceQuery {
    /// Required capabilities for service matching
    pub required_capabilities: Vec<CapabilityRequirement>,
    
    /// Optional primal type filter
    pub primal_type: Option<PrimalType>,
    
    /// Only return healthy services
    pub healthy_only: bool,
    
    /// Metadata filters (key-value pairs)
    pub metadata_filters: HashMap<String, String>,
    
    /// Maximum number of results
    pub limit: Option<usize>,
    
    /// Sort preference for results
    pub sort_preference: SortPreference,
    
    /// Include services from specific regions
    pub regions: Option<Vec<String>>,
    
    /// Include services with specific tags
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub enum SortPreference {
    Health,
    Performance,
    LastSeen,
    PrimalType,
    Capability,
}

impl Default for UniversalServiceQuery {
    fn default() -> Self {
        Self {
            required_capabilities: Vec::new(),
            primal_type: None,
            healthy_only: true,
            metadata_filters: HashMap::new(),
            limit: None,
            sort_preference: SortPreference::Health,
            regions: None,
            tags: None,
        }
    }
}
```

### Universal Capability Management
```rust
pub struct UniversalCapabilityManager {
    capability_index: Arc<RwLock<CapabilityIndex>>,
    capability_validators: HashMap<String, Box<dyn CapabilityValidator>>,
}

impl UniversalCapabilityManager {
    pub fn new() -> Self {
        let mut validators: HashMap<String, Box<dyn CapabilityValidator>> = HashMap::new();
        
        // Register built-in validators
        validators.insert("compute".to_string(), Box::new(ComputeCapabilityValidator));
        validators.insert("storage".to_string(), Box::new(StorageCapabilityValidator));
        validators.insert("security".to_string(), Box::new(SecurityCapabilityValidator));
        validators.insert("ai".to_string(), Box::new(AICapabilityValidator));
        validators.insert("network".to_string(), Box::new(NetworkCapabilityValidator));
        
        Self {
            capability_index: Arc::new(RwLock::new(CapabilityIndex::new())),
            capability_validators: validators,
        }
    }

    pub async fn validate_capabilities(&self, capabilities: &[ServiceCapability]) -> Result<(), RegistryError> {
        for capability in capabilities {
            let category = capability.category();
            if let Some(validator) = self.capability_validators.get(category) {
                validator.validate(capability).await?;
            }
        }
        Ok(())
    }

    pub async fn index_service_capabilities(
        &self,
        service_id: &str,
        capabilities: &[ServiceCapability],
    ) -> Result<(), RegistryError> {
        let mut index = self.capability_index.write().await;
        
        for capability in capabilities {
            index.add_service_capability(service_id, capability.clone());
        }
        
        Ok(())
    }

    pub async fn find_services_by_capabilities(
        &self,
        requirements: &[CapabilityRequirement],
    ) -> Result<Vec<RegisteredService>, RegistryError> {
        let index = self.capability_index.read().await;
        let mut matching_services = HashSet::new();
        
        for requirement in requirements {
            let services = index.find_services_with_capability(requirement);
            if matching_services.is_empty() {
                matching_services = services;
            } else {
                matching_services = matching_services.intersection(&services).cloned().collect();
            }
        }
        
        // Convert service IDs to full service records
        let mut services = Vec::new();
        for service_id in matching_services {
            // This would typically fetch from the service store
            // For now, we'll return a placeholder
            services.push(RegisteredService::placeholder(service_id));
        }
        
        Ok(services)
    }

    pub async fn update_service_capabilities(
        &self,
        service_id: &str,
        capabilities: &[ServiceCapability],
    ) -> Result<(), RegistryError> {
        let mut index = self.capability_index.write().await;
        
        // Remove existing capabilities
        index.remove_service_capabilities(service_id);
        
        // Add new capabilities
        for capability in capabilities {
            index.add_service_capability(service_id, capability.clone());
        }
        
        Ok(())
    }

    pub async fn remove_service_capabilities(&self, service_id: &str) -> Result<(), RegistryError> {
        let mut index = self.capability_index.write().await;
        index.remove_service_capabilities(service_id);
        Ok(())
    }

    pub async fn get_capability_distribution(&self) -> Result<HashMap<String, usize>, RegistryError> {
        let index = self.capability_index.read().await;
        Ok(index.get_capability_distribution())
    }
}
```

### Universal Service Health Tracking
```rust
pub struct UniversalHealthTracker {
    health_checkers: HashMap<String, Arc<dyn HealthChecker>>,
    health_status: Arc<RwLock<HashMap<String, ServiceHealth>>>,
    monitoring_tasks: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
}

impl UniversalHealthTracker {
    pub fn new() -> Self {
        Self {
            health_checkers: HashMap::new(),
            health_status: Arc::new(RwLock::new(HashMap::new())),
            monitoring_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_monitoring(&self, service_id: &str) -> Result<(), RegistryError> {
        let health_checker = self.create_health_checker(service_id).await?;
        
        let service_id = service_id.to_string();
        let health_status = Arc::clone(&self.health_status);
        let checker = Arc::clone(&health_checker);
        
        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                match checker.check_health(&service_id).await {
                    Ok(health) => {
                        let mut status = health_status.write().await;
                        status.insert(service_id.clone(), health);
                    }
                    Err(e) => {
                        tracing::warn!("Health check failed for {}: {}", service_id, e);
                        let mut status = health_status.write().await;
                        status.insert(service_id.clone(), ServiceHealth {
                            status: HealthStatus::Unhealthy,
                            last_check: Utc::now(),
                            response_time: Duration::from_secs(0),
                            error: Some(e.to_string()),
                        });
                    }
                }
            }
        });
        
        let mut tasks = self.monitoring_tasks.write().await;
        tasks.insert(service_id.to_string(), task);
        
        Ok(())
    }

    pub async fn stop_monitoring(&self, service_id: &str) -> Result<(), RegistryError> {
        let mut tasks = self.monitoring_tasks.write().await;
        if let Some(task) = tasks.remove(service_id) {
            task.abort();
        }
        
        let mut status = self.health_status.write().await;
        status.remove(service_id);
        
        Ok(())
    }

    pub async fn get_service_health(&self, service_id: &str) -> Result<ServiceHealth, RegistryError> {
        let status = self.health_status.read().await;
        status.get(service_id)
            .cloned()
            .ok_or(RegistryError::ServiceNotFound(service_id.to_string()))
    }

    pub async fn update_service_health(
        &self,
        service_id: &str,
        health_status: HealthStatus,
    ) -> Result<(), RegistryError> {
        let mut status = self.health_status.write().await;
        let health = status.entry(service_id.to_string())
            .or_insert_with(|| ServiceHealth {
                status: HealthStatus::Unknown,
                last_check: Utc::now(),
                response_time: Duration::from_secs(0),
                error: None,
            });
        
        health.status = health_status;
        health.last_check = Utc::now();
        
        Ok(())
    }

    async fn create_health_checker(&self, service_id: &str) -> Result<Arc<dyn HealthChecker>, RegistryError> {
        // Create appropriate health checker based on service type
        // For now, return a generic HTTP health checker
        Ok(Arc::new(HttpHealthChecker::new()))
    }
}
```

### Universal Service Store Trait
```rust
#[async_trait]
pub trait UniversalServiceStore: Send + Sync {
    async fn store_service(&self, service: RegisteredService) -> Result<(), RegistryError>;
    async fn get_service(&self, service_id: &str) -> Result<Option<RegisteredService>, RegistryError>;
    async fn get_all_services(&self) -> Result<Vec<RegisteredService>, RegistryError>;
    async fn get_services_by_primal(&self, primal_type: PrimalType) -> Result<Vec<RegisteredService>, RegistryError>;
    async fn update_service_health(&self, service_id: &str, health: HealthStatus) -> Result<(), RegistryError>;
    async fn update_service_capabilities(&self, service_id: &str, capabilities: Vec<ServiceCapability>) -> Result<(), RegistryError>;
    async fn remove_service(&self, service_id: &str) -> Result<(), RegistryError>;
    async fn service_exists(&self, service_id: &str) -> Result<bool, RegistryError>;
    async fn count_services(&self) -> Result<u64, RegistryError>;
    async fn count_services_by_primal(&self) -> Result<HashMap<PrimalType, u64>, RegistryError>;
    async fn count_services_by_health(&self) -> Result<HashMap<HealthStatus, u64>, RegistryError>;
    async fn cleanup_expired_services(&self, expiry_duration: Duration) -> Result<Vec<String>, RegistryError>;
}

/// In-memory implementation for development/testing
pub struct InMemoryServiceStore {
    services: Arc<RwLock<HashMap<String, RegisteredService>>>,
}

/// Distributed implementation for production
pub struct DistributedServiceStore {
    // Implementation would use etcd, Consul, or similar
    backend: Arc<dyn DistributedStorage>,
}

/// Database implementation for persistent storage
pub struct DatabaseServiceStore {
    connection_pool: Arc<sqlx::Pool<sqlx::Postgres>>,
}
```

### Universal Configuration
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalRegistryConfig {
    pub store_type: StoreType,
    pub health_check_interval: Duration,
    pub service_expiry_duration: Duration,
    pub capability_validation_enabled: bool,
    pub event_publishing_enabled: bool,
    pub discovery_backends: Vec<DiscoveryBackendConfig>,
    pub replication_factor: usize,
    pub consistency_level: ConsistencyLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StoreType {
    InMemory,
    Distributed { backend: String, endpoints: Vec<String> },
    Database { connection_string: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsistencyLevel {
    Eventual,
    Strong,
    Quorum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryBackendConfig {
    pub backend_type: String,
    pub enabled: bool,
    pub configuration: HashMap<String, String>,
}
```

### Universal Registry Statistics
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStatistics {
    pub total_services: u64,
    pub services_by_primal: HashMap<PrimalType, u64>,
    pub services_by_health: HashMap<HealthStatus, u64>,
    pub capability_distribution: HashMap<String, usize>,
}
```

## Implementation Guidelines

### Universal Integration
- All services MUST implement universal service registration format
- Services MUST provide accurate capability information
- Services MUST support health check endpoints
- Services MUST handle graceful deregistration

### Capability Management
- Define clear capability requirements for all operations
- Use structured capability definitions with validation
- Support dynamic capability updates
- Implement capability-based service matching

### Health Monitoring
- Implement reliable health check mechanisms
- Support multiple health check protocols
- Provide detailed health status information
- Enable custom health check implementations

### Scalability
- Support distributed service storage
- Implement efficient capability indexing
- Use caching for frequently accessed data
- Enable horizontal scaling of registry components

## Success Metrics

### Discovery Performance
- Sub-50ms service discovery latency
- 99.9% successful service registrations
- Efficient capability matching algorithms
- Linear scaling with service count

### Availability
- 99.99% registry uptime
- Automatic failover for distributed deployments
- Graceful degradation during partial failures
- Zero downtime during configuration changes

### Accuracy
- Real-time health status updates
- Accurate capability information
- Consistent service state across replicas
- Reliable service lifecycle management

This specification establishes universal service registry patterns that work with any primal type while maintaining optimal performance, reliability, and scalability for future expansion. 