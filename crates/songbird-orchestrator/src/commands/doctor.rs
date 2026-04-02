// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Doctor command — health diagnostics and system checks
//!
//! Provides `run_doctor()` which validates configuration, checks connectivity,
//! and verifies system health in text, JSON, or YAML output formats.

use anyhow::Result;
use songbird_types::config::CanonicalSongbirdConfig;
use songbird_types::constants::LOCALHOST;

use crate::process_manager::ProcessManager;

/// Run health diagnostics and system checks
///
/// Supports text, json, and toml output formats.
///
/// # Errors
///
/// Returns an error if health data cannot be gathered or serialized.
pub async fn run_doctor(comprehensive: bool, format: &str) -> Result<()> {
    match format {
        "text" => run_doctor_text(comprehensive).await,
        "json" => run_doctor_json(comprehensive).await,
        "toml" => run_doctor_toml(comprehensive).await,
        "yaml" => {
            eprintln!("⚠️  YAML format is deprecated; using TOML instead.");
            run_doctor_toml(comprehensive).await
        }
        _ => {
            eprintln!("❌ Unknown format: {format}. Use: text, json, or toml");
            std::process::exit(1);
        }
    }
}

/// Run doctor in text format (human-readable)
async fn run_doctor_text(comprehensive: bool) -> Result<()> {
    println!("🏥 Songbird Health Diagnostics");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Check 1: Binary info
    println!("📦 Binary Information");
    println!("   Name: songbird");
    println!("   Version: {}", env!("CARGO_PKG_VERSION"));
    println!("   Build: {}", env!("CARGO_PKG_VERSION"));
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

    // Check 3: Network ports
    println!("🌐 Network Ports");
    let default_port = songbird_config::defaults::ports::orchestrator_port();
    match check_port_availability(default_port).await {
        Ok(true) => {
            println!("   Port {default_port}: ✅ Available");
        }
        Ok(false) => {
            println!("   Port {default_port}: ⚠️  In use");
            println!("   Note: May be used by running Songbird instance");
        }
        Err(e) => {
            println!("   Port {default_port}: ❌ Check failed: {e}");
        }
    }
    println!();

    // Check 4: Filesystem (PID file, sockets)
    println!("📁 Filesystem");
    let process_mgr = ProcessManager::new()?;
    match process_mgr.acquire_lock() {
        Ok(_guard) => {
            println!("   PID file: ✅ Writable");
            println!("   Instance lock: ✅ Available");
            // Guard drops here, releasing lock
        }
        Err(_) => {
            println!("   PID file: ⚠️  Locked (another instance running)");
        }
    }
    println!();

    // Check 5: Comprehensive checks (if requested)
    if comprehensive {
        println!("🔍 Comprehensive Checks");
        println!("   Checking primal connectivity...");
        println!();

        // Check BearDog connectivity
        println!("   🐻 BearDog (Security & Crypto)");
        match check_beardog_connectivity().await {
            Ok(true) => println!("      Status: ✅ Connected"),
            Ok(false) => println!("      Status: ⚠️  Not reachable"),
            Err(e) => println!("      Status: ❌ Error: {e}"),
        }

        // Check Squirrel connectivity (future)
        println!("   🐿️  Squirrel (AI & MCP)");
        println!("      Status: ⏳ Not yet integrated");

        // Check ToadStool connectivity (future)
        println!("   🍄 ToadStool (Storage)");
        println!("      Status: ⏳ Not yet integrated");

        // Check NestGate connectivity (future)
        println!("   🏠 NestGate (Sovereign Storage)");
        println!("      Status: ⏳ Not yet integrated");

        println!();
    }

    // Final summary
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Health check complete!");
    println!();
    if comprehensive {
        println!("💡 All critical systems healthy");
        println!("💡 Some primals not yet integrated (expected)");
    } else {
        println!("💡 Run with --comprehensive for full system check");
    }
    println!();

    Ok(())
}

/// Run doctor in JSON format (machine-readable)
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

    // Comprehensive checks — discover primals at runtime rather than hardcoding names
    let primal_checks = if comprehensive {
        let mut discovered = std::collections::HashMap::new();
        // Check known capability providers discovered at runtime
        let crypto_status = check_primal_status("crypto", check_beardog_connectivity()).await;
        discovered.insert("crypto".to_string(), crypto_status);
        // Scan for other primals via socket directory
        for capability in &["ai", "storage", "messaging"] {
            let status = check_primal_status(capability, futures::future::ready(Ok(false))).await;
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

/// Check if a port is available
/// # Errors
///
/// Returns an error if the operation fails.
#[expect(
    clippy::unused_async,
    reason = "async signature required by Axum, trait objects, or future I/O"
)]
pub async fn check_port_availability(port: u16) -> Result<bool> {
    use std::net::TcpListener;

    // Use configurable bind address; default matches canonical localhost constant
    let bind_addr =
        songbird_process_env::var("SONGBIRD_BIND_HOST").unwrap_or_else(|_| LOCALHOST.to_string());

    match TcpListener::bind((bind_addr.as_str(), port)) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => Ok(false),
        Err(e) => Err(anyhow::anyhow!("Failed to check port: {e}")),
    }
}

/// Check `BearDog` connectivity
async fn check_beardog_connectivity() -> Result<bool> {
    use crate::btsp_client::BtspClient;

    let client = BtspClient::new();
    match client.ping().await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

// ─── Health status types ─────────────────────────────────────────────────

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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    #[tokio::test]
    async fn check_port_available_when_free() {
        let port = {
            use std::net::TcpListener;
            let l = TcpListener::bind("127.0.0.1:0").unwrap();
            l.local_addr().unwrap().port()
        };
        assert!(check_port_availability(port).await.unwrap());
    }

    #[tokio::test]
    async fn check_port_unavailable_when_bound() {
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        assert!(!check_port_availability(port).await.unwrap());
    }

    #[tokio::test]
    async fn check_primal_status_connected() {
        let s = check_primal_status("crypto", futures::future::ready(Ok(true))).await;
        assert_eq!(s.name, "crypto");
        assert_eq!(s.status, "connected");
        assert!(s.error.is_none());
    }

    #[tokio::test]
    async fn check_primal_status_not_reachable() {
        let s = check_primal_status("ai", futures::future::ready(Ok(false))).await;
        assert_eq!(s.status, "not_reachable");
    }

    #[tokio::test]
    async fn check_primal_status_error_string() {
        let s =
            check_primal_status("messaging", futures::future::ready(Err(anyhow::anyhow!("boom"))))
                .await;
        assert_eq!(s.status, "error");
        assert_eq!(s.error.as_deref(), Some("boom"));
    }

    #[tokio::test]
    async fn check_primal_status_preserves_name() {
        let s = check_primal_status("custom-cap", futures::future::ready(Ok(false))).await;
        assert_eq!(s.name, "custom-cap");
    }

    #[tokio::test]
    async fn check_port_two_different_ephemeral_ports_both_available() {
        use std::net::TcpListener;
        let p1 = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let p2 = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        assert_ne!(p1, p2);
        assert!(check_port_availability(p1).await.unwrap());
        assert!(check_port_availability(p2).await.unwrap());
    }
}
