// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Tower Command - Single-command orchestrator startup
//!
//! Makes it trivial to start a Songbird tower with automatic resource detection

#![allow(missing_docs, reason = "tower command clap types include inline help strings")]

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use songbird_types::sys_metrics;
use std::process::Command; // Still needed for GPU/storage detection

use crate::errors::SongbirdResult;

/// Bind address for `tower info` / `tower config` when no CLI `--bind` is parsed (matches
/// `SONGBIRD_BIND_ADDRESS` with `0.0.0.0` default, same as [`TowerStartArgs::bind`]).
fn tower_bind_from_env_or_default() -> String {
    songbird_process_env::var("SONGBIRD_BIND_ADDRESS")
        .unwrap_or_else(|_| songbird_types::constants::PRODUCTION_BIND_ADDRESS.to_string())
}

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
    #[arg(short, long, env = "SONGBIRD_HTTP_PORT", default_value_t = songbird_types::defaults::ports::DEFAULT_ORCHESTRATOR_PORT)]
    pub port: u16,

    /// Bootstrap node address (for joining existing federation)
    #[arg(short, long)]
    pub bootstrap: Option<String>,

    /// Bind address (0.0.0.0 for network access, 127.0.0.1 for local only)
    #[arg(long, env = "SONGBIRD_BIND_ADDRESS", default_value = songbird_types::constants::PRODUCTION_BIND_ADDRESS)]
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

    /// Enable Dark Forest mode (encrypted BirdSong beacons, no plaintext fallback)
    #[arg(long, env = "SONGBIRD_DARK_FOREST")]
    pub dark_forest: bool,

    /// PID file directory (for Android/container substrates)
    #[arg(long, env = "SONGBIRD_PID_DIR")]
    pub pid_dir: Option<String>,

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
    songbird_process_env::set_var("SONGBIRD_ENV", "development");
    songbird_process_env::set_var("SONGBIRD_NODE_ID", format!("{}-{}", caps.hostname, args.port));
    songbird_process_env::set_var("NODE_NAME", &caps.hostname);
    songbird_process_env::set_var("NODE_ROLE", determine_role(&caps, &args.role));
    songbird_process_env::set_var("BIND_ADDRESS", &args.bind);
    songbird_process_env::set_var("SERVICE_PORT", args.port.to_string());
    songbird_process_env::set_var("ORCHESTRATOR_PORT", args.port.to_string());
    songbird_process_env::set_var("CPU_CORES", caps.cpu_cores.to_string());
    songbird_process_env::set_var("MEMORY_GB", caps.memory_gb.to_string());

    if let Some(gpu) = &caps.gpu_model {
        songbird_process_env::set_var("GPU_MODEL", gpu);
    }

    if let Some(storage) = caps.storage_gb {
        songbird_process_env::set_var("STORAGE_GB", storage.to_string());
    }

    if args.federation {
        songbird_process_env::set_var("FEDERATION_ENABLED", "true");
    }

    if args.dark_forest {
        songbird_process_env::set_var("SONGBIRD_DARK_FOREST", "true");
    }

    if let Some(ref pid_dir) = args.pid_dir {
        songbird_process_env::set_var("SONGBIRD_PID_DIR", pid_dir);
    }

    if let Some(bootstrap) = &args.bootstrap {
        songbird_process_env::set_var("BOOTSTRAP_NODE", bootstrap);
    }

    let log_level = if args.verbose {
        "debug,songbird=trace"
    } else {
        "info,songbird=debug"
    };
    songbird_process_env::set_var("RUST_LOG", log_level);

    println!("🚀 Launching orchestrator...\n");

    // ✅ FIX: Direct function call instead of cargo run (makes binary standalone)
    // Load configuration from environment (environment vars are already set above)
    let config = songbird_types::config::CanonicalSongbirdConfig::from_env()
        .map_err(|e| format!("Failed to load configuration: {e}"))?;

    // TLS crypto provider: rustls-rustcrypto (pure Rust, zero C — ecoBin compliant).
    // Production TLS is delegated to security provider via Tower Atomic; this provider is
    // the initial bootstrap before security provider discovery completes.
    rustls_rustcrypto::provider().install_default().ok();

    // Start the orchestrator directly (no cargo run needed!)
    songbird_orchestrator::app::start_orchestrator(config)
        .await
        .map_err(|e| format!("Orchestrator failed: {e}"))?;

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
        bind: tower_bind_from_env_or_default(),
        federation: false,
        dark_forest: false,
        pid_dir: None,
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

    let bind = tower_bind_from_env_or_default();
    let args = TowerStartArgs {
        name: None,
        role: "auto".to_string(),
        port: songbird_config::defaults::ports::orchestrator_port(),
        bootstrap: None,
        bind: bind.clone(),
        federation: false,
        dark_forest: false,
        pid_dir: None,
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
BIND_ADDRESS="{}"
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
        bind,
        caps.cpu_cores,
        caps.memory_gb,
        caps.gpu_model.as_ref().map(|gpu| format!("GPU_MODEL=\"{gpu}\"\n")).unwrap_or_default(),
        caps.storage_gb.map(|s| format!("STORAGE_GB=\"{s}\"\n")).unwrap_or_default(),
    );

    std::fs::write(output, config).map_err(|e| format!("Failed to write config file: {e}"))?;

    println!("✅ Configuration written to: {output}\n");
    println!("To use:");
    println!("  $ source {output}");
    println!("  $ songbird tower start");
    println!();

    Ok(())
}

/// Detect system capabilities
async fn detect_capabilities(args: &TowerStartArgs) -> SongbirdResult<TowerCapabilities> {
    // Hostname
    let hostname = args.name.clone().unwrap_or_else(|| {
        gethostname::gethostname().into_string().unwrap_or_else(|_| "songbird-tower".to_string())
    });

    // CPU cores
    let cpu_cores = args
        .cpu_cores
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, std::num::NonZero::get));

    let memory_gb = args.memory_gb.unwrap_or_else(|| sys_metrics::total_memory_gb().max(16));

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
        && let Ok(gpu_name) = String::from_utf8(output.stdout)
    {
        let gpu = gpu_name.trim().to_string();
        if !gpu.is_empty() {
            return Some(gpu);
        }
    }

    // Try lspci for other GPUs
    #[cfg(target_os = "linux")]
    if let Ok(output) = Command::new("lspci").output()
        && let Ok(lspci_output) = String::from_utf8(output.stdout)
    {
        for line in lspci_output.lines() {
            if (line.contains("VGA") || line.contains("3D"))
                && let Some(device) = line.split(':').nth(2)
            {
                return Some(device.trim().to_string());
            }
        }
    }

    None
}

/// Detect available storage (approximate)
fn detect_storage_gb() -> Option<usize> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("df").args(["-B", "1G", "/"]).output()
            && let Ok(df_output) = String::from_utf8(output.stdout)
        {
            // Parse df output (second line, fourth column)
            if let Some(line) = df_output.lines().nth(1)
                && let Some(avail) = line.split_whitespace().nth(3)
                && let Ok(gb) = avail.trim_end_matches('G').parse::<usize>()
            {
                return Some(gb);
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
                if let Some(name) = entry.file_name().to_str()
                    && !name.starts_with("lo")
                {
                    interfaces.push(name.to_string());
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::{TowerCapabilities, determine_role};

    fn caps(cpu: usize, mem: usize, storage: Option<usize>) -> TowerCapabilities {
        TowerCapabilities {
            hostname: "test-host".to_string(),
            cpu_cores: cpu,
            memory_gb: mem,
            storage_gb: storage,
            gpu_model: None,
            network_interfaces: vec![],
            architecture: "x86_64".to_string(),
            os: "linux".to_string(),
        }
    }

    #[test]
    fn determine_role_respects_explicit_non_auto() {
        let c = caps(4, 8, None);
        assert_eq!(determine_role(&c, "orchestrator"), "orchestrator");
        assert_eq!(determine_role(&c, "edge"), "edge");
    }

    #[test]
    fn determine_role_auto_high_end_compute() {
        let c = caps(32, 128, Some(500));
        assert_eq!(determine_role(&c, "auto"), "compute");
    }

    #[test]
    fn determine_role_auto_storage_when_tb_class_disk() {
        let c = caps(4, 16, Some(1000));
        assert_eq!(determine_role(&c, "auto"), "storage");
    }

    #[test]
    fn determine_role_auto_orchestrator_when_enough_cpu_and_moderate_storage() {
        let c = caps(8, 32, Some(100));
        assert_eq!(determine_role(&c, "auto"), "orchestrator");
    }

    #[test]
    fn determine_role_auto_edge_when_low_cpu_and_some_storage() {
        let c = caps(4, 32, Some(100));
        assert_eq!(determine_role(&c, "auto"), "edge");
    }

    #[test]
    fn determine_role_auto_orchestrator_without_storage_and_enough_cpu() {
        let c = caps(8, 32, None);
        assert_eq!(determine_role(&c, "auto"), "orchestrator");
    }

    #[test]
    fn determine_role_auto_edge_without_storage_low_cpu() {
        let c = caps(4, 32, None);
        assert_eq!(determine_role(&c, "auto"), "edge");
    }

    #[test]
    fn determine_role_auto_not_compute_when_cpu_just_below_threshold() {
        let c = caps(31, 128, Some(500));
        assert_ne!(determine_role(&c, "auto"), "compute");
    }

    #[test]
    fn determine_role_auto_not_compute_when_memory_just_below_threshold() {
        let c = caps(32, 127, Some(500));
        assert_ne!(determine_role(&c, "auto"), "compute");
    }

    #[test]
    fn determine_role_auto_storage_at_exactly_1000_gb() {
        let c = caps(16, 64, Some(1000));
        assert_eq!(determine_role(&c, "auto"), "storage");
    }

    #[test]
    fn determine_role_auto_orchestrator_at_999_gb_storage_and_high_cpu() {
        let c = caps(8, 64, Some(999));
        assert_eq!(determine_role(&c, "auto"), "orchestrator");
    }

    #[test]
    fn tower_bind_from_env_or_default_reads_overlay_address() {
        let _guard = songbird_process_env::ScopedEnv::new("SONGBIRD_BIND_ADDRESS", "192.168.99.7");
        assert_eq!(super::tower_bind_from_env_or_default(), "192.168.99.7");
    }

    #[test]
    fn tower_bind_from_env_or_default_accepts_ipv6_literal_from_overlay() {
        let _guard = songbird_process_env::ScopedEnv::new("SONGBIRD_BIND_ADDRESS", "::1");
        assert_eq!(super::tower_bind_from_env_or_default(), "::1");
    }

    #[test]
    fn tower_bind_from_env_or_default_trims_nothing_but_preserves_literal_value() {
        let _guard = songbird_process_env::ScopedEnv::new("SONGBIRD_BIND_ADDRESS", " 127.0.0.1 ");
        assert_eq!(super::tower_bind_from_env_or_default(), " 127.0.0.1 ");
    }
}
