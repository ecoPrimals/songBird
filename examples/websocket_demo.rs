//! WebSocket Communication Demo
//!
//! Demonstrates the real-time WebSocket communication capabilities
//! of the Songbird Orchestrator.

use chrono::Utc;
use songbird_gaming_bridge::{
    communication::{WebSocketCommunication, WebSocketConfig},
    traits::communication::{CommunicationLayer, MessageType, ServiceMessage},
    Orchestrator, OrchestratorConfig,
};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use tracing_subscriber::fmt::init;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    init();

    println!("🚀 Songbird Orchestrator - WebSocket Communication Demo");
    println!("=====================================================");

    // Create orchestrator
    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await?;

    // Start orchestrator
    orchestrator.start().await?;
    println!("✅ Orchestrator started");

    // Create WebSocket communication layer
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

    // Show connection status
    println!("📊 Connection Status:");
    println!("   - Active connections: {}", websocket.connection_count());
    println!("   - Server running: {}", websocket.is_connected().await);

    // Create sample messages
    let messages = vec![
        ServiceMessage {
            id: Uuid::new_v4().to_string(),
            message_type: MessageType::Event,
            topic: Some("service.status".to_string()),
            body: serde_json::json!({
                "service": "demo-service",
                "status": "started",
                "timestamp": Utc::now()
            }),
            headers: HashMap::from([
                ("content-type".to_string(), "application/json".to_string()),
                ("source".to_string(), "orchestrator".to_string()),
            ]),
            timestamp: Utc::now(),
            correlation_id: Some(Uuid::new_v4().to_string()),
            reply_to: None,
            ttl: Some(300),
        },
        ServiceMessage {
            id: Uuid::new_v4().to_string(),
            message_type: MessageType::Notification,
            topic: Some("system.health".to_string()),
            body: serde_json::json!({
                "system": "orchestrator",
                "health": "healthy",
                "metrics": {
                    "memory_usage": "45MB",
                    "cpu_usage": "12%",
                    "uptime": "5 minutes"
                }
            }),
            headers: HashMap::from([("priority".to_string(), "low".to_string())]),
            timestamp: Utc::now(),
            correlation_id: None,
            reply_to: None,
            ttl: Some(600),
        },
    ];

    // Wait a bit to allow for potential connections
    println!("\n⏳ Waiting 2 seconds for potential WebSocket connections...");
    sleep(Duration::from_secs(2)).await;

    // Broadcast messages
    println!("\n📢 Broadcasting sample messages:");
    for (i, message) in messages.iter().enumerate() {
        let message_type_str = match message.message_type {
            MessageType::Request => "Request",
            MessageType::Response => "Response",
            MessageType::Event => "Event",
            MessageType::Command => "Command",
            MessageType::Notification => "Notification",
        };

        println!(
            "   Message {}: {} ({})",
            i + 1,
            message.topic.as_ref().unwrap_or(&"no-topic".to_string()),
            message_type_str
        );

        match websocket.broadcast(message.clone()).await {
            Ok(responses) => {
                println!(
                    "     ✅ Broadcast successful to {} connections",
                    responses.len()
                );
                if responses.is_empty() {
                    println!("     📝 Note: No active connections to receive the message");
                }
            }
            Err(e) => {
                println!("     ❌ Broadcast failed: {}", e);
            }
        }
    }

    // Show communication statistics
    sleep(Duration::from_millis(100)).await;
    println!("\n📊 Communication Statistics:");
    match websocket.get_stats().await {
        Ok(stats) => {
            println!("   - Messages sent: {}", stats.messages_sent);
            println!("   - Messages received: {}", stats.messages_received);
            println!("   - Bytes sent: {}", stats.bytes_sent);
            println!("   - Bytes received: {}", stats.bytes_received);
            println!("   - Active connections: {}", stats.active_connections);
            println!("   - Failed connections: {}", stats.failed_connections);
        }
        Err(e) => {
            println!("   ❌ Failed to get stats: {}", e);
        }
    }

    // Show orchestrator metrics
    println!("\n📈 Orchestrator Metrics:");
    let metrics = orchestrator.get_config().await;
    println!("   - Total services: {}", metrics.total_services);
    println!("   - Healthy services: {}", metrics.healthy_services);
    println!("   - Total requests: {}", metrics.total_requests);
    println!("   - Uptime: {} seconds", metrics.uptime_seconds);

    println!("\n🎯 Demo Instructions:");
    println!("   1. Connect to ws://127.0.0.1:8080 with a WebSocket client");
    println!("   2. Send JSON messages to test two-way communication");
    println!("   3. Observe real-time message handling and metrics");
    println!("   4. Press Ctrl+C to stop the demo");

    // Keep running for demo purposes
    println!("\n🔄 Demo running... Press Ctrl+C to stop");

    // Run for 30 seconds then show final stats
    sleep(Duration::from_secs(30)).await;

    // Final statistics
    println!("\n📊 Final Statistics:");
    match websocket.get_stats().await {
        Ok(stats) => {
            println!("   - Messages sent: {}", stats.messages_sent);
            println!("   - Messages received: {}", stats.messages_received);
            println!("   - Bytes sent: {}", stats.bytes_sent);
            println!("   - Bytes received: {}", stats.bytes_received);
            println!("   - Active connections: {}", stats.active_connections);
        }
        Err(e) => {
            println!("   ❌ Failed to get final stats: {}", e);
        }
    }

    // Cleanup
    println!("\n🧹 Cleaning up...");
    websocket.disconnect().await?;
    orchestrator.stop().await?;
    println!("✅ Demo completed successfully!");

    Ok(())
}
