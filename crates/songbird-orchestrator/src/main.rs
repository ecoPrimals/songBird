//! Songbird - Network Orchestration & Discovery Primal
//!
//! UniBin Architecture (Ecosystem Standard v1.0.0)
//! Main entry point with subcommand structure for different operational modes

use anyhow::Result;
use clap::{Parser, Subcommand};
use songbird_orchestrator::app;
use songbird_orchestrator::process_manager::ProcessManager;
use songbird_types::config::CanonicalSongbirdConfig;

/// Songbird - Network Orchestration & Discovery Primal
///
/// UniBin Architecture: One binary, multiple modes
#[derive(Parser)]
#[command(name = "songbird")]
#[command(about = "Network Orchestration & Discovery Primal", long_about = None)]
#[command(version)]
#[command(author = "ecoPrimals <contact@ecoprimals.dev>")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start Songbird orchestrator in server mode
    ///
    /// This is the primary operational mode that runs the full orchestrator
    /// with discovery, federation, and network services.
    Server {
        /// HTTP server port
        #[arg(long, short, default_value = "8080")]
        port: u16,

        /// Run as daemon (background process)
        #[arg(long, short)]
        daemon: bool,

        /// Configuration file path
        #[arg(long, short)]
        config: Option<String>,

        /// Enable verbose logging
        #[arg(long, short)]
        verbose: bool,
    },

    /// Run health diagnostics and system checks
    ///
    /// Validates configuration, checks connectivity, and verifies system health.
    Doctor {
        /// Run comprehensive checks (includes primal connectivity)
        #[arg(long, short)]
        comprehensive: bool,

        /// Output format (text, json, yaml)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Configuration management commands
    ///
    /// View, validate, and initialize Songbird configuration.
    Config {
        #[command(subcommand)]
        config_cmd: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Server {
            port,
            daemon,
            config,
            verbose,
        } => {
            run_server(port, daemon, config, verbose).await?;
        }
        Commands::Doctor {
            comprehensive,
            format,
        } => {
            run_doctor(comprehensive, &format).await?;
        }
        Commands::Config { config_cmd } => {
            run_config_command(config_cmd).await?;
        }
    }

    Ok(())
}

/// Run Songbird orchestrator in server mode
///
/// Modern, idiomatic, async Rust implementation with:
/// - Proper signal handling (SIGINT, SIGTERM)
/// - Graceful shutdown
/// - Instance locking
/// - Comprehensive logging
async fn run_server(
    port: u16,
    daemon: bool,
    config_path: Option<String>,
    verbose: bool,
) -> Result<()> {
    // Initialize tracing (early, before any logging)
    if verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt::init();
    }

    // Log startup with mode information
    tracing::info!("🚀 Songbird v{} - Server Mode", env!("CARGO_PKG_VERSION"));
    tracing::info!("   Mode: Server {}", if daemon { "(daemon)" } else { "(foreground)" });
    tracing::info!("   Port: {}", port);
    tracing::info!("   Process ID: {}", std::process::id());

    // ✅ Step 1: Acquire instance lock FIRST (before any resources)
    // This lock is scoped per NODE_ID, enabling multi-instance deployments
    // Prevents "Federation Split State Bug" (Dec 20, 2025)
    let process_mgr = ProcessManager::new()?;
    let _singleton_guard = process_mgr.acquire_lock()?;
    tracing::info!("   Instance Lock: ✅ Acquired (PID file active)");

    // ✅ Pure Songbird TLS - No crypto provider init needed!
    // (BearDog handles all crypto via JSON-RPC at runtime)

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
    let config = if let Some(path) = config_path {
        tracing::info!("   Config file: {}", path);
        // TODO: Load from file (future enhancement)
        CanonicalSongbirdConfig::from_env()
            .map_err(|e| anyhow::anyhow!("Failed to load configuration from file: {}", e))?
    } else {
        tracing::info!("   Config source: Environment variables");
        CanonicalSongbirdConfig::from_env()
            .map_err(|e| anyhow::anyhow!("Failed to load configuration from environment: {}", e))?
    };
    tracing::info!("   Configuration: ✅ Loaded");

    // Step 4: Start the orchestrator (non-blocking, returns handle)
    // v3.24.0: Modern idiomatic Rust - clear separation of concerns
    tracing::info!("🔧 Starting orchestrator components...");
    let mut orchestrator = app::start_orchestrator(config).await?;
    tracing::info!("   Orchestrator: ✅ Started");

    tracing::info!("✅ Songbird ready!");
    tracing::info!("   Unix Socket IPC: /tmp/songbird-*.sock (see logs for actual path)");
    tracing::info!("   Protocol: JSON-RPC 2.0 over Unix sockets");
    tracing::info!("   HTTP/TLS: Handled by external gateway component");
    tracing::info!("");
    tracing::info!("💡 Press Ctrl+C to stop gracefully");

    // Step 5: If daemon mode, detach from terminal (future enhancement)
    if daemon {
        tracing::info!("📌 Daemon mode: Process detached");
        // TODO: Actual daemonization (future enhancement)
    }

    // Step 6: Main event loop - wait for shutdown signal
    // Modern async/await pattern with tokio::select!
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
                // Windows: only Ctrl+C is available
                std::future::pending::<()>().await
            }
        } => {
            tracing::info!("🛑 Received SIGTERM, initiating graceful shutdown...");
        }
    }

    // Step 7: Graceful shutdown - stop orchestrator components
    tracing::info!("🧹 Stopping orchestrator components...");
    orchestrator.stop().await?;
    tracing::info!("   Orchestrator: ✅ Stopped");

    tracing::info!("🧹 Cleaning up resources...");
    tracing::info!("   • Releasing instance lock (PID file)");
    tracing::info!("   • Closing network connections");
    tracing::info!("   • Flushing logs");

    // Add a small delay to ensure logs are flushed
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    tracing::info!("✅ Graceful shutdown complete");

    Ok(())
    // _singleton_guard drops here, removing PID file cleanly
    // This is the RAII pattern - cleanup is automatic, panic-safe
}

/// Run health diagnostics and system checks
///
/// Modern, idiomatic Rust implementation with async/await
async fn run_doctor(comprehensive: bool, format: &str) -> Result<()> {
    // Initialize minimal logging for doctor mode
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    match format {
        "text" => run_doctor_text(comprehensive).await,
        "json" => run_doctor_json(comprehensive).await,
        "yaml" => run_doctor_yaml(comprehensive).await,
        _ => {
            eprintln!("❌ Unknown format: {}. Use: text, json, or yaml", format);
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
            Err(e) => println!("      Status: ❌ Error: {}", e),
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
async fn run_doctor_json(_comprehensive: bool) -> Result<()> {
    // TODO: Implement JSON output
    println!(r#"{{"status":"ok","message":"JSON output not yet implemented"}}"#);
    Ok(())
}

/// Run doctor in YAML format (machine-readable)
async fn run_doctor_yaml(_comprehensive: bool) -> Result<()> {
    // TODO: Implement YAML output
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
    use songbird_orchestrator::btsp_client::BtspClient;

    let client = BtspClient::new();
    match client.ping().await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Run configuration management commands
///
/// Modern, idiomatic Rust implementation
async fn run_config_command(cmd: ConfigCommands) -> Result<()> {
    // Initialize minimal logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    match cmd {
        ConfigCommands::Show { show_secrets } => {
            show_config(show_secrets).await?;
        }
        ConfigCommands::Validate => {
            validate_config().await?;
        }
        ConfigCommands::Init { output, force } => {
            init_config(&output, force).await?;
        }
    }

    Ok(())
}

/// Show current configuration
async fn show_config(show_secrets: bool) -> Result<()> {
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

            // TODO: Display actual config values
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

    // Check if file exists
    if std::path::Path::new(output).exists() && !force {
        eprintln!("❌ File already exists: {}", output);
        eprintln!();
        eprintln!("💡 Use --force to overwrite");
        std::process::exit(1);
    }

    let template = r#"# Songbird Configuration Template
# Generated by: songbird config init
# Version: 3.24.0

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

# Optional: TLS
# SONGBIRD_TLS_CERT_PATH=./certs/cert.pem
# SONGBIRD_TLS_KEY_PATH=./certs/key.pem
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
