use songbird_config::unified::*;
//! # 🎼 Canonical Federation Configuration
//!
//! **🚀 CONFIGURATION MANAGEMENT**
//!
//! This module provides configuration management for canonical federation.

use super::CanonicalFederationConfig;
use songbird_errors::SongbirdResult;

/// Load federation configuration from environment and config files
pub async fn load_federation_config() -> SongbirdResult<CanonicalFederationConfig> {
    let mut config = CanonicalFederationConfig::default();

    // Load from environment variables
    if let Ok(node_id) = std::env::var("SONGBIRD_FEDERATION_NODE_ID") {
        config.node_id = node_id;
    }

    if let Ok(max_nodes) = std::env::var("SONGBIRD_FEDERATION_MAX_NODES") {
        if let Ok(max) = max_nodes.parse::<usize>() {
            config.max_nodes = max;
        }
    }

    if let Ok(bind_address) = std::env::var("SONGBIRD_FEDERATION_BIND_ADDRESS") {
        // Note: bind_address field doesn't exist in CanonicalFederationConfig
        // This would need to be added to the config struct or handled differently
        tracing::info!("Bind address from env: {}", bind_address);
    }

    // Load from config files (TOML/YAML) if available
    if let Ok(config_path) = std::env::var("SONGBIRD_FEDERATION_CONFIG_PATH") {
        if let Ok(file_content) = std::fs::read_to_string(&config_path) {
            if config_path.ends_with(".toml") {
                if let Ok(file_config) = toml::from_str::<CanonicalFederationConfig>(&file_content)
                {
                    // Merge file config with environment config
                    config = merge_federation_config(config, file_config);
                }
            }
        }
    }

    validate_federation_config(&config).await?;
    Ok(config)
}

/// Validate federation configuration
pub async fn validate_federation_config(config: &CanonicalFederationConfig) -> SongbirdResult<()> {
    if config.node_id.is_empty() {
        return Err(songbird_errors::SongbirdError::configuration_error(
            "Node ID cannot be empty",
        ));
    }

    if config.max_nodes == 0 {
        return Err(songbird_errors::SongbirdError::configuration_error(
            "Maximum nodes must be greater than 0",
        ));
    }

    Ok(())
}

/// Merge file-based config with environment config (environment takes precedence)
fn merge_federation_config(
    env_config: CanonicalFederationConfig,
    file_config: CanonicalFederationConfig,
) -> CanonicalFederationConfig {
    CanonicalFederationConfig {
        node_id: if env_config.node_id.is_empty() {
            file_config.node_id
        } else {
            env_config.node_id
        },
        cluster_endpoints: if env_config.cluster_endpoints.is_empty() {
            file_config.cluster_endpoints
        } else {
            env_config.cluster_endpoints
        },
        discovery_enabled: env_config.discovery_enabled,
        auto_discovery_enabled: env_config.auto_discovery_enabled,
        health_interval_secs: env_config.health_interval_secs,
        heartbeat_interval_seconds: env_config.heartbeat_interval_seconds,
        health_check_interval_seconds: env_config.health_check_interval_seconds,
        max_nodes: env_config.max_nodes,
        security_enabled: env_config.security_enabled,
        discovery_interval_seconds: env_config.discovery_interval_seconds,
        node_scan_interval_seconds: env_config.node_scan_interval_seconds,
        node_timeout_seconds: env_config.node_timeout_seconds,
        discovery_network_ranges: env_config.discovery_network_ranges,
        seed_nodes: env_config.seed_nodes,
        mdns_discovery_enabled: env_config.mdns_discovery_enabled,
    }
}
