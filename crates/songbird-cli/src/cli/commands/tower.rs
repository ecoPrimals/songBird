//! Tower Command - Single-command orchestrator startup
//!
//! Makes it trivial to start a Songbird tower with automatic resource detection

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::process::Command;
use sysinfo::System;

use crate::errors::SongbirdResult;

/// Tower management commands
#[derive(Debug, Clone, Subcommand)]
pub enum TowerCommand {
    /// Start a Songbird tower with automatic configuration
    Start(TowerStartArgs),

    /// Show detected system capabilities
    Info,

    /// Generate configuration file for this tower
    Config {
        /// Output file path
        #[arg(short, long, default_value = "tower.env")]
        output: String,
    },
}

/// Arguments for tower start command
#[derive(Debug, Clone, Args)]
pub struct TowerStartArgs {
    /// Tower name (defaults to hostname)
    #[arg(short, long)]
    pub name: Option<String>,

    /// Tower role (orchestrator, compute, storage, etc.)
    #[arg(short, long, default_value = "auto")]
    pub role: String,

    /// Port to listen on
    #[arg(short, long, default_value = "8080")]
    pub port: u16,

    /// Bootstrap node address (for joining existing federation)
    #[arg(short, long)]
    pub bootstrap: Option<String>,

    /// Bind address (0.0.0.0 for network access, 127.0.0.1 for local only)
    #[arg(long, default_value = "0.0.0.0")]
    pub bind: String,

    /// Enable federation mode
    #[arg(short, long)]
    pub federation: bool,

    /// Override detected CPU cores
    #[arg(long)]
    pub cpu_cores: Option<usize>,

    /// Override detected memory (GB)
    #[arg(long)]
    pub memory_gb: Option<usize>,

    /// Verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}

/// Detected tower capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerCapabilities {
    pub hostname: String,
    pub cpu_cores: usize,
    pub memory_gb: usize,
    pub storage_gb: Option<usize>,
    pub gpu_model: Option<String>,
    pub network_interfaces: Vec<String>,
    pub architecture: String,
    pub os: String,
}

impl TowerCommand {
    /// Execute the tower command
    pub async fn execute(&self) -> SongbirdResult<()> {
        match self {
            Self::Start(args) => start_tower(args).await,
            Self::Info => show_tower_info().await,
            Self::Config {
                output,
            } => generate_config(output).await,
        }
    }
}

/// Start a Songbird tower with automatic configuration
async fn start_tower(args: &TowerStartArgs) -> SongbirdResult<()> {
    println!("🏰 Starting Songbird Tower...\n");

    // Detect system capabilities
    let caps = detect_capabilities(args).await?;

    // Display configuration
    println!("📊 Tower Configuration:");
    println!("  Name:         {}", caps.hostname);
    println!("  Role:         {}", determine_role(&caps, &args.role));
    println!("  CPU Cores:    {}", caps.cpu_cores);
    println!("  Memory:       {} GB", caps.memory_gb);
    if let Some(gpu) = &caps.gpu_model {
        println!("  GPU:          {gpu}");
    }
    if let Some(storage) = caps.storage_gb {
        println!("  Storage:      {storage} GB");
    }
    println!("  Architecture: {}", caps.architecture);
    println!("  OS:           {}", caps.os);
    println!("  Listen:       {}:{}", args.bind, args.port);
    if let Some(bootstrap) = &args.bootstrap {
        println!("  Bootstrap:    {bootstrap}");
    }
    println!();

    // Set environment variables for the orchestrator
    std::env::set_var("SONGBIRD_ENV", "development");
    std::env::set_var("SONGBIRD_NODE_ID", format!("{}-{}", caps.hostname, args.port));
    std::env::set_var("NODE_NAME", &caps.hostname);
    std::env::set_var("NODE_ROLE", determine_role(&caps, &args.role));
    std::env::set_var("BIND_ADDRESS", &args.bind);
    std::env::set_var("SERVICE_PORT", args.port.to_string());
    std::env::set_var("ORCHESTRATOR_PORT", args.port.to_string());
    std::env::set_var("CPU_CORES", caps.cpu_cores.to_string());
    std::env::set_var("MEMORY_GB", caps.memory_gb.to_string());

    if let Some(gpu) = &caps.gpu_model {
        std::env::set_var("GPU_MODEL", gpu);
    }

    if let Some(storage) = caps.storage_gb {
        std::env::set_var("STORAGE_GB", storage.to_string());
    }

    if args.federation {
        std::env::set_var("FEDERATION_ENABLED", "true");
    }

    if let Some(bootstrap) = &args.bootstrap {
        std::env::set_var("BOOTSTRAP_NODE", bootstrap);
    }

    let log_level = if args.verbose {
        "debug,songbird=trace"
    } else {
        "info,songbird=debug"
    };
    std::env::set_var("RUST_LOG", log_level);

    println!("🚀 Launching orchestrator...\n");

    // Start the orchestrator
    let status = Command::new("cargo")
        .args(["run", "--release", "--bin", "songbird-orchestrator"])
        .status()
        .map_err(|e| format!("Failed to start orchestrator: {e}"))?;

    if !status.success() {
        return Err(format!("Orchestrator exited with status: {status}").into());
    }

    Ok(())
}

/// Show detected tower information
async fn show_tower_info() -> SongbirdResult<()> {
    println!("🏰 Tower System Information\n");

    let args = TowerStartArgs {
        name: None,
        role: "auto".to_string(),
        port: songbird_config::defaults::ports::orchestrator_port(),
        bootstrap: None,
        bind: "0.0.0.0".to_string(),
        federation: false,
        cpu_cores: None,
        memory_gb: None,
        verbose: false,
    };

    let caps = detect_capabilities(&args).await?;

    println!("🖥️  System:");
    println!("  Hostname:     {}", caps.hostname);
    println!("  Architecture: {}", caps.architecture);
    println!("  OS:           {}", caps.os);
    println!();

    println!("💻 Compute:");
    println!("  CPU Cores:    {}", caps.cpu_cores);
    println!("  Memory:       {} GB", caps.memory_gb);
    if let Some(gpu) = &caps.gpu_model {
        println!("  GPU:          {gpu}");
    }
    println!();

    if let Some(storage) = caps.storage_gb {
        println!("📦 Storage:");
        println!("  Available:    {storage} GB");
        println!();
    }

    println!("🌐 Network:");
    for interface in &caps.network_interfaces {
        println!("  Interface:    {interface}");
    }
    println!();

    println!("🎯 Recommended Role: {}", determine_role(&caps, "auto"));
    println!();

    println!("💡 Quick Start:");
    println!("  # Start as standalone tower:");
    println!("  $ songbird tower start");
    println!();
    println!("  # Join existing federation:");
    println!(
        "  $ songbird tower start --bootstrap <other-tower>:{} --federation",
        songbird_config::defaults::ports::orchestrator_port()
    );
    println!();

    Ok(())
}

/// Generate configuration file
async fn generate_config(output: &str) -> SongbirdResult<()> {
    println!("📝 Generating tower configuration...\n");

    let args = TowerStartArgs {
        name: None,
        role: "auto".to_string(),
        port: songbird_config::defaults::ports::orchestrator_port(),
        bootstrap: None,
        bind: "0.0.0.0".to_string(),
        federation: false,
        cpu_cores: None,
        memory_gb: None,
        verbose: false,
    };

    let caps = detect_capabilities(&args).await?;
    let role = determine_role(&caps, "auto");

    let config = format!(
        r#"# Songbird Tower Configuration
# Generated automatically for: {}

# Node Identity
SONGBIRD_ENV="development"
SONGBIRD_NODE_ID="{}-tower"
NODE_NAME="{}"
NODE_ROLE="{}"

# Network
BIND_ADDRESS="0.0.0.0"
SERVICE_PORT="8080"
ORCHESTRATOR_PORT="8080"

# Resources
CPU_CORES="{}"
MEMORY_GB="{}"
{}{}

# Discovery
DISCOVERY_ENABLED="true"
DISCOVERY_INTERVAL="30"

# Federation (uncomment to enable)
# FEDERATION_ENABLED="true"
# BOOTSTRAP_NODE="other-tower.local:8080"

# Logging
RUST_LOG="info,songbird=debug"
"#,
        caps.hostname,
        caps.hostname,
        caps.hostname,
        role,
        caps.cpu_cores,
        caps.memory_gb,
        caps.gpu_model.as_ref().map(|gpu| format!("GPU_MODEL=\"{gpu}\"\n")).unwrap_or_default(),
        caps.storage_gb.map(|s| format!("STORAGE_GB=\"{s}\"\n")).unwrap_or_default(),
    );

    std::fs::write(output, config).map_err(|e| format!("Failed to write config file: {e}"))?;

    println!("✅ Configuration written to: {output}\n");
    println!("To use:");
    println!("  $ source {output}");
    println!("  $ cargo run --release --bin songbird-orchestrator");
    println!();

    Ok(())
}

/// Detect system capabilities
async fn detect_capabilities(args: &TowerStartArgs) -> SongbirdResult<TowerCapabilities> {
    // Hostname
    let hostname = args.name.clone().unwrap_or_else(|| {
        hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "songbird-tower".to_string())
    });

    // CPU cores
    let cpu_cores = args.cpu_cores.unwrap_or_else(num_cpus::get);

    // Memory (using sysinfo)
    let mut sys = System::new_all();
    sys.refresh_memory();

    let memory_bytes = sys.total_memory();
    let memory_gb = args.memory_gb.unwrap_or((memory_bytes / 1024 / 1024 / 1024) as usize);

    // Storage (approximate available)
    let storage_gb = detect_storage_gb();

    // GPU detection
    let gpu_model = detect_gpu();

    // Network interfaces
    let network_interfaces = detect_network_interfaces();

    // Architecture and OS
    let architecture = std::env::consts::ARCH.to_string();
    let os = std::env::consts::OS.to_string();

    Ok(TowerCapabilities {
        hostname,
        cpu_cores,
        memory_gb,
        storage_gb,
        gpu_model,
        network_interfaces,
        architecture,
        os,
    })
}

/// Detect GPU model
fn detect_gpu() -> Option<String> {
    // Try nvidia-smi first
    if let Ok(output) =
        Command::new("nvidia-smi").args(["--query-gpu=name", "--format=csv,noheader"]).output()
    {
        if let Ok(gpu_name) = String::from_utf8(output.stdout) {
            let gpu = gpu_name.trim().to_string();
            if !gpu.is_empty() {
                return Some(gpu);
            }
        }
    }

    // Try lspci for other GPUs
    #[cfg(target_os = "linux")]
    if let Ok(output) = Command::new("lspci").output() {
        if let Ok(lspci_output) = String::from_utf8(output.stdout) {
            for line in lspci_output.lines() {
                if line.contains("VGA") || line.contains("3D") {
                    if let Some(device) = line.split(':').nth(2) {
                        return Some(device.trim().to_string());
                    }
                }
            }
        }
    }

    None
}

/// Detect available storage (approximate)
fn detect_storage_gb() -> Option<usize> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("df").args(["-B", "1G", "/"]).output() {
            if let Ok(df_output) = String::from_utf8(output.stdout) {
                // Parse df output (second line, fourth column)
                if let Some(line) = df_output.lines().nth(1) {
                    if let Some(avail) = line.split_whitespace().nth(3) {
                        if let Ok(gb) = avail.trim_end_matches('G').parse::<usize>() {
                            return Some(gb);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Detect network interfaces
fn detect_network_interfaces() -> Vec<String> {
    let mut interfaces = Vec::new();

    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if !name.starts_with("lo") {
                        interfaces.push(name.to_string());
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Fallback for other OS
        interfaces.push("default".to_string());
    }

    interfaces
}

/// Determine tower role based on capabilities
fn determine_role(caps: &TowerCapabilities, requested_role: &str) -> String {
    if requested_role != "auto" {
        return requested_role.to_string();
    }

    // Heuristics for role determination
    if caps.cpu_cores >= 32 && caps.memory_gb >= 128 {
        "compute".to_string()
    } else if let Some(storage) = caps.storage_gb {
        if storage >= 1000 {
            "storage".to_string()
        } else if caps.cpu_cores >= 8 {
            "orchestrator".to_string()
        } else {
            "edge".to_string()
        }
    } else if caps.cpu_cores >= 8 {
        "orchestrator".to_string()
    } else {
        "edge".to_string()
    }
}
