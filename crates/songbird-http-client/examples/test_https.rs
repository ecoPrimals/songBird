/// Simple HTTPS test binary for debugging TLS handshakes
/// 
/// Usage:
///   RUST_LOG=trace cargo run --example test_https -- https://httpbin.org/get
/// 
/// This binary provides comprehensive logging to identify TLS handshake issues.

use songbird_http_client::{SongbirdHttpClient, Result};
use std::env;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();

    // Get URL from args
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <url>", args[0]);
        eprintln!("Example: {} https://httpbin.org/get", args[0]);
        std::process::exit(1);
    }

    let url = &args[1];
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                                                              ║");
    println!("║           🔍 SONGBIRD HTTPS DEBUG TEST                       ║");
    println!("║                                                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    println!("📋 Test URL: {}", url);
    println!("📊 Logging: TRACE (comprehensive)");
    println!("🎯 Goal: Identify TLS handshake failure point\n");

    // Get Neural API socket from environment or use default
    let neural_socket = env::var("NEURAL_API_SOCKET")
        .unwrap_or_else(|_| "/tmp/neural-api-nat0.sock".to_string());
    println!("🔌 Neural API Socket: {}\n", neural_socket);

    println!("──────────────────────────────────────────────────────────────\n");
    println!("🚀 Starting HTTPS request...\n");

    // Create client
    let client = SongbirdHttpClient::from_env();

    // Make request
    match client.get(url).await {
        Ok(response) => {
            println!("\n──────────────────────────────────────────────────────────────\n");
            println!("✅ SUCCESS! HTTP {}", response.status);
            println!("📊 Headers: {} entries", response.headers.len());
            
            // Show first few headers
            let header_count = std::cmp::min(5, response.headers.len());
            if header_count > 0 {
                println!("\n📋 Headers (first {}):", header_count);
                for (key, value) in response.headers.iter().take(header_count) {
                    let value_preview = if value.len() > 60 {
                        format!("{}...", &value[..60])
                    } else {
                        value.clone()
                    };
                    println!("   {}: {}", key, value_preview);
                }
            }
            
            // Serialize body to string for display
            let body_str = response.body.to_string();
            println!("\n📦 Body: {} bytes (JSON)", body_str.len());
            
            // Show body preview
            let body_preview = if body_str.len() > 200 {
                format!("{}...", &body_str[..200])
            } else {
                body_str.clone()
            };
            println!("\n📄 Body Preview:\n{}", body_preview);
            
            println!("\n──────────────────────────────────────────────────────────────");
            println!("\n🎉 TEST COMPLETE - HTTPS WORKING!\n");
            
            Ok(())
        }
        Err(e) => {
            println!("\n──────────────────────────────────────────────────────────────\n");
            println!("❌ ERROR: {}", e);
            println!("\n💡 DEBUGGING HINTS:");
            println!("──────────────────────────────────────────────────────────────");
            
            let error_msg = format!("{}", e);
            
            if error_msg.contains("early eof") {
                println!("\n🔍 ERROR TYPE: 'early eof'");
                println!("\n📋 WHAT THIS MEANS:");
                println!("   • Server closed TCP connection during handshake");
                println!("   • Could be before or after ServerHello");
                println!("   • Check logs above for exact failure point\n");
                
                println!("🎯 NEXT STEPS:");
                println!("   1. Check logs for '📥 Waiting for ServerHello'");
                println!("   2. If you see that, server rejected ClientHello");
                println!("   3. If you don't see that, issue is before sending\n");
                
                println!("🔧 COMMON CAUSES:");
                println!("   • Missing or malformed ClientHello extension");
                println!("   • Server doesn't support TLS 1.3");
                println!("   • Network/firewall issue");
                println!("   • Server requires specific cipher suite\n");
            } else if error_msg.contains("timeout") {
                println!("\n🔍 ERROR TYPE: Timeout");
                println!("\n📋 WHAT THIS MEANS:");
                println!("   • Server didn't respond within 10 seconds");
                println!("   • Could be network issue or server overload\n");
                
                println!("🎯 NEXT STEPS:");
                println!("   1. Try again (might be temporary)");
                println!("   2. Test with different server (example.com)");
                println!("   3. Check network connectivity\n");
            } else if error_msg.contains("Connection refused") {
                println!("\n🔍 ERROR TYPE: Connection Refused");
                println!("\n📋 WHAT THIS MEANS:");
                println!("   • Can't connect to server");
                println!("   • Server might be down");
                println!("   • Firewall blocking connection\n");
                
                println!("🎯 NEXT STEPS:");
                println!("   1. Verify server is reachable: nc -v <host> 443");
                println!("   2. Check firewall rules");
                println!("   3. Try different server\n");
            } else {
                println!("\n🔍 ERROR TYPE: {}", error_msg);
                println!("\n📋 Check logs above for detailed error information\n");
            }
            
            println!("──────────────────────────────────────────────────────────────");
            println!("\n🔍 REVIEW THE LOGS ABOVE TO FIND:");
            println!("   • Last successful step (e.g., '✅ ClientHello sent')");
            println!("   • First error message (e.g., '❌ Failed to read...')");
            println!("   • Any TLS alerts from server");
            println!("   • Timing information (how long before error)\n");
            
            println!("💡 TIP: Share the complete log output when reporting issue!\n");
            
            Err(e)
        }
    }
}

