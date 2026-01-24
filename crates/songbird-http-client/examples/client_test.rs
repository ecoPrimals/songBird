//! Songbird TLS Client Test Binary
//!
//! Purpose: Used by self-test harness to validate client+server transcript matching
//! Strategy: biomeOS validated approach (18+ hour debugging session)

use songbird_http_client::SongbirdHttpClient;
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
    info!("║   🔗 SONGBIRD TLS CLIENT - SELF-TEST MODE                  ║");
    info!("║                                                              ║");
    info!("╚══════════════════════════════════════════════════════════════╝");
    info!("");

    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut url = "https://localhost:8443".to_string();
    let mut _skip_verify = false;
    let mut beardog_socket = "/tmp/beardog-test.sock".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--url" => {
                if i + 1 < args.len() {
                    url = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --url requires a value");
                    std::process::exit(1);
                }
            }
            "--skip-verify" => {
                _skip_verify = true;
                i += 1;
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
                println!("Songbird TLS Client - Self-Test Mode");
                println!("");
                println!("Usage: client_test [OPTIONS]");
                println!("");
                println!("Options:");
                println!("  --url <URL>                URL to connect to (default: https://localhost:8443)");
                println!("  --skip-verify              Skip certificate verification (for self-signed certs)");
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
    info!("   URL: {}", url);
    info!("   Skip Verify: {}", _skip_verify);
    info!("   BearDog Socket: {}", beardog_socket);
    info!("");

    // Create HTTPS client (uses BearDogClient::from_env() for mode detection)
    info!("🏗️  Creating HTTPS client...");
    info!("   Mode: Controlled by BEARDOG_MODE env var");
    info!("   - BEARDOG_MODE=direct → Direct RPC to BearDog");
    info!("   - BEARDOG_MODE=neural → Via Neural API (default)");
    let client = SongbirdHttpClient::from_env();
    info!("✅ Client ready");
    info!("");

    // Make request
    info!("═══════════════════════════════════════════════════════════════");
    info!("🔗 Connecting to: {}", url);
    info!("═══════════════════════════════════════════════════════════════");
    info!("");

    match client.get(&url).await {
        Ok(response) => {
            info!("");
            info!("═══════════════════════════════════════════════════════════════");
            info!("✅ TLS HANDSHAKE COMPLETE!");
            info!("═══════════════════════════════════════════════════════════════");
            info!("");
            info!("📊 Response:");
            info!("   Status: {}", response.status);
            info!("   Body: {}", response.body);
            info!("");
            info!("🔬 CLIENT TRANSCRIPT logged above");
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

    info!("🏁 Client shutting down (self-test complete)");
    Ok(())
}
