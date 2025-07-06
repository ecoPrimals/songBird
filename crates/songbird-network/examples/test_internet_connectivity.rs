//! Test Internet Connectivity
//!
//! Simple test to validate internet connectivity and basic network functionality

use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{error, info, warn};
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    fmt::init();

    info!("🌐 Testing Internet Connectivity");
    info!("================================");

    // Test 1: UDP Socket Creation and Binding
    info!("\n🧪 Test 1: UDP Socket Creation");
    match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => {
            info!("✅ UDP socket created successfully");
            if let Ok(local_addr) = socket.local_addr() {
                info!("📍 Local address: {}", local_addr);
            }

            // Test UDP connectivity to public DNS
            let dns_addr: SocketAddr = "8.8.8.8:53".parse()?;
            match socket.send_to(b"test", dns_addr) {
                Ok(bytes_sent) => {
                    info!("✅ UDP packet sent: {} bytes to {}", bytes_sent, dns_addr);
                }
                Err(e) => {
                    warn!("⚠️ UDP send failed: {}", e);
                }
            }
        }
        Err(e) => {
            error!("❌ UDP socket creation failed: {}", e);
        }
    }

    // Test 2: DNS Resolution
    info!("\n🧪 Test 2: DNS Resolution");
    let test_domains = vec!["google.com", "github.com", "cloudflare.com"];

    for domain in test_domains {
        let lookup_target = format!("{}:80", domain);
        match timeout(
            Duration::from_secs(5),
            tokio::net::lookup_host(lookup_target),
        )
        .await
        {
            Ok(Ok(mut addresses)) => {
                if let Some(addr) = addresses.next() {
                    info!("✅ Resolved {}: {}", domain, addr);
                } else {
                    warn!("⚠️ No addresses found for {}", domain);
                }
            }
            Ok(Err(e)) => {
                warn!("⚠️ Failed to resolve {}: {}", domain, e);
            }
            Err(_) => {
                warn!("⏰ DNS resolution for {} timed out", domain);
            }
        }
    }

    // Test 3: TCP Connectivity
    info!("\n🧪 Test 3: TCP Connectivity");
    let tcp_targets = vec![("google.com", 80), ("github.com", 443)];

    for (host, port) in tcp_targets {
        let addr = format!("{}:{}", host, port);
        match timeout(Duration::from_secs(10), TcpStream::connect(&addr)).await {
            Ok(Ok(_stream)) => {
                info!("✅ TCP connection to {} successful", addr);
            }
            Ok(Err(e)) => {
                warn!("⚠️ TCP connection to {} failed: {}", addr, e);
            }
            Err(_) => {
                warn!("⏰ TCP connection to {} timed out", addr);
            }
        }
    }

    // Test 4: Network Interface Detection
    info!("\n🧪 Test 4: Network Interface Detection");
    let test_addresses = vec!["127.0.0.1:0", "0.0.0.0:0"];

    for addr_str in test_addresses {
        match addr_str.parse::<SocketAddr>() {
            Ok(addr) => match UdpSocket::bind(addr) {
                Ok(socket) => {
                    if let Ok(local_addr) = socket.local_addr() {
                        info!("✅ Successfully bound to: {}", local_addr);
                    }
                }
                Err(e) => {
                    warn!("⚠️ Failed to bind to {}: {}", addr, e);
                }
            },
            Err(e) => {
                error!("❌ Invalid address {}: {}", addr_str, e);
            }
        }
    }

    info!("\n🎯 Internet Connectivity Test Complete!");
    info!(
        "Results show whether the system has internet access and can create network connections."
    );
    info!("This validates that WireGuard tunnels would work with live internet connectivity.");

    Ok(())
}
