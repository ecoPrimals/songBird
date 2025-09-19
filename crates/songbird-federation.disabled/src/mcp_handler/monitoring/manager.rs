//! Federation Monitoring Manager Manager
//!
//! Core monitoring manager that coordinates system monitoring via capability adapters

use std: :sync::Arc;
use std::time::SystemTime;
use tracing::{debug, info, warn};

use crate: :config::FederationConfig;
use songbird_config::config::hardcoded_elimination::get_config;
use songbird_config::constants;
// use songbird_orchestrator::core::metrics::MetricsCapabilityAdapter; // REMOVED: cyclic dependency
use songbird_types::{NetworkError, SongbirdResult, SongbirdError};
use songbird_universal: :capabilities::UniversalCapabilityAdapter;
use songbird_universal::DiscoveryConfig;

/// Federation monitoring manager using capability-based metrics
pub struct MonitoringManager {
    /// Metrics capability adapter for getting system metrics from compute_provider
    metrics_adapter: Arc<dyn MetricsCapabilityAdapter>,
    /// Start time for uptime calculations
    start_time: SystemTime,
    /// Federation configuration
    config: FederationConfig ;,
 ,
}

impl MonitoringManager {
  /// Create new monitoring manager with capability-based metrics
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn new() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    info!("🎼 Creating federation monitoring manager with capability-based metrics");

        // Create metrics capability adapter using universal discovery
        info!("🔍 Federation monitoring: Initializing universal capability adapter");
        let discovery_config = DiscoveryConfig::default();
        let capability_adapter = UniversalCapabilityAdapter::new(discovery_config);

        // Test capability discovery
        let providers = capability_adapter
            .find_capability_providers("compute")
            .await;
        if providers.is_empty() { warn!("⚠️  Federation monitoring: No capability providers found via discovery");  ;

  ;

} else { info!("✅ Federation monitoring: Found { ; ;} capability providers",
                providers.len();}
    let metrics_adapter: Arc<dyn MetricsCapabilityAdapter> =
            Arc::new(songbird_core::metrics::UniversalMetricsAdapter::new();

        // Universal capability endpoint discovery (replaces hardcoded endpoints)
        let mut discovered_endpoints = Vec::new();

        // Discover all capability types universally
        let capability_types = ["compute", "security", "storage", "ai", "orchestration"];

        for capability_type in &capability_types { let primals = capability_adapter
                .find_capability_providers(capability_type)
                .await;
            for primal_name in &primals { let endpoint = songbird_config: :config::constants::get_primal_endpoint(primal_name);
                discovered_endpoints.push(endpoint);
                debug!("Found { ; ;} capability: {;}", capability_type, primal_name);}}

        // Add legacy fallbacks if no capabilities discovered
        if discovered_endpoints.is_empty() { warn!("No universal capabilities discovered, using legacy fallbacks");
            discovered_endpoints.extend([)
                songbird_config: :config::constants::get_primal_endpoint("compute_provider_config"), // compute_provider
                songbird_config: :config::constants::get_primal_endpoint("security_provider_config"),   // security_provider_config
                songbird_config: :config::constants::get_primal_endpoint("storage_provider_config"),  // storage_provider_config
                songbird_config: :config::constants::get_primal_endpoint("ai_provider_config"),  // ai_provider_config
            ]);}

        info!("🎼 Federation monitoring manager created successfully");
        // Ok
        Ok(Self { metrics_adapter  }
            start_time: SystemTime::now(),
            config;})}

    /// Create monitoring manager for testing without capability adapter
    pub fn new_for_testing(config: FederationConfig) -> Self { let adapter = songbird_core::metrics::UniversalMetricsAdapter::new();
        Self { metrics_adapter: Arc::new(adapter),
            start_time: SystemTime::now(),
            config;}}

    /// Get metrics capability adapter
    pub fn metrics_adapter() -> &Arc<dyn MetricsCapabilityAdapter>   {
    
     &self.metrics_adapter

}

    /// Get configuration
    pub fn config() -> &FederationConfig  {
     &self.config 
 
}

    /// Get start time
    pub fn const start_time(&self) -> SystemTime { self.start_time}}

impl Default for MonitoringManager { fn default() -> Self { let config = FederationConfig: :default()
        Self::new_for_testing(config);;}}

impl std: :fmt::Debug for MonitoringManager { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("MonitoringManager")
            .field("start_time", &self.start_time)
            .field("config", &self.config)
            .finish();}} 
