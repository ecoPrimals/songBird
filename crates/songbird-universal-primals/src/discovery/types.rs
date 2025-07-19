//! Core types and structures for Universal Primal discovery

use crate::router::PrimalHealth;
use crate::PrimalCapability;
use songbird_universal::PrimalType;
use std::collections::HashMap;

/// A discovered primal node with metadata
#[derive(Debug, Clone)]
pub struct PrimalNode {
    /// Unique identifier for the primal node
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
    pub metadata: HashMap<String, String>,
}

/// A primal discovered through various discovery methods
#[derive(Debug, Clone)]
pub struct DiscoveredPrimal {
    /// Unique identifier for the discovered primal
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
    pub metadata: HashMap<String, String>,
}

/// Methods available for discovering primals in the network
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DiscoveryMethod {
    /// Network scanning discovery
    NetworkScan,
    /// Service registry based discovery
    ServiceRegistry,
    /// UDP broadcast discovery
    Broadcast,
    /// Direct connection discovery
    DirectConnection,
    /// Federation network discovery
    Federation,
    /// Manual registration
    Manual,
}

/// Discovery result containing multiple primals
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    /// List of discovered primals
    pub primals: Vec<DiscoveredPrimal>,
    /// Method used for discovery
    pub method: DiscoveryMethod,
    /// Timestamp of discovery
    pub discovered_at: std::time::Instant,
    /// Additional metadata about the discovery process
    pub metadata: HashMap<String, String>,
}

/// Discovery configuration for different methods
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Enable network scanning
    pub enable_network_scan: bool,
    /// Enable service registry discovery
    pub enable_service_registry: bool,
    /// Enable broadcast discovery
    pub enable_broadcast: bool,
    /// Enable federation discovery
    pub enable_federation: bool,
    /// Timeout for discovery operations
    pub discovery_timeout: std::time::Duration,
    /// Network scan IP ranges
    pub scan_ranges: Vec<String>,
    /// Service registry endpoints
    pub registry_endpoints: Vec<String>,
    /// Broadcast ports to scan
    pub broadcast_ports: Vec<u16>,
    /// Federation endpoints
    pub federation_endpoints: Vec<String>,
}

/// Statistics about discovery operations
#[derive(Debug, Clone, Default)]
pub struct DiscoveryStats {
    /// Total number of discovery attempts
    pub total_attempts: u64,
    /// Successful discoveries
    pub successful_discoveries: u64,
    /// Failed discovery attempts
    pub failed_attempts: u64,
    /// Total primals discovered
    pub total_primals_discovered: u64,
    /// Discovery attempts by method
    pub attempts_by_method: HashMap<DiscoveryMethod, u64>,
    /// Success rate by method
    pub success_rate_by_method: HashMap<DiscoveryMethod, f64>,
}

impl PrimalNode {
    /// Create a new primal node
    pub fn new(id: String, name: String, endpoint: String, primal_type: PrimalType) -> Self {
        Self {
            id,
            name,
            endpoint,
            primal_type,
            capabilities: Vec::new(),
            health_status: PrimalHealth::Unknown,
            last_seen: chrono::Utc::now(),
            version: "unknown".to_string(),
            metadata: HashMap::new(),
        }
    }

    /// Check if the primal node is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.health_status, PrimalHealth::Healthy)
    }

    /// Update the last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = chrono::Utc::now();
    }

    /// Add a capability to the primal
    pub fn add_capability(&mut self, capability: PrimalCapability) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
    }

    /// Add metadata entry
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

impl DiscoveredPrimal {
    /// Create a new discovered primal
    pub fn new(
        primal_id: String,
        primal_type: PrimalType,
        endpoint: String,
        discovery_method: DiscoveryMethod,
    ) -> Self {
        Self {
            primal_id,
            primal_type,
            capabilities: Vec::new(),
            endpoint,
            health_status: "unknown".to_string(),
            discovery_method,
            last_seen: std::time::Instant::now(),
            metadata: HashMap::new(),
        }
    }

    /// Check if the discovered primal is healthy
    pub fn is_healthy(&self) -> bool {
        self.health_status == "healthy"
    }

    /// Update the last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = std::time::Instant::now();
    }

    /// Add a capability to the discovered primal
    pub fn add_capability(&mut self, capability: PrimalCapability) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
    }

    /// Add metadata entry
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Convert to PrimalNode
    pub fn to_primal_node(&self, name: String) -> PrimalNode {
        PrimalNode {
            id: self.primal_id.clone(),
            name,
            endpoint: self.endpoint.clone(),
            primal_type: self.primal_type.clone(),
            capabilities: self.capabilities.clone(),
            health_status: if self.is_healthy() {
                PrimalHealth::Healthy
            } else {
                PrimalHealth::Unhealthy
            },
            last_seen: chrono::Utc::now(),
            version: self
                .metadata
                .get("version")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string()),
            metadata: self.metadata.clone(),
        }
    }
}

impl DiscoveryResult {
    /// Create a new discovery result
    pub fn new(primals: Vec<DiscoveredPrimal>, method: DiscoveryMethod) -> Self {
        Self {
            primals,
            method,
            discovered_at: std::time::Instant::now(),
            metadata: HashMap::new(),
        }
    }

    /// Get the number of discovered primals
    pub fn count(&self) -> usize {
        self.primals.len()
    }

    /// Filter primals by type
    pub fn filter_by_type(&self, primal_type: &PrimalType) -> Vec<&DiscoveredPrimal> {
        self.primals
            .iter()
            .filter(|p| &p.primal_type == primal_type)
            .collect()
    }

    /// Filter healthy primals only
    pub fn filter_healthy(&self) -> Vec<&DiscoveredPrimal> {
        self.primals.iter().filter(|p| p.is_healthy()).collect()
    }
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            enable_network_scan: true,
            enable_service_registry: true,
            enable_broadcast: true,
            enable_federation: true,
            discovery_timeout: std::time::Duration::from_secs(30),
            scan_ranges: vec![
                "127.0.0.1/32".to_string(),
                "10.0.0.0/24".to_string(),
                "192.168.1.0/24".to_string(),
            ],
            registry_endpoints: vec![
                "http://localhost:8500".to_string(), // Consul
                "http://localhost:2379".to_string(), // etcd
            ],
            broadcast_ports: vec![8080, 8081, 8082, 8083, 8084, 8085],
            federation_endpoints: vec![
                "http://localhost:9090".to_string(),
                "http://localhost:9091".to_string(),
            ],
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

        *self.attempts_by_method.entry(method.clone()).or_insert(0) += 1;

        // Update success rate
        let attempts = *self.attempts_by_method.get(&method).unwrap_or(&0) as f64;
        let successes = if success { 1.0 } else { 0.0 };
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
            DiscoveryMethod::NetworkScan => write!(f, "network_scan"),
            DiscoveryMethod::ServiceRegistry => write!(f, "service_registry"),
            DiscoveryMethod::Broadcast => write!(f, "broadcast"),
            DiscoveryMethod::DirectConnection => write!(f, "direct_connection"),
            DiscoveryMethod::Federation => write!(f, "federation"),
            DiscoveryMethod::Manual => write!(f, "manual"),
        }
    }
}
