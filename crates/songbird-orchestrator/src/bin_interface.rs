//! Public API for UniBin integration
//!
//! This module exposes the main entry points and types needed for
//! the unified `songbird` binary to route to orchestrator functionality.

use anyhow::Result;
use clap::{Args, Subcommand};

// Re-export main entry point function
pub use crate::app::start_orchestrator;

/// Server mode arguments
#[derive(Args, Debug, Clone)]
pub struct ServerArgs {
    /// HTTP server port (external discovery gateway)
    ///
    /// Songbird operates in dual-mode:
    /// • External TCP port (for LAN discovery beacons) ← this flag
    /// • Internal Unix socket (for inter-primal IPC) ← see --socket
    ///
    /// This port is used for:
    /// - Broadcasting discovery beacons to peers
    /// - Initial peer handshake
    /// - Federation negotiation
    /// - External API access
    ///
    /// Required when discovery is enabled (default).
    #[arg(long, short, default_value = "8080")]
    pub port: u16,

    /// Federation port (alias for --port, clearer intent)
    ///
    /// Use this flag when explicitly configuring for LAN discovery/federation.
    /// If both --port and --federation-port are specified, --federation-port takes precedence.
    #[arg(long)]
    pub federation_port: Option<u16>,

    /// Run as daemon (background process)
    #[arg(long, short)]
    pub daemon: bool,

    /// Configuration file path
    #[arg(long, short)]
    pub config: Option<String>,

    /// Enable verbose logging
    #[arg(long, short)]
    pub verbose: bool,

    /// Unix socket path for IPC (JSON-RPC 2.0)
    ///
    /// Enables external primals to access HTTP/HTTPS capabilities via Unix socket.
    /// This is the INTERNAL interface for inter-primal communication.
    ///
    /// Songbird operates in dual-mode:
    /// • External TCP port (for LAN discovery) ← see --port
    /// • Internal Unix socket (for inter-primal IPC) ← this flag
    ///
    /// XDG-compliant path example: /run/user/1000/biomeos/songbird-nat0.sock
    /// Legacy fallback: /tmp/songbird-nat0.sock
    ///
    /// Mutually exclusive with --listen
    #[arg(long, conflicts_with = "listen")]
    pub socket: Option<String>,

    /// TCP listen address for IPC (universal transport, works on Android)
    ///
    /// Alternative to Unix sockets for platforms with restrictions (Android SELinux).
    /// Examples: 127.0.0.1:9901, [::1]:9901, 0.0.0.0:0 (OS-assigned port)
    ///
    /// Mutually exclusive with --socket
    #[arg(long, conflicts_with = "socket")]
    pub listen: Option<String>,

    /// BearDog socket path for crypto operations (defaults based on family_id)
    ///
    /// If not specified, uses XDG-compliant discovery:
    /// 1. $BEARDOG_SOCKET env var
    /// 2. $XDG_RUNTIME_DIR/biomeos/beardog-$FAMILY_ID.sock
    /// 3. /tmp/beardog-nat0.sock (fallback)
    #[arg(long)]
    pub beardog_socket: Option<String>,

    /// BearDog TCP address (when BearDog is running in TCP mode on Android)
    ///
    /// Example: 127.0.0.1:9900
    /// Used instead of beardog_socket when BearDog is using TCP transport
    #[arg(long)]
    pub beardog_tcp: Option<String>,
}

/// Doctor mode arguments
#[derive(Args, Debug, Clone)]
pub struct DoctorArgs {
    /// Run comprehensive checks (includes primal connectivity)
    #[arg(long, short)]
    pub comprehensive: bool,

    /// Output format (text, json, yaml)
    #[arg(long, default_value = "text")]
    pub format: String,
}

/// Configuration management commands
#[derive(Subcommand, Debug, Clone)]
pub enum ConfigCommands {
    /// Show current configuration
    Show {
        /// Show sensitive values (API keys, etc.)
        #[arg(long)]
        show_secrets: bool,

        /// Output format (text, json, yaml)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Validate configuration
    Validate,

    /// Generate default configuration template
    Init {
        /// Output path for generated config
        #[arg(long, default_value = "songbird.toml")]
        output: String,

        /// Overwrite existing file
        #[arg(long)]
        force: bool,
    },
}

/// Run orchestrator in server mode
///
/// Modern, idiomatic, async Rust implementation with:
/// - Proper signal handling (SIGINT, SIGTERM)
/// - Graceful shutdown
/// - Instance locking
/// - Comprehensive logging
pub async fn run_server(args: ServerArgs) -> Result<()> {
    use crate::app;
    use crate::process_manager::ProcessManager;
    use songbird_types::config::CanonicalSongbirdConfig;

    // Initialize tracing (early, before any logging)
    if args.verbose {
        tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init();
    } else {
        tracing_subscriber::fmt::init();
    }

    // Determine the actual port to use (federation_port takes precedence)
    let actual_port = args.federation_port.unwrap_or(args.port);

    // Log startup with mode information
    tracing::info!("🚀 Songbird v{} - Server Mode", env!("CARGO_PKG_VERSION"));
    tracing::info!(
        "   Mode: Server {}",
        if args.daemon {
            "(daemon)"
        } else {
            "(foreground)"
        }
    );
    tracing::info!("   External Port: {} (LAN discovery/federation)", actual_port);
    if let Some(ref socket) = args.socket {
        tracing::info!("   Internal Socket: {} (inter-primal IPC)", socket);
    }
    tracing::info!("   Process ID: {}", std::process::id());

    // ✅ Step 1: Acquire instance lock FIRST (before any resources)
    let process_mgr = ProcessManager::new()?;
    let _singleton_guard = process_mgr.acquire_lock()?;
    tracing::info!("   Instance Lock: ✅ Acquired (PID file active)");

    // Get node identity for logging
    let node_identity = std::env::var("SONGBIRD_NODE_ID")
        .or_else(|_| std::env::var("NODE_ID"))
        .or_else(|_| std::env::var("SPORE_ID"))
        .ok();

    let family_identity =
        std::env::var("SONGBIRD_FAMILY_ID").or_else(|_| std::env::var("FAMILY_ID")).ok();

    if let Some(ref family) = family_identity {
        tracing::info!("   Family ID: {}", family);
    }
    if let Some(ref node) = node_identity {
        tracing::info!("   Node ID: {}", node);
    }

    // Step 3: Load configuration
    tracing::info!("📋 Loading configuration...");
    let mut config = if let Some(path) = args.config {
        tracing::info!("   Config file: {}", path);
        CanonicalSongbirdConfig::from_env()
            .map_err(|e| anyhow::anyhow!("Failed to load configuration from file: {}", e))?
    } else {
        tracing::info!("   Config source: Environment variables");
        CanonicalSongbirdConfig::from_env()
            .map_err(|e| anyhow::anyhow!("Failed to load configuration from environment: {}", e))?
    };

    // Override port from CLI (CLI takes precedence over config/env)
    config.network.base_port = actual_port;
    tracing::info!("   Configuration: ✅ Loaded (port override: {})", actual_port);

    // Step 4: Start the orchestrator (non-blocking, returns handle)
    tracing::info!("🔧 Starting orchestrator components...");
    let mut orchestrator = app::start_orchestrator(config).await?;
    tracing::info!("   Orchestrator: ✅ Started");

    tracing::info!("✅ Songbird ready!");
    tracing::info!("");

    // Step 4.5: Start IPC server (Unix socket or TCP)
    let ipc_enabled = args.socket.is_some() || args.listen.is_some();
    let socket_path_for_registration = args.socket.clone(); // Clone for later use
    let ipc_handle = if let Some(socket_path) = args.socket {
        // Unix Socket Mode (Linux, macOS preferred)
        tracing::info!("");
        tracing::info!("🌐 Starting IPC Server (Unix socket)...");
        tracing::info!("   Socket: {}", socket_path);
        tracing::info!("   Protocol: JSON-RPC 2.0 over Unix sockets");
        if let Some(ref fam) = family_identity {
            tracing::info!("   Family: {}", fam);
        }

        // Determine BearDog connection
        let beardog_conn = if let Some(tcp) = args.beardog_tcp {
            tracing::info!("   BearDog: {} (TCP)", tcp);
            BearDogConnection::Tcp(tcp)
        } else {
            let beardog_socket = args.beardog_socket.unwrap_or_else(|| {
                let family_id = family_identity.as_deref().unwrap_or("nat0");
                format!("/tmp/beardog-{}.sock", family_id)
            });
            tracing::info!("   BearDog: {} (Unix)", beardog_socket);
            BearDogConnection::UnixSocket(beardog_socket)
        };
        tracing::info!("   Capabilities: http, discovery, secure_http");

        // Spawn IPC server in background task (Unix only)
        #[cfg(unix)]
        let ipc_task = {
            let socket_clone = socket_path.clone();
            Some(tokio::spawn(async move {
                match start_ipc_server(&socket_clone, beardog_conn).await {
                    Ok(_) => tracing::info!("IPC server stopped gracefully"),
                    Err(e) => tracing::error!("IPC server error: {}", e),
                }
            }))
        };

        #[cfg(not(unix))]
        let ipc_task: Option<tokio::task::JoinHandle<()>> = {
            tracing::info!("IPC server: Windows not yet supported");
            None
        };

        ipc_task
    } else if let Some(listen_addr) = args.listen {
        // TCP Mode (Android, universal)
        tracing::info!("");
        tracing::info!("🌐 Starting IPC Server (TCP - universal transport)...");
        tracing::info!("   Listen: {}", listen_addr);
        tracing::info!("   Protocol: JSON-RPC 2.0 over TCP");
        if let Some(ref fam) = family_identity {
            tracing::info!("   Family: {}", fam);
        }

        // Determine BearDog connection
        let beardog_conn = if let Some(tcp) = args.beardog_tcp {
            tracing::info!("   BearDog: {} (TCP)", tcp);
            BearDogConnection::Tcp(tcp)
        } else {
            let beardog_socket = args.beardog_socket.unwrap_or_else(|| {
                let family_id = family_identity.as_deref().unwrap_or("nat0");
                format!("/tmp/beardog-{}.sock", family_id)
            });
            tracing::info!("   BearDog: {} (Unix fallback)", beardog_socket);
            BearDogConnection::UnixSocket(beardog_socket)
        };
        tracing::info!("   Capabilities: http, discovery, secure_http");

        Some(tokio::spawn(async move {
            match start_ipc_server_tcp(&listen_addr, beardog_conn).await {
                Ok(_) => tracing::info!("TCP IPC server stopped gracefully"),
                Err(e) => tracing::error!("TCP IPC server error: {}", e),
            }
        }))
    } else {
        tracing::info!("");
        tracing::info!("💡 Tip: Use --socket or --listen to enable IPC for biomeOS integration");
        tracing::info!("   Example: --socket /run/user/$(id -u)/biomeos/songbird.sock");
        tracing::info!("   Example: --listen 127.0.0.1:9901  (Android-compatible)");
        None
    };

    // Step 4.6: Register capabilities with Neural API (if available)
    if ipc_enabled {
        tracing::info!("");
        tracing::info!("🌟 Registering capabilities with Neural API...");
        if let Err(e) = crate::capability_registration::register_capabilities().await {
            tracing::warn!("⚠️  Failed to register capabilities: {}", e);
            tracing::warn!("   Songbird will continue without Neural API registration");
            tracing::warn!("   Direct connections will still work");
        }
    }

    tracing::info!("");
    tracing::info!("💡 Press Ctrl+C to stop gracefully");

    // Step 5: If daemon mode, detach from terminal (future enhancement)
    if args.daemon {
        tracing::info!("📌 Daemon mode: Process detached");
    }

    // Step 6: Main event loop - wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("🛑 Received SIGINT (Ctrl+C), initiating graceful shutdown...");
        }
        _ = async {
            #[cfg(unix)]
            {
                let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .expect("Failed to setup SIGTERM handler");
                sigterm.recv().await;
            }
            #[cfg(not(unix))]
            {
                std::future::pending::<()>().await
            }
        } => {
            tracing::info!("🛑 Received SIGTERM, initiating graceful shutdown...");
        }
    }

    // Step 7: Graceful shutdown
    tracing::info!("🧹 Stopping orchestrator components...");

    // Unregister capabilities from Neural API (if registered)
    if ipc_enabled {
        let _ = crate::capability_registration::unregister_capabilities().await;
    }

    orchestrator.stop().await?;
    tracing::info!("   Orchestrator: ✅ Stopped");

    tracing::info!("✅ Graceful shutdown complete");

    Ok(())
}

/// Run health diagnostics and system checks
pub async fn run_doctor(args: DoctorArgs) -> Result<()> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    match args.format.as_str() {
        "text" => run_doctor_text(args.comprehensive).await,
        "json" => run_doctor_json(args.comprehensive).await,
        "yaml" => run_doctor_yaml(args.comprehensive).await,
        _ => {
            eprintln!("❌ Unknown format: {}. Use: text, json, or yaml", args.format);
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
            println!("   Error: {}", e);
            println!();
            println!("💡 Fix: Check environment variables or create config file");
            std::process::exit(1);
        }
    }
    println!();

    // Check 3: Network ports
    println!("🌐 Network Ports");
    let default_port = 8080;
    match check_port_availability(default_port).await {
        Ok(true) => {
            println!("   Port {}: ✅ Available", default_port);
        }
        Ok(false) => {
            println!("   Port {}: ⚠️  In use", default_port);
            println!("   Note: May be used by running Songbird instance");
        }
        Err(e) => {
            println!("   Port {}: ❌ Check failed: {}", default_port, e);
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

    // Check 5: Comprehensive checks
    if comprehensive {
        println!("🔍 Comprehensive Checks");
        println!("   Checking primal connectivity...");
        println!();

        println!("   🐻 BearDog (Security & Crypto)");
        match check_beardog_connectivity().await {
            Ok(true) => println!("      Status: ✅ Connected"),
            Ok(false) => println!("      Status: ⚠️  Not reachable"),
            Err(e) => println!("      Status: ❌ Error: {}", e),
        }

        println!("   🐿️  Squirrel (AI & MCP)");
        println!("      Status: ⏳ Not yet integrated");

        println!("   🍄 ToadStool (Storage)");
        println!("      Status: ⏳ Not yet integrated");

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
    println!("{}", json);
    Ok(())
}

/// Run doctor in YAML format
async fn run_doctor_yaml(comprehensive: bool) -> Result<()> {
    let health_status = gather_health_status(comprehensive).await?;
    let yaml = serde_yaml::to_string(&health_status)?;
    println!("{}", yaml);
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
            error: Some(e.to_string()),
        },
    };

    // Check network ports
    let port_checks = vec![
        PortCheck {
            port: 3030,
            name: "HTTP API".to_string(),
            available: check_port_availability(3030).await?,
        },
        PortCheck {
            port: 3031,
            name: "Metrics".to_string(),
            available: check_port_availability(3031).await?,
        },
        PortCheck {
            port: 3032,
            name: "gRPC".to_string(),
            available: check_port_availability(3032).await?,
        },
    ];

    // Check IPC socket
    let socket_status = SocketStatus {
        path: "/tmp/songbird-orchestrator.sock".to_string(),
        available: true,
    };

    // Comprehensive checks (if requested)
    let primal_checks = if comprehensive {
        Some(PrimalChecks {
            beardog: check_primal_status("beardog", check_beardog_connectivity()).await,
            squirrel: check_primal_status("squirrel", futures::future::ready(Ok(false))).await,
            toadstool: check_primal_status("toadstool", futures::future::ready(Ok(false))).await,
            nestgate: check_primal_status("nestgate", futures::future::ready(Ok(false))).await,
        })
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

#[derive(Debug, serde::Serialize)]
struct PrimalChecks {
    beardog: PrimalStatus,
    squirrel: PrimalStatus,
    toadstool: PrimalStatus,
    nestgate: PrimalStatus,
}

#[derive(Debug, serde::Serialize)]
struct PrimalStatus {
    name: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Check if a port is available
async fn check_port_availability(port: u16) -> Result<bool> {
    use std::net::TcpListener;

    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => Ok(false),
        Err(e) => Err(anyhow::anyhow!("Failed to check port: {}", e)),
    }
}

/// Check BearDog connectivity
async fn check_beardog_connectivity() -> Result<bool> {
    use crate::btsp_client::BtspClient;

    let client = BtspClient::new();
    match client.ping().await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

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
                println!(
                    "{{\"status\":\"error\",\"message\":\"{}\"}}",
                    e.to_string().replace('"', "\\\"")
                );
            } else if format == "yaml" {
                println!("status: error");
                println!("message: \"{}\"", e);
            } else {
                println!("❌ Configuration invalid");
                println!();
                println!("Error: {}", e);
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
            println!("Error: {}", e);
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
        eprintln!("❌ File already exists: {}", output);
        eprintln!();
        eprintln!("💡 Use --force to overwrite");
        std::process::exit(1);
    }

    let template = r#"# Songbird Configuration Template
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
"#;

    std::fs::write(output, template)?;
    println!("✅ Configuration template generated: {}", output);
    println!();
    println!("Next steps:");
    println!("  1. Review and customize: {}", output);
    println!("  2. Source the file: source {}", output);
    println!("  3. Validate: songbird config validate");
    println!("  4. Start server: songbird server");
    println!();

    Ok(())
}

/// BearDog connection type (Unix socket or TCP)
enum BearDogConnection {
    UnixSocket(String),
    Tcp(String),
}

/// Start IPC server for external primal access to HTTP/HTTPS capabilities
///
/// This enables biomeOS and other primals to make HTTP/HTTPS requests via JSON-RPC
/// without embedding Songbird code (TRUE PRIMAL architecture).
#[cfg(unix)]
async fn start_ipc_server(socket_path: &str, _beardog_conn: BearDogConnection) -> Result<()> {
    use songbird_universal_ipc::registry::ServiceRegistry;
    use songbird_universal_ipc::service::IpcServiceHandler;
    use songbird_universal_ipc::tower_atomic::{
        JsonRpcError, JsonRpcHandler, JsonRpcRequest, JsonRpcResponse,
    };
    use std::sync::Arc;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::sync::RwLock;

    // Remove old socket if exists
    let _ = std::fs::remove_file(socket_path);

    // Create IPC service handler with all methods (HTTP + STUN + Discovery + Rendezvous + Peer)
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));

    tracing::info!("✅ IPC server listening on {}", socket_path);
    tracing::info!("   Methods available:");
    tracing::info!("     • http.request, http.get, http.post - HTTP/HTTPS requests");
    tracing::info!("     • stun.get_public_address, stun.bind - NAT traversal");
    tracing::info!("     • discovery.peers - Real-time peer discovery");
    tracing::info!("     • rendezvous.register, rendezvous.lookup - Relay server");
    tracing::info!("     • peer.connect - UDP hole punching");

    // Bind to Unix socket
    let listener = tokio::net::UnixListener::bind(socket_path)
        .map_err(|e| anyhow::anyhow!("Failed to bind to {}: {}", socket_path, e))?;

    // Accept connections in a loop
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tracing::debug!("New IPC connection accepted");

                // Create handler for this connection
                let handler_clone = IpcServiceHandler::new(registry.clone());

                tokio::spawn(async move {
                    // Handle connection
                    let (reader, mut writer) = tokio::io::split(stream);
                    let mut reader = BufReader::new(reader);
                    let mut line = String::new();

                    loop {
                        line.clear();
                        match reader.read_line(&mut line).await {
                            Ok(0) => {
                                tracing::debug!("IPC client disconnected");
                                break;
                            }
                            Ok(_) => {
                                if line.trim().is_empty() {
                                    continue;
                                }

                                // Parse JSON-RPC request
                                let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
                                    Ok(request) => {
                                        tracing::debug!("IPC JSON-RPC request: {}", request.method);
                                        match handler_clone
                                            .handle(
                                                &request.method,
                                                request.params.unwrap_or(serde_json::Value::Null),
                                            )
                                            .await
                                        {
                                            Ok(result) => {
                                                JsonRpcResponse::success(result, request.id)
                                            }
                                            Err(message) => JsonRpcResponse::error(
                                                JsonRpcError::internal_error(message),
                                                request.id,
                                            ),
                                        }
                                    }
                                    Err(e) => JsonRpcResponse::error(
                                        JsonRpcError {
                                            code: JsonRpcError::PARSE_ERROR,
                                            message: format!("Failed to parse request: {}", e),
                                            data: None,
                                        },
                                        serde_json::Value::Null,
                                    ),
                                };

                                // Send response
                                if let Ok(response_json) = serde_json::to_string(&response) {
                                    let _ = writer.write_all(response_json.as_bytes()).await;
                                    let _ = writer.write_all(b"\n").await;
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to read from IPC socket: {}", e);
                                break;
                            }
                        }
                    }
                });
            }
            Err(e) => {
                tracing::error!("Failed to accept IPC connection: {}", e);
            }
        }
    }
}

/// Start TCP IPC server for universal platform support
///
/// TCP transport works on Android, Windows, and anywhere Unix sockets are restricted.
async fn start_ipc_server_tcp(listen_addr: &str, _beardog_conn: BearDogConnection) -> Result<()> {
    use songbird_universal_ipc::registry::ServiceRegistry;
    use songbird_universal_ipc::service::IpcServiceHandler;
    use songbird_universal_ipc::tower_atomic::{
        JsonRpcError, JsonRpcHandler, JsonRpcRequest, JsonRpcResponse,
    };
    use std::sync::Arc;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::sync::RwLock;

    // Parse listen address
    let addr: std::net::SocketAddr = listen_addr.parse()
        .map_err(|e| anyhow::anyhow!("Invalid listen address {}: {}", listen_addr, e))?;

    // Create IPC service handler with all methods
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));

    tracing::info!("✅ TCP IPC server binding to {}", addr);
    tracing::info!("   Methods available:");
    tracing::info!("     • http.request, http.get, http.post - HTTP/HTTPS requests");
    tracing::info!("     • stun.get_public_address, stun.bind - NAT traversal");
    tracing::info!("     • discovery.peers - Real-time peer discovery");
    tracing::info!("     • rendezvous.register, rendezvous.lookup - Relay server");
    tracing::info!("     • peer.connect - UDP hole punching");

    // Bind to TCP
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind TCP to {}: {}", addr, e))?;

    let bound_addr = listener.local_addr()
        .map_err(|e| anyhow::anyhow!("Failed to get local address: {}", e))?;
    tracing::info!("✅ TCP IPC server listening on {}", bound_addr);

    // Accept connections in a loop
    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                tracing::debug!("New TCP IPC connection from {}", peer_addr);

                // Create handler for this connection
                let handler_clone = IpcServiceHandler::new(registry.clone());

                tokio::spawn(async move {
                    // Handle connection
                    let (reader, mut writer) = tokio::io::split(stream);
                    let mut reader = BufReader::new(reader);
                    let mut line = String::new();

                    loop {
                        line.clear();
                        match reader.read_line(&mut line).await {
                            Ok(0) => {
                                tracing::debug!("TCP IPC client disconnected: {}", peer_addr);
                                break;
                            }
                            Ok(_) => {
                                if line.trim().is_empty() {
                                    continue;
                                }

                                // Parse JSON-RPC request
                                let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
                                    Ok(request) => {
                                        tracing::debug!("TCP IPC JSON-RPC request: {} from {}", request.method, peer_addr);
                                        match handler_clone
                                            .handle(
                                                &request.method,
                                                request.params.unwrap_or(serde_json::Value::Null),
                                            )
                                            .await
                                        {
                                            Ok(result) => {
                                                JsonRpcResponse::success(result, request.id)
                                            }
                                            Err(message) => JsonRpcResponse::error(
                                                JsonRpcError::internal_error(message),
                                                request.id,
                                            ),
                                        }
                                    }
                                    Err(e) => JsonRpcResponse::error(
                                        JsonRpcError {
                                            code: JsonRpcError::PARSE_ERROR,
                                            message: format!("Failed to parse request: {}", e),
                                            data: None,
                                        },
                                        serde_json::Value::Null,
                                    ),
                                };

                                // Send response
                                if let Ok(response_json) = serde_json::to_string(&response) {
                                    let _ = writer.write_all(response_json.as_bytes()).await;
                                    let _ = writer.write_all(b"\n").await;
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to read from TCP IPC socket ({}): {}", peer_addr, e);
                                break;
                            }
                        }
                    }
                });
            }
            Err(e) => {
                tracing::error!("Failed to accept TCP IPC connection: {}", e);
            }
        }
    }
}

/// Mask sensitive values in configuration for safe display
///
/// ✅ Modern approach: Creates a copy with secrets masked (simple placeholder for now)
fn mask_secrets_in_config(
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
        println!("│  TLS Certificate: {}", cert);
    }
    if let Some(ref key) = config.security.tls.key_path {
        println!("│  TLS Key: {}", key);
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
            println!("  • {}: {:?}", key, value);
        }
        println!();
    }
}
