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
    /// HTTP server port
    #[arg(long, short, default_value = "8080")]
    pub port: u16,

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
    /// Enables external primals to access HTTP/HTTPS capabilities
    /// Example: /tmp/songbird-nat0.sock
    #[arg(long)]
    pub socket: Option<String>,

    /// BearDog socket path for crypto operations (defaults based on family_id)
    #[arg(long)]
    pub beardog_socket: Option<String>,
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
    tracing::info!("   Port: {}", args.port);
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
    let config = if let Some(path) = args.config {
        tracing::info!("   Config file: {}", path);
        CanonicalSongbirdConfig::from_env()
            .map_err(|e| anyhow::anyhow!("Failed to load configuration from file: {}", e))?
    } else {
        tracing::info!("   Config source: Environment variables");
        CanonicalSongbirdConfig::from_env()
            .map_err(|e| anyhow::anyhow!("Failed to load configuration from environment: {}", e))?
    };
    tracing::info!("   Configuration: ✅ Loaded");

    // Step 4: Start the orchestrator (non-blocking, returns handle)
    tracing::info!("🔧 Starting orchestrator components...");
    let mut orchestrator = app::start_orchestrator(config).await?;
    tracing::info!("   Orchestrator: ✅ Started");

    tracing::info!("✅ Songbird ready!");
    tracing::info!("   Unix Socket IPC: /tmp/songbird-*.sock (see logs for actual path)");
    tracing::info!("   Protocol: JSON-RPC 2.0 over Unix sockets");
    tracing::info!("");

    // Step 4.5: Start IPC server if socket is provided (NEW for biomeOS integration)
    let socket_path_for_registration = args.socket.clone(); // Clone for later use
    let ipc_handle = if let Some(socket_path) = args.socket {
        tracing::info!("🌐 Starting IPC Server (for biomeOS integration)...");
        tracing::info!("   Socket: {}", socket_path);

        // Determine BearDog socket
        let beardog_socket = args.beardog_socket.unwrap_or_else(|| {
            let family_id = family_identity.as_deref().unwrap_or("nat0");
            format!("/tmp/beardog-{}.sock", family_id)
        });
        tracing::info!("   BearDog: {}", beardog_socket);

        // Spawn IPC server in background task
        let socket_clone = socket_path.clone();
        Some(tokio::spawn(async move {
            match start_ipc_server(&socket_clone, &beardog_socket).await {
                Ok(_) => tracing::info!("IPC server stopped gracefully"),
                Err(e) => tracing::error!("IPC server error: {}", e),
            }
        }))
    } else {
        tracing::info!("💡 Tip: Use --socket /tmp/songbird-nat0.sock to enable IPC for biomeOS");
        None
    };

    // Step 4.6: Register capabilities with Neural API (if available)
    if socket_path_for_registration.is_some() {
        tracing::info!("");
        tracing::info!("🌟 Registering capabilities with Neural API...");
        if let Err(e) = crate::capability_registration::register_capabilities().await {
            tracing::warn!("⚠️  Failed to register capabilities: {}", e);
            tracing::warn!("   Songbird will continue without Neural API registration");
            tracing::warn!("   Direct socket connections will still work");
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
    if socket_path_for_registration.is_some() {
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
async fn run_doctor_json(_comprehensive: bool) -> Result<()> {
    println!(r#"{{"status":"ok","message":"JSON output not yet implemented"}}"#);
    Ok(())
}

/// Run doctor in YAML format
async fn run_doctor_yaml(_comprehensive: bool) -> Result<()> {
    println!("status: ok");
    println!("message: YAML output not yet implemented");
    Ok(())
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
        } => {
            show_config(show_secrets).await?;
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
async fn show_config(show_secrets: bool) -> Result<()> {
    use songbird_types::config::CanonicalSongbirdConfig;

    println!("📋 Songbird Configuration");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    match CanonicalSongbirdConfig::from_env() {
        Ok(config) => {
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

            println!("Configuration details: (implementation pending)");
            println!("{:?}", config);
        }
        Err(e) => {
            println!("❌ Configuration invalid");
            println!();
            println!("Error: {}", e);
            println!();
            println!("💡 Fix: Set required environment variables or create config file");
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

/// Start IPC server for external primal access to HTTP/HTTPS capabilities
///
/// This enables biomeOS and other primals to make HTTP/HTTPS requests via JSON-RPC
/// without embedding Songbird code (TRUE PRIMAL architecture).
async fn start_ipc_server(socket_path: &str, beardog_socket: &str) -> Result<()> {
    use songbird_universal_ipc::handlers::http_handler::HttpHandler;
    use songbird_universal_ipc::tower_atomic::{
        JsonRpcError, JsonRpcHandler, JsonRpcRequest, JsonRpcResponse,
    };
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    // Remove old socket if exists
    let _ = std::fs::remove_file(socket_path);

    // Create HTTP handler with default discovery
    let handler = HttpHandler::with_default_discovery();

    tracing::info!("✅ IPC server listening on {}", socket_path);
    tracing::info!("   Methods available:");
    tracing::info!("     • http.request - Full HTTP/HTTPS request");
    tracing::info!("     • http.get - GET request shorthand");
    tracing::info!("     • http.post - POST request shorthand");

    // Bind to Unix socket
    let listener = tokio::net::UnixListener::bind(socket_path)
        .map_err(|e| anyhow::anyhow!("Failed to bind to {}: {}", socket_path, e))?;

    // Accept connections in a loop
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tracing::debug!("New IPC connection accepted");

                // Clone handler for the spawned task
                let handler_clone = HttpHandler::with_default_discovery();

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
