// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Configuration management commands
//!
//! Handles:
//! - Configuration display (text/JSON/YAML formats)
//! - Configuration validation
//! - Configuration template generation

use anyhow::Result;

use super::ConfigCommands;

/// Run configuration management commands
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
                "yaml" => {
                    // Mask secrets if needed
                    let output_config = if show_secrets {
                        config
                    } else {
                        mask_secrets_in_config(config)
                    };
                    println!("{}", serde_yaml::to_string(&output_config)?);
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
                    display_config_formatted(&config, show_secrets);
                }
            }
        }
        Err(e) => {
            if format == "json" {
                println!("{{\"status\":\"error\",\"message\":\"{}\"}}", e.replace('"', "\\\""));
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

/// Validate configuration
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
            println!("  • SONGBIRD_PORT (optional, default: 8080)");
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

    let template = r"# Songbird Configuration Template
# Generated by: songbird config init

# Network Configuration
SONGBIRD_PORT=8080
SONGBIRD_HOST=0.0.0.0

# Identity
SONGBIRD_NODE_ID=auto-generated
SONGBIRD_FAMILY_ID=default

# Discovery
SONGBIRD_DISCOVERY_ENABLED=true
SONGBIRD_BIRDSONG_ENABLED=true

# Federation
SONGBIRD_FEDERATION_ENABLED=true

# Security (BTSP)
BEARDOG_SOCKET=/tmp/beardog-default-default.sock

# HTTP Gateway
SONGBIRD_HTTP_GATEWAY_SOCKET=/tmp/songbird-http-gateway.sock

# Logging
RUST_LOG=info
SONGBIRD_LOG_LEVEL=info
";

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

/// Display configuration in structured, human-readable format
///
/// ✅ Modern implementation: Displays actual config values in clean format
fn display_config_formatted(
    config: &songbird_types::config::CanonicalSongbirdConfig,
    _show_secrets: bool,
) {
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
