//! Simple server example using universal IPC

use songbird_universal_ipc::ipc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Initialize universal IPC
    ipc::init()?;

    println!("🚀 Starting simple server...");

    // Register this primal
    let endpoint = ipc::register("simple-server", vec!["demo".to_string()]).await?;

    println!("✅ Registered as 'simple-server' at: {}", endpoint.path);

    // Listen for connections
    let mut listener = ipc::listen(endpoint).await?;

    println!("👂 Listening for connections...");

    // Accept connections
    loop {
        match listener.accept().await {
            Ok(mut stream) => {
                println!("📥 Accepted connection!");

                tokio::spawn(async move {
                    let mut buf = vec![0u8; 1024];

                    // Read message
                    match stream.read(&mut buf).await {
                        Ok(n) if n > 0 => {
                            let message = String::from_utf8_lossy(&buf[..n]);
                            println!("📨 Received: {}", message);

                            // Echo back
                            let response = format!("Echo: {}", message);
                            if let Err(e) = stream.write_all(response.as_bytes()).await {
                                eprintln!("❌ Failed to write response: {}", e);
                            } else {
                                println!("📤 Sent response");
                            }
                        }
                        Ok(_) => {
                            println!("🔌 Client disconnected");
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to read: {}", e);
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("❌ Failed to accept connection: {}", e);
            }
        }
    }
}

