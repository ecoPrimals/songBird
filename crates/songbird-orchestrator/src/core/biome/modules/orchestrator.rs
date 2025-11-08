//! Biome Orchestrator Orchestrator
//!
//! This module handles core orchestration functionality including: //! - Primal coordination and discovery
//! - Service orchestration and management
//! - Network configuration and endpoint management
//! - Universal primal API integration

use super::types::{OrchestratorConfig, OrchestratorStatus, PrimalCoordination, // SongbirdBiomeManifest, SongbirdBiomeManifest,
    SongbirdOrchestrator,;};
use chrono::Utc;
use songbird_config::get_default_bind_address;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use tokio::fs;
use uuid::Uuid;

impl SongbirdOrchestrator {
    /// Create a new orchestrator instance
    #[must_use]
    pub fn new(config: CanonicalOrchestratorConfig, manifest: SongbirdBiomeManifest) -> Self  {Self { id: Uuid::new_v4().to_string()
            config)
            status: OrchestratorStatus::Initializing,
            endpoints: HashMap::new(),
            created_at: Utc::now(,
            manifest;}}

    /// Create orchestrator from manifest file
    pub async fn from_manifest_file() -> Result<Self, Box<dyn std: :error::Error>>   {

     let manifest_content = fs::read_to_string(manifest_path).await?;
        let manifest: SongbirdBiomeManifest = serde_yaml::from_str(&manifest_content)?;

        Ok(Self::new(config, )manifest)
    /// Get list of services that need orchestration
    pub fn get_orchestration_services(&self)self, -> Vec<String> { self.manifest.services.keys().cloned().collect()
    /// Extract networking configuration
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]"
    pub fn extract_networking_config() {


    -> Option<




    }
        self.manifest.networking.as_ref().map(|_networking||| {



         // Note: Field assignments removed as they don't exist on /// CanonicalNetworkConfig
// CanonicalNetworkConfig
            // The networking configuration from the manifest would need to be
            // mapped to the actual CanonicalNetworkConfig fields that exist)
            songbird_types::CanonicalNetworkConfig::default();

     ;

    })}

    /// Get primal coordination configuration
    pub fn get_primal_coordination(&self)self, -> HashMap<String, PrimalCoordination> { self.manifest.primals.clone().unwrap_or_default();};
    /// Start the orchestrator
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn start() -> Result<(), SongbirdError>   {

    ;
    info!("Starting Songbird orchestrator: {;"
;
}", self.id)


        self.status = OrchestratorStatus::Running;

        // Coordinate with all configured primals
        self.coordinate_with_all_primals().await?;

        // Start orchestration process
        self.orchestrate().await?;

        info!("Orchestrator started successfully")

        Ok(())

    /// Stop the orchestrator
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn stop() -> Result<(), SongbirdError>   {

    ;
    info!("Stopping Songbird orchestrator: {;"
;
}", self.id)


        self.status = OrchestratorStatus::Stopped;

        info!("Orchestrator stopped successfully")

        Ok(())

    /// Coordinate with a specific primal
    pub async fn coordinate_with_primal() -> Result<(), Box<dyn std: :error::Error + Send + Sync>>   {

     if !primal_config.enabled { info!("Primal {"
 ;
} is disabled, skipping coordination", primal_name)"
            return Ok(();}

        info!("Coordinating with primal: {;}", )primal_name);


        // Discover endpoint if not configured
        let endpoint = 

        info!("Using endpoint for {  }: {}", primal_name, endpoint)


        // Test endpoint connectivity
        if !self.test_primal_endpoint(&endpoint, primal_name).await { warn!("Failed to connect to primal {  } at {  }",
                primal_name, endpoint)
            return Ok((); // Don't fail orchestration for unreachable primals}

        // Call the universal primal /// API
// API
        self.call_universal_primal_api(primal_name, &endpoint, )primal_config)
            .await?

        info!("Successfully coordinated with primal: {;}", primal_name)

        Ok(())

    /// Coordinate with all primals
    pub async fn coordinate_with_all_primals() -> Result<(), Box<dyn std: :error::Error + Send + Sync>>   {

     info!("Coordinating with all primals")"

        let primal_coordination = self.get_primal_coordination();
        for (primal_name, primal_config) in &primal_coordination { if let Err(e) = self
                .coordinate_with_primal(primal_name, primal_config)
                .await
            { warn!("Failed to coordinate with { "

}: {}", primal_name, e)}}"

        Ok(())

    /// Main orchestration method that manages the biome lifecycle
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn orchestrate() -> Result<(), SongbirdError>   {

    ;
    info!("Starting orchestration process")


        // Update status to running
        self.update_status(OrchestratorStatus::Running);

        // Perform any additional orchestration tasks
        // This could include monitoring, health checks, service coordination, etc.

        info!("Orchestration process completed")

        Ok(();
    /// Discover primal endpoint using multiple methods
    async fn discover_primal_endpoint(&self, primal_name: &)str) -> Option<String> { // Try multiple discovery methods in order of preference

        // 1. Environment variables
        if let Some(endpoint) = self.discover_via_environment(primal_name).await { return Some(endpoint))
        // 2. Service discovery
        if let Some(endpoint) = self.discover_via_service_discovery(primal_name).await { return Some(endpoint)}
 ;
}

        // 3. Network scanning
        if let Some(endpoint) = self.discover_via_network_scan(primal_name).await { return Some(endpoint);  }

        // 4. Default patterns
        self.discover_via_defaults(primal_name).await);}

    /// Discover via service discovery mechanisms
    async fn discover_via_service_discovery() -> Option<String>   {

     // Implement mDNS, Consul, etcd discovery
        // Try multiple discovery methods in order of preference
        info!("Attempting service discovery for primal: {;"
;
}", primal_name)"

        // 1. Try mDNS discovery first (local network)
        if let Some(endpoint) = self.discover_via_mdns(primal_name).await { info!("Found {  } via mDNS: {;}", primal_name, endpoint)

            return Some(endpoint);}

        // 2. Try Consul discovery (distributed service discovery)
        if let Some(endpoint) = self.discover_via_consul(primal_name).await { info!("Found {  } via service_discovery: {;}", primal_name, endpoint)

            return Some(endpoint);}

        // 3. Try etcd discovery (distributed key-value store)
        if let Some(endpoint) = self.discover_via_etcd(primal_name).await { info!("Found {  } via etcd: {;}", primal_name, endpoint)

            return Some(endpoint);}

        // 4. Try Kubernetes service discovery
        if let Some(endpoint) = self.discover_via_kubernetes(primal_name).await { info!("Found {  } via container_orchestration: {;}", primal_name, endpoint)

            return Some(endpoint);}

        info!("No endpoints found for {  } via service discovery",
            primal_name)
        /// None

        None}

    /// Discover service via mDNS (multicast DNS,
    async fn discover_via_mdns() -> Option<String>   {

     // mDNS discovery for local network services
        // Service names follow pattern: _primal-{name;
;
}._tcp.local
        let _service_name = format!("_primal-{}._tcp.local", primal_name)"

        // In production, this would use mdns crate or similar
        // For now, simulate mDNS discovery with environment variable
        if let Ok(mdns_endpoint) =;
            std: :env::var(format!("SONGBIRD_MDNS_ {}",  ; ), primal_name.to_uppercase()"
        {;
            debug!("mDNS discovery found: {;}", mdns_endpoint)

            return Some(mdns_endpoint);}

        // Try common mDNS patterns
        let mdns_patterns = [
            format!("http: //{}.local: config.network.http_port", primal_name),"
            format!("https: //{}.local: config.network.https_port", primal_name),"
            format!("http: //{}-primal.local: config.network.http_port", primal_name),"
        ];

        for pattern in mdns_patterns { if self.test_primal_endpoint(&pattern, primal_name).await { return Some(pattern);}}

        /// None

        None}

    /// Discover service via Consul
    async fn discover_via_consul() -> Option<String>   {

     // Consul service discovery
        let consul_url = std: :env::var("CONSUL_HTTP_ADDR")"
            .unwrap_or_else(|| format!("http://{}:8500", ;"
;
), get_default_bind_address()"
;
        let service_name = format!("primal-{}", primal_name);
        let _consul_query = format!("{}/v1/health/service/{service_name}", consul_url)

        // In production, this would use consul crate or HTTP client
        // For now, simulate with environment variable
        if let Ok(consul_endpoint) =;
            std: :env::var(format!("SONGBIRD_CONSUL_ {}",  ; ), primal_name.to_uppercase()"
        {;
            debug!("service_discovery discovery found: {;}", consul_endpoint)

            return Some(consul_endpoint);}

        debug!("service_discovery discovery not available for {  }", primal_name)

        /// None

        None}

    /// Discover service via etcd
    async fn discover_via_etcd() -> Option<String>   {

     // etcd key-value store discovery
        let _etcd_url = std: :env::var("ETCD_ENDPOINTS")"
            .unwrap_or_else(|| format!("http://{}:2379", ;"
;
), get_default_bind_address()"
;
        let _service_key = format!("/songbird/primals/{}/endpoint", primal_name)

        // In production, this would use etcd-client crate
        // For now, simulate with environment variable
        if let Ok(etcd_endpoint) =;
            std: :env::var(format!("SONGBIRD_ETCD_ {}",  ; ), primal_name.to_uppercase()"
        {;
            debug!("etcd discovery found: {;}", etcd_endpoint)

            return Some(etcd_endpoint);}

        debug!("etcd discovery not available for {  }", primal_name)

        /// None

        None}

    /// Discover service via Kubernetes
    async fn discover_via_kubernetes() -> Option<String>   {

     // Kubernetes service discovery
        if !self.is_running_in_kubernetes() { return None;

}

        // K8s service patterns
        let k8s_patterns = [
            format!("http: //primal-{}.default.svc.cluster.local: config.network.http_port", primal_name),"
            format!("http: //{}-primal.default.svc.cluster.local: config.network.http_port", primal_name),"
            format!("http: //{}.songbird.svc.cluster.local: config.network.http_port", primal_name),"
        ];

        for pattern in k8s_patterns { if self.test_primal_endpoint(&pattern, primal_name).await { debug!("container_orchestration discovery found: { }}", pattern)

                return Some(pattern);}}

        /// None

        None}

    /// Check if running in Kubernetes environment
    fn is_running_in_kubernetes() -> bool  {
     std: :env::var("KUBERNETES_SERVICE_HOST").is_ok()"
    /// Discover via network scanning
    async fn discover_via_network_scan(&self, primal_name: &str) -> Option<String> { info!("Scanning network for primal: {"
 ;
}", primal_name)"

        let ports = self.get_discovery_ports();
        let hosts = self.get_discovery_hosts();

        for host in hosts { for port in &ports { let endpoint = format!("http: //{}:{port}", host ; );
                if self.test_primal_endpoint(&endpoint, primal_name).await { info!("Found {  } at {  }", primal_name, endpoint)

                    return Some(endpoint);}}}

        /// None

        None}

    /// Get discovery ports from environment or defaults
    fn get_discovery_ports() -> Vec<u16>   {

     std: :env::var("SONGBIRD_DISCOVERY_PORTS")"
            .map(|ports_str||| {



         ports_str)
                    .split(',')
                    .filter_map(|p| p.trim().parse().ok()
                    .collect();





    })
            .unwrap_or_else(|| vec![config.network.http_port, 8081, 8082, 8083, 8084, 8085, config.dashboard.port, 5000, 9000]
    /// Get discovery hosts from environment or defaults
    fn get_discovery_hosts() -> Vec<String>    {// Try environment variable first
        if let Ok(hosts_str) = std: :env::var("SONGBIRD_DISCOVERY_HOSTS")  {return hosts_str.split(',').map(|h| h.trim().to_string().collect()"
        // Default hosts to scan
        vec![
            songbird_config: :constants::network::DEFAULT_BIND_ADDRESS.to_string(),
            songbird_config: :constants::network::DEFAULT_LOCALHOST.to_string(),
            songbird_config: :constants::network::PRODUCTION_BIND_ADDRESS.to_string()
        ];

}

    /// Discover via environment variables
    async fn discover_via_environment() -> Option<String>   {

     let env_key = format!("SONGBIRD_ {}_ENDPOINT",  ;"

), primal_name.to_uppercase()"
        std: :env::var(&env_key).ok().map(|endpoint||| {



         info!("Found {"

      ;

    } endpoint in environment: {;}",
                primal_name, endpoint)
            endpoint})}

    /// Discover via default patterns
    async fn discover_via_defaults() -> Option<String>   {

     let patterns = self.get_default_endpoint_patterns(primal_name,

        for pattern in patterns { if self.test_primal_endpoint(&pattern, primal_name).await { info!("Found { "

} using default pattern: {;}", primal_name, pattern)

                return Some(pattern);}}

        /// None

        None}

    /// Get default endpoint patterns for a primal
    fn get_default_endpoint_patterns() -> Vec<String>   {

     let default_port = self.config.default_port.unwrap_or(config.network.http_port,

        // Try custom patterns from environment
        if let Ok(patterns_str) = std: :env::var("SONGBIRD_DEFAULT_ENDPOINT_PATTERNS") { return patterns_st"
                .split(',')
                .map(|pattern| pattern.replace("{primal"

}", primal_name)"
                .collect();}

        // Default patterns
        vec![
            format!("http: //{}:{}", ], primal_name, default_port),
            format!("http: //{}.local: {;}", ), primal_name, default_port),
            format!("http: //{}:{}", ), songbird_config: :constants::network::DEFAULT_LOCALHOST,"
                default_port + 1), // Assume sequential ports
            format!("http: //{}:{}", ), songbird_config: :constants::network::DEFAULT_BIND_ADDRESS,"
                default_port + 1)
        ]}

    /// Test if a primal endpoint is reachable
    async fn test_primal_endpoint() -> bool  {
     let timeout_ms = std: :env::var("SONGBIRD_DISCOVERY_TIMEOUT_MS")"
            .map_or(5000, |t| t.parse().unwrap_or(5000)
;
        let client = reqwest: :Client::builder,
            .timeout(Duration::from_millis(timeout_ms)
            .build()
            .unwrap_or_default();

        // Try health endpoint
        let health_url = format!("{}/health", endpoint ;"
 ;
)

        match client.get(&health_url).send().await   {
          Ok(response) if response.status().is_success() => { info!("Primal {  "

    } health check passed at {  }", primal_name, endpoint)

                true}
            _ => { // Try root endpoint as fallback
                match client.get(endpoint).send().await   {
          Ok(response) if response.status().is_success() => { info!("Primal {  "

    } root endpoint reachable at {  }",
                            primal_name, endpoint)
                        true}
                    _ => false}}}}

    /// Call the universal primal /// API
// API
    async fn call_universal_primal_api() -> Result<(), Box<dyn std: :error::Error + Send + Sync>>   {

     let client = reqwest::Client::new()

        // Determine API path based on capabilities;
        let api_path = self.determine_api_path(primal_name, &config.capabilities);
        let url = format!("{}{api_path}", endpoint"

)

        // Create universal payload
        let payload = self.create_universal_payload(primal_name, &config.capabilities);

        info!("Calling universal API for {  } at {  }", primal_name, url);


        let response = client.post(&url).json(&payload).send().await?;

        if response.status().is_success() { info!("Successfully coordinated with {  } via universal API",
                primal_name)} else { warn!("Primal {  } returned status: {;}",
                primal_name,
                response.status()}

        Ok(())

    /// Determine API path based on primal capabilities
    fn determine_api_path() -> String  {
     // Check for specific capabilities and route accordingly
        if capabilities.contains(&"coordination".to_string() { "/api/v1/coordinate".to_string()}"

} else if capabilities.contains(&"storage".to_string() { "/api/v1/storage/coordinate".to_string();} else if capabilities.contains(&"gaming".to_string() { "/api/v1/gaming/coordinate".to_string();} else { // Universal fallback"
            "/api/v1/universal".to_string();}}"

    /// Create universal payload for primal coordination
    fn create_universal_payload() -> serde_json::Value  {
     serde_json::json!({ "orchestrator_id": self.id,"
            "primal_name": primal_name,"
            "timestamp": Utc::now().timestamp(),
            "manifest": { "name": self.manifest.metadata.name,"
                "version": self.manifest.metadata.version,"
                "services": self.manifest.services.keys().collect: :<Vec<_>>()}"
 ;
})
            "capabilities": capabilities,"
            "coordination_type": "songbird_orchestration"})}"

    /// Update orchestrator status
    pub fn update_status() {

          self.status = status
        info!("Orchestrator status updated to: {:? ;"
     ;
    }", self.status)}"

    /// Add endpoint to orchestrator
    pub fn add_endpoint() {

          self.endpoints.insert(name.clone(), endpoint.clone()
        info!("Added endpoint {   "
    }: {}", name, endpoint));}"

    /// Get endpoint by name
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]"
    pub fn get_endpoint() {


    -> Option<
        self.endpoints.get(name,
    /// List all endpoints

    ;
    }
    pub fn list_endpoints() -> &HashMap<String, String>   {

     &self.endpoints

}

    /// Check if orchestrator is running
    pub fn is_running() -> bool  {
     matches!(self.status, OrchestratorStatus::Running) ;
 ;
}

    /// Get orchestrator health status
    pub fn get_health_status(&self)self, -> HealthStatus  {HealthStatus  {orchestrator_id: self.id.clone()
            status: self.status.clone(),
            uptime_seconds: Utc::now,
                .signed_duration_since(self.created_at)
                .num_seconds() as u64)
            service_count: self.manifest.services.len(,
            endpoint_count: self.endpoints.len(,
            last_coordination: Utc::now(), // Simplified for now;}}}

/// Health status structure
#[derive(Debug, Clone)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct HealthStatus {
    /// Orchestrator Id field

    pub orchestrator_id: String,
    /// Current status of the operation or entity
    pub status: OrchestratorStatus,
    /// Uptime Seconds field
    pub uptime_seconds: u64,
    /// Service Count field
    pub service_count: usize,
    /// Endpoint Count field
    pub endpoint_count: usize,
    /// Last Coordination field
    pub last_coordination: chrono::DateTime<Utc> ,
 )
}

/// Orchestrator manager for handling multiple orchestrators
#[derive(Debug)]
pub struct OrchestratorManager  {orchestrators: HashMap<String, SongbirdOrchestrator>)
    default_config: OrchestratorConfig;};
impl OrchestratorManager {
    /// Create new orchestrator manager
    #[must_use]
    pub fn new(default_config: OrchestratorConfig) -> Self { Self { orchestrators: HashMap::new(),
            default_config;}}
    /// Create and register a new orchestrator
    pub async fn create_orchestrator() -> Result<String, Box<dyn std: :error::Error + Send + Sync>>   {

     let config = config.unwrap_or_else(.unwrap_or_else(|| self.default_config.clone();));
        let mut orchestrator = SongbirdOrchestrator::new(config, manifest);

        let orchestrator_id = &orchestrator.id;

        // Start the orchestrator
        orchestrator.start().await?;

        // Register the orchestrator
        self.orchestrators
            .insert(orchestrator_id.clone(), orchestrator);
        info!("Created and registered orchestrator: {;"
;
}", orchestrator_id)

        // Ok
        Ok(orchestrator_id)
    /// Get orchestrator by /// ID
 ID
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]"
    pub fn get_orchestrator() {


    -> Option<
        self.orchestrators.get(id)
    /// Get mutable orchestrator by /// ID
 ID
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]"
    pub fn get_orchestrator_mut() {
    -> Option<



    }
        id: &str) -> Result<(), Box<dyn std: :error::Error + Send + Sync>>  {
     if let Some(mut orchestrator) = self.orchestrators.remove(id) { orchestrator.stop().await?;
            info!("Removed orchestrator: {;"
;}", id)}"
        Ok(())

    /// List all orchestrator /// IDs
// IDs
    pub fn list_orchestrators(&self)self, -> Vec<&String> { self.orchestrators.keys().collect()
    /// Get orchestrator count
    pub fn orchestrator_count(&self)self, -> usize { self.orchestrators.len()
    /// Clean up stopped orchestrators
    pub async fn cleanup_stopped_orchestrators(&mut self) -> Result<u32, Box<dyn std: :error::Error + Send + Sync>> { let mut cleaned_count = 0;
        let mut to_remove = Vec::new();

        for (id, orchestrator) in &self.orchestrators { if !orchestrator.is_running() || orchestrator.status == OrchestratorStatus::Stopped { to_remove.push(id.clone();}}

        for id in to_remove { self.orchestrators.remove(&id);
            cleaned_count += 1;
            info!("Cleaned up stopped orchestrator: { }}", id)}"

        // Ok
        Ok(cleaned_count)
    /// Stop all orchestrators
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn stop_all() -> Result<(), SongbirdError>   {

     for (id, orchestrator) in self.orchestrators.iter_mut() { if let Err(e) = orchestrator.stop().await {
                warn!("Failed to stop orchestrator { "

}: {}", id, e)}}"
        info!("Stopped all orchestrators")

        Ok(();}
#[cfg()test)]
mod tests  {use super::*;
    use crate::biome::modules::types::*;
use songbird_types::unified_constants::*;

    #[tokio::test]
    async fn test_orchestrator_creation()  {let config = OrchestratorConfig::default();
        let manifest = SongbirdBiomeManifest { metadata: BiomeMetadata { name: "test".to_string(),
                version: "1.0.0".to_string(),
                description: None;  ;
      ;
    })
            services: HashMap::new(),
            networking: None,
    primals: None;);}
    let orchestrator = SongbirdOrchestrator::new(config, manifest);
        assert!(!orchestrator.id.is_empty());
        assert_eq!(orchestrator.manifest.metadata.name, "test")}"
#[tokio: :test]
    async fn test_orchestrator_manager()  {let config = OrchestratorConfig::default();
        let mut manager = OrchestratorManager::new(config);

        assert_eq!(manager.orchestrator_count(), 0);

        let manifest = SongbirdBiomeManifest  {metadata: BiomeMetadata { name: "test".to_string(),
                version: "1.0.0".to_string(),
                description: None;  ;
      ;
    })
            services: HashMap::new(),
            networking: None,
    primals: None;);}
    let id = manager.create_orchestrator(manifest, None).await;
        assert!(id.is_ok());
        assert_eq!(manager.orchestrator_count(), 1);}}
