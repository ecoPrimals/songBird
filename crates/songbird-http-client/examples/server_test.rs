//! Songbird TLS Server Test Binary
//! 
//! Purpose: Used by self-test harness to validate client+server transcript matching
//! Strategy: biomeOS validated approach (18+ hour debugging session)

use songbird_http_client::{
    beardog_client::BearDogClient,
    tls::server_complete::TlsServer,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging (CRITICAL for transcript comparison!)
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_line_number(false)
        .init();

    info!("╔══════════════════════════════════════════════════════════════╗");
    info!("║                                                              ║");
    info!("║   🔒 SONGBIRD TLS SERVER - SELF-TEST MODE                  ║");
    info!("║                                                              ║");
    info!("╚══════════════════════════════════════════════════════════════╝");
    info!("");

    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut port = 8443;
    let mut cert_file = "test-data/test-cert.pem".to_string();
    let mut key_file = "test-data/test-key.pem".to_string();
    let mut beardog_socket = "/tmp/beardog-test.sock".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                if i + 1 < args.len() {
                    port = args[i + 1].parse()?;
                    i += 2;
                } else {
                    eprintln!("Error: --port requires a value");
                    std::process::exit(1);
                }
            }
            "--cert" => {
                if i + 1 < args.len() {
                    cert_file = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --cert requires a value");
                    std::process::exit(1);
                }
            }
            "--key" => {
                if i + 1 < args.len() {
                    key_file = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --key requires a value");
                    std::process::exit(1);
                }
            }
            "--beardog-socket" => {
                if i + 1 < args.len() {
                    beardog_socket = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --beardog-socket requires a value");
                    std::process::exit(1);
                }
            }
            "--help" | "-h" => {
                println!("Songbird TLS Server - Self-Test Mode");
                println!("");
                println!("Usage: server_test [OPTIONS]");
                println!("");
                println!("Options:");
                println!("  --port <PORT>              Port to listen on (default: 8443)");
                println!("  --cert <FILE>              Certificate file (default: test-data/test-cert.pem)");
                println!("  --key <FILE>               Private key file (default: test-data/test-key.pem)");
                println!("  --beardog-socket <PATH>    BearDog socket path (default: /tmp/beardog-test.sock)");
                println!("  --help, -h                 Show this help message");
                std::process::exit(0);
            }
            _ => {
                eprintln!("Error: Unknown option: {}", args[i]);
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
        }
    }

    info!("📋 Configuration:");
    info!("   Port: {}", port);
    info!("   Certificate: {}", cert_file);
    info!("   Private Key: {}", key_file);
    info!("   BearDog Socket: {}", beardog_socket);
    info!("");

    // Load certificate and private key
    info!("🔐 Loading certificate and private key...");
    let cert_pem = std::fs::read_to_string(&cert_file)
        .map_err(|e| anyhow::anyhow!("Failed to read certificate: {}", e))?;
    let key_pem = std::fs::read_to_string(&key_file)
        .map_err(|e| anyhow::anyhow!("Failed to read private key: {}", e))?;
    
    // Parse PEM (simple extraction of base64 content)
    let cert_chain = parse_pem(&cert_pem)?;
    let private_key = parse_pem(&key_pem)?;
    
    info!("✅ Certificate chain: {} bytes", cert_chain.len());
    info!("✅ Private key: {} bytes", private_key.len());
    info!("");

    // Connect to BearDog (DIRECT MODE for self-test!)
    info!("🐻 Connecting to BearDog in DIRECT MODE...");
    let beardog = Arc::new(BearDogClient::new_direct(&beardog_socket));
    info!("✅ BearDog connected (direct RPC, no Neural API needed)");
    info!("");

    // Create TLS server
    info!("🏗️  Creating TLS server...");
    let mut server = TlsServer::new(beardog, cert_chain, private_key);
    info!("✅ TLS server ready");
    info!("");

    // Bind TCP listener
    info!("🔌 Binding to 0.0.0.0:{}...", port);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("✅ Listening on port {}", port);
    info!("");

    info!("🚀 Server started - waiting for connections...");
    info!("");

    // Accept ONE connection (for self-test)
    match listener.accept().await {
        Ok((mut stream, addr)) => {
            info!("═══════════════════════════════════════════════════════════════");
            info!("📥 New connection from: {}", addr);
            info!("═══════════════════════════════════════════════════════════════");
            info!("");

            // Handle TLS handshake
            match server.accept_connection(&mut stream).await {
                Ok(()) => {
                    info!("");
                    info!("═══════════════════════════════════════════════════════════════");
                    info!("✅ TLS HANDSHAKE COMPLETE!");
                    info!("═══════════════════════════════════════════════════════════════");
                    info!("");
                    info!("🔬 SERVER TRANSCRIPT logged above");
                    info!("   Look for: 'COMPLETE TRANSCRIPT HEX DUMP'");
                    info!("");
                }
                Err(e) => {
                    error!("");
                    error!("═══════════════════════════════════════════════════════════════");
                    error!("❌ TLS HANDSHAKE FAILED!");
                    error!("═══════════════════════════════════════════════════════════════");
                    error!("Error: {}", e);
                    error!("");
                    return Err(anyhow::anyhow!("TLS handshake failed: {}", e));
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to accept connection: {}", e);
            return Err(e.into());
        }
    }

    info!("🏁 Server shutting down (self-test complete)");
    Ok(())
}

/// Simple PEM parser - extracts base64 content and decodes
fn parse_pem(pem: &str) -> anyhow::Result<Vec<u8>> {
    let lines: Vec<&str> = pem.lines()
        .filter(|line| !line.starts_with("-----"))
        .collect();
    
    let base64_content = lines.join("");
    
    use base64::{Engine as _, engine::general_purpose};
    let decoded = general_purpose::STANDARD.decode(&base64_content)
        .map_err(|e| anyhow::anyhow!("Failed to decode PEM: {}", e))?;
    
    Ok(decoded)
}

