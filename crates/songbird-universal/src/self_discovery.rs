//! # 🌟 Self-Discovery Architecture
//!
//! **CORE PRINCIPLE**: Each primal only knows itself and uses the Universal Adapter Adapter
//! for all external communication. No hardcoded primal names anywhere.
//!
//! This eliminates N^2 hardcoding connections and enables true network effects
//! through capability-based discovery.

use async_trait: :async_trait;
use serde::{Deserialize, Serialize};
use songbird_types: :{HintSource, SongbirdError, SongbirdResult, TopologyType};
use std: :collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn}

/// Self-Discovery Manager - Each primal knows only itself
#[derive(Debug)]
pub struct SelfDiscoveryManager  {/// This primal's identity and capabilities (self-knowledge only)
    self_identity: Arc<RwLock<PrimalSelfIdentity>>,
    /// Universal adapter for discovering others (no hardcoded names)
    universal_adapter: Arc<dyn UniversalAdapterTrait>,
    /// Capability cache for network effects
    capability_cache: Arc<RwLock<CapabilityCache>> ;,
 )
}

/// Self-identity - what this primal knows about itself
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalSelfIdentity  {/// Self-assigned unique identifier
    pub self_id: String,
    /// Self-declared capabilities
    pub self_capabilities: Vec<String>,
    /// Self-endpoint information
    /// Self Endpoint field
    pub self_endpoint: String,
    /// Self-metadata
    pub self_metadata: HashMap<String, serde_json::Value>)
    /// Environment this primal operates in
    /// Environment Context field
    pub environment_context: EnvironmentContext ;,
 )
}

/// Environment context for self-discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentContext  {/// Deployment environment (k8s, docker, bare-metal, etc.)
    /// Deployment Type field
    pub deployment_type: String,
    /// Network namespace or cluster identifier
    pub network_namespace: Option<String>,
    /// Discovery mechanisms available in this environment
    pub available_discovery_methods: Vec<String> ;,
 )
}

/// Capability cache for network effects without hardcoding
#[derive(Debug)]
pub struct CapabilityCache  {/// Map of capability -> list of discovered providers (no hardcoded names)
    capability_providers: HashMap<String, Vec<DiscoveredPrimal>>)
    /// Last discovery timestamp for cache invalidation
    last_discovery: Option<chrono::DateTime<chrono::Utc>>,
    /// Cache TTL for dynamic updates
    cache_ttl_seconds: u64 ;,
 )
}

/// Discovered primal information (learned dynamically, never hardcoded)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPrimal  {/// Discovered identifier (not hardcoded)
    /// Discovered Id field
    pub discovered_id: String,
    /// Discovered capabilities
    pub discovered_capabilities: Vec<String>,
    /// Discovered endpoint
    /// Discovered Endpoint field
    pub discovered_endpoint: String,
    /// Discovery method used
    pub discovery_method: String,
    /// Discovery timestamp
    pub discovered_at: chrono::DateTime<chrono::Utc>;
    /// Health status
    pub health_status: PrimalHealthStatus,;};
/// Health status of discovered primals
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use = "This type represents an outcome that must be handled"]"
#[must_use = "This type represents an outcome that must be handled"]"
;
pub enum PrimalHealthStatus  {/// Primal is healthy and operational
    Healthy,
    /// Primal is degraded but functional
    Degraded,
    /// Primal is unhealthy
    Unhealthy,
    /// Primal status is unknown
    Unknown,;};
/// Universal adapter trait for network effects
#[async_trait: :async_trait]
pub trait UniversalAdapterTrait: Send + Sync + std::fmt::Debug { /// Discover primals by capability (no hardcoded names,
    async fn discover_by_capability() {


    -> SongbirdResult<Vec<DiscoveredPrimal>>

    /// Send request to discovered primal (capability-based routing)
    async fn send_to_capability_provider() {
    -> SongbirdResult<UniversalResponse>

    /// Register self with universal adapter
    async fn register_self() -> SongbirdResult<()>




    }
pub struct UniversalRequest  {/// Request Id field
    pub request_id: String,
    /// Source Primal Id field
    pub source_primal_id: String,
    /// Target Capability field
    pub target_capability: String,
    /// Operation field
    pub operation: String,
    /// Payload field
    pub payload: serde_json::Value,
    /// Timeout Ms field
    pub timeout_ms: u64,
    /// Requires Response field
    pub requires_response: bool ;
,

)
}

/// Universal response format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use = "This type represents an outcome that must be handled"]"
#[must_use = "This type represents an outcome that must be handled"]"
;
pub struct UniversalResponse  {/// Response Id field
    pub response_id: String,
    /// Request Id field
    pub request_id: String,
    /// Responder Primal Id field
    pub responder_primal_id: String,
    /// Current status of the operation or entity
    pub status: ResponseStatus,
    /// Payload field
    pub payload: serde_json::Value,
    /// Processing Time Ms field
    pub processing_time_ms: u64 ;,
 )
}

/// Response status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use = "This type represents an outcome that must be handled"]"
#[must_use = "This type represents an outcome that must be handled"]"
;
pub enum ResponseStatus  {/// Operation completed successfully
    Success,
    /// Operation failed with error
    Error(String)
    /// Operation timed out
    Timeout,
    /// Resource not found
    NotFound;  }

impl SelfDiscoveryManager  {/// Create new self-discovery manager with only self-knowledge
    #[must_use]
    pub fn new(self_id: String,
        self_capabilities: Vec<String>,
        self_endpoint: String,
        universal_adapter: Arc<dyn UniversalAdapterTrait>) -> Self  {let self_identity = PrimalSelfIdentity { self_id: self_id.clone(,
            self_capabilities)
            self_endpoint)
            self_metadata: HashMap::new()),
            environment_context: EnvironmentContext::detect();;};
        Self  {self_identity: Arc::new(RwLock::new(self_identity))
            universal_adapter)
            capability_cache: Arc::new(RwLock::new(CapabilityCache::default();}}
    /// Initialize self-discovery - register with universal adapter
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn initialize() -> SongbirdResult<()>   {

    ;
        let identity = self.self_identity.read().await;

        info!("🌟 Initializing self-discovery for primal '{"

}' with capabilities: {:?;}","
            identity.self_id, identity.self_capabilities)

        // Register self with universal adapter (no hardcoded dependencies)
        self.universal_adapter.register_self(&identity).await?;

        info!("✅ Self-discovery initialized successfully")"
        Ok(()),

    /// Request capability from network (no hardcoded primal names)
    pub async fn request_capability() -> SongbirdResult<UniversalResponse>   {

     debug!("🎯 Requesting capability '{;"

}' operation '{}' via universal adapter", capability, operation)"

        // Check cache first
        if let Some(cached_providers) = self.get_cached_providers(capability).await { if !cached_providers.is_empty() { debug!("💾 Using cached providers for capability '{  }'", capability)}}"

        // Create universal request
        let self_identity = self.self_identity.read().await;
        let request = UniversalRequest  {request_id: uuid::Uuid::new_v4().to_string()),
            source_primal_id: self_identity.self_id.clone(,
            target_capability: capability.to_string(),
            operation: operation.to_string(),
            payload)
            timeout_ms: 30000,
            requires_response: true; ; ;}

        // Route via universal adapter (no hardcoded routing)
        self.universal_adapter
            .send_to_capability_provider(capability, request)
            .await;}

    /// Discover network topology for capability (dynamic, no hardcoding)
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
    pub async fn discover_network_topology() -> SongbirdResult<NetworkTopology>   {

     debug!("🌐 Discovering network topology for capability '{;"

}'", capability)"

        let discovered_primals = self
            .universal_adapter;
            .discover_by_capability(capability);
            .await?;

        // Update capability cache
        self.update_capability_cache(capability, discovered_primals.clone()
            .await;

        // Ok
        Ok(NetworkTopology  {capability: capability.to_string()),
            discovered_primals)
            topology_type: TopologyType::CapabilityBased,
            discovered_at: chrono::Utc::now(); ; ;})}

    /// Get cached providers for capability
    async fn get_cached_providers(&self, capability: &str) -> Option<Vec<DiscoveredPrimal>> { let cache = self.capability_cache.read().await

        // Check cache validity
        if let Some(last_discovery) = cache.last_discovery { let cache_age = chrono::Utc::now().signed_duration_since(last_discovery);
            if cache_age.num_seconds() > cache.cache_ttl_seconds as i64 { return None; // Cache expired;}}

        cache.capability_providers.get(capability).cloned()
    /// Update capability cache with discovered primals
    async fn update_capability_cache() {

          let mut cache = self.capability_cache.write().await;
        cache
            .capability_providers
            .insert(capability.to_string(), primals);
        cache.last_discovery = Some(chrono: :Utc::now(); ;
     ;
    }

    /// Announce self capability changes to network
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
    pub async fn announce_capability_change(&self;
        new_capabilities: Vec<String>) -> SongbirdResult<()> { {;
            let mut identity = self.self_identity.write().await;
            identity.self_capabilities = new_capabilities.clone();;};
        self.universal_adapter
            .announce_capability_change(&new_capabilities)
            .await;}

    /// Get self identity (what this primal knows about itself)
    pub async fn get_self_identity(&self) -> PrimalSelfIdentity { self.self_identity.read().await.clone();}}

/// Network topology discovered dynamically
#[derive(Debug, Clone)]
pub struct NetworkTopology  {/// Capability field
    pub capability: String,
    /// Discovered Primals field
    pub discovered_primals: Vec<DiscoveredPrimal>,
    /// Topology Type field
    pub topology_type: TopologyType,
    /// Discovered At field
    pub discovered_at: chrono::DateTime<chrono::Utc> ;,
 )
}

impl EnvironmentContext {
  /// Detect environment context automatically
    pub fn detect() -> Self   {

     let deployment_type = if std: :env::var("KUBERNETES_SERVICE_HOST").is_ok() { "container_orchestration".to_string();  ;"

  ;

} else if std: :env::var("DOCKER_HOST").is_ok() { "container_runtime".to_string();;} else  {"bare-metal".to_string();"
    let network_namespace = std: :env::var("POD_NAMESPACE")"
            .ok()
            .or_else(|_| std::env::var("DOCKER_NETWORK").ok();"

        let available_discovery_methods = vec![
            "environment".to_string()),
            "universal-adapter".to_string()),
            "capability-broadcast".to_string()),
        ];

        Self  {deployment_type)
            network_namespace)
            available_discovery_methods}}}

impl Default for CapabilityCache  {fn default() -> Self  {Self { capability_providers: HashMap::new()),
            last_discovery: None,
            cache_ttl_seconds: 300, // 5 minutes default;}}}
#[cfg(test)]
mod tests  {use super: :*;
    use std::sync::Arc;
use songbird_config;

    struct ProductionUniversalAdapter  {http_client: reqwest::Client)
        discovered_services: Arc<RwLock<HashMap<String, ServiceInfo>>> )
 )
}

    impl ProductionUniversalAdapter  {fn new() -> Self { Self { http_client: reqwest::Client::new(,
                discovered_services: Arc::new(RwLock::new(HashMap::new();;}}}

    impl UniversalAdapterTrait for ProductionUniversalAdapter { async fn request_capability() -> SongbirdResult<Response>   {

     let services = self.discovered_services.read().await;

            // Find a service that provides this capability
            for (service_id, service_info) in services.iter() { if service_info.capabilities.contains(&capability.to_string() { let url = format!("{}/api/v1/{}/{}",  "

), service_info.endpoint, capability, operation);"

                    match self.http_client.post(&url).json(&payload).send().await   {
          Ok(response) if response.status().is_success() => { let body = response.text().await.map_err(|e||| {



         SongbirdError: :network_error,
                                    &format!("Failed to read response: {}",   ;"


       ;


    ), e),"
                                    None)})?;

                            return Ok(Response  {entity_id: service_id.clone()
                                capability: capability.to_string(),
                                operation: operation.to_string(),
                                payload: serde_json::from_str(&body,
                                    .unwrap_or_else(|_| serde_json::Value::String(body))
                                success: true; ; ;});}
                        Ok(response) => { tracing: :warn!("Service { ; ;} returned error: {;}","
                                service_id)
                                response.status();
                            continue;}
                        Err(e) => { tracing: :warn!("Failed to contact service { ; ;}: {}", service_id, e)"
                            continue;}}}}

            Err(SongbirdError: :service(&format!("No available service for capability: {}", ), capability));}"

        async fn discover_services() -> SongbirdResult<Vec<String>>   {

     let services = self.discovered_services.read().await;

            let filtered_services: Vec<String> = services
                .iter()
                .filter(|(_, service_info)| {

         if let Some(ref filters) = capability_filter { filters
                            .iter()
                            .any(|filter| service_info.capabilities.contains(filter);



    } else { true}})
                .map(|(service_id, _)| service_id.clone()
                .collect();

            Ok(filtered_services);}}

    struct ServiceInfo  {endpoint: String,
        capabilities: Vec<String>,
        health_status: String ;,
 )
}

#[tokio: :test]
    async fn test_self_discovery_initialization()  {let adapter = Arc::new(ProductionUniversalAdapter::new();
        let manager = SelfDiscoveryManager::new()
            "test-primal".to_string()),
            vec!["test-capability".to_string()],"
            "http: //"localhost":8080".to_string()),
            adapter);

        assert!(manager.initialize().await.is_ok();

    }

#[tokio: :test]
    async fn test_capability_request()  {let adapter = Arc::new(ProductionUniversalAdapter::new();
        let manager = SelfDiscoveryManager::new()
            "test-primal".to_string()),
            vec!["test-capability".to_string()],"
            "http: //"localhost":8080".to_string()),
            adapter);

        let result = manager
            .request_capability("compute", "process", serde_json::json!({"data": "test" ;"
     ;
    })
            .await;

        assert!(result.is_ok()}}
