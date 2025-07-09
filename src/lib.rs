//! Songbird Gaming Bridge - Comprehensive Gaming Network Orchestrator
//!
//! High-performance, distributed orchestration platform for gaming infrastructure.
//! Provides seamless gaming platform integration, service discovery, load balancing,
//! and advanced security features including BearDog integration.
//!
// Songbird Orchestrator - Core Library
// Main library entry point with minimal modules for BearDog integration

// Core modules that compile and are needed for BearDog
pub mod api; // API endpoints and server
pub mod cli; // CLI module for error handling
pub mod config; // Configuration management
pub mod errors; // Error handling
pub mod federation; // Federation with encrypted snapshots
pub mod load_balancer;
pub mod network; // Network layer with gaming support
pub mod observability; // Basic observability
pub mod proxy; // Basic proxy functionality
pub mod security; // Security providers (including BearDog)
pub mod traits; // Core traits // Load balancer module

// Additional modules
pub mod accessibility;
pub mod basic_iot; // Universal IoT device connectivity
pub mod communication; // Communication protocols
pub mod discovery; // Service discovery
pub mod firewall; // System-agnostic firewall
pub mod health; // Health monitoring
pub mod http_server; // HTTP server for web interface
pub mod orchestrator; // Main orchestrator
pub mod robustness; // Circuit breakers, retry mechanisms, rate limiting
pub mod scalability; // Auto-scaling and performance optimization
pub mod zero_touch; // Zero touch deployment // Universal accessibility - "Free for All"
pub mod biome;

// Re-export core types for easier access
pub use config::SongbirdConfig;
pub use errors::{Result, SongbirdError};

// Re-export scalability types
pub use scalability::{
    AutoScaler, OptimizationEvent, OptimizationRecommendation, OptimizationType, 
    PerformanceConfig, PerformanceMetrics, PerformanceOptimizer, ResourceConfig, 
    ResourcePool, ResourceUsage, ScalabilityStats, ScalingDecision, ScalingEvent, 
    ServiceScalingConfig,
};

// Re-export BearDog integration types
pub use security::{
    BearDogAction, BearDogAuditLevel, BearDogComplianceReport, BearDogEncryptedData,
    BearDogKeyHandle, BearDogKeyPurpose, BearDogKeySpec, BearDogPrincipal, BearDogResource,
    BearDogRotationPolicy, BearDogSecurityContext, BearDogSecurityEvent, BearDogSecurityEventType,
    BearDogSecurityLevel, BearDogSecurityProvider,
};

// Re-export federation types
pub use federation::{
//     EncryptedSnapshotManager, Federation, FederationConfig, FederationManager, FederationMode,
//     FederationStatus, ProductionSnapshotSecurityAdapter, SnapshotDistributionStats,
//     SnapshotFilters, SnapshotMetadata, SnapshotRequest, SnapshotRequestType,
//     SnapshotSecurityProvider, SnapshotType, StoragePreferences,
};

// Re-export gaming network types
pub use network::gaming::{
    DetectedGameSession, GameProtocolClass, GamingManager, PlayerEndpoint, VirtualNetwork,
};

// Universal Security and Accessibility - "Free for All, Secure for All"
pub use accessibility::{
    convenience, AccessibilityConfig, InterfaceMode, UniversalAccessManager, UniversalHelpSystem,
    UserSkillLevel,
};

pub use security::{
    ConnectionSecurityStatus, DeviceSecurityPolicy, FamilyProtectionConfig, FriendTrustLevel,
    ScammerProtectionConfig, ScammerProtectionResult, SecurityLevel, UniversalSecurityManager,
};

// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Library description
pub const DESCRIPTION: &str =
    "Songbird Orchestrator - Federated Service Management with BearDog Security Integration";

/// Initialize the Songbird Orchestrator library
pub fn init() -> Result<()> {
    // Initialize logging if not already done
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    // Initialize tracing
    tracing_subscriber::fmt::init();

    tracing::info!("Songbird Orchestrator {} initialized", VERSION);
    tracing::info!("BearDog security integration available");

    Ok(())

}

// CLI types for internet connection commands
#[derive(Debug)]
pub enum CliError {
    Config(String),
    Network(String),
    Io(std::io::Error),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CliError::Config(msg) => write!(f, "Configuration error: {}", msg),
            CliError::Network(msg) => write!(f, "Network error: {}", msg),
            CliError::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for CliError {}

pub mod commands {
    use std::path::PathBuf;
    
    #[derive(Debug)]
    pub enum InternetCommands {
        Wizard {
            environment: Option<String>,
            tunnel: Option<String>,
            network_name: Option<String>,
            no_discovery: bool,
        },
        Status,
        Connect { network: String },
        Disconnect,
        Config { action: InternetConfigAction },
    }
    
    #[derive(Debug)]
    pub enum InternetConfigAction {
        Show,
        Validate { config: Option<PathBuf> },
        Ports,
    }
}

pub mod ui {
    use colored::*;
    
    pub fn title(text: &str) -> String {
        text.bright_blue().bold().to_string()
    }
    
    pub fn info(text: &str) -> String {
        text.bright_cyan().to_string()
    }
    
    pub fn success(text: &str) -> String {
        text.bright_green().to_string()
    }
}

// Internet connection configuration types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InternetConnectionConfig {
    pub connection_type: String,
    pub security_enabled: bool,
    pub monitoring_enabled: bool,
    pub family_safe_mode: bool,
}

impl Default for InternetConnectionConfig {
    fn default() -> Self {
        Self {
            connection_type: "direct".to_string(),
            security_enabled: true,
            monitoring_enabled: false,
            family_safe_mode: false,
        }
    }
}

#[derive(Debug)]
pub struct InternetConnectionWizard {
    #[allow(dead_code)] config: InternetConnectionConfig,
}

impl InternetConnectionWizard {
    pub fn new(config: InternetConnectionConfig) -> Self {
        Self { config }
    }
    
    pub async fn discover_songbird_ports(&self) -> std::result::Result<SongbirdPorts, String> {
        Ok(SongbirdPorts {
            orchestrator_port: 8080,
            federation_port: 8081,
            metrics_port: 8082,
            discovery_port: 8083,
            additional_service_ports: vec![8084, 8085],
        })
    }
}

// Port discovery types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SongbirdPorts {
    pub orchestrator_port: u16,
    pub federation_port: u16,
    pub metrics_port: u16,
    pub discovery_port: u16,
    pub additional_service_ports: Vec<u16>,
}

impl SongbirdPorts {
    pub fn get_all_required_ports(&self) -> Vec<u16> {
        let mut ports = vec![
            self.orchestrator_port,
            self.federation_port,
            self.metrics_port,
            self.discovery_port,
        ];
        ports.extend(&self.additional_service_ports);
        ports
    }
}

impl Default for SongbirdPorts {
    fn default() -> Self {
        Self {
            orchestrator_port: 8080,
            federation_port: 8081,
            metrics_port: 8082,
            discovery_port: 8083,
            additional_service_ports: Vec::new(),
        }
    }
}
pub use orchestrator::Orchestrator as SongbirdOrchestrator;
pub use config::NetworkConfig;
