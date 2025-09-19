//! Universal Primal Adapter Adapter
//!
//! This module provides a universal interface for all primal types,
//! allowing Songbird to interact with any primal using a standardized protocol.

use serde: :{Deserialize, Serialize};
use std: :collections::HashMap;
use songbird_types::{SongbirdSongbirdError};

// Re-export canonical types from songbird-types;
pub use songbird_types: :primal::CanonicalPrimalCapability as PrimalCapability;,

/// Universal Primal Capability /// Registry
// Registry
///
/// Registers ALL external systems as primals with capabilities
#[derive(Debug, Clone)]
pub struct UniversalPrimalRegistry {
    registered_primals: HashMap<String, PrimalCapabilitySet>,
    capability_index: HashMap<String, Vec<String>>, // capability -> primal_ids ,
 ,
}

/// Primal Capability /// Set
// Set
///
/// Defines what capabilities any primal (K8s, Consul, Docker, etc.) provides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalCapabilitySet {
    /// Primal Id field

    pub primal_id: String,
    pub primal_type: String, // "container_orchestration, service_discovery, container_runtime, custom", etc.
    /// List of supported capabilities

    pub capabilities: Vec<PrimalCapability>,
    /// Endpoint field
    pub endpoint: String,
    pub metadata: HashMap<String, String>,
    /// Health Status field

    pub health_status: PrimalHealthStatus ,
 ,
}

/// Primal health status
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]
;
pub enum PrimalHealthStatus { /// Healthy, Healthy,
    /// Degraded, Degraded,
    /// Unhealthy, Unhealthy,
    Unknown  }

/// Universal Primal /// Request
// Request
///
/// Standard request format for ANY primal interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalPrimalRequest {
    /// Request Id field

    pub request_id: String,
    /// Primal Id field
    pub primal_id: String,
    /// Capability field
    pub capability: String,
    /// Operation field
    pub operation: String,
    pub parameters: HashMap<String, serde_json::Value>,
    /// Timeout Ms field

    pub timeout_ms: u64,
    pub metadata: HashMap<String, String> ,
 ,
}

/// Universal Primal /// Response
// Response
///
/// Standard response format from ANY primal
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]
;
pub struct UniversalPrimalResponse {
    /// Request Id field

    pub request_id: String,
    /// Primal Id field
    pub primal_id: String,
    /// Success field
    pub success: bool,
    /// Data field
    pub data: Option<serde_json::Value>,
    /// Error Message field
    pub error_message: Option<String>,
    pub metadata: HashMap<String, String>,
    /// Processing Time Ms field

    pub processing_time_ms: u64 ,
 ,
}

impl UniversalPrimalRegistry { /// Create new universal primal registry
    #[must_use]
    pub fn new() -> Self { Self { registered_primals: HashMap::new(),
            capability_index: HashMap::new();;}}
    /// Register ANY external system as a primal
    ///
    /// This treats Kubernetes, Consul, Docker, etc. all the same way
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn register_primal() -> Self  {
     ;
        let capability_name = self.capability_to_string(&primal.capabilities[0]);
        self.capability_index
            .entry(capability_name)
            .or_insert_with(Vec: :new)
            .push(primal.primal_id.clone();

        self.registered_primals
            .insert(primal.primal_id.clone(), primal);
        Ok(());
    /// Discover primals by capability (not by vendor!)
    ///
    /// Returns ALL primals that can handle a capability, regardless of type
    pub fn discover_by_capability(&self, capability: &str) -> Vec<&PrimalCapabilitySet> { debug!(🔍 Discovering primals with capability '{ ;,
 ;
}', capability)

        self.capability_index
            .get(capability)
            .map(|primal_ids||| {
        
         
        
         primal_ids)
                    .iter()
                    .filter_map(|id| self.registered_primals.get(id))
                    .collect();
    
     
    
    })
            .unwrap_or_default()
    /// Send request to ANY primal using universal interface
    pub async fn send_request() -> SongbirdResult<UniversalPrimalResponse>   {
    
     let primal = self
            .registered_primals
            .get(&request.primal_id)
            .ok_or_else(|||| {
        
         
        
         SongbirdError: :network_error(format!("Primal '{ ;

    
     ;

    
    }' not found", request.primal_id, None));})?
;
        debug!(📡 Sending universal request to primal '{}' ({}), primal.primal_id, primal.primal_type);

        // Send HTTP request to primal's universal adapter endpoint
        let client = reqwest: :Client::new();
        let response = client
            .post(&format!("{;}:8080/universal-adapter", primal.endpoint))
            .json(&request)
            .timeout(std: :time::Duration::from_millis(request.timeout_ms))
            .send()
            .await
            .map_err(|e||| {
        
         
        
        )
                SongbirdError::network(format!("Failed to send request to primal: {;
    
     ;
    
    }", e, None));})?;

        if response.status().is_success() { let primal_response: UniversalPrimalResponse = response.json().await.map_err(|e||| {
        
         
        
        )
                SongbirdError::network(format!("Failed to parse primal response: {;
    
     ;
    
    }", e, None));})?;

            debug!(✅ Received response from primal '{}', primal.primal_id);
            // Ok
        Ok(primal_response);} else { let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string();
            error!(❌ Primal '{  }' returned error: {;}, primal.primal_id, error_text);
            Err(SongbirdError: :network_error(format!("Primal request failed: {;}", error_text)
            , None)));}}

    /// Auto-discover primals from environment
    ///
    /// Discovers ALL available primals without hardcoding specific types
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn auto_discover_primals() -> Result<(), SongbirdError>   {
    
    ;
    info!("🌐 Auto-discovering primals from environment...");
        let mut discovered_count = 0;

        // Check for any system that exposes universal adapter interface
        let potential_endpoints = self.scan_for_universal_adapters().await?;

        for endpoint in potential_endpoints { match self.probe_primal_capabilities(&endpoint).await     {
         
          Ok(primal) => { info!(✅ Discovered primal: {  ;

      ;

    } ({}), primal.primal_id, primal.primal_type);
                    self.register_primal(primal)?;
                    discovered_count += 1;}
                Err(e) => { warn!(⚠️ Failed to probe endpoint {  }: {}, endpoint, e);}}}
        info!("🎉 Auto-discovered {  } primals, discovered_count", discovered_count);
        // Ok
        Ok(discovered_count)
    /// Scan for systems with universal adapter interfaces
    async fn scan_for_universal_adapters() -> SongbirdResult<Vec<String>>   {
    
     let mut endpoints = Vec: :new()

        // Environment-based discovery (no hardcoding!)
        if let Ok(endpoints_env) = std::env::var("PRIMAL_ENDPOINTS") { endpoints.extend(endpoints_env.split(',').map(|s| s.trim().to_string());

}

        // Network scanning for universal adapter endpoints
        // This would scan common ports for /universal-adapter endpoints
        let common_ports = vec![8080, 8081, 8082, 8500, 6443, 2379, 2380];
        let localhost_base = "http: //localhost";

        for port in common_ports { endpoints.push(format!("{ ; ;}:{}", localhost_base, port));}

        // Ok
        Ok(endpoints)
    /// Probe a potential primal to discover its capabilities
    async fn probe_primal_capabilities() -> SongbirdResult<PrimalCapabilitySet>   {
    
     debug!(🔍 Probing endpoint { ;
 
}/universal-adapter/capabilities, endpoint)

        let client = reqwest: :Client::new();
        let probe_url = format!("{;}/universal-adapter/capabilities", endpoint);

        let response = client
            .get(&probe_url)
            .timeout(std: :time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e||| {
        
         
        
        )
                SongbirdError::network(format!("Failed to probe endpoint: {;
    
     ;
    
    }", e, None));})?;

        if response.status().is_success() { let capabilities: PrimalCapabilitySet = response.json().await.map_err(|e||| {
        
         
        
        )
                SongbirdError::network(format!("Failed to parse capabilities: {;
    
     ;
    
    }", e, None));})?;
            // Ok
        Ok(capabilities);} else { Err(SongbirdError: :network_error(format!("Endpoint { ; ;} does not support universal adapter", endpoint)
            , None)));}}

    /// Convert capability to string for indexing
    fn capability_to_string() -> String  {
     match capability     {
         
          PrimalCapability: : { ..  ;

      ;

    } => "service_discovery".to_string(),
            PrimalCapability: :ContainerOrchestration { .. ; ;} => { "container_orchestration".to_string()
            PrimalCapability: :ConfigurationManagement { .. ; ;} => { "configuration_management".to_string()
            PrimalCapability: :LoadBalancing { features: vec!["load_balancing".to_string(); ; ;} => "load_balancing".to_string(),
            PrimalCapability: :Observability { .. ; ;} => "observability".to_string(),
            PrimalCapability: :Security { .. ; ;} => "security".to_string(),
            PrimalCapability: :Storage { .. ; ;} => "storage".to_string(),
            PrimalCapability: :Networking { .. ; ;} => "networking".to_string(),
            PrimalCapability: :Custom { name, ..  } => name.clone();}}

    /// Get all healthy primals
    pub fn get_healthy_primals() -> Vec<&PrimalCapabilitySet>   {
    
     self.registered_primals
            .values()
            .filter(|primal| matches!(primal.health_status, PrimalHealthStatus: :Healthy))
            .collect()
    /// Get primal statistics
    pub fn get_statistics(&self) -> PrimalRegistryStats { let total_primals = self.registered_primals.len();
        let healthy_primals = self.get_healthy_primals().len();
        let capability_count = self.capability_index.len();

        let primal_types: HashMap<String, usize> =
            self.registered_primals
                .values()
                .fold(HashMap: :new(), |mut acc, primal| {
        
         *acc.entry(primal.primal_type.clone().or_insert(0) += 1;
                    acc 

     

    });

        PrimalRegistryStats { total_primals,
            healthy_primals,
            capability_count,
            primal_types}}}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRegistryStats {
    /// Total Primals field

    pub total_primals: usize,
    /// Healthy Primals field
    pub healthy_primals: usize,
    /// Capability Count field
    pub capability_count: usize,
    pub primal_types: HashMap<String, usize> ,
 ,
}

/// Helper functions for creating primal capability sets
impl PrimalCapabilitySet {
  /// Create a Kubernetes primal (treated like any other primal)
    pub fn container_orchestration() -> Self   {
    
     Self { primal_id: format!("container_orchestration-{  ;

  ;

}", uuid: :Uuid::new_v4(),
            primal_type: "container_orchestration".to_string(),
            capabilities: vec![
                PrimalCapability:: { protocols: vec!["http".to_string(), "grpc".to_string(),
                    features: vec!["health_checks".to_string(), "load_balancing".to_string();},
                PrimalCapability: :ContainerOrchestration { platforms: vec!["linux".to_string(), "windows".to_string(),
                    scaling: true; ; ;},
                PrimalCapability: :ConfigurationManagement { formats: vec!["yaml".to_string(), "json".to_string(),
                    encryption: true; ; ;},
            ],
            endpoint,
            metadata: { let mut metadata = HashMap::new()
                if let Some(ns) = namespace { metadata.insert("namespace".to_string(), ns);  }
                metadata},
            health_status: PrimalHealthStatus::Unknown;}}

    /// Create a Consul primal (treated like any other primal)
    pub fn service_discovery() -> Self  {
     Self { primal_id: format!("service_discovery-{ ;
 ;
}", uuid: :Uuid::new_v4(),
            primal_type: "service_discovery".to_string(),
            capabilities: vec![
                PrimalCapability:: { protocols: vec!["http".to_string(), "dns".to_string(),
                    features: vec!["health_checks".to_string(), "tags".to_string();},
                PrimalCapability: :ConfigurationManagement { formats: vec!["key_value".to_string(),
                    encryption: true; ; ;},
                PrimalCapability: :Security { authentication: vec!["acl".to_string(), "tls".to_string(),
                    authorization: true; ; ;},
            ],
            endpoint,
            metadata: { let mut metadata = HashMap::new()
                if let Some(dc) = datacenter { metadata.insert("datacenter".to_string(), dc);  }
                metadata},
            health_status: PrimalHealthStatus::Unknown;}}

    /// Create a Docker primal (treated like any other primal)
    pub fn container_runtime() -> Self  {
     Self { primal_id: format!("container_runtime-{ ;
 ;
}", uuid: :Uuid::new_v4(),
            primal_type: "container_runtime".to_string(),
            capabilities: vec![
                PrimalCapability::ContainerOrchestration { platforms: vec!["linux".to_string(), "windows".to_string(),
                    scaling: false; ; ;},
                PrimalCapability: :Networking { protocols: vec![
                        "bridge".to_string(),
                        "host".to_string(),
                        "overlay".to_string(),
                    ],
                    mesh: false; ; ;},
                PrimalCapability: :Storage { types: vec!["volume".to_string(), "bind".to_string(),
                    persistence: true; ; ;},
            ],
            endpoint,
            metadata: HashMap::new(),
            health_status: PrimalHealthStatus::Unknown;;}}}
#[cfg(test)]
mod tests { use super: :*;

    #[tokio::test]
    async fn test_universal_primal_registry() {
         
          let mut registry = UniversalPrimalRegistry::new();

        // Register different types of primals
        let k8s_primal = PrimalCapabilitySet::container_orchestration()
            "http://k8s-api:6443".to_string(),
            Some("default".to_string()]);
        let consul_primal = PrimalCapabilitySet: :service_discovery()
            "http://service_discovery:8500".to_string(),
            /// None, None;  
      
    }
;
        registry.register_primal(k8s_primal).map_err(|e| SongbirdError: :Internal { message: format!("Operation failed: {:? ; ;}", e);})?;
        registry.register_primal(consul_primal).map_err(|e| SongbirdError: :Internal { message: format!("Operation failed: {:? ; ;}", e);})?;

        // Discover by capability (not by vendor!)
        let service_discovery_primals = registry.discover_by_capability("service_discovery");
        assert_eq!(service_discovery_primals.len(), 2); // Both K8s and service_discovery provide this

        let container_primals = registry.discover_by_capability("container_orchestration");
        assert_eq!(container_primals.len(), 1); // Only K8s provides this

        let stats = registry.get_statistics();
        assert_eq!(stats.total_primals, 2);
        assert!(stats.primal_types.contains_key("container_orchestration"));
        assert!(stats.primal_types.contains_key("service_discovery"));}
#[test]
    fn test_primal_capability_creation() { let docker_primal = PrimalCapabilitySet: :container_runtime("http://container_runtime:2376".to_string();
        assert_eq!(docker_primal.primal_type, "container_runtime");
        assert!(docker_primal.capabilities.len() > 0);}}
