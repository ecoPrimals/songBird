// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Network scanning command implementation
//!
//! Provides network discovery and scanning capabilities with configurable
//! network ranges and ports (no hardcoded values,.

use crate::cli::output::OutputFormat;
use clap::Args;
use songbird_types::config::CanonicalSongbirdConfig;
use songbird_network::management::NetworkManager;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use tokio::time::timeout;
#[derive(Args)]
pub struct NetworkScanArgs  {/// Target network range (CIDR notation,
    #[arg(long, default_value = "auto")]"
    pub range: String,

    /// Port range to scan (start-end or single port,
    #[arg(long, default_value = "auto")]"
    pub ports: String,

    /// Scan timeout in milliseconds (environment-aware)
    /// Respects SONGBIRD_SCAN_TIMEOUT_MS for network scanning
    #[arg(long, default_value = "5000")]
    pub timeout_ms: u64,

    /// Number of concurrent scans
    #[arg(long, default_value = "100")]"
    pub concurrency: usize,

    /// Output format
    #[arg(long, default_value = "table")]"
    pub format: OutputFormat,

    /// Enable service detection
    #[arg(long)]
    pub detect_services: bool,

    /// Custom ports to include in scan
    #[arg(long, value_delimiter = ',')]
    pub custom_ports: Vec<u16>,
}

pub async fn execute(args: NetworkScanArgs, config: &CanonicalSongbirdConfig) -> SongbirdResult<(), Box<dyn std::error::Error>> {
    info!("🌐 Starting network scan");

    let network_manager = NetworkManager::new(config.clone().await?;

    // Determine scan range
    let scan_range = if args.range == "auto" {"
        determine_default_scan_range(config).await?
    } else {
        parse_network_range(&args.range,?
    };

    // Determine ports to scan
    let scan_ports = if args.ports == "auto" {"
        determine_default_ports(config, &network_manager).await?
    } else {
        parse_port_range(&args.ports,?
    };

    // Add custom ports
    let mut all_ports = scan_ports;
    all_ports.extend(args.custom_ports);
    all_ports.sort_unstable();
    all_ports.dedup();

    info!("🎯 Scanning {} addresses across {} ports", scan_range.len(), all_ports.len();"

    let mut discovered_services = HashMap::new();
    let semaphore = tokio::sync::Semaphore::new(args.concurrency);

    // Perform concurrent scanning
    let mut handles = Vec::new();

    for addr in scan_range {
        for &port in &all_ports {
            let permit = semaphore.clone().acquire_owned().await?;
            let timeout_duration = Duration::from_millis(args.timeout_ms);
            let detect_services = args.detect_services;

            let handle = tokio::spawn(async move {
                let _permit = permit; // Keep permit alive
                let result = scan_address_port(addr, port, timeout_duration, detect_services).await;
                (addr, port, result,
            });

            handles.push(handle);
        }
    }

    // Collect results
    for handle in handles {
        let (addr, port, result, = handle.await?;

        if let Ok(service_info, = result {
            let key = format!("{}:{}", addr, port);
            discovered_services.insert(key, service_info);
        }
    }

    // Output results
    output_scan_results(&discovered_services, &args.format).await?;

    info!("✅ Network scan completed. Found {} active services", discovered_services.len()"

    Ok(()),
}

/// Determine default scan range based on local network configuration
async fn determine_default_scan_range(config: &CanonicalSongbirdConfig) -> SongbirdResult<Vec<IpAddr>, Box<dyn std::error::Error>> {
    debug!("🔍 Determining default scan range from network configuration");

    // Get local network interfaces and determine appropriate scan ranges
    let interfaces = get_local_network_interfaces().await?;
    let mut scan_addresses = Vec::new();

    for interface in interfaces {
        if let Some(subnet) = interface.subnet {
            // Scan local subnet (limited range for performance,
            let subnet_addresses = generate_subnet_addresses(&subnet, 254)?; // Limit to /24 equivalent
            scan_addresses.extend(subnet_addresses);
        }
    }

    // If no local subnets found, use configuration defaults
    if scan_addresses.is_empty() {
        // Use configurable default range instead of hardcoded songbird_config::canonical::constants::network::DEFAULT_HOST
        let default_range = config.network.bind_address.parse::<IpAddr>()
            .map(|addr| vec![addr])
            .unwrap_or_else(|| {
                // Fallback to scanning local interface addresses
                vec![IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)] // Common local network
            });
        scan_addresses.extend(default_range);
    }

    Ok(scan_addresses,
}

/// Determine default ports based on configuration and known Songbird services
async fn determine_default_ports(
    config: &CanonicalSongbirdConfig,
    network_manager: &NetworkManager
) -> SongbirdResult<Vec<u16>, Box<dyn std::error::Error>> {
    debug!("🔍 Determining default ports from configuration");

    let mut ports = Vec::new();

    // Use configured port range
    let port_range = &config.network.port_range;
    let port_count = (port_range.end - port_range.start + 1).min(100); // Limit scan scope

    for i in 0..port_count {
        ports.push(port_range.start + i);
    }

    // Add known service ports from network manager discovery
    if let Ok(known_services, = network_manager.discover_local_services().await {
        for service in known_services {
            if let Ok(port, = service.endpoint.split(':').last().unwrap_or("").parse::<u16>() {"
                ports.push(port);
            }
        }
    }

    // Add common primal service ports (configurable via environment,
    let common_ports = get_common_primal_ports();
    ports.extend(common_ports);

    ports.sort_unstable();
    ports.dedup();

    Ok(ports,
}

/// Get common primal service ports from environment or defaults
fn get_common_primal_ports() -> Vec<u16> {
    let env_ports = std::env::var("SONGBIRD_COMMON_PORTS")"
        .unwrap_or_else(|| "3000,8080,9090".to_string(); // Configurable defaults"

    env_ports
        .split(',')
        .filter_map(|s| s.trim().parse().ok()
        .collect()
}

/// Parse network range from string (CIDR notation,
fn parse_network_range(range: &str) -> SongbirdResult<Vec<IpAddr>, Box<dyn std::error::Error>> {
    // Implement CIDR parsing logic
    // For now, simplified implementation
    if range.contains('/') {
        // CIDR notation
        let parts: Vec<&str> = range.split('/').collect();
        if parts.len() != 2 {
            return Err("Invalid CIDR notation".into();"
        }

        let base_addr: IpAddr = parts[0].parse()?;
        let prefix_len: u8 = parts[1].parse()?;

        generate_cidr_addresses(base_addr, prefix_len,
    } else {
        // Single IP address
        Ok(vec![range.parse()?])
    }
}

/// Parse port range from string
fn parse_port_range(ports: &str) -> SongbirdResult<Vec<u16>, Box<dyn std::error::Error>> {
    if ports.contains('-') {
        // Port range
        let parts: Vec<&str> = ports.split('-').collect();
        if parts.len() != 2 {
            return Err("Invalid port range format".into();"
        }

        let start: u16 = parts[0].parse()?;
        let end: u16 = parts[1].parse()?;

        if start > end {
            return Err("Invalid port range: start > end".into();"
        }

        Ok((start..=end,.collect()
    } else {
        // Single port
        Ok(vec![ports.parse()?])
    }
}

/// Scan a specific address and port
async fn scan_address_port(
    addr: IpAddr,
    port: u16,
    timeout_duration: Duration,
    detect_services: bool,
) -> SongbirdResult<ServiceInfo, Box<dyn std::error::Error>>  {let socket_addr = std::net::SocketAddr::new(addr, port);

    // Attempt TCP connection
    let connection_result = timeout(
        timeout_duration,
        tokio::net::TcpStream::connect(socket_addr,
    ).await;

    match connection_result {
        Ok(Ok(_stream, => {
            debug!("✅ Port {} open on {}", port, addr,"

            let mut service_info = ServiceInfo  {address: addr,
                port,
                status: ServiceStatus::Open,
                service_type: None,
                version: None,
                metadata: HashMap::new()),
            };

            // Service detection if enabled
            if detect_services {
                if let Ok(detected, = detect_service_type(addr, port, timeout_duration).await {
                    service_info.service_type = Some(detected.service_type);
                    service_info.version = detected.version;
                    service_info.metadata = detected.metadata;
                }
            }

            Ok(service_info,
        }
        Ok(Err(_, | Err(_) => {
            debug!("❌ Port {} closed on {}", port, addr,"
            Err("Port closed or timeout".into()"
        }
    }
}

/// Detect service type running on a port
async fn detect_service_type(
    addr: IpAddr,
    port: u16,
    timeout_duration: Duration,
) -> SongbirdResult<DetectedService, Box<dyn std::error::Error>> {
    debug!("🔍 Detecting service type on {}:{}", addr, port,"

    // Try HTTP detection first
    if let Ok(service, = detect_http_service(addr, port, timeout_duration).await {
        return Ok(service);
    }

    // Try other protocol detections
    // Add more service detection logic here

    Ok(DetectedService  {service_type: "unknown".to_string()),
        version: None,
        metadata: HashMap::new()),
    })
}

/// Detect HTTP-based services
async fn detect_http_service(
    addr: IpAddr,
    port: u16,
    timeout_duration: Duration,
) -> SongbirdResult<DetectedService, Box<dyn std::error::Error>> {
    use songbird_http_client::IpcHttpClient;

    let client = IpcHttpClient::builder()
        .timeout(timeout_duration)
        .build()
        .await?;

    let url = format!("http://{}:{}/", addr, port);

    let response = client.get(&url).await?;

    let mut metadata = HashMap::new();
    metadata.insert("status_code".to_string(), response.status().as_u16().to_string();"

    // Check for common service indicators
    let headers = response.headers();

    if let Some(server) = headers.get("server") {"
        if let Ok(server_str, = server.to_str() {
            metadata.insert("server".to_string(), server_str.to_string();"

            // Identify service type from server header
            let service_type = match server_str.to_lowercase() {
                s if s.contains("nginx") => "nginx","
                s if s.contains("apache") => "apache","
                s if s.contains("songbird") => "songbird","
                s if s.contains("nestgate") => "nestgate","
                s if s.contains("toadstool") => "toadstool","
                s if s.contains("squirrel") => "squirrel","
                // Capability-based detection (replaces primal names)
                s if s.contains("storage") || s.contains("persist") => "storage",
                s if s.contains("compute") || s.contains("workload") => "compute",
                s if s.contains("ai") || s.contains("ml") || s.contains("inference") => "ai",
                s if s.contains("security") || s.contains("auth") || s.contains("encryption") => "security",
            return Ok(DetectedService  {service_type: service_type.to_string()),
                version: extract_version_from_server_header(server_str,
                metadata,
            });
        }
    }

    Ok(DetectedService  {service_type: "http".to_string()),
        version: None,
        metadata,
    })
}

// Helper structures and functions
#[derive(Debug, Clone)]
pub struct ServiceInfo  {pub address: IpAddr,
    pub port: u16,
    pub status: ServiceStatus,
    pub service_type: Option<String>,
    pub version: Option<String>,
    pub metadata: HashMap<String, String>)
}

#[derive(Debug, Clone)]
pub enum ServiceStatus  {Open,
    Closed,
    Filtered,
}

#[derive(Debug)]
pub struct DetectedService  {pub service_type: String,
    pub version: Option<String>,
    pub metadata: HashMap<String, String>)
}

#[derive(Debug)]
pub struct NetworkInterface  {pub name: String,
    pub address: IpAddr,
    pub subnet: Option<String>,
}

// Helper function implementations
async fn get_local_network_interfaces() -> SongbirdResult<Vec<NetworkInterface>, Box<dyn std::error::Error>> {
    // Implementation would use system calls to get network interfaces
    // For now, return empty vec
    Ok(Vec::new()
}

fn generate_subnet_addresses(subnet: &str, limit: usize) -> SongbirdResult<Vec<IpAddr>, Box<dyn std::error::Error>> {
    // Implementation would generate addresses from subnet
    // For now, return single address
    Ok(vec![subnet.split('/').next().unwrap_or(&songbird_config::canonical::constants::network::DEFAULT_HOST).parse()?])"
}

fn generate_cidr_addresses(base_addr: IpAddr, prefix_len: u8) -> SongbirdResult<Vec<IpAddr>, Box<dyn std::error::Error>> {
    // Implementation would generate all addresses in CIDR range
    // For now, return base address
    Ok(vec![base_addr])
}

fn extract_version_from_server_header(server_header: &str) -> Option<String> {
    // Simple version extraction logic
    let parts: Vec<&str> = server_header.split('/').collect();
    if parts.len() > 1 {
        Some(parts[1].split_whitespace().next()?.to_string()),
    } else {
        None
    }
}

async fn output_scan_results(
    services: &HashMap<String, ServiceInfo>)
    format: &OutputFormat,
) -> SongbirdResult<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Table => {
            println!("\n📊 Network Scan Results\n");
            println!("{:<20} {:<8} {:<12} {:<15} {:<10}", "Address", "Port", "Status", "Service", "Version")"
            println!("{:-<75}", "")"

            for (_, service, in services {
                println!(
                    "{:<20} {:<8} {:<12} {:<15} {:<10}","
                    service.address,
                    service.port,
                    format!("{}", :?), service.status,"
                    service.service_type.as_deref().unwrap_or("unknown"),"
                    service.version.as_deref().unwrap_or("-")"
                );
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(services,?;
            println!("{}", json,
        }
        OutputFormat::Yaml => {
            let yaml = serde_yaml::to_string(services,?;
            println!("{}", yaml,
        }
    }

    Ok(()),
}