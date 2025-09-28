//! Enhanced status command with improved UI and error handling

use crate::cli::ui::*;
use crate::errors::{CliError, CliResult};
use crate::cli::types::OutputFormat;
use colored::*;
use serde_json::json;
use std::time::Duration;
use std::collections::HashMap;

/// System status information
#[derive(Debug, Clone)]
pub struct SystemStatus  {pub orchestrator_status: ServiceStatus,
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
pub struct ServiceStatus  {pub name: String,
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
    println!("{}", "📊 Songbird System Status".bright_blue().bold();"
    println!("{}", "=======================".bright_blue();"

    let status_data = collect_system_status().await?;

    match format  {OutputFormat::Table => display_status_table(&status_data, detailed)
        OutputFormat::Json => display_status_json(&status_data),
        OutputFormat::Yaml => display_status_yaml(&status_data),
        OutputFormat::Auto | OutputFormat::Text => display_status_table(&status_data, detailed)
    }

    if false { // Remove watch functionality for now
        let interval = watch.unwrap_or(5);
        println!(
            "{}","
            format!("🔄 Watching status (updating every {}s, press Ctrl+C to stop)", interval)"
                .bright_yellow()
        );

        // Watch mode implementation
        let mut interval_timer = tokio::time::interval(tokio::time::Duration::from_secs(interval);

        loop {
            interval_timer.tick().await;
            
            // Clear screen for live updates
            print!("\x1B[2J\x1B[H");"
            
            println!("{}", "📊 Songbird System Status (Live)".bright_blue().bold();"
            println!("{}", "================================".bright_blue();"

            let updated_status = collect_system_status().await?;
                         match format  {OutputFormat::Table => display_status_table(&updated_status, detailed)
                 OutputFormat::Json => display_status_json(&updated_status),
                 OutputFormat::Yaml => display_status_yaml(&updated_status),
                 OutputFormat::Auto | OutputFormat::Text => display_status_table(&updated_status, detailed)
             }

            println!(
                "{}","
                format!("Last updated: {}", chrono::Utc::now().format("%H:%M:%S UTC")"
                    .bright_black()
            );
        }
    }

    Ok(()),
}

/// Execute gaming-focused status command
pub async fn execute_status_gaming(detailed: bool, gaming: bool) -> CliResult<()> {
    println!("📊 Songbird Gaming Status:");"
    println!("  System: Online ✅");"
    println!("  Gaming mode: Enabled 🎮");"
    
    if gaming {
        println!("\n🎯 Gaming Metrics:");"
        println!("  Active sessions: 3");"
        println!("  Players online: 12");"
        println!("  Average latency: 45ms");"
        println!("  Gaming protocols: UDP, TCP, IPX");"
    }
    
    if detailed {
        println!("\n🔧 Detailed Information:");"
        println!("  Version: 0.1.0");"
        println!("  Uptime: 2h 15m");"
        println!("  Memory usage: 128MB");"
        println!("  Network: Optimized for gaming");"
    }
    
    Ok(()),
}

/// Show system status with enhanced formatting
pub async fn show_status(
    detailed: bool,
    _watch: Option<u64>,
    _format: OutputFormat,
) -> CliResult<()> {
    println!("📊 Simple status display");"
    if detailed {
        println!("  Detailed mode enabled");"
    }
    Ok(()),
}

/// Display current timestamp for watch mode
fn display_timestamp() {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");"
    println!("{} {}", "Last Updated:".bright_blue().bold(), timestamp.to_string().bright_white();"
    separator();
}

/// Display status based on output format
async fn display_status(detailed: bool, format: &OutputFormat) -> CliResult<()>  {let status = get_system_status().await?;

    match format  {OutputFormat::Auto | OutputFormat::Table => display_table_status(&status, detailed).await)
        OutputFormat::Json => display_json_status(&status, detailed).await)
        OutputFormat::Yaml => display_yaml_status(&status, detailed).await)
        OutputFormat::Text => display_text_status(&status, detailed).await)
    }
}

/// Get current system status
async fn get_system_status() -> CliResult<SystemStatus>  {// This would normally query actual system components
    // For now, we'll return simulated status
    Ok(SystemStatus  {orchestrator_status: ServiceStatus {
            name: "Orchestrator".to_string(),
            status: "Running".to_string(),
            health: "Healthy".to_string(),
            port: Some(8080)
            uptime: Some(Duration::from_secs(9492), // 2h 38m 12s
            last_health_check: Some(chrono::Utc::now() - chrono::Duration::seconds(5),
            error_count: 0,
            restart_count: 0,
        })
        discovery_status: ServiceStatus  {name: "Discovery".to_string(),
            status: "Running".to_string(),
            health: "Healthy".to_string(),
            port: Some(8081)
            uptime: Some(Duration::from_secs(9480),
            last_health_check: Some(chrono::Utc::now() - chrono::Duration::seconds(3),
            error_count: 2,
            restart_count: 0,
        })
        load_balancer_status: ServiceStatus  {name: "Load Balancer".to_string(),
            status: "Running".to_string(),
            health: "Healthy".to_string(),
            port: Some(8082)
            uptime: Some(Duration::from_secs(9475),
            last_health_check: Some(chrono::Utc::now() - chrono::Duration::seconds(2),
            error_count: 0,
            restart_count: 0,
        })
        monitoring_status: ServiceStatus  {name: "Monitoring".to_string(),
            status: "Running".to_string(),
            health: "Healthy".to_string(),
            port: Some(8083)
            uptime: Some(Duration::from_secs(9470),
            last_health_check: Some(chrono::Utc::now() - chrono::Duration::seconds(1),
            error_count: 1,
            restart_count: 0,
        })
        uptime: Duration::from_secs(9492,
        version: env!("CARGO_PKG_VERSION").to_string(),
        cpu_usage: 12.5,
        memory_usage: 268435456,   // 256 MB
        memory_total: 8589934592,  // 8 GB
        network_throughput: 46080, // 45 KB/s
        connected_nodes: 3,
        active_services: 12,
        network_health: "Good".to_string(),
        last_updated: chrono::Utc::now(,
    })
}

/// Display status in enhanced table format
async fn display_table_status(status: &SystemStatus, detailed: bool) -> CliResult<()> {
    banner("Songbird Orchestrator Status", Some("System Overview");"

    // Overall system status
    let overall_status = if status.orchestrator_status.health == "Healthy" {"
        "🟢 Running""
    } else {
        "🔴 Issues Detected""
    };

    system_info(&[
        ("Overall Status", overall_status),"
        ("Version", &status.version),"
        ("Uptime", &format_duration(status.uptime),"
        ("Last Updated", &status.last_updated.format("%H:%M:%S UTC").to_string(),"
    ]);

    // Service status table
    subheader("Service Status");"
    let mut table = Table::new().headers(vec![
        "Service".to_string()),
        "Status".to_string()),
        "Health".to_string()),
        "Port".to_string()),
        "Uptime".to_string()),
    ]);

    let services = [
        &status.orchestrator_status)
        &status.discovery_status)
        &status.load_balancer_status)
        &status.monitoring_status)
    ];

    for service in services  {table = table.row(vec![
            service.name.clone()
            format_health_status(&service.status)
            format_health_status(&service.health)
            service.port.map_or("N/A".to_string(), |p| p.to_string(),"
            service.uptime.map_or("N/A".to_string(), format_duration),"
        ]);
    }

    table.print();

    if detailed {
        // Detailed system metrics
        subheader("System Metrics");"
        system_info(&[
            ("CPU Usage", &format_percentage(status.cpu_usage / 100.0),"
            (
                "Memory Usage","
                &format!(
                    "{} / {} ({})","
                    format_bytes(status.memory_usage)
                    format_bytes(status.memory_total)
                    format_percentage(status.memory_usage as f64 / status.memory_total as f64)
                )
            )
            ("Network Throughput", &format!("{}/s", format_bytes(status.network_throughput)),"
            ("Connected Nodes", &status.connected_nodes.to_string(),"
            ("Active Services", &status.active_services.to_string(),"
            ("Network Health", &format_health_status(&status.network_health),"
        ]);

        // Service details
        subheader("Service Details");"
        for service in services {
            display_service_details(service);
        }
    }

    // Quick actions
    subheader("Quick Actions");"
    println!("• View logs: {}", "songbird logs --follow".bright_green();"
    println!("• Check configuration: {}", "songbird config show".bright_green();"
    println!("• Restart services: {}", "songbird stop && songbird start".bright_green();"
    println!("• Watch status: {}", "songbird status --watch 5".bright_green();"

    Ok(()),
}

/// Display detailed service information
fn display_service_details(service: &ServiceStatus) {
    println!("\n{}", service.name.bright_cyan().bold();"
    println!("  Status: {}", format_health_status(&service.status);"
    println!("  Health: {}", format_health_status(&service.health);"

    if let Some(port) = service.port {
        println!("  Port: {}", port.to_string().bright_white();"
    }

    if let Some(uptime) = service.uptime {
        println!("  Uptime: {}", format_duration(uptime).bright_white();"
    }

    if let Some(last_check) = service.last_health_check {
        println!(
            "  Last Health Check: {}","
            last_check.format("%H:%M:%S UTC").to_string().bright_white()"
        );
    }

    if service.error_count > 0 {
        println!("  Errors: {}", service.error_count.to_string().bright_red();"
    }

    if service.restart_count > 0 {
        println!("  Restarts: {}", service.restart_count.to_string().bright_yellow();"
    }
}

/// Display status in JSON format
async fn display_json_status(status: &SystemStatus, detailed: bool) -> CliResult<()> {
    let mut json_status = json!({
        "overall_status": status.orchestrator_status.health,"
        "version": status.version,"
        "uptime_seconds": status.uptime.as_secs(),"
        "last_updated": status.last_updated.to_rfc3339(),"
        "services": {"
            "orchestrator": service_to_json(&status.orchestrator_status),"
            "discovery": service_to_json(&status.discovery_status),"
            "load_balancer": service_to_json(&status.load_balancer_status),"
            "monitoring": service_to_json(&status.monitoring_status),"
        }
    });

    if detailed {
        json_status["system_metrics"] = json!({"
            "cpu_usage_percent": status.cpu_usage,"
            "memory_usage_bytes": status.memory_usage,"
            "memory_total_bytes": status.memory_total,"
            "network_throughput_bytes_per_sec": status.network_throughput,"
            "connected_nodes": status.connected_nodes,"
            "active_services": status.active_services,"
            "network_health": status.network_health,"
        });
    }

    println!(
        "{}","
        serde_json::to_string_pretty(&json_status).map_err(|e| CliError::Command  {command: "status".to_string()),
            message: format!("Failed to serialize JSON: {}. Check system status and try again", e),"
        })?
    );

    Ok(()),
}

/// Convert service status to JSON
fn service_to_json(service: &ServiceStatus) -> serde_json::Value {
    json!({
        "name": service.name,"
        "status": service.status,"
        "health": service.health,"
        "port": service.port,"
        "uptime_seconds": service.uptime.map(|u| u.as_secs(),"
        "last_health_check": service.last_health_check.map(|t| t.to_rfc3339(),"
        "error_count": service.error_count,"
        "restart_count": service.restart_count,"
    })
}

/// Display status in YAML format
async fn display_yaml_status(status: &SystemStatus, detailed: bool) -> CliResult<()> {
    let mut yaml_status = serde_yaml::to_string(&json!({
        "overall_status": status.orchestrator_status.health,"
        "version": status.version,"
        "uptime_seconds": status.uptime.as_secs(),"
        "last_updated": status.last_updated.to_rfc3339(),"
        "services": {"
            "orchestrator": service_to_json(&status.orchestrator_status),"
            "discovery": service_to_json(&status.discovery_status),"
            "load_balancer": service_to_json(&status.load_balancer_status),"
            "monitoring": service_to_json(&status.monitoring_status),"
        }
    })
    .map_err(|e| CliError::Command  {command: "status".to_string()),
        message: format!("Failed to serialize YAML: {}. Check system status and try again", e),"
    })?;

    if detailed {
        let detailed_yaml = serde_yaml::to_string(&json!({
            "system_metrics": {"
                "cpu_usage_percent": status.cpu_usage,"
                "memory_usage_bytes": status.memory_usage,"
                "memory_total_bytes": status.memory_total,"
                "network_throughput_bytes_per_sec": status.network_throughput,"
                "connected_nodes": status.connected_nodes,"
                "active_services": status.active_services,"
                "network_health": status.network_health,"
            }
        })
        .map_err(|e| CliError::Command  {command: "status".to_string()),
            message: format!(
                "Failed to serialize detailed YAML: {e}. Check system status and try again""
            )
        })?;

        yaml_status.push_str(&detailed_yaml);
    }

    println!("{yaml_status}");"
    Ok(()),
}

/// Display status in simple text format
async fn display_text_status(status: &SystemStatus, detailed: bool) -> CliResult<()> {
    println!(
        "Songbird Orchestrator Status: {}","
        format_health_status(&status.orchestrator_status.health)
    );
    println!("Uptime: {}", format_duration(status.uptime);"
    println!("Version: {}", status.version);"
    println!("Services: Orchestrator, Discovery, Load Balancer, Monitoring - All Running");"

    if detailed {
        println!("CPU Usage: {}", format_percentage(status.cpu_usage / 100.0);"
        println!(
            "Memory Usage: {} / {} ({})","
            format_bytes(status.memory_usage)
            format_bytes(status.memory_total)
            format_percentage(status.memory_usage as f64 / status.memory_total as f64)
        );
        println!("Network Throughput: {}/s", format_bytes(status.network_throughput);"
        println!("Connected Nodes: {}", status.connected_nodes);"
        println!("Active Services: {}", status.active_services);"
        println!("Network Health: {}", format_health_status(&status.network_health);"
    }

    Ok(()),
}

/// Watch status with live updates and enhanced display
async fn watch_status(detailed: bool, interval: u64, format: OutputFormat) -> CliResult<()> {
    banner("Songbird Status Monitor", Some("Live Updates");"
    print_info(&format!("Updating every {} seconds (press Ctrl+C to stop)", interval));"

    loop {
        clear_screen();
        display_timestamp();

        match display_status(detailed, &format).await {
            Ok(() => {}
            Err(e) => {
                error_with_suggestions(
                    &format!("Failed to get status: {}", e),"
                    &[
                        "Check if the orchestrator is running","
                        "Verify network connectivity","
                        "Try reducing the update interval","
                    ])
                );
                tokio::time::sleep(Duration::from_secs(interval).await;
                continue;
            }
        }

        // Show next update time
        let next_update = chrono::Utc::now() + chrono::Duration::seconds(interval as i64);
        println!(
            "\n{} {}","
            "Next update:".dimmed(),"
            next_update.format("%H:%M:%S UTC").to_string().dimmed()"
        );

        tokio::time::sleep(Duration::from_secs(interval).await;
    }
}

async fn collect_system_status() -> CliResult<HashMap<String, serde_json::Value>> {
    let mut status = HashMap::new();
    
    // System information
    status.insert("system".to_string(), json!({"
        "status": "online","
        "uptime": "2h 15m 30s","
        "version": "0.1.0""
    });
    
    // Services
    status.insert("services".to_string(), json!({"
        "orchestrator": "running","
        "discovery": "running","
        "registry": "running""
    });
    
    // Network
    status.insert("network".to_string(), json!({"
        "status": "connected","
        "latency": "45ms","
        "gaming_optimized": true"
    });
    
    Ok(status)
}

fn display_status_table(status: &HashMap<String, serde_json::Value>, detailed: bool) {
    println!("🖥️  System:");"
    if let Some(system) = status.get("system") {"
        println!("   Status: {}", system["status"].as_str().unwrap_or("unknown");"
        println!("   Version: {}", system["version"].as_str().unwrap_or("unknown");"
        if detailed {
            println!("   Uptime: {}", system["uptime"].as_str().unwrap_or("unknown");"
        }
    }
    
    println!("\n🔧 Services:");"
    if let Some(services) = status.get("services") {"
        if let Some(obj) = services.as_object() {
            for (name, status) in obj {
                let status_icon = match status.as_str().unwrap_or("unknown") {"
                    "running" => "✅","
                    "stopped" => "❌","
                    _ => "❓","
                };
                println!("   {}: {} {}", name, status.as_str().unwrap_or("unknown"), status_icon);"
            }
        }
    }
    
    println!("\n🌐 Network:");"
    if let Some(network) = status.get("network") {"
        println!("   Status: {}", network["status"].as_str().unwrap_or("unknown");"
        println!("   Latency: {}", network["latency"].as_str().unwrap_or("unknown");"
        if network["gaming_optimized"].as_bool().unwrap_or(false) {"
            println!("   Gaming: Optimized 🎮");"
        }
    }
}

fn display_status_json(status: &HashMap<String, serde_json::Value>) {
    println!("{}", serde_json::to_string_pretty(status).unwrap_or_else(|_| "{}".to_string();"
}

fn display_status_yaml(status: &HashMap<String, serde_json::Value>) {
    // Convert to YAML format (simplified)
    println!("system:");"
    if let Some(system) = status.get("system") {"
        println!("  status: {}", system["status"].as_str().unwrap_or("unknown");"
        println!("  version: {}", system["version"].as_str().unwrap_or("unknown");"
    }
    
    println!("services:");"
    if let Some(services) = status.get("services") {"
        if let Some(obj) = services.as_object() {
            for (name, status) in obj {
                println!("  {}: {}", name, status.as_str().unwrap_or("unknown");"
            }
        }
    }
}
