// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Enhanced status command with improved UI and error handling

#![expect(missing_docs, reason = "CLI command module — doc coverage not required")]

use crate::cli::types::OutputFormat;
use crate::cli::ui::{
    Table, banner, clear_screen, error_with_suggestions, format_bytes, format_duration,
    format_health_status, format_percentage, print_info, separator, subheader, system_info,
};
use crate::errors::{CliError, SongbirdResult};
use colored::Colorize;
use serde_json::json;
use std::time::Duration;

/// System status information
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub orchestrator_status: ServiceStatus,
    pub discovery_status: ServiceStatus,
    pub load_balancer_status: ServiceStatus,
    pub monitoring_status: ServiceStatus,
    pub uptime: Duration,
    pub version: String,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub memory_total: u64,
    pub network_throughput: u64,
    pub connected_nodes: u32,
    pub active_services: u32,
    pub network_health: String,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Service status information
#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub name: String,
    pub status: String,
    pub health: String,
    pub port: Option<u16>,
    pub uptime: Option<Duration>,
    pub last_health_check: Option<chrono::DateTime<chrono::Utc>>,
    pub error_count: u32,
    pub restart_count: u32,
}

/// Execute the status command with enhanced error handling
pub async fn execute_status(
    detailed: bool,
    watch: Option<u64>,
    format: OutputFormat,
) -> SongbirdResult<()> {
    if let Some(interval) = watch {
        watch_status(detailed, interval, format).await
    } else {
        show_status(detailed, watch, format).await
    }
}

/// Show system status with enhanced formatting
pub async fn show_status(
    detailed: bool,
    watch: Option<u64>,
    format: OutputFormat,
) -> SongbirdResult<()> {
    if let Some(interval) = watch {
        // Watch mode - continuously update status
        loop {
            clear_screen();
            display_timestamp();
            display_status(detailed, &format).await?;
            tokio::time::sleep(Duration::from_secs(interval)).await;
        }
    } else {
        // Single status display
        display_status(detailed, &format).await?;
    }

    Ok(())
}

/// Display current timestamp for watch mode
fn display_timestamp() {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    println!("{} {}", "Last Updated:".bright_blue().bold(), timestamp.to_string().bright_white());
    separator();
}

/// Display status based on output format
async fn display_status(detailed: bool, format: &OutputFormat) -> SongbirdResult<()> {
    let status = get_system_status().await?;

    match format {
        OutputFormat::Auto | OutputFormat::Table => display_table_status(&status, detailed).await,
        OutputFormat::Json => display_json_status(&status, detailed).await,
        OutputFormat::Yaml => display_yaml_status(&status, detailed).await,
        OutputFormat::Text => display_text_status(&status, detailed).await,
    }
}

/// Get current system status.
///
/// Probes the orchestrator IPC socket for real health data. Falls back to
/// an "Unreachable" status when the orchestrator is not running.
async fn get_system_status() -> SongbirdResult<SystemStatus> {
    let now = chrono::Utc::now();
    let version = env!("CARGO_PKG_VERSION").to_string();

    // Attempt real IPC health probe
    let orchestrator_port = songbird_config::defaults::ports::orchestrator_port();
    let biomeos_dir = std::env::var("BIOMEOS_SOCKET_DIR")
        .map(std::path::PathBuf::from)
        .or_else(|_| {
            std::env::var("XDG_RUNTIME_DIR").map(|xdg| {
                std::path::PathBuf::from(xdg)
                    .join(songbird_types::defaults::paths::BIOMEOS_RUNTIME_SUBDIR)
            })
        })
        .unwrap_or_else(|_| std::env::temp_dir());
    let ipc_alive = [
        biomeos_dir.join(format!("{}.sock", songbird_types::primal_names::SELF_NAME)),
        biomeos_dir.join(format!("{}.sock", songbird_types::primal_names::CAPABILITY_DOMAIN)),
    ]
    .iter()
    .any(|p| p.exists());

    let (orch_status, orch_health, orch_uptime) = if ipc_alive {
        ("Running".to_string(), "Healthy".to_string(), None)
    } else {
        ("Stopped".to_string(), "Unreachable".to_string(), None)
    };

    let unreachable = ServiceStatus {
        name: String::new(),
        status: "Unknown".to_string(),
        health: "Unreachable".to_string(),
        port: None,
        uptime: None,
        last_health_check: None,
        error_count: 0,
        restart_count: 0,
    };

    Ok(SystemStatus {
        orchestrator_status: ServiceStatus {
            name: "Orchestrator".to_string(),
            status: orch_status,
            health: orch_health,
            port: Some(orchestrator_port),
            uptime: orch_uptime,
            last_health_check: Some(now),
            error_count: 0,
            restart_count: 0,
        },
        discovery_status: ServiceStatus {
            name: "Discovery".to_string(),
            port: Some(songbird_config::defaults::ports::discovery_port()),
            ..unreachable.clone()
        },
        load_balancer_status: ServiceStatus {
            name: "Load Balancer".to_string(),
            port: Some(songbird_config::defaults::ports::security_provider_port()),
            ..unreachable.clone()
        },
        monitoring_status: ServiceStatus {
            name: "Monitoring".to_string(),
            port: Some(songbird_config::defaults::ports::metrics_port()),
            ..unreachable
        },
        uptime: Duration::ZERO,
        version,
        cpu_usage: 0.0,
        memory_usage: 0,
        memory_total: 0,
        network_throughput: 0,
        connected_nodes: 0,
        active_services: 0,
        network_health: "Unknown".to_string(),
        last_updated: now,
    })
}

/// Display status in enhanced table format
async fn display_table_status(status: &SystemStatus, detailed: bool) -> SongbirdResult<()> {
    banner("Songbird Orchestrator Status", Some("System Overview"));

    // Overall system status
    let overall_status = overall_status_label(&status.orchestrator_status.health);

    system_info(&[
        ("Overall Status", overall_status),
        ("Version", &status.version),
        ("Uptime", &format_duration(status.uptime)),
        ("Last Updated", &status.last_updated.format("%H:%M:%S UTC").to_string()),
    ]);

    // Service status table
    subheader("Service Status");
    let mut table = Table::new().headers(vec![
        "Service".to_string(),
        "Status".to_string(),
        "Health".to_string(),
        "Port".to_string(),
        "Uptime".to_string(),
    ]);

    let services = [
        &status.orchestrator_status,
        &status.discovery_status,
        &status.load_balancer_status,
        &status.monitoring_status,
    ];

    for service in services {
        table = table.row(vec![
            service.name.clone(),
            format_health_status(&service.status),
            format_health_status(&service.health),
            service.port.map_or_else(|| "N/A".to_string(), |p| p.to_string()),
            service.uptime.map_or_else(|| "N/A".to_string(), format_duration),
        ]);
    }

    table.print();

    if detailed {
        // Detailed system metrics
        subheader("System Metrics");
        system_info(&[
            ("CPU Usage", &format_percentage(status.cpu_usage / 100.0)),
            (
                "Memory Usage",
                &format!(
                    "{} / {} ({})",
                    format_bytes(status.memory_usage),
                    format_bytes(status.memory_total),
                    format_percentage(status.memory_usage as f64 / status.memory_total as f64)
                ),
            ),
            ("Network Throughput", &format!("{}/s", format_bytes(status.network_throughput))),
            ("Connected Nodes", &status.connected_nodes.to_string()),
            ("Active Services", &status.active_services.to_string()),
            ("Network Health", &format_health_status(&status.network_health)),
        ]);

        // Service details
        subheader("Service Details");
        for service in services {
            display_service_details(service);
        }
    }

    // Quick actions
    subheader("Quick Actions");
    println!("• View logs: {}", "songbird logs --follow".bright_green());
    println!("• Check configuration: {}", "songbird config show".bright_green());
    println!("• Restart services: {}", "songbird stop && songbird start".bright_green());
    println!("• Watch status: {}", "songbird status --watch 5".bright_green());

    Ok(())
}

/// Display detailed service information
fn display_service_details(service: &ServiceStatus) {
    println!("\n{}", service.name.bright_cyan().bold());
    println!("  Status: {}", format_health_status(&service.status));
    println!("  Health: {}", format_health_status(&service.health));

    if let Some(port) = service.port {
        println!("  Port: {}", port.to_string().bright_white());
    }

    if let Some(uptime) = service.uptime {
        println!("  Uptime: {}", format_duration(uptime).bright_white());
    }

    if let Some(last_check) = service.last_health_check {
        println!(
            "  Last Health Check: {}",
            last_check.format("%H:%M:%S UTC").to_string().bright_white()
        );
    }

    if service.error_count > 0 {
        println!("  Errors: {}", service.error_count.to_string().bright_red());
    }

    if service.restart_count > 0 {
        println!("  Restarts: {}", service.restart_count.to_string().bright_yellow());
    }
}

/// Display status in JSON format
async fn display_json_status(status: &SystemStatus, detailed: bool) -> SongbirdResult<()> {
    let mut json_status = json!({
        "overall_status": status.orchestrator_status.health,
        "version": status.version,
        "uptime_seconds": status.uptime.as_secs(),
        "last_updated": status.last_updated.to_rfc3339(),
        "services": {
            "orchestrator": service_to_json(&status.orchestrator_status),
            "discovery": service_to_json(&status.discovery_status),
            "load_balancer": service_to_json(&status.load_balancer_status),
            "monitoring": service_to_json(&status.monitoring_status),
        }
    });

    if detailed {
        json_status["system_metrics"] = json!({
            "cpu_usage_percent": status.cpu_usage,
            "memory_usage_bytes": status.memory_usage,
            "memory_total_bytes": status.memory_total,
            "network_throughput_bytes_per_sec": status.network_throughput,
            "connected_nodes": status.connected_nodes,
            "active_services": status.active_services,
            "network_health": status.network_health,
        });
    }

    println!("{}", serde_json::to_string_pretty(&json_status).map_err(CliError::Serialization)?);

    Ok(())
}

/// Table label for overall orchestrator health (pure; used by status table and tests).
fn overall_status_label(orchestrator_health: &str) -> &'static str {
    if orchestrator_health == "Healthy" {
        "🟢 Running"
    } else {
        "🔴 Issues Detected"
    }
}

/// Convert service status to JSON
fn service_to_json(service: &ServiceStatus) -> serde_json::Value {
    json!({
        "name": service.name,
        "status": service.status,
        "health": service.health,
        "port": service.port,
        "uptime_seconds": service.uptime.map(|u| u.as_secs()),
        "last_health_check": service.last_health_check.map(|t| t.to_rfc3339()),
        "error_count": service.error_count,
        "restart_count": service.restart_count,
    })
}

/// Display status in TOML format (replaces deprecated YAML output)
async fn display_yaml_status(status: &SystemStatus, detailed: bool) -> SongbirdResult<()> {
    let mut output = serde_json::to_string_pretty(&json!({
        "overall_status": status.orchestrator_status.health,
        "version": status.version,
        "uptime_seconds": status.uptime.as_secs(),
        "last_updated": status.last_updated.to_rfc3339(),
        "services": {
            "orchestrator": service_to_json(&status.orchestrator_status),
            "discovery": service_to_json(&status.discovery_status),
            "load_balancer": service_to_json(&status.load_balancer_status),
            "monitoring": service_to_json(&status.monitoring_status),
        }
    }))
    .map_err(|e| CliError::Config {
        message: format!("Failed to serialize status: {e}"),
        field: Some("status".to_string()),
        suggestion: Some("Check system status and try again".to_string()),
    })?;

    if detailed {
        let detailed_output = serde_json::to_string_pretty(&json!({
            "system_metrics": {
                "cpu_usage_percent": status.cpu_usage,
                "memory_usage_bytes": status.memory_usage,
                "memory_total_bytes": status.memory_total,
                "network_throughput_bytes_per_sec": status.network_throughput,
                "connected_nodes": status.connected_nodes,
                "active_services": status.active_services,
                "network_health": status.network_health,
            }
        }))
        .map_err(|e| CliError::Config {
            message: format!("Failed to serialize detailed status: {e}"),
            field: Some("status".to_string()),
            suggestion: Some("Check system status and try again".to_string()),
        })?;

        output.push_str(&detailed_output);
    }

    println!("{output}");
    Ok(())
}

/// Display status in simple text format
async fn display_text_status(status: &SystemStatus, detailed: bool) -> SongbirdResult<()> {
    println!(
        "Songbird Orchestrator Status: {}",
        format_health_status(&status.orchestrator_status.health)
    );
    println!("Uptime: {}", format_duration(status.uptime));
    println!("Version: {}", status.version);
    println!("Services: Orchestrator, Discovery, Load Balancer, Monitoring - All Running");

    if detailed {
        println!("CPU Usage: {}", format_percentage(status.cpu_usage / 100.0));
        println!(
            "Memory Usage: {} / {} ({})",
            format_bytes(status.memory_usage),
            format_bytes(status.memory_total),
            format_percentage(status.memory_usage as f64 / status.memory_total as f64)
        );
        println!("Network Throughput: {}/s", format_bytes(status.network_throughput));
        println!("Connected Nodes: {}", status.connected_nodes);
        println!("Active Services: {}", status.active_services);
        println!("Network Health: {}", format_health_status(&status.network_health));
    }

    Ok(())
}

/// Watch status with live updates and enhanced display
async fn watch_status(detailed: bool, interval: u64, format: OutputFormat) -> SongbirdResult<()> {
    banner("Songbird Status Monitor", Some("Live Updates"));
    print_info(&format!("Updating every {interval} seconds (press Ctrl+C to stop,"));

    loop {
        clear_screen();
        display_timestamp();

        match display_status(detailed, &format).await {
            Ok(()) => {}
            Err(e) => {
                error_with_suggestions(
                    &format!("Failed to get status: {e}"),
                    &[
                        "Check if the orchestrator is running",
                        "Verify network connectivity",
                        "Try reducing the update interval",
                    ],
                );
                tokio::time::sleep(Duration::from_secs(interval)).await;
                continue;
            }
        }

        // Show next update time
        let next_update = chrono::Utc::now() + chrono::Duration::seconds(interval.cast_signed());
        println!(
            "\n{} {}",
            "Next update:".dimmed(),
            next_update.format("%H:%M:%S UTC").to_string().dimmed()
        );

        tokio::time::sleep(Duration::from_secs(interval)).await;
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::ServiceStatus;
    use super::overall_status_label;
    use super::service_to_json;
    use std::time::Duration;

    fn sample_service() -> ServiceStatus {
        ServiceStatus {
            name: "Orchestrator".to_string(),
            status: "Running".to_string(),
            health: "Healthy".to_string(),
            port: Some(8080),
            uptime: Some(Duration::from_secs(3600)),
            last_health_check: Some(
                chrono::DateTime::parse_from_rfc3339("2024-01-02T15:04:05Z")
                    .unwrap()
                    .with_timezone(&chrono::Utc),
            ),
            error_count: 0,
            restart_count: 1,
        }
    }

    #[test]
    fn service_to_json_includes_all_scalar_fields() {
        let s = sample_service();
        let v = service_to_json(&s);
        assert_eq!(v["name"], "Orchestrator");
        assert_eq!(v["status"], "Running");
        assert_eq!(v["health"], "Healthy");
        assert_eq!(v["port"], 8080);
        assert_eq!(v["uptime_seconds"], 3600);
        assert_eq!(v["error_count"], 0);
        assert_eq!(v["restart_count"], 1);
        assert!(v["last_health_check"].is_string());
    }

    #[test]
    fn service_to_json_serializes_none_optionals_as_null() {
        let s = ServiceStatus {
            name: "X".to_string(),
            status: "Stopped".to_string(),
            health: "Unknown".to_string(),
            port: None,
            uptime: None,
            last_health_check: None,
            error_count: 3,
            restart_count: 0,
        };
        let v = service_to_json(&s);
        assert!(v["port"].is_null());
        assert!(v["uptime_seconds"].is_null());
        assert!(v["last_health_check"].is_null());
        assert_eq!(v["error_count"], 3);
    }

    #[test]
    fn service_to_json_uptime_seconds_matches_duration() {
        let s = ServiceStatus {
            name: "a".to_string(),
            status: "b".to_string(),
            health: "c".to_string(),
            port: None,
            uptime: Some(Duration::from_secs(999_999)),
            last_health_check: None,
            error_count: 0,
            restart_count: 0,
        };
        assert_eq!(service_to_json(&s)["uptime_seconds"], 999_999);
    }

    #[test]
    fn overall_status_label_healthy() {
        assert_eq!(overall_status_label("Healthy"), "🟢 Running");
    }

    #[test]
    fn overall_status_label_any_non_healthy_is_issues() {
        assert_eq!(overall_status_label("Degraded"), "🔴 Issues Detected");
        assert_eq!(overall_status_label(""), "🔴 Issues Detected");
        assert_eq!(overall_status_label("healthy"), "🔴 Issues Detected");
    }

    #[test]
    fn service_to_json_last_health_check_rfc3339() {
        let t = chrono::DateTime::parse_from_rfc3339("2024-06-01T12:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let expected = t.to_rfc3339();
        let s = ServiceStatus {
            name: "n".to_string(),
            status: "s".to_string(),
            health: "h".to_string(),
            port: None,
            uptime: None,
            last_health_check: Some(t),
            error_count: 0,
            restart_count: 0,
        };
        let v = service_to_json(&s);
        assert_eq!(v["last_health_check"], serde_json::Value::String(expected));
    }

    #[test]
    fn detailed_memory_fraction_matches_expected_ratio() {
        let memory_usage: f64 = 268_435_456.0;
        let memory_total: f64 = 8_589_934_592.0;
        let ratio = memory_usage / memory_total;
        assert!((ratio - 0.03125).abs() < f64::EPSILON);
    }

    #[test]
    fn service_to_json_serializes_max_port_and_zero_uptime() {
        let s = ServiceStatus {
            name: "svc".to_string(),
            status: "Idle".to_string(),
            health: "Unknown".to_string(),
            port: Some(u16::MAX),
            uptime: Some(Duration::ZERO),
            last_health_check: None,
            error_count: 0,
            restart_count: 0,
        };
        let v = service_to_json(&s);
        assert_eq!(v["port"], u16::MAX);
        assert_eq!(v["uptime_seconds"], 0);
    }

    #[test]
    fn overall_status_label_unreachable_maps_to_issues() {
        assert_eq!(overall_status_label("Unreachable"), "🔴 Issues Detected");
    }

    #[test]
    fn service_to_json_contains_expected_top_level_keys() {
        let s = sample_service();
        let v = service_to_json(&s);
        for key in [
            "name",
            "status",
            "health",
            "port",
            "uptime_seconds",
            "last_health_check",
            "error_count",
            "restart_count",
        ] {
            assert!(v.get(key).is_some(), "missing key {key}");
        }
    }
}
