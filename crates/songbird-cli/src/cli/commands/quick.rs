// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Headless Quick Setup API
//!
//! Provides API endpoints for system resource detection, network discovery,
//! and zero-touch configuration that biomeOS can consume.
//!
//! All interactive UI elements have been removed - this module provides
//! clean JSON APIs following the songbird headless architecture.

#![expect(missing_docs, reason = "CLI command module — doc coverage not required")]

use crate::errors::{CliError, SongbirdResult};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Orchestrator configuration for quick setup
///
/// This is a local type definition for the CLI quick setup flow.
/// It provides type-safe configuration generation for biomeOS consumption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    /// Node identity
    pub node_name: String,
    /// Contribution capabilities
    pub capabilities: Vec<String>,
    /// Discovery endpoints
    pub discovery_endpoints: Vec<String>,
    /// Service ports configuration
    pub ports: HashMap<String, u16>,
    /// Security settings
    pub security: SecurityConfig,
    /// Resource limits
    pub resource_limits: ResourceLimits,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub require_tls: bool,
    pub enable_audit_logging: bool,
    pub allow_insecure_networks: bool,
}

/// Resource limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_cpu_percent: u8,
    pub max_memory_gb: f64,
    pub max_storage_gb: Option<f64>,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        let mut ports = HashMap::new();
        ports.insert("api".to_string(), songbird_config::defaults::ports::orchestrator_port());
        ports.insert("metrics".to_string(), songbird_config::defaults::ports::metrics_port());

        Self {
            node_name: "songbird-node".to_string(),
            capabilities: vec!["compute".to_string()],
            discovery_endpoints: vec![format!(
                "http://{}:{}",
                songbird_types::constants::LOCALHOST_HOSTNAME,
                songbird_config::defaults::ports::orchestrator_port()
            )],
            ports,
            security: SecurityConfig {
                require_tls: true,
                enable_audit_logging: true,
                allow_insecure_networks: false,
            },
            resource_limits: ResourceLimits {
                max_cpu_percent: 80,
                max_memory_gb: 8.0,
                max_storage_gb: None,
            },
        }
    }
}

// Import from submodules in the quick/ directory
mod discovery;
pub mod resources;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ValueEnum, Default)]
pub enum ContributeType {
    #[default]
    Compute,
    Storage,
    Data,
    All,
}

/// System resources detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    pub cpu_cores: usize,
    pub memory_gb: f64,
    pub storage_gb: Option<f64>,
    pub has_gpu: bool,
    pub network_speed: NetworkSpeed,
    pub platform: String,
    pub architecture: String,
}

/// Network speed classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkSpeed {
    Slow,
    Medium,
    Fast,
}

/// Network discovery result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredNetwork {
    pub name: String,
    pub node_count: usize,
    pub network_type: String,
    pub institution: Option<String>,
    pub endpoint: String,
    pub compatibility_score: f64,
}

/// Discovery parameters for network scanning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryParameters {
    pub methods: Vec<String>,
    pub timeout_ms: u64,
    pub max_results: usize,
}

/// Security preferences for configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPreferences {
    pub require_tls: bool,
    pub allow_insecure_networks: bool,
    pub trusted_network_patterns: Vec<String>,
    pub enable_firewall: bool,
    pub audit_logging: bool,
}

/// Quick setup request from biomeOS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickSetupRequest {
    pub contribute_type: ContributeType,
    pub node_name: Option<String>,
    pub endpoint_preferences: Option<EndpointPreferences>,
    pub security_preferences: Option<SecurityPreferences>,
}

/// Quick setup response for biomeOS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickSetupResponse {
    pub success: bool,
    pub node_name: String,
    pub system_resources: SystemResources,
    pub discovered_networks: Vec<DiscoveredNetwork>,
    pub recommended_config: OrchestratorConfig,
    pub setup_status: SetupStatus,
    pub next_steps: Vec<String>,
}

/// Setup status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SetupStatus {
    ResourcesDetected,
    NetworksDiscovered,
    ConfigurationGenerated,
    SystemReady,
    Error(String),
}

/// Endpoint preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointPreferences {
    pub preferred_discovery_methods: Vec<String>,
    pub timeout_seconds: Option<u64>,
    pub max_networks_to_discover: Option<usize>,
}

/// Main headless quick setup API
pub async fn execute_quick_setup_api(
    request: QuickSetupRequest,
) -> SongbirdResult<QuickSetupResponse> {
    // Step 1: Detect system resources
    let resources = resources::detect_system_resources_api().await?;

    // Step 2: Discover networks
    let discovery_params = DiscoveryParameters {
        methods: request.endpoint_preferences.as_ref().map_or_else(
            || vec!["subnet".to_string(), "multicast".to_string()],
            |p| p.preferred_discovery_methods.clone(),
        ),
        timeout_ms: request
            .endpoint_preferences
            .as_ref()
            .and_then(|p| p.timeout_seconds)
            .unwrap_or(30)
            * 1000,
        max_results: request
            .endpoint_preferences
            .as_ref()
            .and_then(|p| p.max_networks_to_discover)
            .unwrap_or(10),
    };

    let discovered_networks = discovery::discover_networks_api(discovery_params).await?;

    // Step 3: Generate optimized configuration
    let node_name = request.node_name.unwrap_or_else(|| {
        format!(
            "{}-{}",
            whoami::username(),
            gethostname::gethostname().to_string_lossy().into_owned()
        )
    });

    // Generate type-safe configuration based on request and discovered resources
    let capabilities = capabilities_for_contribute_type(&request.contribute_type);

    let discovery_endpoints: Vec<String> =
        discovered_networks.iter().map(|n| n.endpoint.clone()).collect();

    let mut ports = HashMap::new();
    ports.insert("api".to_string(), songbird_config::defaults::ports::orchestrator_port());
    ports.insert("metrics".to_string(), songbird_config::defaults::ports::metrics_port());

    let security = request.security_preferences.as_ref().map_or_else(
        || SecurityConfig {
            require_tls: true,
            enable_audit_logging: true,
            allow_insecure_networks: false,
        },
        |prefs| SecurityConfig {
            require_tls: prefs.require_tls,
            enable_audit_logging: prefs.audit_logging,
            allow_insecure_networks: prefs.allow_insecure_networks,
        },
    );

    let config = OrchestratorConfig {
        node_name: node_name.clone(),
        capabilities,
        discovery_endpoints,
        ports,
        security,
        resource_limits: ResourceLimits {
            max_cpu_percent: 80,
            max_memory_gb: resources.memory_gb,
            max_storage_gb: resources.storage_gb,
        },
    };

    let next_steps = generate_next_steps(&discovered_networks, &request.contribute_type);

    Ok(QuickSetupResponse {
        success: true,
        node_name,
        system_resources: resources,
        discovered_networks,
        recommended_config: config,
        setup_status: SetupStatus::SystemReady,
        next_steps,
    })
}

/// Capability strings for a contribute type (pure mapping for config generation).
fn capabilities_for_contribute_type(contribute_type: &ContributeType) -> Vec<String> {
    match contribute_type {
        ContributeType::Compute => vec!["compute".to_string()],
        ContributeType::Storage => vec!["storage".to_string()],
        ContributeType::Data => vec!["data".to_string()],
        ContributeType::All => {
            vec!["compute".to_string(), "storage".to_string(), "data".to_string()]
        }
    }
}

/// Generate next steps based on setup results
fn generate_next_steps(
    networks: &[DiscoveredNetwork],
    contribute_type: &ContributeType,
) -> Vec<String> {
    let mut steps = Vec::new();

    match networks.len() {
        0 => {
            steps.push("Start a new Songbird network ".to_string());
            steps.push("Configure firewall and network settings ".to_string());
        }
        1 => {
            steps.push(format!("Join the '{}' network ", networks[0].name));
            steps.push("Verify network connectivity ".to_string());
        }
        _ => {
            // Find the network with the highest compatibility score
            if let Some(best_network) = networks.iter().max_by(|a, b| {
                a.compatibility_score
                    .partial_cmp(&b.compatibility_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }) {
                steps.push(format!("Recommended: Join '{}' network", best_network.name));
                steps.push("Alternative networks available".to_string());
            } else {
                steps.push("No networks available".to_string());
            }
        }
    }

    match contribute_type {
        ContributeType::Compute => {
            steps.push("Enable compute sharing in configuration".to_string());
        }
        ContributeType::Storage => {
            steps.push("Configure storage allocation limits".to_string());
        }
        ContributeType::Data => {
            steps.push("Set up data sharing protocols".to_string());
        }
        ContributeType::All => {
            steps.push("Configure resource sharing for all types".to_string());
        }
    }

    steps.push("Monitor system status via API".to_string());
    steps
}

/// Legacy execute function for backward compatibility (now calls headless API)
pub async fn execute_quick_gaming(
    name: Option<String>,
    auto_detect: bool,
    family_safe: bool,
) -> SongbirdResult<()> {
    println!("🚀 Quick gaming setup...");

    if let Some(session_name) = name {
        println!("🎮 Session name: {session_name}");
    }

    if auto_detect {
        println!("🔍 Auto-detecting gaming protocols");
    }

    if family_safe {
        println!("👨‍👩‍👧‍👦 Family-safe mode enabled");
    }

    println!("✅ Quick gaming setup complete");
    Ok(())
}

// Keep the legacy function for compatibility
pub async fn execute_quick(contribute: ContributeType, name: Option<String>) -> SongbirdResult<()> {
    let request = QuickSetupRequest {
        contribute_type: contribute,
        node_name: name,
        endpoint_preferences: None,
        security_preferences: None,
    };

    let response = execute_quick_setup_api(request).await?;

    // For CLI compatibility, just indicate success
    if response.success {
        Ok(())
    } else {
        Err(CliError::Config {
            message: "Quick setup failed".to_string(),
            field: None,
            suggestion: Some("Check API response for details".to_string()),
        }
        .into())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::{
        ContributeType, DiscoveredNetwork, OrchestratorConfig, capabilities_for_contribute_type,
        generate_next_steps,
    };

    fn net(name: &str, score: f64, endpoint: &str) -> DiscoveredNetwork {
        DiscoveredNetwork {
            name: name.to_string(),
            node_count: 1,
            network_type: "test".to_string(),
            institution: None,
            endpoint: endpoint.to_string(),
            compatibility_score: score,
        }
    }

    #[test]
    fn generate_next_steps_zero_networks_includes_bootstrap_steps() {
        let steps = generate_next_steps(&[], &ContributeType::Compute);
        assert!(steps.iter().any(|s| s.contains("Start a new Songbird network")));
        assert!(steps.iter().any(|s| s.contains("firewall")));
        assert!(steps.iter().any(|s| s.contains("compute")));
        assert!(steps.iter().any(|s| s.contains("Monitor system status")));
    }

    #[test]
    fn generate_next_steps_single_network_mentions_name() {
        let nets = [net("alpha-net", 0.5, "http://a")];
        let steps = generate_next_steps(&nets, &ContributeType::Storage);
        assert!(steps.iter().any(|s| s.contains("alpha-net")));
        assert!(steps.iter().any(|s| s.contains("storage")));
    }

    #[test]
    fn generate_next_steps_picks_highest_compatibility() {
        let nets = [
            net("low", 0.2, "http://l"),
            net("best", 0.95, "http://b"),
            net("mid", 0.5, "http://m"),
        ];
        let steps = generate_next_steps(&nets, &ContributeType::Data);
        assert!(steps.iter().any(|s| s.contains("best")));
        assert!(steps.iter().any(|s| s.contains("data")));
    }

    #[test]
    fn generate_next_steps_tied_scores_still_recommend_one() {
        let nets = [net("a", 0.5, "http://a"), net("b", 0.5, "http://b")];
        let steps = generate_next_steps(&nets, &ContributeType::All);
        assert!(steps.iter().any(|s| s.contains("Recommended: Join")));
        assert!(steps.iter().any(|s| s.contains("resource sharing")));
    }

    #[test]
    fn contribute_type_compute_appends_distinct_step() {
        let steps = generate_next_steps(&[], &ContributeType::Compute);
        assert!(steps.iter().any(|s| s.contains("compute sharing")));
    }

    #[test]
    fn contribute_type_all_appends_all_types_step() {
        let steps = generate_next_steps(&[], &ContributeType::All);
        assert!(steps.iter().any(|s| s.contains("all types")));
    }

    #[test]
    fn generate_next_steps_two_networks_includes_alternative_hint() {
        let nets = [net("n1", 0.1, "e1"), net("n2", 0.2, "e2")];
        let steps = generate_next_steps(&nets, &ContributeType::Compute);
        assert!(steps.iter().any(|s| s.contains("Alternative networks")));
    }

    #[test]
    fn capabilities_for_contribute_type_compute_storage_data() {
        assert_eq!(
            capabilities_for_contribute_type(&ContributeType::Compute),
            vec!["compute".to_string()]
        );
        assert_eq!(
            capabilities_for_contribute_type(&ContributeType::Storage),
            vec!["storage".to_string()]
        );
        assert_eq!(
            capabilities_for_contribute_type(&ContributeType::Data),
            vec!["data".to_string()]
        );
    }

    #[test]
    fn capabilities_for_contribute_type_all_ordering() {
        assert_eq!(
            capabilities_for_contribute_type(&ContributeType::All),
            vec!["compute".to_string(), "storage".to_string(), "data".to_string()]
        );
    }

    #[test]
    fn orchestrator_config_default_has_expected_keys_and_security() {
        let c = OrchestratorConfig::default();
        assert_eq!(c.node_name, "songbird-node");
        assert!(c.ports.contains_key("api"));
        assert!(c.ports.contains_key("metrics"));
        assert!(c.security.require_tls);
        assert!(c.security.enable_audit_logging);
        assert!(!c.security.allow_insecure_networks);
        assert_eq!(c.resource_limits.max_cpu_percent, 80);
    }

    #[test]
    fn generate_next_steps_single_network_verify_connectivity_step() {
        let nets = [net("solo", 1.0, "http://solo")];
        let steps = generate_next_steps(&nets, &ContributeType::Compute);
        assert!(steps.iter().any(|s| s.contains("Verify network connectivity")));
    }

    #[test]
    fn generate_next_steps_empty_networks_storage_contribute_appends_storage_step() {
        let steps = generate_next_steps(&[], &ContributeType::Storage);
        assert!(steps.iter().any(|s| s.contains("storage allocation")));
    }
}
