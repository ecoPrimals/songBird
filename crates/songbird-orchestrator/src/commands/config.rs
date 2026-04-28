// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Config command — configuration management
//!
//! Provides `run_config()` for showing, validating, and initializing
//! Songbird configuration from environment variables.

use anyhow::Result;
use songbird_types::config::CanonicalSongbirdConfig;

/// Configuration action to perform
pub enum ConfigAction {
    /// Show current configuration
    Show {
        /// Whether to show sensitive values
        show_secrets: bool,
        /// Output format (text, json, yaml)
        format: String,
    },
    /// Validate configuration
    Validate,
    /// Generate default configuration template
    Init {
        /// Output file path
        output: String,
        /// Whether to overwrite existing file
        force: bool,
    },
}

/// Run configuration management commands
///
/// # Errors
///
/// Returns an error if configuration cannot be loaded, serialized, or written.
pub async fn run_config(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Show {
            show_secrets,
            format,
        } => show_config(show_secrets, &format).await,
        ConfigAction::Validate => validate_config().await,
        ConfigAction::Init {
            output,
            force,
        } => init_config(&output, force).await,
    }
}

/// Show current configuration
#[expect(
    clippy::unused_async,
    reason = "async signature required by Axum, trait objects, or future I/O"
)]
async fn show_config(show_secrets: bool, format: &str) -> Result<()> {
    match CanonicalSongbirdConfig::from_env() {
        Ok(config) => {
            // Handle different output formats
            match format {
                "json" => {
                    let output_config = if show_secrets {
                        config
                    } else {
                        mask_secrets_in_config(config)
                    };
                    println!("{}", serde_json::to_string_pretty(&output_config)?);
                }
                "toml" => {
                    let output_config = if show_secrets {
                        config
                    } else {
                        mask_secrets_in_config(config)
                    };
                    println!("{}", toml::to_string_pretty(&output_config)?);
                }
                "yaml" => {
                    eprintln!("⚠️  YAML format is deprecated; using TOML instead.");
                    let output_config = if show_secrets {
                        config
                    } else {
                        mask_secrets_in_config(config)
                    };
                    println!("{}", toml::to_string_pretty(&output_config)?);
                }
                _ => {
                    // Text format (default)
                    println!("📋 Songbird Configuration");
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!();
                    println!("✅ Configuration loaded successfully");
                    println!();
                    println!("Source: Environment variables");
                    println!();

                    if show_secrets {
                        println!("⚠️  Showing all values (including secrets)");
                    } else {
                        println!("💡 Use --show-secrets to display sensitive values");
                    }
                    println!();

                    display_config_formatted(&config, show_secrets);
                }
            }
        }
        Err(e) => {
            if format == "json" {
                let msg = e.to_string();
                println!("{{\"status\":\"error\",\"message\":\"{}\"}}", msg.replace('"', "\\\""));
            } else if format == "yaml" {
                println!("status: error");
                println!("message: \"{e}\"");
            } else {
                println!("❌ Configuration invalid");
                println!();
                println!("Error: {e}");
                println!();
                println!("💡 Fix: Set required environment variables or create config file");
            }
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Mask sensitive values in configuration for safe display
const fn mask_secrets_in_config(config: CanonicalSongbirdConfig) -> CanonicalSongbirdConfig {
    // For now, return as-is. Future: mask sensitive fields in TLS config, etc.
    // The current config structure doesn't have explicit API keys to mask
    config
}

/// Validate configuration
#[expect(
    clippy::unused_async,
    reason = "async signature required by Axum, trait objects, or future I/O"
)]
async fn validate_config() -> Result<()> {
    println!("🔍 Validating Songbird Configuration");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    match CanonicalSongbirdConfig::from_env() {
        Ok(_config) => {
            println!("✅ Configuration is valid!");
            println!();
            println!("All required settings present:");
            println!("  • Network configuration");
            println!("  • Discovery settings");
            println!("  • Federation parameters");
            println!();
            println!("💡 Ready to start: songbird server");
        }
        Err(e) => {
            println!("❌ Configuration validation failed");
            println!();
            println!("Error: {e}");
            println!();
            println!("Required environment variables:");
            let dp = songbird_config::defaults::ports::orchestrator_port();
            println!("  • SONGBIRD_PORT (optional, default: {dp})");
            println!("  • SONGBIRD_NODE_ID (optional, auto-generated)");
            println!("  • SONGBIRD_FAMILY_ID (optional, default: 'default')");
            println!();
            println!("💡 Run 'songbird config init' to generate a template");
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Initialize configuration template
#[expect(
    clippy::unused_async,
    reason = "async signature required by Axum, trait objects, or future I/O"
)]
async fn init_config(output: &str, force: bool) -> Result<()> {
    println!("🔧 Generating Songbird Configuration Template");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    if std::path::Path::new(output).exists() && !force {
        eprintln!("❌ File already exists: {output}");
        eprintln!();
        eprintln!("💡 Use --force to overwrite");
        std::process::exit(1);
    }

    let default_port = songbird_config::defaults::ports::orchestrator_port();
    let default_host = songbird_config::defaults::hosts::default_host();
    let template = format!(
        r"# Songbird Configuration Template
# Generated by: songbird config init

# Network Configuration
SONGBIRD_PORT={default_port}
SONGBIRD_HOST={default_host}

# Identity
SONGBIRD_NODE_ID=auto-generated
SONGBIRD_FAMILY_ID=default

# Discovery
SONGBIRD_DISCOVERY_ENABLED=true
SONGBIRD_BIRDSONG_ENABLED=true

# Federation
SONGBIRD_FEDERATION_ENABLED=true

# Security — discovered at runtime via capability system.
# Override only if running security provider at a non-standard path:
# SECURITY_PROVIDER_SOCKET=/run/user/${{UID}}/biomeos/security.sock

# Logging
RUST_LOG=info
SONGBIRD_LOG_LEVEL=info

# Optional: TLS
# SONGBIRD_TLS_CERT_PATH=./certs/cert.pem
# SONGBIRD_TLS_KEY_PATH=./certs/key.pem
"
    );

    std::fs::write(output, template)?;
    println!("✅ Configuration template generated: {output}");
    println!();
    println!("Next steps:");
    println!("  1. Review and customize: {output}");
    println!("  2. Source the file: source {output}");
    println!("  3. Validate: songbird config validate");
    println!("  4. Start server: songbird server");
    println!();

    Ok(())
}

/// Display configuration in structured, human-readable format
fn display_config_formatted(config: &CanonicalSongbirdConfig, _show_secrets: bool) {
    println!("┌─ System Configuration");
    println!("│  System ID: {}", config.system.system_id);
    println!("│  Instance ID: {}", config.system.instance_id);
    println!("│  Environment: {}", config.system.environment);
    println!("│  App Name: {}", config.system.app_name);
    println!("│  Version: {}", config.system.version);
    println!("│  Data Directory: {}", config.system.data_dir);
    println!("│  Config Directory: {}", config.system.config_dir);
    println!("│  Cache Directory: {}", config.system.cache_dir);
    println!("│  Log Directory: {}", config.system.log_dir);
    println!("│  Temp Directory: {}", config.system.temp_dir);
    println!("│  Log Level: {}", config.system.logging.level);
    println!("│  Log Format: {}", config.system.logging.format);
    println!("│");
    println!("├─ Network Configuration");
    println!("│  Bind Host: {}", config.network.bind_host);
    println!("│  Base Port: {}", config.network.base_port);
    println!("│  Primary Address: {}", config.network.bind.address);
    println!("│  Primary Port: {}", config.network.bind.port);
    println!("│  IPv6 Enabled: {}", config.network.bind.ipv6_enabled);
    println!("│  Client Max Connections: {}", config.network.client.max_connections);
    println!("│  Connect Timeout: {:?}", config.network.timeouts.connect);
    println!("│  Request Timeout: {:?}", config.network.timeouts.request);
    println!("│");
    println!("├─ Security Configuration");
    println!("│  Security Level: {:?}", config.security.security_level);
    println!("│  Auth Method: {}", config.security.auth_method);
    println!("│  Initial Trust Level: {:?}", config.security.initial_trust_level);
    println!("│  TLS Cert Policy: {:?}", config.security.tls.cert_policy);
    if let Some(ref cert) = config.security.tls.cert_path {
        println!("│  TLS Certificate: {cert}");
    }
    if let Some(ref key) = config.security.tls.key_path {
        println!("│  TLS Key: {key}");
    }
    println!("│  Require Valid Certs: {}", config.security.tls.require_valid_certs);
    println!("│");
    println!("├─ Performance Configuration");
    println!("│  Enabled: {}", config.performance.enabled);
    println!("│  Thread Pool Size: {}", config.performance.thread_pool_size);
    println!("│");
    println!("├─ Discovery Configuration");
    println!("│  Mode: {:?}", config.discovery.mode);
    println!("│  Backend: {}", config.discovery.backend);
    println!("│  Port: {}", config.discovery.port);
    println!("│  Protocol Version: {}", config.discovery.protocol_version);
    println!("│  Session Rotation: {}s", config.discovery.session_rotation_interval);
    println!("│");
    println!("├─ Observability Configuration");
    println!("│  Enabled: {}", config.observability.enabled);
    println!("│  Metrics Interval: {}s", config.observability.metrics_interval);
    println!("│  Metrics Enabled: {}", config.observability.metrics.enabled);
    println!("│  Tracing Enabled: {}", config.observability.tracing.enabled);
    println!("│  Tracing Level: {}", config.observability.tracing.level);
    println!("│  Health Checks Enabled: {}", config.observability.health_checks.enabled);
    println!("│");
    println!("├─ Gaming Configuration");
    println!("│  Enabled: {}", config.gaming.enabled);
    println!("│  Protocol Version: {}", config.gaming.protocol_version);
    println!("│");
    println!("├─ Primal Configuration (Runtime Discovery)");
    println!("│  Enabled: {}", config.primals.enabled);
    println!("│  Discovery Method: {}", config.primals.discovery_method);
    println!("│");
    println!("├─ Federation Configuration");
    println!("│  Cluster Name: {:?}", config.federation.cluster_name);
    println!("│  Trust Escalation Policy: {:?}", config.federation.trust_escalation_policy);
    println!("│  Initial Trust Level: {}", config.federation.initial_trust_level);
    println!("│  Acceptance Policy: {:?}", config.federation.acceptance_policy);
    println!("│  Require Hardware for Admin: {}", config.federation.require_hardware_for_admin);
    println!("│");
    println!("└─ Environment Configuration");
    println!("   Name: {}", config.environment.name);
    println!("   Deployment Mode: {}", config.environment.deployment_mode);
    println!();

    // Show custom fields if any
    if !config.custom.is_empty() {
        println!("Custom Fields:");
        for (key, value) in &config.custom {
            println!("  • {key}: {value:?}");
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use songbird_types::config::CanonicalSongbirdConfig;

    #[test]
    fn mask_secrets_is_identity_for_default_config() {
        let c = CanonicalSongbirdConfig::default();
        let m = mask_secrets_in_config(c.clone());
        assert_eq!(m.system.system_id, c.system.system_id);
        assert_eq!(m.network.base_port, c.network.base_port);
    }

    #[test]
    fn mask_secrets_preserves_custom_map() {
        let mut c = CanonicalSongbirdConfig::default();
        c.custom.insert("k".to_string(), serde_json::json!("v"));
        let m = mask_secrets_in_config(c.clone());
        assert_eq!(m.custom.get("k"), c.custom.get("k"));
    }

    #[test]
    fn config_action_show_variants_distinct() {
        let a = ConfigAction::Show {
            show_secrets: true,
            format: "json".to_string(),
        };
        let b = ConfigAction::Show {
            show_secrets: false,
            format: "text".to_string(),
        };
        match (a, b) {
            (
                ConfigAction::Show {
                    format: fa,
                    ..
                },
                ConfigAction::Show {
                    format: fb,
                    ..
                },
            ) => {
                assert_ne!(fa, fb);
            }
            _ => panic!("expected Show variants"),
        }
    }

    #[test]
    fn config_action_validate_is_unit() {
        assert!(matches!(ConfigAction::Validate, ConfigAction::Validate));
    }

    #[test]
    fn config_action_init_fields() {
        let a = ConfigAction::Init {
            output: "/tmp/out".to_string(),
            force: true,
        };
        match a {
            ConfigAction::Init {
                output,
                force,
            } => {
                assert_eq!(output, "/tmp/out");
                assert!(force);
            }
            _ => panic!("expected Init"),
        }
    }

    #[test]
    fn canonical_config_roundtrips_json_through_mask() {
        let c = CanonicalSongbirdConfig::default();
        let json_before = serde_json::to_value(&c).unwrap();
        let m = mask_secrets_in_config(c);
        let json_after = serde_json::to_value(&m).unwrap();
        assert_eq!(json_before, json_after);
    }

    #[test]
    fn canonical_config_roundtrips_toml_through_mask() {
        let c = CanonicalSongbirdConfig::default();
        let toml_before = toml::to_string_pretty(&c).unwrap();
        let m = mask_secrets_in_config(c);
        let toml_after = toml::to_string_pretty(&m).unwrap();
        assert_eq!(toml_before, toml_after);
    }
}
