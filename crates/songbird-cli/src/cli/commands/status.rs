//! Enhanced status command with improved UI and error handling

use crate::cli::ui::*;
use crate::cli::{CliError, CliResult, OutputFormat};
use colored::*;
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
) -> CliResult<()> {
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
) -> CliResult<()> {
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
    println!(
        "{} {}",
        "Last Updated:".bright_blue().bold(),
        timestamp.to_string().bright_white()
    );
    separator();
}

/// Display status based on output format
async fn display_status(detailed: bool, format: &OutputFormat) -> CliResult<()> {
    let status = get_system_status().await?;

    match format {
        OutputFormat::Auto | OutputFormat::Table => display_table_status(&status, detailed).await,
        OutputFormat::Json => display_json_status(&status, detailed).await,
        OutputFormat::Yaml => display_yaml_status(&status, detailed).await,
        OutputFormat::Text => display_text_status(&status, detailed).await,
    }
}

/// Get current system status
async fn get_system_status() -> CliResult<SystemStatus> {
    // This would normally query actual system components
    // For now, we'll return simulated status
    Ok(SystemStatus {
        orchestrator_status: ServiceStatus {
            name: "Orchestrator".to_string(),
            status: "Running".to_string(),
            health: "Healthy".to_string(),
            port: Some(8080),
            uptime: Some(Duration::from_secs(9492)), // 2h 38m 12s
            last_health_check: Some(chrono::Utc::now() - chrono::Duration::seconds(5)),
            error_count: 0,
            restart_count: 0,
        },
        discovery_status: ServiceStatus {
            name: "Discovery".to_string(),
            status: "Running".to_string(),
            health: "Healthy".to_string(),
            port: Some(8081),
            uptime: Some(Duration::from_secs(9480)),
            last_health_check: Some(chrono::Utc::now() - chrono::Duration::seconds(3)),
            error_count: 2,
            restart_count: 0,
        },
        load_balancer_status: ServiceStatus {
            name: "Load Balancer".to_string(),
            status: "Running".to_string(),
            health: "Healthy".to_string(),
            port: Some(8082),
            uptime: Some(Duration::from_secs(9475)),
            last_health_check: Some(chrono::Utc::now() - chrono::Duration::seconds(2)),
            error_count: 0,
            restart_count: 0,
        },
        monitoring_status: ServiceStatus {
            name: "Monitoring".to_string(),
            status: "Running".to_string(),
            health: "Healthy".to_string(),
            port: Some(8083),
            uptime: Some(Duration::from_secs(9470)),
            last_health_check: Some(chrono::Utc::now() - chrono::Duration::seconds(1)),
            error_count: 1,
            restart_count: 0,
        },
        uptime: Duration::from_secs(9492),
        version: env!("CARGO_PKG_VERSION").to_string(),
        cpu_usage: 12.5,
        memory_usage: 268435456,   // 256 MB
        memory_total: 8589934592,  // 8 GB
        network_throughput: 46080, // 45 KB/s
        connected_nodes: 3,
        active_services: 12,
        network_health: "Good".to_string(),
        last_updated: chrono::Utc::now(),
    })
}

/// Display status in enhanced table format
async fn display_table_status(status: &SystemStatus, detailed: bool) -> CliResult<()> {
    banner("Songbird Orchestrator Status", Some("System Overview"));

    // Overall system status
    let overall_status = if status.orchestrator_status.health == "Healthy" {
        "🟢 Running"
    } else {
        "🔴 Issues Detected"
    };

    system_info(&[
        ("Overall Status", overall_status),
        ("Version", &status.version),
        ("Uptime", &format_duration(status.uptime)),
        (
            "Last Updated",
            &status.last_updated.format("%H:%M:%S UTC").to_string(),
        ),
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
            service.port.map_or("N/A".to_string(), |p| p.to_string()),
            service
                .uptime
                .map_or("N/A".to_string(), format_duration),
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
            (
                "Network Throughput",
                &format!("{}/s", format_bytes(status.network_throughput)),
            ),
            ("Connected Nodes", &status.connected_nodes.to_string()),
            ("Active Services", &status.active_services.to_string()),
            (
                "Network Health",
                &format_health_status(&status.network_health),
            ),
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
    println!(
        "• Check configuration: {}",
        "songbird config show".bright_green()
    );
    println!(
        "• Restart services: {}",
        "songbird stop && songbird start".bright_green()
    );
    println!(
        "• Watch status: {}",
        "songbird status --watch 5".bright_green()
    );

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
        println!(
            "  Restarts: {}",
            service.restart_count.to_string().bright_yellow()
        );
    }
}

/// Display status in JSON format
async fn display_json_status(status: &SystemStatus, detailed: bool) -> CliResult<()> {
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

    println!(
        "{}",
        serde_json::to_string_pretty(&json_status).map_err(|e| CliError::command_error(
            &format!("Failed to serialize JSON: {e}"),
            Some("status"),
            "Check system status and try again"
        ))?
    );

    Ok(())
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

/// Display status in YAML format
async fn display_yaml_status(status: &SystemStatus, detailed: bool) -> CliResult<()> {
    let mut yaml_status = serde_yaml::to_string(&json!({
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
    .map_err(|e| {
        CliError::command_error(
            &format!("Failed to serialize YAML: {e}"),
            Some("status"),
            "Check system status and try again",
        )
    })?;

    if detailed {
        let detailed_yaml = serde_yaml::to_string(&json!({
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
        .map_err(|e| {
            CliError::command_error(
                &format!("Failed to serialize detailed YAML: {e}"),
                Some("status"),
                "Check system status and try again",
            )
        })?;

        yaml_status.push_str(&detailed_yaml);
    }

    println!("{yaml_status}");
    Ok(())
}

/// Display status in simple text format
async fn display_text_status(status: &SystemStatus, detailed: bool) -> CliResult<()> {
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
        println!(
            "Network Throughput: {}/s",
            format_bytes(status.network_throughput)
        );
        println!("Connected Nodes: {}", status.connected_nodes);
        println!("Active Services: {}", status.active_services);
        println!(
            "Network Health: {}",
            format_health_status(&status.network_health)
        );
    }

    Ok(())
}

/// Watch status with live updates and enhanced display
async fn watch_status(detailed: bool, interval: u64, format: OutputFormat) -> CliResult<()> {
    banner("Songbird Status Monitor", Some("Live Updates"));
    print_info(&format!(
        "Updating every {interval} seconds (press Ctrl+C to stop)"
    ));

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
        let next_update = chrono::Utc::now() + chrono::Duration::seconds(interval as i64);
        println!(
            "\n{} {}",
            "Next update:".dimmed(),
            next_update.format("%H:%M:%S UTC").to_string().dimmed()
        );

        tokio::time::sleep(Duration::from_secs(interval)).await;
    }
}
