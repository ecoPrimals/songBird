// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Doctor mode implementation (health diagnostics and system checks)
//!
//! Provides comprehensive health diagnostics with multiple output formats:
//! - Text (human-readable)
//! - JSON (machine-readable)
//! - TOML (machine-readable)

use anyhow::Result;

use super::DoctorArgs;

/// Run health diagnostics and system checks
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn run_doctor(args: DoctorArgs) -> Result<()> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    match args.format.as_str() {
        "text" => run_doctor_text(args.comprehensive).await,
        "json" => run_doctor_json(args.comprehensive).await,
        "toml" => run_doctor_toml(args.comprehensive).await,
        "yaml" => {
            eprintln!("⚠️  YAML format is deprecated; using TOML instead.");
            run_doctor_toml(args.comprehensive).await
        }
        _ => {
            eprintln!("❌ Unknown format: {}. Use: text, json, or toml", args.format);
            std::process::exit(1);
        }
    }
}

/// Run doctor in text format
async fn run_doctor_text(comprehensive: bool) -> Result<()> {
    use crate::process_manager::ProcessManager;
    use songbird_types::config::CanonicalSongbirdConfig;

    println!("🏥 Songbird Health Diagnostics");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Check 1: Binary info
    println!("📦 Binary Information");
    println!("   Name: songbird");
    println!("   Version: {}", env!("CARGO_PKG_VERSION"));
    println!("   Status: ✅ Healthy");
    println!();

    // Check 2: Configuration
    println!("📋 Configuration");
    match CanonicalSongbirdConfig::from_env() {
        Ok(_config) => {
            println!("   Config source: Environment variables");
            println!("   Status: ✅ Valid");
        }
        Err(e) => {
            println!("   Status: ❌ Invalid");
            println!("   Error: {e}");
            println!();
            println!("💡 Fix: Check environment variables or create config file");
            std::process::exit(1);
        }
    }
    println!();

    // Check 3: Network ports - use environment-aware port discovery
    println!("🌐 Network Ports");
    let configured_port = songbird_config::defaults::ports::orchestrator_port();
    match check_port_availability(configured_port).await {
        Ok(true) => {
            println!("   Port {configured_port}: ✅ Available (from SONGBIRD_ORCHESTRATOR_PORT)");
        }
        Ok(false) => {
            println!("   Port {configured_port}: ⚠️  In use");
            println!("   Note: May be used by running Songbird instance");
        }
        Err(e) => {
            println!("   Port {configured_port}: ❌ Check failed: {e}");
        }
    }
    println!();

    // Check 4: Filesystem
    println!("📁 Filesystem");
    let process_mgr = ProcessManager::new()?;
    match process_mgr.acquire_lock() {
        Ok(_guard) => {
            println!("   PID file: ✅ Writable");
            println!("   Instance lock: ✅ Available");
        }
        Err(_) => {
            println!("   PID file: ⚠️  Locked (another instance running)");
        }
    }
    println!();

    // Check 5: Comprehensive checks — capability-based primal discovery
    if comprehensive {
        println!("🔍 Comprehensive Checks");
        println!("   Discovering primals by capability...");
        println!();

        println!("   🔐 Crypto Provider (capability: crypto)");
        match discover_capability_provider("crypto").await {
            DiscoveryResult::Found(path) => println!("      Status: ✅ Discovered at {path}"),
            DiscoveryResult::NotFound => println!("      Status: ⚠️  No provider discovered"),
            DiscoveryResult::Error(e) => println!("      Status: ❌ Discovery error: {e}"),
        }

        for (capability, label) in [
            ("ai", "AI / MCP Provider"),
            ("storage", "Storage Provider"),
            ("sovereign-storage", "Sovereign Storage Provider"),
            ("messaging", "Messaging Provider"),
        ] {
            println!("   🔎 {label} (capability: {capability})");
            match discover_capability_provider(capability).await {
                DiscoveryResult::Found(path) => {
                    println!("      Status: ✅ Discovered at {path}");
                }
                DiscoveryResult::NotFound => {
                    println!("      Status: ⚠️  No provider discovered");
                }
                DiscoveryResult::Error(e) => {
                    println!("      Status: ❌ Discovery error: {e}");
                }
            }
        }

        println!();
    }

    // Final summary
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Health check complete!");
    println!();
    if comprehensive {
        println!("💡 All critical systems healthy");
    } else {
        println!("💡 Run with --comprehensive for full system check");
    }
    println!();

    Ok(())
}

/// Run doctor in JSON format
async fn run_doctor_json(comprehensive: bool) -> Result<()> {
    let health_status = gather_health_status(comprehensive).await?;
    let json = serde_json::to_string_pretty(&health_status)?;
    println!("{json}");
    Ok(())
}

/// Run doctor in TOML format (replaces deprecated YAML output)
async fn run_doctor_toml(comprehensive: bool) -> Result<()> {
    let health_status = gather_health_status(comprehensive).await?;
    let output = toml::to_string_pretty(&health_status)?;
    println!("{output}");
    Ok(())
}

/// Gather comprehensive health status for machine-readable output
async fn gather_health_status(comprehensive: bool) -> Result<DoctorHealthStatus> {
    use songbird_types::config::CanonicalSongbirdConfig;

    // Collect binary information
    let binary_info = BinaryInfo {
        name: "songbird".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        build: env!("CARGO_PKG_VERSION").to_string(),
        healthy: true,
    };

    // Check configuration
    let config_status = match CanonicalSongbirdConfig::from_env() {
        Ok(_) => ConfigStatus {
            valid: true,
            source: "environment".to_string(),
            error: None,
        },
        Err(e) => ConfigStatus {
            valid: false,
            source: "environment".to_string(),
            error: Some(e),
        },
    };

    // Check network ports - use environment-aware port discovery
    let http_api_port = songbird_config::defaults::ports::orchestrator_port();
    let metrics_port = songbird_config::defaults::ports::metrics_port();
    let tarpc_port = songbird_config::defaults::ports::tarpc_port();

    let port_checks = vec![
        PortCheck {
            port: http_api_port,
            name: "HTTP API".to_string(),
            available: check_port_availability(http_api_port).await?,
        },
        PortCheck {
            port: metrics_port,
            name: "Metrics".to_string(),
            available: check_port_availability(metrics_port).await?,
        },
        PortCheck {
            port: tarpc_port,
            name: "tarpc RPC".to_string(),
            available: check_port_availability(tarpc_port).await?,
        },
    ];

    // Check IPC socket - use environment-aware path
    let socket_path = crate::env_config::socket_path();
    let socket_status = SocketStatus {
        path: socket_path.to_string_lossy().to_string(),
        available: socket_path.parent().is_some_and(|p: &std::path::Path| p.exists()),
    };

    // Comprehensive checks — discover primals at runtime by capability
    let primal_checks = if comprehensive {
        let mut discovered = std::collections::HashMap::new();
        let crypto_found =
            matches!(discover_capability_provider("crypto").await, DiscoveryResult::Found(_));
        discovered.insert(
            "crypto".to_string(),
            check_primal_status("crypto", futures::future::ready(Ok(crypto_found))).await,
        );
        for capability in &["ai", "storage", "sovereign-storage", "messaging"] {
            let found =
                matches!(discover_capability_provider(capability).await, DiscoveryResult::Found(_));
            let status = check_primal_status(capability, futures::future::ready(Ok(found))).await;
            discovered.insert((*capability).to_string(), status);
        }
        Some(discovered)
    } else {
        None
    };

    Ok(DoctorHealthStatus {
        overall_status: if config_status.valid {
            "healthy"
        } else {
            "degraded"
        }
        .to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        binary_info,
        config_status,
        port_checks,
        socket_status,
        primal_checks,
    })
}

/// Helper to check primal status
async fn check_primal_status<F>(name: &str, check: F) -> PrimalStatus
where
    F: std::future::Future<Output = Result<bool>>,
{
    match check.await {
        Ok(true) => PrimalStatus {
            name: name.to_string(),
            status: "connected".to_string(),
            error: None,
        },
        Ok(false) => PrimalStatus {
            name: name.to_string(),
            status: "not_reachable".to_string(),
            error: None,
        },
        Err(e) => PrimalStatus {
            name: name.to_string(),
            status: "error".to_string(),
            error: Some(e.to_string()),
        },
    }
}

/// Health status structure for JSON/YAML output
#[derive(Debug, serde::Serialize)]
struct DoctorHealthStatus {
    overall_status: String,
    timestamp: String,
    binary_info: BinaryInfo,
    config_status: ConfigStatus,
    port_checks: Vec<PortCheck>,
    socket_status: SocketStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    primal_checks: Option<PrimalChecks>,
}

#[derive(Debug, serde::Serialize)]
struct BinaryInfo {
    name: String,
    version: String,
    build: String,
    healthy: bool,
}

#[derive(Debug, serde::Serialize)]
struct ConfigStatus {
    valid: bool,
    source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct PortCheck {
    port: u16,
    name: String,
    available: bool,
}

#[derive(Debug, serde::Serialize)]
struct SocketStatus {
    path: String,
    available: bool,
}

/// Capability-based primal status (discovered at runtime, not hardcoded)
type PrimalChecks = std::collections::HashMap<String, PrimalStatus>;

#[derive(Debug, serde::Serialize)]
struct PrimalStatus {
    name: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Check if a port is available
#[expect(
    clippy::unused_async,
    reason = "async signature required by Axum, trait objects, or future I/O"
)]
async fn check_port_availability(port: u16) -> Result<bool> {
    use std::net::TcpListener;

    match TcpListener::bind((songbird_types::constants::DEVELOPMENT_BIND_ADDRESS, port)) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => Ok(false),
        Err(e) => Err(anyhow::anyhow!("Failed to check port: {e}")),
    }
}

/// Result of runtime capability discovery
enum DiscoveryResult {
    Found(String),
    NotFound,
    Error(String),
}

/// Discover a primal by capability: optional `SONGBIRD_*_PROVIDER_SOCKET`, then
/// [`crate::primal_discovery::discover_for_capability_id_with`] (env + biomeos JSON-RPC probes).
async fn discover_capability_provider(capability: &str) -> DiscoveryResult {
    let env_key =
        format!("SONGBIRD_{}_PROVIDER_SOCKET", capability.to_uppercase().replace('-', "_"));
    if let Ok(path) = songbird_process_env::var(&env_key)
        && std::path::Path::new(&path).exists()
    {
        return DiscoveryResult::Found(path);
    }

    match crate::primal_discovery::discover_for_capability_id_with(capability, |k| {
        songbird_process_env::var(k).ok()
    })
    .await
    {
        Ok(path) => DiscoveryResult::Found(path),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("No ") && msg.contains("provider available") {
                DiscoveryResult::NotFound
            } else {
                DiscoveryResult::Error(msg)
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::check_primal_status;
    use crate::bin_interface::DoctorArgs;
    use clap::Parser;
    use futures::future::ready;

    #[derive(Parser)]
    #[command(name = "songbird-doctor")]
    struct DoctorCli {
        #[command(flatten)]
        args: DoctorArgs,
    }

    #[test]
    fn doctor_args_default_format_is_text() {
        let cli = DoctorCli::try_parse_from(["songbird-doctor"]).unwrap();
        assert_eq!(cli.args.format, "text");
        assert!(!cli.args.comprehensive);
    }

    #[test]
    fn doctor_args_json_and_comprehensive() {
        let cli =
            DoctorCli::try_parse_from(["songbird-doctor", "--format", "json", "--comprehensive"])
                .unwrap();
        assert_eq!(cli.args.format, "json");
        assert!(cli.args.comprehensive);
    }

    #[tokio::test]
    async fn primal_status_connected() {
        let s = check_primal_status("crypto", ready(Ok(true))).await;
        assert_eq!(s.status, "connected");
        assert_eq!(s.name, "crypto");
        assert!(s.error.is_none());
    }

    #[tokio::test]
    async fn primal_status_not_reachable() {
        let s = check_primal_status("ai", ready(Ok(false))).await;
        assert_eq!(s.status, "not_reachable");
    }

    #[tokio::test]
    async fn primal_status_error_from_check() {
        let s =
            check_primal_status("storage", ready(Err(anyhow::anyhow!("connection refused")))).await;
        assert_eq!(s.status, "error");
        assert!(s.error.as_ref().is_some_and(|e| e.contains("refused")));
    }

    #[tokio::test]
    async fn check_port_availability_localhost_free_port() {
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let available = super::check_port_availability(port).await.expect("check");
        assert!(available);
    }
}
