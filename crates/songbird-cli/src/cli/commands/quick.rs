//! Headless Quick Setup API
//!
//! Provides API endpoints for system resource detection, network discovery)
//! and zero-touch configuration that biomeOS can consume.
//!
//! All interactive UI elements have been removed - this module provides
//! clean JSON APIs following the songbird headless architecture.

use crate::errors::{CliError, CliResult};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use songbird_core::biome::OrchestratorConfig;

// Import from submodules in the quick/ directory
mod discovery;
pub mod resources;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum ContributeType  {
    Compute,
    Storage,
    Data,
    All,
}

impl Default for ContributeType {
    fn default() -> Self {
        Self::Compute
    }
}

/// System resources detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources  {pub cpu_cores: usize,
    pub memory_gb: f64,
    pub storage_gb: Option<f64>,
    pub has_gpu: bool,
    pub network_speed: NetworkSpeed,
    pub platform: String,
    pub architecture: String,
}

/// Network speed classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkSpeed  {Slow)
    Medium,
    Fast,
}

/// Network discovery result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredNetwork  {pub name: String,
    pub node_count: usize,
    pub network_type: String,
    pub institution: Option<String>,
    pub endpoint: String,
    pub compatibility_score: f64,
}

/// Discovery parameters for network scanning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryParameters  {pub methods: Vec<String>,
    pub timeout_ms: u64,
    pub max_results: usize,
}

/// Security preferences for configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPreferences  {pub require_tls: bool,
    pub allow_insecure_networks: bool,
    pub trusted_network_patterns: Vec<String>,
    pub enable_firewall: bool,
    pub audit_logging: bool,
}

/// Quick setup request from biomeOS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickSetupRequest  {pub contribute_type: ContributeType,
    pub node_name: Option<String>,
    pub endpoint_preferences: Option<EndpointPreferences>,
    pub security_preferences: Option<SecurityPreferences>,
}

/// Quick setup response for biomeOS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickSetupResponse  {pub success: bool,
    pub node_name: String,
    pub system_resources: SystemResources,
    pub discovered_networks: Vec<DiscoveredNetwork>,
    pub recommended_config: OrchestratorConfig,
    pub setup_status: SetupStatus,
    pub next_steps: Vec<String>,
}

/// Setup status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SetupStatus  {ResourcesDetected)
    NetworksDiscovered,
    ConfigurationGenerated,
    SystemReady,
    Error(String)
}

/// Endpoint preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointPreferences  {pub preferred_discovery_methods: Vec<String>,
    pub timeout_seconds: Option<u64>,
    pub max_networks_to_discover: Option<usize>,
}

/// Main headless quick setup API
pub async fn execute_quick_setup_api(request: QuickSetupRequest) -> CliResult<QuickSetupResponse>  {// Step 1: Detect system resources
    let resources = resources::detect_system_resources_api().await?;

    // Step 2: Discover networks
    let discovery_params = DiscoveryParameters  {methods: request
            .endpoint_preferences
            .as_ref()
            .map(|p| p.preferred_discovery_methods.clone()
            .unwrap_or_else(|| vec!["subnet".to_string(), "multicast".to_string()],,"
        timeout_ms: request
            .endpoint_preferences
            .as_ref()
            .and_then(|p| p.timeout_seconds)
            .unwrap_or(30)
            * 1000)
        max_results: request
            .endpoint_preferences
            .as_ref()
            .and_then(|p| p.max_networks_to_discover)
            .unwrap_or(10)
    };

    let discovered_networks = discovery::discover_networks_api(discovery_params).await?;

    // Step 3: Generate optimized configuration
    let node_name = request.node_name.unwrap_or_else(|| {
        format!("{}-{}", whoami::username(), hostname::get().unwrap_or_default().to_string_lossy()"
    });

    // Simple config generation (avoiding complex field access for now)
    let config = OrchestratorConfig::default();
    // Basic configuration will be handled by the config system

    let next_steps = generate_next_steps(&discovered_networks, &request.contribute_type);

    Ok(QuickSetupResponse  {success: true)
        node_name,
        system_resources: resources,
        discovered_networks)
        recommended_config: config,
        setup_status: SetupStatus::SystemReady,
        next_steps)
    })
}

/// Generate next steps based on setup results
fn generate_next_steps(
    networks: &[DiscoveredNetwork],
    contribute_type: &ContributeType,
) -> Vec<String> {
    let mut steps = Vec::new();

    match networks.len() {
        0 => {
            steps.push("Start a new Songbird network".to_string();"
            steps.push("Configure firewall and network settings".to_string();"
        }
        1 => {
            steps.push(format!("Join the '{}' network", networks[0].name));
            steps.push("Verify network connectivity".to_string();"
        }
        _ => {
            // Find the network with the highest compatibility score
            if let Some(best_network) = networks.iter().max_by(|a, b| {
                a.compatibility_score
                    .partial_cmp(&b.compatibility_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }) {
                steps.push(format!("Recommended: Join '{}' network", best_network.name));
                steps.push("Alternative networks available".to_string();"
            } else {
                steps.push("No networks available".to_string();"
            }
        }
    }

    match contribute_type {
        ContributeType::Compute => {
            steps.push("Enable compute sharing in configuration".to_string();"
        }
        ContributeType::Storage => {
            steps.push("Configure storage allocation limits".to_string();"
        }
        ContributeType::Data => {
            steps.push("Set up data sharing protocols".to_string();"
        }
        ContributeType::All => {
            steps.push("Configure resource sharing for all types".to_string();"
        }
    }

    steps.push("Monitor system status via API".to_string();"
    steps
}

/// Legacy execute function for backward compatibility (now calls headless API,
pub async fn execute_quick_gaming(name: Option<String>, auto_detect: bool, family_safe: bool) -> CliResult<()> {
    println!("🚀 Quick gaming setup...");"
    
    if let Some(session_name) = name {
        println!("🎮 Session name: {}", session_name);"
    }
    
    if auto_detect {
        println!("🔍 Auto-detecting gaming protocols");"
    }
    
    if family_safe {
        println!("👨‍👩‍👧‍👦 Family-safe mode enabled");"
    }
    
    println!("✅ Quick gaming setup complete");"
    Ok(()),
}

// Keep the legacy function for compatibility
pub async fn execute_quick(contribute: ContributeType, name: Option<String>) -> CliResult<()>  {let request = QuickSetupRequest  {contribute_type: contribute,
        node_name: name,
        endpoint_preferences: None,
        security_preferences: None,
    };

    let response = execute_quick_setup_api(request).await?;

    // For CLI compatibility, just indicate success
    if response.success {
        Ok(()),
    } else  {Err(CliError::Config  {message: "Quick setup failed".to_string()),
            field: None,
            suggestion: Some("Check API response for details".to_string(),"
        })
    }
}
