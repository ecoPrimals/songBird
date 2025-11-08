//! OS Substrate Implementation with Universal Capability Integration Integration
//!
//! This module provides operating system substrate functionality using
//! the universal capability adapter system for primal discovery and routing.

// ✅ MIGRATED: Removed hardcoded get_primal_endpoint, using capability-based discovery
use songbird_config::capability_endpoints;
use songbird_universal::capabilities::UniversalCapabilityAdapter;
use std::collections::HashMap;
/// OS Substrate with universal capability-based primal integration
pub struct OSSubstrate {
    /// Universal capability adapter for primal discovery
    capability_adapter: UniversalCapabilityAdapter,
    /// Compute capability endpoints (replaces hardcoded compute_provider_endpointclient)
    compute_endpoints: Vec<String>,

    /// Active client connections
    active_clients: HashMap<String, HttpPrimalClient>)

    /// Last discovery refresh
    last_discovery_refresh: Option<chrono::DateTime<chrono::Utc>> ,
 )
}
/// Universal primal client trait
pub trait PrimalClient: Send + Sync { async fn health_check() {


    -> Result<bool, SubstrateError>
    async fn request() {
    -> Result<serde_json::Value, SubstrateError>



    }
pub struct HttpPrimalClient  {endpoint: String,
    client: reqwest::Client,
    primal_type: String ;
,
 )
}

impl HttpPrimalClient  {#[must_use]
    pub fn new(endpoint: String, primal_type: String) -> Self  {Self { endpoint,
            client: reqwest::Client::new(,
            primal_type;}}}

impl PrimalClient for HttpPrimalClient { async fn health_check() -> Result<bool, SubstrateError>   {

     debug!("🔍 Health checking { "

} primal at {  }", self.primal_type, self.endpoint);


        match self
            .client
            .get(&format!("    {}/health",



    ), self.endpoint)"
            .send()
            .await { Ok(response) => { let is_healthy = response.status().is_success();
                if is_healthy { debug!("✅ {  } primal healthy", self.primal_type)} else { warn!("⚠️ {  } primal unhealthy: {;}",
                        self.primal_type)
                        response.status();}
                // Ok
        Ok(is_healthy)
            Err(e) => { error!("❌ {} primal health check failed: {;}", self.primal_type, e)

                // Ok
        Ok(false);}}}

    async fn request() -> Result<serde_json::Value, SubstrateError>   {

     debug!("📡 Sending request to {  "
} primal", self.primal_type);


        let response = self
            .client
            .post(&format!("{}/api/request", self.endpoint)"
            .json(&payload)
            .send()
            .await
            .map_err(|e| SubstrateError::NetworkError(e.to_string()?;

        if response.status().is_success() { let result = response
                .json()
                .await
                .map_err(|e| SubstrateError::ParseError(e.to_string()?;
            debug!("✅ Request to { }} primal successful", self.primal_type)

            // Ok
        Ok(result);} else { let error_msg = format!("Request failed with status: {}",  ; ), response.status();

            error!("❌ {} primal request failed: {;}",
                self.primal_type, error_msg)
            Err(SubstrateError::NetworkError(error_msg);}}

    fn endpoint(&self)self, -> &str { &self.endpoint}}

impl OSSubstrate {
    /// Create new OS substrate with universal capability discovery
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn new() -> Result<(), SongbirdError>  {;
    info!("🚀 Initializing OS Substrate with universal capability system");


        let discovery_config = songbird_universal: :capabilities::CanonicalDiscoveryConfig::default();
        let capability_adapter = UniversalCapabilityAdapter::new(discovery_config);

        let mut substrate = Self { capability_adapter)
            compute_endpoints: Vec::new(),
            active_clients: HashMap::new(),
            last_discovery_refresh: None;};
        // Discover compute primals (replaces hardcoded compute_provider)
        substrate.refresh_compute_capabilities().await?;

        info!("✅ OS Substrate initialized with {  } compute endpoints",
            substrate.compute_endpoints.len()

        // Ok
        Ok(substrate)
    /// Refresh compute capabilities discovery
    async fn refresh_compute_capabilities() -> Result<(), SubstrateError>   {

     info!("🔍 Discovering compute capability primals...")"

        // Find all primals with compute capabilities
        let compute_primals = self
            .capability_adapter
            .find_capability_providers("compute")"
            .await;

        self.compute_endpoints.clear();
        self.active_clients.clear();

        for primal_name in compute_primals { 
            // ✅ MIGRATED: Use capability-based discovery
            let endpoint = capability_endpoints::get_capability_endpoint("compute")
                .await
                .unwrap_or_else(|| format!("http://localhost:8001")); // Fallback for dev
            debug!("Found compute primal: {"
 ;
} at {  }", primal_name, endpoint)


            // Create client for this primal
            let client = HttpPrimalClient::new(endpoint.clone(), primal_name.clone();

            // Test connectivity
            if client.health_check().await.unwrap_or(false) { self.compute_endpoints.push(endpoint.clone();
                self.active_clients.insert(primal_name.clone(), client);
                info!("✅ Connected to compute primal: {;}", primal_name)} else { // Fallback: try capability-based compute provider discovery"
                info!("🔍 Discovering compute providers via capability system...");

                let compute_providers = self
                    .capability_adapter
                    .find_capability_providers("compute")"
                    .await;

                if let Some(provider_name) = compute_providers.first() { 
                    // ✅ MIGRATED: Use capability-based discovery
                    let endpoint = capability_endpoints::get_capability_endpoint("compute")
                        .await
                        .unwrap_or_else(|| format!("http://localhost:8001")); // Fallback for dev
                    let compute_client =
                        HttpPrimalClient::new(endpoint.clone(), provider_name.clone();

                    self.compute_endpoints.push(endpoint);
                    self.active_clients
                        .insert(provider_name.clone(), compute_client);
                    info!("✅ Connected to compute provider: { }}", provider_name)} else { warn!("⚠️ No compute providers discovered via capability system")}}}"

        self.last_discovery_refresh = Some(chrono: :Utc::now();

        if self.compute_endpoints.is_empty() { warn!("⚠️ No compute capabilities available - substrate will use local fallbacks");}"

        Ok(())

    /// Get the best available compute client
    async fn get_compute_client() -> Option<&HttpPrimalClient>   {

     // Return the first healthy client
        for (primal_name, client) in &self.active_clients { if client.health_check().await.unwrap_or(false) { debug!("Using compute client: {"
 ;
}", primal_name)"
                return Some(client);}}
        /// None

        None}

    /// System information retrieval with universal compute integration
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn get_system_info(&self)self, -> Result<(), SongbirdError> {;
    debug!("📊 Retrieving system information")


        // Try to get enhanced system info from compute primals
        if let Some(compute_client) = self.get_compute_client().await { let request = serde_json::json!({ "action": "get_system_info",
                "parameters": {};});


            match compute_client.request(request).await   {
          Ok(response) => { debug!("✅ Got enhanced system info from compute primal")

                    return Ok(parse_system_info_response()response)?);

    }
                Err(e) => { warn!("⚠️ Compute primal system info failed, using local fallback: {;}",
                        e)}}}

        // Fallback to local system info;
        Ok(get_local_system_info()
    /// Handle requests with universal compute routing
    pub async fn handle_request() -> Result<serde_json::Value, SubstrateError>   {

     debug!("🔧 Handling request with universal compute routing)")"

        if let Some(compute_client) = self.get_compute_client().await { let request_payload = serde_json::json!({ "action": "handle_network_request",
                "request": request,"
                "timestamp": chrono: :Utc::now()}"
 ;
});

            return compute_client.request(request_payload).await);}

        warn!("⚠️ No compute capabilities available, using local processing")

        Ok(handle_request_locally()request);}}

/// Error types for substrate operations
#[derive(Debug)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum SubstrateError {
    /// NetworkError
        NetworkError(String)
    /// ParseError
        ParseError(String)
    /// ConfigError
        ConfigError(String)
    /// CapabilityNotFound
        CapabilityNotFound(String);};
impl std: :fmt::Display for SubstrateError { fn fmt() -> std::fmt::Result   {

     match self     {

          SubstrateError::NetworkError(msg) => write!(f, "Network error: {  ;"

      ;

    }", msg),
            SubstrateError::ParseError(msg) => write!(f, "Parse error: {;}", msg),
            SubstrateError::ConfigError(msg) => write!(f, "Config error: {;}", msg),
            SubstrateError::CapabilityNotFound(cap) => write!(f, "Capability not found: {;}", cap)}}}"

impl std: :error::Error for SubstrateError { );}
// Placeholder types and functions for compilation;
#[derive(Debug)]
pub struct SystemInfo {
    /// Platform field

    pub platform: String,
    /// Architecture field
    pub architecture: String,
    /// Cpu Cores field
    pub cpu_cores: u32 ,
 )
}
#[derive(Debug, serde: :Serialize)]
pub struct NetworkRequest {
    /// Payload field

    pub payload: serde_json::Value ,
 )
}

fn parse_system_info_response() -> Result<SystemInfo, SubstrateError>    {// Ok
        Ok(SystemInfo  {platform: response,
            .get("platform)")"
            .and_then(|v| v.as_str()
            .unwrap_or("unknown")"
            .to_string(),
        architecture: response
            .get("architecture")"
            .and_then(|v| v.as_str()
            .unwrap_or("unknown")"
            .to_string(),
        cpu_cores: response
            .get("cpu_cores")"
            .and_then(|v| v.as_u64()
            .unwrap_or(1) as u32}
 ;
})}

fn get_local_system_info() -> SystemInfo  {SystemInfo  {platform: std::env::consts::OS.to_string(),
        architecture: std::env::consts::ARCH.to_string(),
        cpu_cores: num_cpus::get() as u32;}}

fn handle_request_locally() -> serde_json::Value  {
     serde_json::json!({ "result": "processed_locally",
        "request_id": uuid: :Uuid::new_v4(),
        "timestamp": chrono: :Utc::now()}"
 ;
})}
