// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Configuration management commands
//!
//! Handles:
//! - Configuration display (text/JSON/TOML formats)
//! - Configuration validation
//! - Configuration template generation

use std::fmt::Write as _;

use anyhow::Result;

use super::ConfigCommands;

/// Run configuration management commands
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn run_config(cmd: ConfigCommands) -> Result<()> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    match cmd {
        ConfigCommands::Show {
            show_secrets,
            format,
        } => {
            show_config(show_secrets, &format).await?;
        }
        ConfigCommands::Validate => {
            validate_config().await?;
        }
        ConfigCommands::Init {
            output,
            force,
        } => {
            init_config(&output, force).await?;
        }
    }

    Ok(())
}

/// Show current configuration
#[expect(
    clippy::unused_async,
    reason = "async signature required by Axum, trait objects, or future I/O"
)]
async fn show_config(show_secrets: bool, format: &str) -> Result<()> {
    use songbird_types::config::CanonicalSongbirdConfig;

    match CanonicalSongbirdConfig::from_env() {
        Ok(config) => {
            // Handle different output formats
            match format {
                "json" => {
                    // Mask secrets if needed
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

                    // ✅ Display structured configuration values
                    print!("{}", format_config_display(&config, show_secrets));
                }
            }
        }
        Err(e) => {
            if format == "json" {
                let msg = e.to_string();
                println!("{{\"status\":\"error\",\"message\":\"{}\"}}", msg.replace('"', "\\\""));
            } else if format == "toml" || format == "yaml" {
                println!("status = \"error\"");
                println!("message = \"{e}\"");
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

/// Validate configuration
#[expect(
    clippy::unused_async,
    reason = "async signature required by Axum, trait objects, or future I/O"
)]
async fn validate_config() -> Result<()> {
    use songbird_types::config::CanonicalSongbirdConfig;

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
    let family_id = crate::env_config::family_id();
    let template = format!(
        r"# Songbird Configuration Template
# Generated by: songbird config init
# All values shown are runtime defaults — override via environment variables.

# Network Configuration
SONGBIRD_PORT={default_port}
SONGBIRD_HOST={default_host}

# Identity (auto-detected at runtime if not set)
# SONGBIRD_NODE_ID=<auto-generated from hostname>
SONGBIRD_FAMILY_ID={family_id}

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

/// Mask sensitive values in configuration for safe display
///
/// ✅ Modern approach: Creates a copy with secrets masked (simple placeholder for now)
const fn mask_secrets_in_config(
    config: songbird_types::config::CanonicalSongbirdConfig,
) -> songbird_types::config::CanonicalSongbirdConfig {
    // For now, return as-is. Future: mask sensitive fields in TLS config, etc.
    // The current config structure doesn't have explicit API keys to mask
    config
}

/// Human-readable configuration text (same content as printed by `songbird config show`).
fn format_config_display(
    config: &songbird_types::config::CanonicalSongbirdConfig,
    show_secrets: bool,
) -> String {
    let mut out = String::new();
    // write! to String is infallible — fmt::Error is unreachable
    write_config_display(&mut out, config, show_secrets).ok();
    out
}

fn write_config_display(
    out: &mut String,
    config: &songbird_types::config::CanonicalSongbirdConfig,
    _show_secrets: bool,
) -> std::fmt::Result {
    writeln!(out, "┌─ System Configuration")?;
    writeln!(out, "│  System ID: {}", config.system.system_id)?;
    writeln!(out, "│  Instance ID: {}", config.system.instance_id)?;
    writeln!(out, "│  Environment: {}", config.system.environment)?;
    writeln!(out, "│  App Name: {}", config.system.app_name)?;
    writeln!(out, "│  Version: {}", config.system.version)?;
    writeln!(out, "│  Data Directory: {}", config.system.data_dir)?;
    writeln!(out, "│  Config Directory: {}", config.system.config_dir)?;
    writeln!(out, "│  Cache Directory: {}", config.system.cache_dir)?;
    writeln!(out, "│  Log Directory: {}", config.system.log_dir)?;
    writeln!(out, "│  Temp Directory: {}", config.system.temp_dir)?;
    writeln!(out, "│  Log Level: {}", config.system.logging.level)?;
    writeln!(out, "│  Log Format: {}", config.system.logging.format)?;
    writeln!(out, "│")?;
    writeln!(out, "├─ Network Configuration")?;
    writeln!(out, "│  Bind Host: {}", config.network.bind_host)?;
    writeln!(out, "│  Base Port: {}", config.network.base_port)?;
    writeln!(out, "│  Primary Address: {}", config.network.bind.address)?;
    writeln!(out, "│  Primary Port: {}", config.network.bind.port)?;
    writeln!(out, "│  IPv6 Enabled: {}", config.network.bind.ipv6_enabled)?;
    writeln!(out, "│  Client Max Connections: {}", config.network.client.max_connections)?;
    writeln!(out, "│  Connect Timeout: {:?}", config.network.timeouts.connect)?;
    writeln!(out, "│  Request Timeout: {:?}", config.network.timeouts.request)?;
    writeln!(out, "│")?;
    writeln!(out, "├─ Security Configuration")?;
    writeln!(out, "│  Security Level: {:?}", config.security.security_level)?;
    writeln!(out, "│  Auth Method: {}", config.security.auth_method)?;
    writeln!(out, "│  Initial Trust Level: {:?}", config.security.initial_trust_level)?;
    writeln!(out, "│  TLS Cert Policy: {:?}", config.security.tls.cert_policy)?;
    if let Some(ref cert) = config.security.tls.cert_path {
        writeln!(out, "│  TLS Certificate: {cert}")?;
    }
    if let Some(ref key) = config.security.tls.key_path {
        writeln!(out, "│  TLS Key: {key}")?;
    }
    writeln!(out, "│  Require Valid Certs: {}", config.security.tls.require_valid_certs)?;
    writeln!(out, "│")?;
    writeln!(out, "├─ Performance Configuration")?;
    writeln!(out, "│  Enabled: {}", config.performance.enabled)?;
    writeln!(out, "│  Thread Pool Size: {}", config.performance.thread_pool_size)?;
    writeln!(out, "│")?;
    writeln!(out, "├─ Discovery Configuration")?;
    writeln!(out, "│  Mode: {:?}", config.discovery.mode)?;
    writeln!(out, "│  Backend: {}", config.discovery.backend)?;
    writeln!(out, "│  Port: {}", config.discovery.port)?;
    writeln!(out, "│  Protocol Version: {}", config.discovery.protocol_version)?;
    writeln!(out, "│  Session Rotation: {}s", config.discovery.session_rotation_interval)?;
    writeln!(out, "│")?;
    writeln!(out, "├─ Observability Configuration")?;
    writeln!(out, "│  Enabled: {}", config.observability.enabled)?;
    writeln!(out, "│  Metrics Interval: {}s", config.observability.metrics_interval)?;
    writeln!(out, "│  Metrics Enabled: {}", config.observability.metrics.enabled)?;
    writeln!(out, "│  Tracing Enabled: {}", config.observability.tracing.enabled)?;
    writeln!(out, "│  Tracing Level: {}", config.observability.tracing.level)?;
    writeln!(out, "│  Health Checks Enabled: {}", config.observability.health_checks.enabled)?;
    writeln!(out, "│")?;
    writeln!(out, "├─ Gaming Configuration")?;
    writeln!(out, "│  Enabled: {}", config.gaming.enabled)?;
    writeln!(out, "│  Protocol Version: {}", config.gaming.protocol_version)?;
    writeln!(out, "│")?;
    writeln!(out, "├─ Primal Configuration (Runtime Discovery)")?;
    writeln!(out, "│  Enabled: {}", config.primals.enabled)?;
    writeln!(out, "│  Discovery Method: {}", config.primals.discovery_method)?;
    writeln!(out, "│")?;
    writeln!(out, "├─ Federation Configuration")?;
    writeln!(out, "│  Cluster Name: {:?}", config.federation.cluster_name)?;
    writeln!(out, "│  Trust Escalation Policy: {:?}", config.federation.trust_escalation_policy)?;
    writeln!(out, "│  Initial Trust Level: {}", config.federation.initial_trust_level)?;
    writeln!(out, "│  Acceptance Policy: {:?}", config.federation.acceptance_policy)?;
    writeln!(
        out,
        "│  Require Hardware for Admin: {}",
        config.federation.require_hardware_for_admin
    )?;
    writeln!(out, "│")?;
    writeln!(out, "└─ Environment Configuration")?;
    writeln!(out, "   Name: {}", config.environment.name)?;
    writeln!(out, "   Deployment Mode: {}", config.environment.deployment_mode)?;
    writeln!(out)?;

    if !config.custom.is_empty() {
        writeln!(out, "Custom Fields:")?;
        for (key, value) in &config.custom {
            writeln!(out, "  • {key}: {value:?}")?;
        }
        writeln!(out)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use serde_json::json;
    use songbird_types::config::consolidated_canonical::DiscoveryMode;
    use songbird_types::config::{
        CanonicalDiscoveryConfig, CanonicalEnvironmentConfigNew, CanonicalSongbirdConfig,
        CanonicalSystemConfigNew,
    };

    #[test]
    fn default_canonical_config_constructs_and_validates() {
        let config = CanonicalSongbirdConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn from_env_loads_valid_defaults() {
        let config = CanonicalSongbirdConfig::from_env().expect("from_env");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn builder_merges_explicit_sections_with_defaults() {
        let built = CanonicalSongbirdConfig::builder()
            .environment(CanonicalEnvironmentConfigNew {
                name: String::from("staging"),
                deployment_mode: String::from("cluster"),
            })
            .build()
            .expect("builder");
        assert_eq!(built.environment.name, "staging");
        assert_eq!(built.environment.deployment_mode, "cluster");
        let defaults = CanonicalSongbirdConfig::default();
        assert_eq!(built.system.system_id, defaults.system.system_id);
        assert!(built.validate().is_ok());
    }

    #[test]
    fn validate_rejects_empty_system_environment() {
        let mut config = CanonicalSongbirdConfig::default();
        config.system.environment.clear();
        assert!(config.validate().is_err());
    }

    #[test]
    fn validate_rejects_empty_system_id() {
        let mut config = CanonicalSongbirdConfig::default();
        config.system.system_id.clear();
        assert!(config.validate().is_err());
    }

    #[test]
    fn validate_rejects_ephemeral_port_when_discovery_enabled() {
        let mut config = CanonicalSongbirdConfig::default();
        config.network.base_port = 0;
        config.discovery.mode = DiscoveryMode::Anonymous;
        assert!(config.validate().is_err());
    }

    #[test]
    fn validate_allows_ephemeral_port_when_discovery_disabled() {
        let mut config = CanonicalSongbirdConfig::default();
        config.network.base_port = 0;
        config.discovery = CanonicalDiscoveryConfig {
            mode: DiscoveryMode::Disabled,
            ..CanonicalDiscoveryConfig::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn builder_rejects_invalid_merged_config() {
        let bad_system = CanonicalSystemConfigNew {
            environment: String::new(),
            ..CanonicalSystemConfigNew::default()
        };
        assert!(CanonicalSongbirdConfig::builder().system(bad_system).build().is_err());
    }

    #[test]
    fn mask_secrets_is_identity_until_tls_fields_need_redaction() {
        let config = CanonicalSongbirdConfig::default();
        let masked = super::mask_secrets_in_config(config.clone());
        assert_eq!(config.system.system_id, masked.system.system_id);
        assert_eq!(config.network.base_port, masked.network.base_port);
    }

    #[tokio::test]
    async fn init_config_writes_expected_template_keys() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("songbird-init.env");
        let path_str = path.to_str().expect("utf8 path");
        super::init_config(path_str, false).await.expect("init_config");
        let contents = std::fs::read_to_string(&path).expect("read template");
        let default_port = songbird_config::defaults::ports::orchestrator_port();
        assert!(contents.contains(&format!("SONGBIRD_PORT={default_port}")));
        assert!(contents.contains("SONGBIRD_FAMILY_ID="));
        assert!(contents.contains("SONGBIRD_DISCOVERY_ENABLED=true"));
    }

    #[tokio::test]
    async fn init_config_overwrites_when_forced() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("twice.env");
        let path_str = path.to_str().expect("utf8 path");
        super::init_config(path_str, false).await.expect("first write");
        super::init_config(path_str, true).await.expect("force overwrite");
        let contents = std::fs::read_to_string(&path).expect("read");
        assert!(contents.contains("SONGBIRD_PORT="));
    }

    #[tokio::test]
    async fn init_config_template_family_id_follows_env_priority() {
        let _g = songbird_process_env::test_env_lock();
        for key in [
            "SONGBIRD_ORCHESTRATOR_FAMILY_ID",
            "SONGBIRD_ORCHESTRATOR_FAMILY",
            "BIOMEOS_FAMILY_ID",
            "SONGBIRD_FAMILY_ID",
            "FAMILY_ID",
        ] {
            songbird_process_env::remove_var(key);
        }
        songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID", "env-test-family");
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("family.env");
        let path_str = path.to_str().expect("utf8 path");
        super::init_config(path_str, false).await.expect("init_config");
        let contents = std::fs::read_to_string(&path).expect("read template");
        assert!(
            contents.contains("SONGBIRD_FAMILY_ID=env-test-family"),
            "template should embed resolved family_id from env chain: {contents}"
        );
        songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    }

    #[tokio::test]
    async fn init_config_empty_path_errors() {
        let err = super::init_config("", false).await;
        assert!(err.is_err(), "expected write to empty path to fail: {err:?}");
    }

    #[test]
    fn format_config_display_includes_sections_and_bind_port() {
        let config = CanonicalSongbirdConfig::default();
        let text = super::format_config_display(&config, false);
        assert!(text.contains("┌─ System Configuration"));
        assert!(text.contains("├─ Network Configuration"));
        assert!(text.contains("Primary Port:"));
        assert!(text.contains("├─ Security Configuration"));
    }

    #[test]
    fn format_config_display_includes_tls_lines_when_paths_set() {
        let mut config = CanonicalSongbirdConfig::default();
        config.security.tls.cert_path = Some(String::from("/path/with/secret-cert.pem"));
        config.security.tls.key_path = Some(String::from("/path/with/secret-key.pem"));
        let text = super::format_config_display(&config, false);
        assert!(text.contains("TLS Certificate: /path/with/secret-cert.pem"));
        assert!(text.contains("TLS Key: /path/with/secret-key.pem"));
    }

    #[test]
    fn format_config_display_show_secrets_matches_until_tls_redaction() {
        let mut config = CanonicalSongbirdConfig::default();
        config.security.tls.cert_path = Some(String::from("/c.pem"));
        let masked_view = super::format_config_display(&config, false);
        let show_all = super::format_config_display(&config, true);
        assert_eq!(masked_view, show_all);
    }

    #[test]
    fn format_config_display_lists_custom_fields() {
        let mut config = CanonicalSongbirdConfig::default();
        config.custom.insert(String::from("extra_key"), json!({"nested": true}));
        let text = super::format_config_display(&config, false);
        assert!(text.contains("Custom Fields:"));
        assert!(text.contains("extra_key"));
    }

    #[test]
    fn mask_secrets_identity_for_tls_material_paths() {
        let mut config = CanonicalSongbirdConfig::default();
        config.security.tls.cert_path = Some(String::from("/tls/ca-chain.pem"));
        config.security.tls.key_path = Some(String::from("/tls/private.key"));
        let masked = super::mask_secrets_in_config(config.clone());
        assert_eq!(masked.security.tls.cert_path, config.security.tls.cert_path);
        assert_eq!(masked.security.tls.key_path, config.security.tls.key_path);
        assert_eq!(masked.network.base_port, config.network.base_port);
    }
}
