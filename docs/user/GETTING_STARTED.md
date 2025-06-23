# Getting Started with Songbird Orchestrator

## What is Songbird Orchestrator?

Songbird Orchestrator is a **Rust library** that provides enterprise-grade service orchestration capabilities. It enables you to add service management, load balancing, health monitoring, and communication to your existing Rust applications through a simple, trait-based interface.

## Key Features

### 🚀 **Universal Service Interface**
- Works with any Rust service through the `UniversalService` trait
- No vendor lock-in or domain-specific requirements
- Pluggable architecture for different backends

### 🔧 **Enterprise-Grade Capabilities**
- **Service Management**: Registration, lifecycle, health monitoring
- **Load Balancing**: Round-robin, health-aware, least-connections algorithms
- **Health Monitoring**: Comprehensive health checks and automatic recovery
- **Communication**: WebSocket and HTTP communication layers
- **Metrics**: Prometheus-compatible metrics and monitoring
- **Configuration**: File-based and programmatic configuration

### ⚡ **Performance Optimized**
- Written in Rust for memory safety and performance
- Async-first design for modern Rust applications
- Minimal resource footprint

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
songbird-orchestrator = "0.1.0"

# Optional: Enable specific features
songbird-orchestrator = { version = "0.1.0", features = ["full"] }
```

## Quick Start Guide

### Step 1: Define Your Service

Implement the `UniversalService` trait for your service:

```rust
use songbird_orchestrator::{
    traits::service::{UniversalService, ServiceInfo, ServiceRequest, ServiceResponse, ServiceMetrics},
    errors::SongbirdError,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// Your service configuration
#[derive(Clone, Debug, Deserialize)]
pub struct MyServiceConfig {
    pub port: u16,
    pub name: String,
}

// Your service implementation
pub struct MyService {
    config: Option<MyServiceConfig>,
    is_running: bool,
}

impl MyService {
    pub fn new() -> Self {
        Self {
            config: None,
            is_running: false,
        }
    }
}

#[async_trait]
impl UniversalService for MyService {
    type Config = MyServiceConfig;
    type Health = serde_json::Value;
    type Error = SongbirdError;
    
    async fn initialize(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        self.config = Some(config);
        println!("Service initialized");
        Ok(())
    }
    
    async fn start(&mut self) -> Result<(), Self::Error> {
        self.is_running = true;
        println!("Service started: {}", self.config.as_ref().unwrap().name);
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<(), Self::Error> {
        self.is_running = false;
        println!("Service stopped");
        Ok(())
    }
    
    async fn health_check(&self) -> Result<Self::Health, Self::Error> {
        Ok(serde_json::json!({
            "status": if self.is_running { "healthy" } else { "unhealthy" },
            "uptime": "5m30s"
        }))
    }
    
    async fn handle_request(&self, request: ServiceRequest) -> Result<ServiceResponse, Self::Error> {
        Ok(ServiceResponse {
            request_id: request.id,
            status: songbird_orchestrator::traits::service::ResponseStatus::Success,
            headers: std::collections::HashMap::new(),
            payload: serde_json::json!({"message": "Hello from service!"}),
            timestamp: chrono::Utc::now(),
            duration: std::time::Duration::from_millis(10),
            processing_time: 10,
            metadata: std::collections::HashMap::new(),
        })
    }
    
    async fn get_metrics(&self) -> Result<ServiceMetrics, Self::Error> {
        Ok(ServiceMetrics {
            request_count: 100,
            error_count: 0,
            average_response_time: 15.5,
            active_connections: 5,
            cpu_usage: 25.0,
            memory_usage: 128.0,
            custom_metrics: std::collections::HashMap::new(),
        })
    }
    
    fn service_info(&self) -> ServiceInfo {
        ServiceInfo {
            id: "my-service".to_string(),
            name: "My Service".to_string(),
            version: "1.0.0".to_string(),
            service_type: "api".to_string(),
            description: "My example service".to_string(),
            endpoints: vec![],
            capabilities: vec!["http".to_string()],
            tags: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
        }
    }
    
    async fn can_handle_load(&self) -> Result<bool, Self::Error> {
        Ok(self.is_running)
    }
    
    async fn get_load_factor(&self) -> Result<f64, Self::Error> {
        Ok(if self.is_running { 0.5 } else { 1.0 })
    }
    
    async fn update_config(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        self.config = Some(config);
        Ok(())
    }
}
```

### Step 2: Create and Configure the Orchestrator

```rust
use songbird_orchestrator::{Orchestrator, OrchestratorConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create orchestrator with default configuration
    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await?;
    
    // Start the orchestrator
    orchestrator.start().await?;
    println!("✅ Orchestrator started");
    
    // Register your service
    let service = MyService::new();
    let service_config = MyServiceConfig {
        port: 8080,
        name: "MyService".to_string(),
    };
    
    let service_id = orchestrator.register_service(service, service_config).await?;
    println!("✅ Service registered with ID: {}", service_id);
    
    // Keep running
    println!("🚀 Service orchestration is running...");
    println!("   Press Ctrl+C to stop");
    
    tokio::signal::ctrl_c().await?;
    
    // Graceful shutdown
    orchestrator.stop().await?;
    println!("✅ Orchestrator stopped");
    
    Ok(())
}
```

### Step 3: Run Your Application

```bash
cargo run
```

You should see output like:
```
✅ Orchestrator started
Service initialized
Service started: MyService
✅ Service registered with ID: abc123...
🚀 Service orchestration is running...
   Press Ctrl+C to stop
```

## Next Steps

### Add REST API Access

To expose the orchestrator's REST API for external management:

```rust
use songbird_orchestrator::{
    api::start_server as start_api_server,
    communication::{WebSocketCommunication, WebSocketConfig},
};
use std::net::SocketAddr;

// Add to your main function:
let websocket = WebSocketCommunication::with_config(
    "127.0.0.1".to_string(), 
    8080, 
    WebSocketConfig::default()
);
websocket.connect().await?;

let api_addr: SocketAddr = "127.0.0.1:3000".parse()?;
start_api_server(
    std::sync::Arc::new(orchestrator.clone()),
    std::sync::Arc::new(websocket),
    api_addr,
).await?;

println!("🔗 REST API available at http://127.0.0.1:3000");
```

### Available API Endpoints

Once the API server is running, you can access:

- **Health**: `GET http://127.0.0.1:3000/health`
- **Services**: `GET http://127.0.0.1:3000/services`
- **Metrics**: `GET http://127.0.0.1:3000/metrics`
- **Dashboard**: `GET http://127.0.0.1:3000/dashboard`

### Configuration Options

Create an `orchestrator.toml` file for more advanced configuration:

```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4

[load_balancer]
strategy = "round_robin"
health_check_enabled = true
max_retries = 3

[monitoring]
enable_metrics = true
metrics_port = 9090
```

Load it in your application:

```rust
let config = OrchestratorConfig::from_file("orchestrator.toml")?;
let orchestrator = Orchestrator::new(config).await?;
```

## Working Examples

The repository includes complete working examples:

```bash
# Run the API demo
cargo run --example api_demo

# Run the WebSocket demo  
cargo run --example websocket_demo

# Run the federation demo
cargo run --example federation_demo
```

These examples demonstrate:
- Complete service integration
- REST API usage
- WebSocket communication
- Multi-service orchestration

## Common Patterns

### Multiple Services

```rust
// Register multiple services
let service1 = MyService::new();
let service2 = AnotherService::new();

orchestrator.register_service(service1, config1).await?;
orchestrator.register_service(service2, config2).await?;
```

### Health Monitoring

```rust
// Check service health
let health = orchestrator.get_service_health(&service_id).await?;
println!("Service health: {:?}", health);
```

### Load Balancing

The orchestrator automatically load balances requests across healthy service instances using configurable algorithms.

## Home HPC Federation Setup

> **✅ PERFECT FOR HOME HPC** - Connect multiple consumer towers into a unified cluster!

### Overview

Songbird's federation system is ideal for home HPC setups where you have multiple consumer towers (gaming rigs, workstations, old desktops) that you want to coordinate as a unified compute cluster.

### Step 1: Main Tower Setup (Coordinator)

Set up your most powerful machine as the cluster coordinator:

```rust
use songbird_orchestrator::federation::{McpFederation, FederationMode, FederationConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Main tower configuration
    let federation_config = FederationConfig {
        cluster_endpoints: vec![
            "192.168.1.10:8080".to_string(), // Gaming rig
            "192.168.1.11:8080".to_string(), // Workstation
            "192.168.1.12:8080".to_string(), // Old desktop
        ],
        heartbeat_interval: 15, // Fast heartbeat for HPC
        connection_timeout: 10,
        auto_discovery: true,
        node_id: Some("hpc-main".to_string()),
        cluster_id: Some("home-hpc-cluster".to_string()),
        ..Default::default()
    };

    // Create main federation coordinator
    let main_federation = McpFederation::new(FederationMode::Server, federation_config);
    
    // Start federation
    main_federation.start().await?;
    println!("✅ Main HPC coordinator started");
    
    // Monitor cluster status
    let status = main_federation.get_status().await;
    println!("🖥️  HPC Cluster Status:");
    println!("   - Cluster ID: {:?}", status.cluster_id);
    println!("   - Connected Nodes: {}", status.node_count);
    println!("   - Federation Active: {}", status.connected);
    
    // Keep running
    tokio::signal::ctrl_c().await?;
    main_federation.stop().await?;
    
    Ok(())
}
```

### Step 2: Worker Tower Setup (Consumer Towers)

Set up each of your consumer towers as worker nodes:

```rust
use songbird_orchestrator::federation::{McpFederation, FederationMode, FederationConfig};

#[tokio::main] 
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Worker tower configuration
    let worker_config = FederationConfig {
        cluster_endpoints: vec!["192.168.1.9:8080".to_string()], // Main tower IP
        heartbeat_interval: 15,
        node_id: Some("hpc-worker-gaming-rig".to_string()), // Unique ID per tower
        cluster_id: Some("home-hpc-cluster".to_string()),
        auto_discovery: true,
        ..Default::default()
    };

    // Create worker federation client
    let worker_federation = McpFederation::new(FederationMode::Client, worker_config);
    
    // Join the cluster
    worker_federation.start().await?;
    println!("✅ Worker tower joined HPC cluster");
    
    // Register capabilities
    let provider_info = ServiceProviderInfo {
        name: "Gaming Rig Compute Node".to_string(),
        description: "High-end gaming rig for compute tasks".to_string(),
        capabilities: vec![
            "compute".to_string(),
            "gpu".to_string(),
            "ml_training".to_string(),
        ],
        endpoints: vec!["http://192.168.1.10:8080".to_string()],
        version: "1.0.0".to_string(),
        metadata: {
            let mut metadata = std::collections::HashMap::new();
            metadata.insert("cpu_cores".to_string(), serde_json::json!(16));
            metadata.insert("memory_gb".to_string(), serde_json::json!(32));
            metadata.insert("gpu_model".to_string(), serde_json::json!("RTX 4080"));
            metadata
        },
    };
    
    worker_federation.register_service_provider(provider_info).await?;
    println!("🎮 Gaming rig registered as compute provider");
    
    // Keep running
    tokio::signal::ctrl_c().await?;
    worker_federation.stop().await?;
    
    Ok(())
}
```

### Step 3: Test Your HPC Cluster

Run the federation demo to verify everything works:

```bash
# Test federation capabilities
cargo run --example federation_demo
```

Expected output:
```
🚀 Songbird Orchestrator - Federation Demo
✓ Standalone federation demo completed
✓ Cluster federation demo completed  
✓ Federation mode demo completed
✅ All federation demos completed successfully!
```

### Step 4: Production HPC Configuration

Create `hpc-cluster.toml` for production deployment:

```toml
[federation]
mode = "Server"  # Main tower
cluster_id = "home-hpc-cluster"
node_id = "hpc-main"
auto_discovery = true
heartbeat_interval = 15
connection_timeout = 10
max_retries = 3

# Your consumer towers
cluster_endpoints = [
    "192.168.1.10:8080",  # Gaming rig
    "192.168.1.11:8080",  # Workstation
    "192.168.1.12:8080",  # Old desktop
    "192.168.1.13:8080",  # Laptop (optional)
]

[hpc]
# HPC-specific configuration
max_compute_nodes = 8
resource_balancing = "performance"
job_distribution = "least_loaded"
enable_gpu_scheduling = true

[monitoring]
enable_cross_node_metrics = true
federation_dashboard = true
cluster_health_checks = true
```

Load the configuration:

```rust
// Load HPC cluster configuration
let config = FederationConfig::from_file("hpc-cluster.toml")?;
let federation = McpFederation::new(FederationMode::Server, config);
```

### HPC Cluster Management

#### Monitor Cluster Status

```rust
// Get real-time cluster status
let status = federation.get_status().await;
println!("🖥️  HPC Cluster Dashboard:");
println!("   - Total Nodes: {}", status.node_count);
println!("   - Cluster Health: {}", if status.connected { "Healthy" } else { "Degraded" });
println!("   - Last Heartbeat: {:?}", status.last_heartbeat);

// Discover all compute nodes
let services = federation.discover_federated_services().await?;
for service in services {
    println!("📍 Node: {} | Service: {} | Status: {}", 
             service.node_id, service.service_name, service.health_status);
}
```

#### Distribute Compute Jobs

```rust
// Send compute job to specific node
let job_request = FederationRequest {
    request_id: "job-12345".to_string(),
    request_type: FederationRequestType::ResourceAllocation,
    target_node: Some("hpc-worker-gaming-rig".to_string()),
    data: serde_json::json!({
        "job_type": "ml_training",
        "dataset": "image_classification",
        "resources_needed": {
            "cpu_cores": 8,
            "memory_gb": 16,
            "gpu_required": true,
            "estimated_runtime": "2h"
        }
    }),
    timestamp: chrono::Utc::now(),
    source_node: Some("hpc-main".to_string()),
};

let response = federation.handle_federation_request(job_request).await?;
if response.success {
    println!("✅ Job dispatched to gaming rig");
} else {
    println!("❌ Job dispatch failed: {:?}", response.error_message);
}
```

#### Broadcast Cluster Alerts

```rust
// Broadcast maintenance alert to all towers
let maintenance_alert = FederationMessage {
    message_id: "maintenance-001".to_string(),
    message_type: FederationMessageType::Announcement,
    data: serde_json::json!({
        "type": "maintenance_window",
        "message": "Scheduled maintenance in 30 minutes",
        "affected_services": ["ml_training"],
        "estimated_downtime": "15 minutes"
    }),
    timestamp: chrono::Utc::now(),
    source_node: "hpc-main".to_string(),
};

federation.broadcast_message(maintenance_alert).await?;
println!("📢 Maintenance alert sent to all towers");
```

### Network Configuration

Ensure your home network is configured for the HPC cluster:

```bash
# Allow federation ports through firewall
sudo ufw allow 8080/tcp
sudo ufw allow 9090/tcp  # Metrics port

# Test connectivity between towers
ping 192.168.1.10  # Gaming rig
ping 192.168.1.11  # Workstation  
ping 192.168.1.12  # Old desktop

# Verify ports are accessible
telnet 192.168.1.10 8080
```

### Performance Optimization

#### For Gaming Rigs (High-End)
```toml
[hpc.gaming_rig]
priority = "high"
max_parallel_jobs = 4
gpu_acceleration = true
memory_limit_gb = 24
```

#### For Older Desktops (Lower-End)
```toml
[hpc.old_desktop]
priority = "normal"
max_parallel_jobs = 2
gpu_acceleration = false
memory_limit_gb = 8
```

### Fault Tolerance

The federation automatically handles:
- **Tower Failures**: If gaming rig goes offline, jobs redirect to workstation
- **Network Issues**: Automatic reconnection with exponential backoff
- **Split Brain**: Server mode prevents multiple coordinators
- **Graceful Degradation**: Cluster continues with reduced capacity

### Monitoring Your HPC Cluster

```rust
// Set up cluster monitoring
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    loop {
        interval.tick().await;
        
        let status = federation.get_status().await;
        let services = federation.discover_federated_services().await.unwrap_or_default();
        
        println!("🔍 HPC Cluster Health Check:");
        println!("   - Active Nodes: {}/{}", 
                 services.len(), 
                 federation_config.cluster_endpoints.len() + 1);
        
        for service in services {
            println!("   - {}: {}", service.node_id, service.health_status);
        }
    }
});
```

## Summary

**Your home HPC cluster setup:**
1. **Main Tower**: Acts as coordinator (`FederationMode::Server`)
2. **Consumer Towers**: Join as workers (`FederationMode::Client`)  
3. **Auto-Discovery**: Towers find each other automatically
4. **Load Balancing**: Jobs distributed based on capacity
5. **Fault Tolerance**: Continues working if towers go offline
6. **Real-time Monitoring**: Live status of all towers

**The federation system is fully implemented and ready for your HPC setup!** 🚀

## Troubleshooting

### Common Issues

1. **Compilation Errors**: Ensure Rust 1.70+
2. **Port Conflicts**: Check that ports 8080/3000 are available
3. **Service Registration**: Verify your service implements all required trait methods

### Getting Help

- Check the [API Reference](API_REFERENCE.md) for detailed documentation
- Review the [examples](https://github.com/your-org/songbird-orchestrator/tree/main/examples) directory
- Open an issue on [GitHub](https://github.com/your-org/songbird-orchestrator/issues)

---

**Next**: Check out the [API Reference](API_REFERENCE.md) for complete API documentation. 