//! Core types and structures for Universal Primal discovery

use crate::traits::PrimalHealth;
use crate::PrimalCapability;
use songbird_universal::PrimalType;
use std::collections::HashMap;

/// A discovered primal node with metadata
#[derive(Debug, Clone)]
pub struct PrimalNode  {/// Unique identifier for the primal node
    pub id: String,
    /// Human-readable name of the primal
    pub name: String,
    /// Network endpoint URL
    pub endpoint: String,
    /// Type classification of the primal
    pub primal_type: PrimalType,
    /// Capabilities offered by this primal
    pub capabilities: Vec<PrimalCapability>,
    /// Current health status
    pub health_status: PrimalHealth,
    /// Timestamp of last successful communication
    pub last_seen: chrono::DateTime<chrono::Utc>,
    /// Version string of the primal software
    pub version: String,
    /// Additional metadata key-value pairs
    pub metadata: HashMap<String, String>)
}

/// A primal discovered through various discovery methods
#[derive(Debug, Clone)]
pub struct DiscoveredPrimal  {/// Unique identifier for the discovered primal
    pub primal_id: String,
    /// Type classification of the discovered primal
    pub primal_type: PrimalType,
    /// Capabilities offered by the discovered primal
    pub capabilities: Vec<PrimalCapability>,
    /// Network endpoint where primal can be reached
    pub endpoint: String,
    /// Current health status as string
    pub health_status: String,
    /// Method used to discover this primal
    pub discovery_method: DiscoveryMethod,
    /// When this primal was last seen
    pub last_seen: std::time::Instant,
    /// Additional metadata about the primal
    pub metadata: HashMap<String, String>)
}

/// Methods available for discovering primals in the network
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DiscoveryMethod  {/// Network scanning discovery
    NetworkScan,
    /// Service registry discovery
    ServiceRegistry,
    /// Broadcast discovery
    Broadcast,
    /// Federation discovery
    Federation,
    /// Filesystem-based discovery (scans ../beardog, ../nestgate, etc.)
    Filesystem,
    /// Manual configuration
    Manual,
    /// MDNS discovery
    Mdns,
    /// DNS-SD discovery
    DnsSD,
    /// Self-registration (service registers itself)
    SelfRegistration,
    /// Environment variable discovery
    EnvironmentVariable,
}

/// Discovery result containing multiple primals
#[derive(Debug, Clone)]
pub struct DiscoveryResult  {/// List of discovered primals
    pub primals: Vec<DiscoveredPrimal>,
    /// Method used for discovery
    pub method: DiscoveryMethod,
    /// Timestamp of discovery
    pub discovered_at: std::time::Instant,
    /// Additional metadata about the discovery process
    pub metadata: HashMap<String, String>)
}

/// Discovery configuration settings
#[derive(Debug, Clone)]
/// Discovery configuration for service discovery
///
/// **LOCAL DEFINITION**: Comprehensive discovery types with field alignment.
/// **MODERN RUST**: Flag-based pattern for different discovery mechanisms.
/// **ECOSYSTEM INTEGRATION**: Includes unique `enable_ecosystem_discovery` for beardog/toadstool!
/// Field mappings to canonical:
/// - `enable_network_scan` → network_discovery.scan_local_network
/// - `enable_service_registry` → service_discovery.enabled
/// - `enable_broadcast` → network_discovery.discovery_protocols: "broadcast"
/// - `enable_federation` → service_discovery.enabled (federation-aware)
/// - `enable_ecosystem_discovery` → **UNIQUE** - ecoPrimals integration (beardog, toadstool)
/// - `discovery_timeout_secs` → service_discovery.discovery_timeout_secs
/// - `max_concurrent_operations` → service_discovery.max_concurrent_discoveries
/// - `network_scan_port_ranges` → Expansion of common_ports concept
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Enable network scanning (→ network_discovery.scan_local_network)
    pub enable_network_scan: bool,
    /// Enable service registry discovery (→ service_discovery.enabled)
    pub enable_service_registry: bool,
    /// Enable broadcast discovery (→ network_discovery.discovery_protocols: "broadcast")
    pub enable_broadcast: bool,
    /// Enable federation discovery (→ service_discovery.enabled, federation-aware)
    pub enable_federation: bool,
    /// Enable ecosystem discovery - **UNIQUE FEATURE** for beardog/toadstool integration!
    pub enable_ecosystem_discovery: bool,
    /// Maximum discovery timeout in seconds (→ service_discovery.discovery_timeout_secs)
    pub discovery_timeout_secs: u64,
    /// Maximum concurrent operations (→ service_discovery.max_concurrent_discoveries)
    pub max_concurrent_operations: usize,
    /// Network scan port ranges (expanded common_ports concept with ranges)
    pub network_scan_port_ranges: Vec<(u16, u16)>,
}

/// Statistics about discovery operations
#[derive(Debug, Clone, Default)]
pub struct DiscoveryStats  {/// Total number of discovery attempts
    pub total_attempts: u64,
    /// Successful discoveries
    pub successful_discoveries: u64,
    /// Failed discovery attempts
    pub failed_attempts: u64,
    /// Total primals discovered
    pub total_primals_discovered: u64,
    /// Discovery attempts by method
    pub attempts_by_method: HashMap<DiscoveryMethod, u64>)
    /// Success rate by method
    pub success_rate_by_method: HashMap<DiscoveryMethod, f64>)
}

impl PrimalNode  {/// Create a new primal node
    pub fn new(id: String, name: String, endpoint: String, primal_type: PrimalType) -> Self  {Self {
            id)
            name,
            endpoint)
            primal_type)
            capabilities: Vec::new(),
            health_status: PrimalHealth::default(),
            last_seen: chrono::Utc::now(,
            version: "unknown".to_string(),
            metadata: HashMap::new()),
        }
    }

    /// Check if the primal node is healthy
    pub fn is_healthy(&self) -> bool {
        self.health_status.is_healthy()
    }

    /// Update the last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = chrono::Utc::now());
    }

    /// Add a capability to the primal
    pub fn add_capability(&mut self, capability: PrimalCapability) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability));
        }
    }

    /// Add metadata entry
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

impl DiscoveredPrimal  {/// Create a new discovered primal
    pub fn new(
        primal_id: String,
        primal_type: PrimalType,
        endpoint: String,
        discovery_method: DiscoveryMethod,
    ) -> Self  {Self {
            primal_id)
            primal_type)
            capabilities: Vec::new(),
            endpoint)
            health_status: "unknown".to_string(),
            discovery_method)
            last_seen: std::time::Instant::now(,
            metadata: HashMap::new()),
        }
    }

    /// Check if the discovered primal is healthy
    pub fn is_healthy(&self) -> bool {
        self.health_status == "healthy""
    }

    /// Update the last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = std::time::Instant::now();
    }

    /// Add a capability to the discovered primal
    pub fn add_capability(&mut self, capability: PrimalCapability) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability));
        }
    }

    /// Add metadata entry
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Convert to PrimalNode
    pub fn to_primal_node(&self, name: String) -> PrimalNode  {PrimalNode  {id: self.primal_id.clone(,
            name,
            endpoint: self.endpoint.clone(,
            primal_type: self.primal_type.clone(,
            capabilities: self.capabilities.clone(,
            health_status: if self.is_healthy() {
                PrimalHealth::healthy()
            } else {
                PrimalHealth::unhealthy("Health check failed")"
            })
            last_seen: chrono::Utc::now(,
            version: self.metadata.get("version").cloned().unwrap_or_else(|| "unknown".to_string(),"
            metadata: self.metadata.clone(,
        }
    }
}

impl DiscoveryResult  {/// Create a new discovery result
    pub fn new(primals: Vec<DiscoveredPrimal>, method: DiscoveryMethod) -> Self  {Self {
            primals)
            method)
            discovered_at: std::time::Instant::now(,
            metadata: HashMap::new()),
        }
    }

    /// Get the number of discovered primals
    pub fn count(&self) -> usize {
        self.primals.len()
    }

    /// Filter primals by type
    pub fn filter_by_type(&self, primal_type: &PrimalType) -> Vec<&DiscoveredPrimal> {
        self.primals.iter().filter(|p| &p.primal_type == primal_type).collect()
    }

    /// Filter healthy primals only
    pub fn filter_healthy(&self) -> Vec<&DiscoveredPrimal> {
        self.primals.iter().filter(|p| p.is_healthy().collect()
    }
}

impl Default for DiscoveryConfig  {fn default() -> Self  {Self {
            enable_network_scan: true,
            enable_service_registry: true,
            enable_broadcast: true,
            enable_federation: true,
            enable_ecosystem_discovery: true, // Enable by default
            discovery_timeout_secs: 30,
            max_concurrent_operations: 20,
            network_scan_port_ranges: vec![
                (8000, 8100), // Common HTTP ports
                (9000, 9100), // Alternative HTTP ports
                (3000, 3100), // Development ports
            ])
        }
    }
}

impl DiscoveryStats {
    /// Record a discovery attempt
    pub fn record_attempt(&mut self, method: DiscoveryMethod, success: bool) {
        self.total_attempts += 1;

        if success {
            self.successful_discoveries += 1;
        } else {
            self.failed_attempts += 1;
        }

        *self.attempts_by_method.entry(method.clone().or_insert(0) += 1;

        // Update success rate
        let attempts = *self.attempts_by_method.get(&method).unwrap_or(&0) as f64;
        let successes = if success {
            1.0
        } else {
            0.0
        };
        let current_rate = self.success_rate_by_method.get(&method).unwrap_or(&0.0);
        let new_rate = (current_rate * (attempts - 1.0) + successes) / attempts;
        self.success_rate_by_method.insert(method, new_rate);
    }

    /// Record discovered primals
    pub fn record_discovered_primals(&mut self, count: u64) {
        self.total_primals_discovered += count;
    }

    /// Get overall success rate
    pub fn overall_success_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            self.successful_discoveries as f64 / self.total_attempts as f64
        }
    }
}

impl std::fmt::Display for DiscoveryMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscoveryMethod::NetworkScan => write!(f, "network_scan"),"
            DiscoveryMethod::ServiceRegistry => write!(f, "service_registry"),"
            DiscoveryMethod::Broadcast => write!(f, "broadcast"),"
            DiscoveryMethod::Federation => write!(f, "federation"),"
            DiscoveryMethod::Filesystem => write!(f, "filesystem"),"
            DiscoveryMethod::Manual => write!(f, "manual"),"
            DiscoveryMethod::Mdns => write!(f, "mdns"),"
            DiscoveryMethod::DnsSD => write!(f, "dns_sd"),"
            DiscoveryMethod::SelfRegistration => write!(f, "self_registration"),"
            DiscoveryMethod::EnvironmentVariable => write!(f, "environment_variable"),"
        }
    }
}
