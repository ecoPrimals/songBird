//! REST API Demo
//!
//! Demonstrates the HTTP REST API endpoints for service management,
//! communication, and monitoring in the Songbird Orchestrator.

use chrono::Utc;
use songbird_orchestrator::{
    api::start_server as start_api_server,
    communication::{WebSocketCommunication, WebSocketConfig},
    traits::communication::CommunicationLayer,
    Orchestrator, OrchestratorConfig,
};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::sleep;
use tracing_subscriber::fmt::init;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    init();

    println!("🚀 Songbird Orchestrator - REST API Demo");
    println!("=======================================");

    // Create orchestrator
    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await?;

    // Start orchestrator
    orchestrator.start().await?;
    println!("✅ Orchestrator started");

    // Create WebSocket communication
    let ws_config = WebSocketConfig {
        max_connections: 100,
        connection_timeout: Duration::from_secs(30),
        heartbeat_interval: Duration::from_secs(15),
        message_buffer_size: 500,
    };

    let websocket = WebSocketCommunication::with_config("127.0.0.1".to_string(), 8080, ws_config);

    // Start WebSocket server
    println!("🌐 Starting WebSocket server on ws://127.0.0.1:8080");
    websocket.connect().await?;
    println!("✅ WebSocket server started");

    // Start REST API server
    let api_addr: SocketAddr = "127.0.0.1:3000".parse()?;
    println!("🔗 Starting REST API server on http://{}", api_addr);

    start_api_server(
        std::sync::Arc::new(orchestrator.clone()),
        std::sync::Arc::new(websocket.clone()),
        api_addr,
    )
    .await?;

    println!("✅ REST API server started");

    // Give servers time to start
    sleep(Duration::from_secs(1)).await;

    // Display available endpoints
    println!("\n📋 Available REST API Endpoints:");
    println!("   Health & System:");
    println!("     GET  http://127.0.0.1:3000/health");
    println!("     GET  http://127.0.0.1:3000/health/detailed");
    println!("     GET  http://127.0.0.1:3000/system/info");
    println!("     GET  http://127.0.0.1:3000/system/metrics");

    println!("\n   Service Management:");
    println!("     GET  http://127.0.0.1:3000/services");
    println!("     POST http://127.0.0.1:3000/services");
    println!("     GET  http://127.0.0.1:3000/services/:id");
    println!("     POST http://127.0.0.1:3000/services/:id/start");
    println!("     POST http://127.0.0.1:3000/services/:id/stop");

    println!("\n   Communication:");
    println!("     POST http://127.0.0.1:3000/communication/send");
    println!("     POST http://127.0.0.1:3000/communication/broadcast");
    println!("     GET  http://127.0.0.1:3000/communication/stats");

    println!("\n   Monitoring & Metrics:");
    println!("     GET  http://127.0.0.1:3000/metrics");
    println!("     GET  http://127.0.0.1:3000/metrics/prometheus");
    println!("     GET  http://127.0.0.1:3000/metrics/services");

    println!("\n   Real-time Streams:");
    println!("     GET  http://127.0.0.1:3000/stream/events (Server-Sent Events)");
    println!("     GET  http://127.0.0.1:3000/stream/metrics (Server-Sent Events)");

    println!("\n   Dashboard:");
    println!("     GET  http://127.0.0.1:3000/dashboard");

    // Test the API with some HTTP requests using reqwest
    println!("\n🧪 Testing API endpoints...");

    let client = reqwest::Client::new();

    // Test health endpoint
    println!("\n1. Testing health endpoint:");
    match client.get("http://127.0.0.1:3000/health").send().await {
        Ok(response) => {
            println!(
                "   ✅ Health: {} - {}",
                response.status(),
                response.text().await?
            );
        }
        Err(e) => {
            println!("   ❌ Health check failed: {}", e);
        }
    }

    // Test system info
    println!("\n2. Testing system info:");
    match client.get("http://127.0.0.1:3000/system/info").send().await {
        Ok(response) => {
            if response.status().is_success() {
                let text = response.text().await?;
                println!("   ✅ System Info received ({} bytes)", text.len());

                // Pretty print JSON
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    println!("   📊 Data: {}", serde_json::to_string_pretty(&json)?);
                }
            } else {
                println!("   ❌ System info failed: {}", response.status());
            }
        }
        Err(e) => {
            println!("   ❌ System info request failed: {}", e);
        }
    }

    // Test services list
    println!("\n3. Testing services list:");
    match client.get("http://127.0.0.1:3000/services").send().await {
        Ok(response) => {
            if response.status().is_success() {
                let text = response.text().await?;
                println!("   ✅ Services list received ({} bytes)", text.len());
            } else {
                println!("   ❌ Services list failed: {}", response.status());
            }
        }
        Err(e) => {
            println!("   ❌ Services list request failed: {}", e);
        }
    }

    // Test communication stats
    println!("\n4. Testing communication stats:");
    match client
        .get("http://127.0.0.1:3000/communication/stats")
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                let text = response.text().await?;
                println!("   ✅ Communication stats received ({} bytes)", text.len());

                // Pretty print JSON
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    println!("   📊 Stats: {}", serde_json::to_string_pretty(&json)?);
                }
            } else {
                println!("   ❌ Communication stats failed: {}", response.status());
            }
        }
        Err(e) => {
            println!("   ❌ Communication stats request failed: {}", e);
        }
    }

    // Test broadcast message via API
    println!("\n5. Testing message broadcast via API:");
    let broadcast_request = serde_json::json!({
        "message_type": "Event",
        "topic": "api.test",
        "payload": {
            "test": true,
            "timestamp": Utc::now(),
            "message": "Hello from REST API!"
        },
        "headers": {
            "source": "api-demo"
        },
        "ttl": 300
    });

    match client
        .post("http://127.0.0.1:3000/communication/broadcast")
        .json(&broadcast_request)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                let text = response.text().await?;
                println!("   ✅ Broadcast successful: {}", text);
            } else {
                println!(
                    "   ❌ Broadcast failed: {} - {}",
                    response.status(),
                    response.text().await?
                );
            }
        }
        Err(e) => {
            println!("   ❌ Broadcast request failed: {}", e);
        }
    }

    // Test metrics endpoint
    println!("\n6. Testing metrics endpoint:");
    match client.get("http://127.0.0.1:3000/metrics").send().await {
        Ok(response) => {
            if response.status().is_success() {
                let text = response.text().await?;
                println!("   ✅ Metrics received ({} bytes)", text.len());

                // Pretty print JSON
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    println!("   📊 Metrics: {}", serde_json::to_string_pretty(&json)?);
                }
            } else {
                println!("   ❌ Metrics failed: {}", response.status());
            }
        }
        Err(e) => {
            println!("   ❌ Metrics request failed: {}", e);
        }
    }

    // Test Prometheus metrics
    println!("\n7. Testing Prometheus metrics:");
    match client
        .get("http://127.0.0.1:3000/metrics/prometheus")
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                let text = response.text().await?;
                println!("   ✅ Prometheus metrics received:");
                println!("   {}", text);
            } else {
                println!("   ❌ Prometheus metrics failed: {}", response.status());
            }
        }
        Err(e) => {
            println!("   ❌ Prometheus metrics request failed: {}", e);
        }
    }

    // Test dashboard endpoint
    println!("\n8. Testing dashboard endpoint:");
    match client.get("http://127.0.0.1:3000/dashboard").send().await {
        Ok(response) => {
            if response.status().is_success() {
                let text = response.text().await?;
                println!("   ✅ Dashboard data received ({} bytes)", text.len());
            } else {
                println!("   ❌ Dashboard failed: {}", response.status());
            }
        }
        Err(e) => {
            println!("   ❌ Dashboard request failed: {}", e);
        }
    }

    println!("\n🎯 Demo Instructions:");
    println!("   1. Open your browser to http://127.0.0.1:3000/health");
    println!("   2. Try the dashboard at http://127.0.0.1:3000/dashboard");
    println!("   3. Use curl or Postman to test other endpoints");
    println!("   4. Connect to ws://127.0.0.1:8080 for WebSocket communication");
    println!("   5. Use Server-Sent Events: curl http://127.0.0.1:3000/stream/events");

    println!("\n📝 Example curl commands:");
    println!("   curl http://127.0.0.1:3000/health");
    println!("   curl http://127.0.0.1:3000/system/info");
    println!("   curl http://127.0.0.1:3000/services");
    println!("   curl http://127.0.0.1:3000/communication/stats");
    println!("   curl -X POST http://127.0.0.1:3000/communication/broadcast \\");
    println!("        -H 'Content-Type: application/json' \\");
    println!("        -d '{{\"message_type\":\"Event\",\"topic\":\"test\",\"payload\":{{\"hello\":\"world\"}}}}'");

    // Keep running for demo purposes
    println!("\n🔄 Demo running... Press Ctrl+C to stop");

    // Run for 60 seconds then show final stats
    sleep(Duration::from_secs(60)).await;

    // Final status
    println!("\n📊 Final Status:");
    let metrics = orchestrator.get_metrics().await;
    println!(
        "   - Orchestrator uptime: {} seconds",
        metrics.uptime_seconds
    );
    println!("   - Total services: {}", metrics.total_services);
    println!("   - Healthy services: {}", metrics.healthy_services);
    println!("   - Total requests: {}", metrics.total_requests);

    if let Ok(comm_stats) = websocket.get_stats().await {
        println!(
            "   - WebSocket connections: {}",
            comm_stats.active_connections
        );
        println!("   - Messages sent: {}", comm_stats.messages_sent);
        println!("   - Messages received: {}", comm_stats.messages_received);
    }

    // Cleanup
    println!("\n🧹 Cleaning up...");
    websocket.disconnect().await?;
    orchestrator.stop().await?;
    println!("✅ Demo completed successfully!");

    Ok(())
}
