//! # 🚀 Songbird Remote Deploy (Agnostic)
//!
//! **Philosophy:** Deploy ANY service to ANY tower via SSH
//!
//! This tool enables Songbird to orchestrate service deployment across
//! federation towers without hardcoding service names or configurations.
//!
//! ## Features
//! - ✅ **Agnostic:** Deploy any binary/service
//! - ✅ **SSH-based:** Secure, standard protocol
//! - ✅ **Environment-driven:** All config via env vars or CLI
//! - ✅ **No hardcoding:** Works with any service
//! - ✅ **Federation-aware:** Queries Songbird for tower info
//!
//! ## Usage
//! ```bash
//! # Deploy compute bridge to Tower B
//! songbird-deploy \
//!   --tower tower-b-strandgate \
//!   --binary ./target/release/songbird-compute-bridge \
//!   --env COMPUTE_SERVICE_NAME="Tower B Compute" \
//!   --env COMPUTE_HOST=192.168.1.134 \
//!   --env COMPUTE_PORT=9000 \
//!   --env SONGBIRD_FEDERATION_ENDPOINT=http://192.168.1.144:8080
//! ```

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use tracing::{debug, info, warn};

mod http_deploy;

#[derive(Parser, Debug)]
#[command(name = "songbird-deploy")]
#[command(about = "Agnostic remote service deployment for Songbird federation")]
struct Args {
    #[command(subcommand)]
    command: Commands,

    /// Songbird federation endpoint
    #[arg(long, env = "SONGBIRD_FEDERATION_ENDPOINT", default_value = "http://localhost:8080")]
    songbird_endpoint: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Deploy a service to a remote tower via SSH
    Deploy {
        /// Target tower ID or address
        #[arg(long)]
        tower: String,

        /// Binary path to deploy
        #[arg(long)]
        binary: String,

        /// Remote destination path
        #[arg(long, default_value = "/tmp/deployed-service")]
        remote_path: String,

        /// Environment variables (can be specified multiple times)
        #[arg(long = "env", value_parser = parse_env_var)]
        env_vars: Vec<(String, String)>,

        /// SSH user
        #[arg(long, env = "SSH_USER", default_value = "eastgate")]
        ssh_user: String,

        /// SSH key path
        #[arg(long, env = "SSH_KEY")]
        ssh_key: Option<String>,

        /// Auto-start after deployment
        #[arg(long, default_value = "true")]
        auto_start: bool,
    },

    /// Deploy a service via HTTP deployment API (adaptive)
    DeployHttp {
        /// Target tower HTTP endpoint (e.g. http://192.168.1.144:8080)
        #[arg(long)]
        tower: String,

        /// Binary path to deploy
        #[arg(long)]
        binary: String,

        /// Service name
        #[arg(long)]
        service: String,

        /// Environment variables (can be specified multiple times)
        #[arg(long = "env", value_parser = parse_env_var)]
        env_vars: Vec<(String, String)>,
    },

    /// List available towers from federation
    List {
        /// Show detailed information
        #[arg(long)]
        detailed: bool,
    },

    /// Check status of deployed services
    Status {
        /// Target tower ID or address
        #[arg(long)]
        tower: String,

        /// Service port to check
        #[arg(long)]
        port: Option<u16>,
    },
}

/// Parse environment variable in KEY=VALUE format
fn parse_env_var(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid env var format: {}. Expected KEY=VALUE", s));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

#[derive(Debug, Deserialize)]
struct NodeInfo {
    node_id: String,
    node_name: String,
    node_address: String,
    capabilities: Vec<String>,
    cpu_cores: usize,
    memory_gb: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,songbird_remote_deploy=debug".to_string()),
        )
        .init();

    let args = Args::parse();

    match args.command {
        Commands::Deploy {
            tower,
            binary,
            remote_path,
            env_vars,
            ssh_user,
            ssh_key,
            auto_start,
        } => {
            deploy_service(DeploymentConfig {
                songbird_endpoint: &args.songbird_endpoint,
                tower_id: &tower,
                binary_path: &binary,
                remote_path: &remote_path,
                env_vars: &env_vars,
                ssh_user: &ssh_user,
                ssh_key: ssh_key.as_deref(),
                auto_start,
            })
            .await?;
        }
        Commands::DeployHttp {
            tower,
            binary,
            service,
            env_vars,
        } => {
            let env_map: HashMap<String, String> = env_vars.into_iter().collect();
            let response =
                http_deploy::deploy_via_http_adaptive(&tower, &binary, &service, env_map).await?;

            info!("✅ Deployment successful!");
            info!("   Deployment ID: {}", response.deployment_id);
            info!("   Status: {}", response.status);
            info!("   Message: {}", response.message);
            if let Some(url) = response.service_url {
                info!("   Service URL: {}", url);
            }
        }
        Commands::List {
            detailed,
        } => {
            list_towers(&args.songbird_endpoint, detailed).await?;
        }
        Commands::Status {
            tower,
            port,
        } => {
            check_status(&args.songbird_endpoint, &tower, port).await?;
        }
    }

    Ok(())
}

/// Deployment configuration
struct DeploymentConfig<'a> {
    songbird_endpoint: &'a str,
    tower_id: &'a str,
    binary_path: &'a str,
    remote_path: &'a str,
    env_vars: &'a [(String, String)],
    ssh_user: &'a str,
    ssh_key: Option<&'a str>,
    auto_start: bool,
}

/// Deploy service to remote tower
async fn deploy_service(config: DeploymentConfig<'_>) -> Result<()> {
    info!("🚀 Deploying service to tower: {}", config.tower_id);

    // Get tower information from Songbird
    let tower_info = get_tower_info(config.songbird_endpoint, config.tower_id).await?;
    let tower_address = parse_tower_address(&tower_info.node_address)?;

    info!("📡 Target: {} ({})", tower_info.node_name, tower_address);
    info!("📦 Binary: {}", config.binary_path);
    info!("📍 Remote path: {}", config.remote_path);

    // Step 1: Copy binary via SCP
    info!("📤 Copying binary...");
    scp_copy(
        config.binary_path,
        &tower_address,
        config.remote_path,
        config.ssh_user,
        config.ssh_key,
    )?;
    info!("✅ Binary copied successfully");

    // Step 2: Make it executable
    info!("🔧 Making binary executable...");
    ssh_exec(
        &tower_address,
        &format!("chmod +x {}", config.remote_path),
        config.ssh_user,
        config.ssh_key,
    )?;
    info!("✅ Binary is executable");

    // Step 3: Start service if requested
    if config.auto_start {
        info!("🎬 Starting service...");
        start_remote_service(
            &tower_address,
            config.remote_path,
            config.env_vars,
            config.ssh_user,
            config.ssh_key,
        )?;
        info!("✅ Service started");

        // Poll for actual service readiness instead of fixed sleep
        // Use health check endpoint or process status
        if let Some(port) = config
            .env_vars
            .iter()
            .find(|(k, _)| k.ends_with("PORT"))
            .and_then(|(_, v)| v.parse::<u16>().ok())
        {
            info!("🔍 Verifying service health...");
            if let Err(e) = verify_service_health(&tower_address, port).await {
                warn!("⚠️  Service may not be healthy yet: {}", e);
            } else {
                info!("✅ Service is healthy!");
            }
        }
    }

    info!("");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("🎉 Deployment Complete!");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");
    info!("Tower: {} ({})", tower_info.node_name, tower_address);
    info!("Service: {}", config.remote_path);
    info!(
        "Status: {}",
        if config.auto_start {
            "Running"
        } else {
            "Deployed (not started)"
        }
    );

    Ok(())
}

/// Get tower information from Songbird federation
async fn get_tower_info(songbird_endpoint: &str, tower_id: &str) -> Result<NodeInfo> {
    let url = format!("{}/api/federation/nodes", songbird_endpoint);
    debug!("Fetching tower info from: {}", url);

    let client = reqwest::Client::new();
    let nodes: Vec<NodeInfo> = client
        .get(&url)
        .send()
        .await
        .context("Failed to query Songbird federation")?
        .json()
        .await
        .context("Failed to parse tower list")?;

    nodes
        .into_iter()
        .find(|n| {
            n.node_id == tower_id || n.node_name.to_lowercase().contains(&tower_id.to_lowercase())
        })
        .ok_or_else(|| anyhow::anyhow!("Tower '{}' not found in federation", tower_id))
}

/// Parse tower address (IP:PORT) to just IP
fn parse_tower_address(address: &str) -> Result<String> {
    let parts: Vec<&str> = address.split(':').collect();
    Ok(parts[0].to_string())
}

/// Copy file via SCP
fn scp_copy(
    local_path: &str,
    remote_host: &str,
    remote_path: &str,
    ssh_user: &str,
    ssh_key: Option<&str>,
) -> Result<()> {
    let mut cmd = Command::new("scp");

    if let Some(key) = ssh_key {
        cmd.arg("-i").arg(key);
    }

    cmd.arg(local_path)
        .arg(format!("{}@{}:{}", ssh_user, remote_host, remote_path))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    debug!("Executing: {:?}", cmd);

    let status = cmd.status().context("Failed to execute scp")?;

    if !status.success() {
        anyhow::bail!("SCP failed with status: {}", status);
    }

    Ok(())
}

/// Execute command via SSH
fn ssh_exec(remote_host: &str, command: &str, ssh_user: &str, ssh_key: Option<&str>) -> Result<()> {
    let mut cmd = Command::new("ssh");

    if let Some(key) = ssh_key {
        cmd.arg("-i").arg(key);
    }

    cmd.arg(format!("{}@{}", ssh_user, remote_host))
        .arg(command)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    debug!("Executing: {:?}", cmd);

    let status = cmd.status().context("Failed to execute ssh")?;

    if !status.success() {
        anyhow::bail!("SSH command failed with status: {}", status);
    }

    Ok(())
}

/// Start service on remote host
fn start_remote_service(
    remote_host: &str,
    remote_path: &str,
    env_vars: &[(String, String)],
    ssh_user: &str,
    ssh_key: Option<&str>,
) -> Result<()> {
    // Build environment variable string
    let env_string =
        env_vars.iter().map(|(k, v)| format!("{}=\"{}\"", k, v)).collect::<Vec<_>>().join(" ");

    // Build command to run service in background with nohup
    let command = format!("nohup {} {} > /tmp/service.log 2>&1 &", env_string, remote_path);

    ssh_exec(remote_host, &command, ssh_user, ssh_key)?;

    Ok(())
}

/// Verify service health
async fn verify_service_health(host: &str, port: u16) -> Result<()> {
    let url = format!("http://{}:{}/health", host, port);
    debug!("Health check: {}", url);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .timeout(tokio::time::Duration::from_secs(5))
        .send()
        .await
        .context("Health check request failed")?;

    if response.status().is_success() {
        Ok(())
    } else {
        anyhow::bail!("Health check failed with status: {}", response.status())
    }
}

/// List towers in federation
async fn list_towers(songbird_endpoint: &str, detailed: bool) -> Result<()> {
    let url = format!("{}/api/federation/nodes", songbird_endpoint);

    let client = reqwest::Client::new();
    let nodes: Vec<NodeInfo> = client
        .get(&url)
        .send()
        .await
        .context("Failed to query Songbird federation")?
        .json()
        .await
        .context("Failed to parse tower list")?;

    info!("📡 Available Towers in Federation");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");

    for node in nodes {
        info!("🏗️  {} ({})", node.node_name, node.node_id);
        info!("   Address: {}", node.node_address);

        if detailed {
            info!("   CPU Cores: {}", node.cpu_cores);
            info!("   Memory: {}GB", node.memory_gb);
            info!("   Capabilities: {}", node.capabilities.join(", "));
        }

        info!("");
    }

    Ok(())
}

/// Check status of services on a tower
async fn check_status(songbird_endpoint: &str, tower_id: &str, port: Option<u16>) -> Result<()> {
    let tower_info = get_tower_info(songbird_endpoint, tower_id).await?;
    let tower_address = parse_tower_address(&tower_info.node_address)?;

    info!("🔍 Checking status on: {} ({})", tower_info.node_name, tower_address);

    if let Some(port) = port {
        // Check specific port
        match verify_service_health(&tower_address, port).await {
            Ok(_) => info!("✅ Service on port {} is healthy", port),
            Err(e) => info!("❌ Service on port {} is not responding: {}", port, e),
        }
    } else {
        // Query federation for services on this tower
        let url = format!("{}/api/federation/services", songbird_endpoint);
        let client = reqwest::Client::new();

        #[derive(Deserialize)]
        struct ServiceInfo {
            service_name: String,
            service_type: String,
            endpoint: String,
            health_status: String,
        }

        let services: Vec<ServiceInfo> = client.get(&url).send().await?.json().await?;

        let tower_services: Vec<_> =
            services.into_iter().filter(|s| s.endpoint.contains(&tower_address)).collect();

        if tower_services.is_empty() {
            info!("ℹ️  No services registered for this tower");
        } else {
            info!("📊 Registered Services:");
            for svc in tower_services {
                info!("   • {} ({})", svc.service_name, svc.service_type);
                info!("     Endpoint: {}", svc.endpoint);
                info!("     Status: {}", svc.health_status);
            }
        }
    }

    Ok(())
}
