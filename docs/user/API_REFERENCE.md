# Songbird Orchestrator API Reference

## Overview

The Songbird Orchestrator provides a comprehensive API for service orchestration, management, and monitoring. This document covers all public APIs, configuration options, and integration patterns.

## Core Orchestrator API

### `Orchestrator`

The main orchestrator class that manages service lifecycle and coordination.

```rust
use songbird_orchestrator::{Orchestrator, OrchestratorConfig};

// Create orchestrator with default configuration
let orchestrator = Orchestrator::new(OrchestratorConfig::default()).await?;

// Register a service
let service_id = orchestrator.register_service(service, config).await?;

// Start orchestrator
orchestrator.start().await?;

// List all services
let services = orchestrator.list_services().await;

// Get orchestrator metrics
let metrics = orchestrator.get_metrics().await;

// Stop orchestrator
orchestrator.stop().await?;
```

#### Methods

- `new(config: OrchestratorConfig) -> Result<Self, SongbirdError>`
- `register_service<S>(&self, service: S, config: S::Config) -> Result<String, SongbirdError>`
- `start(&self) -> Result<(), SongbirdError>`
- `stop(&self) -> Result<(), SongbirdError>`
- `list_services(&self) -> Vec<ServiceInfo>`
- `get_metrics(&self) -> OrchestratorMetrics`
- `get_service_health(&self, service_id: &str) -> Result<ServiceHealth, SongbirdError>`
- `get_service_metrics(&self, service_id: &str) -> Result<ServiceMetrics, SongbirdError>`
- `stop_service(&self, service_id: &str) -> Result<(), SongbirdError>`

## Service Management API

### `UniversalService` Trait

All services must implement the `UniversalService` trait:

```rust
use async_trait::async_trait;
use songbird_orchestrator::traits::service::*;

#[async_trait]
impl UniversalService for MyService {
    type Config = MyConfig;
    type Health = serde_json::Value;
    type Error = SongbirdError;

    async fn initialize(&mut self, config: Self::Config) -> Result<(), Self::Error>;
    async fn start(&mut self) -> Result<(), Self::Error>;
    async fn stop(&mut self) -> Result<(), Self::Error>;
    async fn health_check(&self) -> Result<Self::Health, Self::Error>;
    async fn handle_request(&self, request: ServiceRequest) -> Result<ServiceResponse, Self::Error>;
    async fn get_metrics(&self) -> Result<ServiceMetrics, Self::Error>;
    fn service_info(&self) -> ServiceInfo;
    async fn can_handle_load(&self) -> Result<bool, Self::Error>;
    async fn get_load_factor(&self) -> Result<f64, Self::Error>;
    async fn update_config(&mut self, config: Self::Config) -> Result<(), Self::Error>;
}
```

### Service Types

#### `ServiceInfo`
```rust
pub struct ServiceInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub service_type: String,
    pub description: String,
    pub endpoints: Vec<ServiceEndpoint>,
    pub capabilities: Vec<String>,
    pub tags: HashMap<String, String>,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

#### `ServiceRequest`
```rust
pub struct ServiceRequest {
    pub id: String,
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub timeout: Option<Duration>,
    pub client_info: Option<ClientInfo>,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

#### `ServiceResponse`
```rust
pub struct ServiceResponse {
    pub request_id: String,
    pub status: ResponseStatus,
    pub headers: HashMap<String, String>,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub duration: Duration,
    pub processing_time: u32,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

#### `ServiceMetrics`
```rust
pub struct ServiceMetrics {
    pub request_count: u64,
    pub error_count: u64,
    pub average_response_time: f64,
    pub active_connections: u32,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub custom_metrics: HashMap<String, serde_json::Value>,
}
```

## REST API Endpoints

### Health and System

#### `GET /health`
Basic health check endpoint.

**Response:**
```json
{
  "success": true,
  "data": "healthy",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

#### `GET /health/detailed`
Detailed health check with comprehensive status.

#### `GET /system/info`
System information including version, uptime, and service counts.

**Response:**
```json
{
  "success": true,
  "data": {
    "name": "Songbird Orchestrator",
    "version": "0.1.0",
    "uptime_seconds": 3600,
    "total_services": 5,
    "healthy_services": 4,
    "active_connections": 12,
    "total_requests": 1000,
    "api_endpoints": ["/health", "/services", "/metrics"]
  }
}
```

#### `GET /system/metrics`
Orchestrator-level metrics.

### Service Management

#### `GET /services`
List all registered services.

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "service-123",
      "name": "My Service",
      "version": "1.0.0",
      "service_type": "api",
      "description": "Example service"
    }
  ]
}
```

#### `POST /services`
Register a new service.

**Request Body:**
```json
{
  "name": "My Service",
  "service_type": "api",
  "version": "1.0.0",
  "description": "My example service",
  "endpoints": [],
  "capabilities": ["http"],
  "tags": {},
  "metadata": {}
}
```

#### `GET /services/:id`
Get specific service information.

#### `PUT /services/:id`
Update service configuration.

#### `DELETE /services/:id`
Unregister a service.

#### `POST /services/:id/start`
Start a specific service.

#### `POST /services/:id/stop`
Stop a specific service.

#### `POST /services/:id/restart`
Restart a specific service.

#### `GET /services/:id/health`
Get service health status.

**Response:**
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "uptime": "5m30s"
  }
}
```

#### `GET /services/:id/metrics`
Get service metrics.

### Communication

#### `POST /communication/send`
Send a message to a specific service.

**Request Body:**
```json
{
  "target_service": "service-123",
  "message_type": "Request",
  "topic": "process",
  "payload": {"data": "example"},
  "headers": {},
  "ttl": 30000
}
```

#### `POST /communication/broadcast`
Broadcast a message to all services.

#### `GET /communication/stats`
Get communication layer statistics.

#### `GET /communication/connections`
List active connections.

### Metrics and Monitoring

#### `GET /metrics`
Get orchestrator metrics.

**Response:**
```json
{
  "success": true,
  "data": {
    "total_services": 5,
    "healthy_services": 4,
    "total_requests": 1000,
    "uptime_seconds": 3600
  }
}
```

#### `GET /metrics/prometheus`
Prometheus-format metrics endpoint.

#### `GET /metrics/services`
Get metrics for all services.

### Real-time Streams

#### `GET /stream/events`
Server-Sent Events stream for real-time service events.

#### `GET /stream/metrics`
Server-Sent Events stream for real-time metrics updates.

### Dashboard

#### `GET /dashboard`
Complete dashboard data for monitoring interfaces.

## Load Balancing API

### `LoadBalancer` Trait

```rust
use songbird_orchestrator::load_balancer::LoadBalancer;

#[async_trait]
pub trait LoadBalancer {
    async fn select_service(&self, services: &[ServiceInstance]) -> Result<Option<ServiceInstance>, SongbirdError>;
    async fn update_service_health(&self, service_id: &str, is_healthy: bool) -> Result<(), SongbirdError>;
    async fn get_stats(&self) -> Result<LoadBalancerStats, SongbirdError>;
}
```

### `DefaultLoadBalancer`

```rust
use songbird_orchestrator::load_balancer::{DefaultLoadBalancer, LoadBalancerConfig, LoadBalancingStrategy};

// Create load balancer
let config = LoadBalancerConfig {
    strategy: LoadBalancingStrategy::RoundRobin,
    health_check_enabled: true,
    max_retries: 3,
    health_check_interval: Duration::from_secs(30),
    retry_delay: Duration::from_secs(1),
};
let lb = DefaultLoadBalancer::new(config);

// Select service
if let Some(service) = lb.select_service(&services).await? {
    // Use selected service
}
```

#### Load Balancing Strategies

- `RoundRobin` - Distribute requests evenly across services
- `LeastConnections` - Route to service with fewest active connections
- `WeightedRoundRobin` - Consider service weights in distribution
- `Random` - Random service selection
- `HealthAware` - Only select healthy services

## Communication API

### `CommunicationLayer` Trait

```rust
use songbird_orchestrator::traits::communication::CommunicationLayer;

#[async_trait]
pub trait CommunicationLayer {
    async fn send_message(&self, target: ServiceAddress, message: ServiceMessage) -> Result<CommunicationResponse>;
    async fn broadcast(&self, message: ServiceMessage) -> Result<Vec<CommunicationResponse>>;
    async fn connect(&self) -> Result<()>;
    async fn disconnect(&self) -> Result<()>;
    async fn is_connected(&self) -> bool;
    async fn get_stats(&self) -> Result<CommunicationStats>;
}
```

### `WebSocketCommunication`

```rust
use songbird_orchestrator::communication::{WebSocketCommunication, WebSocketConfig};

// Create WebSocket communication
let config = WebSocketConfig {
    max_connections: 100,
    connection_timeout: Duration::from_secs(30),
    heartbeat_interval: Duration::from_secs(15),
    message_buffer_size: 500,
};

let websocket = WebSocketCommunication::with_config("127.0.0.1".to_string(), 8080, config);

// Connect and use
websocket.connect().await?;
```

## Configuration API

### `OrchestratorConfig`

```rust
use songbird_orchestrator::OrchestratorConfig;

// Default configuration
let config = OrchestratorConfig::default();

// From file
let config = OrchestratorConfig::from_file("orchestrator.toml")?;
```

### Configuration Structure

```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4

[load_balancer]
strategy = "round_robin"
health_check_enabled = true
max_retries = 3
health_check_interval = "30s"
retry_delay = "1s"

[security]
enable_authentication = false
enable_authorization = false
rate_limit_requests_per_minute = 1000

[monitoring]
enable_metrics = true
metrics_port = 9090
enable_tracing = true
```

## Error Handling

### `SongbirdError`

All APIs use the centralized `SongbirdError` type:

```rust
use songbird_orchestrator::errors::SongbirdError;

// Handle errors
match orchestrator.register_service(service, config).await {
    Ok(service_id) => println!("Service registered: {}", service_id),
    Err(SongbirdError::ServiceError { service_id, message }) => {
        eprintln!("Service error in {}: {}", service_id, message);
    },
    Err(e) => eprintln!("Other error: {}", e),
}
```

### Error Types

- `ServiceError` - Service-specific errors
- `ConfigError` - Configuration validation errors
- `NetworkError` - Network communication errors
- `HealthCheckFailed` - Health check failures
- `Serialization` - Data serialization errors
- `LoadBalancerError` - Load balancing failures

## Examples

### Complete Service Integration

```rust
use songbird_orchestrator::{
    Orchestrator, OrchestratorConfig,
    traits::service::{UniversalService, ServiceInfo, ServiceRequest, ServiceResponse, ServiceMetrics},
    api::start_server,
    communication::WebSocketCommunication,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create orchestrator
    let orchestrator = Orchestrator::new(OrchestratorConfig::default()).await?;
    orchestrator.start().await?;
    
    // Register service
    let service = MyService::new();
    orchestrator.register_service(service, MyConfig::default()).await?;
    
    // Start API server
    let websocket = WebSocketCommunication::default();
    websocket.connect().await?;
    
    start_server(
        std::sync::Arc::new(orchestrator),
        std::sync::Arc::new(websocket),
        "127.0.0.1:3000".parse()?,
    ).await?;
    
    Ok(())
}
```

---

For complete examples, see the [examples directory](https://github.com/your-org/songbird-orchestrator/tree/main/examples) and the [Getting Started Guide](GETTING_STARTED.md).

## Federation API (Multicluster Support)

> **✅ FULLY IMPLEMENTED & WORKING** - Perfect for home HPC setups with multiple consumer towers!

### `McpFederation` - Advanced Federation

```rust
use songbird_orchestrator::federation::{McpFederation, FederationMode, FederationConfig};

// Create federation handler
let config = FederationConfig {
    cluster_endpoints: vec![
        "192.168.1.10:8080".to_string(),
        "192.168.1.11:8080".to_string(),
    ],
    heartbeat_interval: 30,
    auto_discovery: true,
    node_id: Some("hpc-tower-main".to_string()),
    cluster_id: Some("home-hpc-cluster".to_string()),
    ..Default::default()
};

let federation = McpFederation::new(FederationMode::Server, config);

// Start federation
federation.start().await?;

// Register as service provider
let provider_info = ServiceProviderInfo {
    name: "HPC Compute Node".to_string(),
    description: "High-performance computing node".to_string(),
    capabilities: vec!["compute".to_string(), "gpu".to_string()],
    endpoints: vec!["http://192.168.1.9:8080".to_string()],
    version: "1.0.0".to_string(),
    metadata: HashMap::new(),
};
federation.register_service_provider(provider_info).await?;
```

### Federation Modes

#### `FederationMode::Server`
Acts as the main coordinator (your most powerful tower):
```rust
let main_federation = McpFederation::new(FederationMode::Server, config);
```

#### `FederationMode::Client` 
Worker nodes (your consumer towers):
```rust
let worker_federation = McpFederation::new(FederationMode::Client, config);
```

#### `FederationMode::Hybrid`
Can act as both coordinator and worker:
```rust
let hybrid_federation = McpFederation::new(FederationMode::Hybrid, config);
```

### Federation Configuration

```rust
#[derive(Debug, Clone)]
pub struct FederationConfig {
    pub cluster_endpoints: Vec<String>,      // Other nodes to connect to
    pub heartbeat_interval: u64,             // Heartbeat frequency (seconds)
    pub connection_timeout: u64,             // Connection timeout (seconds)
    pub max_retries: u32,                    // Connection retry attempts
    pub auto_discovery: bool,                // Auto-discover other nodes
    pub node_id: Option<String>,             // This node's identifier
    pub cluster_id: Option<String>,          // Cluster identifier
}
```

### Federation Methods

#### Connection Management
- `start() -> Result<(), SongbirdError>` - Start federation
- `stop() -> Result<(), SongbirdError>` - Stop federation
- `is_connected() -> bool` - Check connection status
- `auto_detect() -> Result<(), SongbirdError>` - Auto-discover cluster nodes

#### Service Discovery
- `discover_federated_services() -> Result<Vec<FederatedServiceInfo>, SongbirdError>` - Find services across cluster
- `register_service_provider(ServiceProviderInfo) -> Result<(), SongbirdError>` - Register as service provider

#### Communication
- `send_heartbeat() -> Result<(), SongbirdError>` - Send cluster heartbeat
- `broadcast_message(FederationMessage) -> Result<(), SongbirdError>` - Broadcast to all nodes
- `handle_federation_request(FederationRequest) -> Result<FederationResponse, SongbirdError>` - Handle requests

### Federation Status

```rust
#[derive(Debug, Clone)]
pub struct FederationStatus {
    pub enabled: bool,                       // Federation active
    pub connected: bool,                     // Connected to cluster
    pub node_count: u32,                     // Number of nodes
    pub last_heartbeat: Option<DateTime<Utc>>, // Last heartbeat
    pub cluster_id: Option<String>,          // Cluster ID
    pub node_id: Option<String>,             // This node's ID
    pub protocol_version: String,            // Federation protocol version
}

// Get federation status
let status = federation.get_status().await;
println!("Cluster Status: {} nodes connected", status.node_count);
```

### HPC Home Cluster Example

```rust
// Main tower (Server mode)
let main_config = FederationConfig {
    cluster_endpoints: vec![
        "192.168.1.10:8080".to_string(), // Gaming rig
        "192.168.1.11:8080".to_string(), // Workstation  
        "192.168.1.12:8080".to_string(), // Old desktop
    ],
    heartbeat_interval: 15, // Fast heartbeat for HPC
    node_id: Some("hpc-main".to_string()),
    cluster_id: Some("home-hpc".to_string()),
    auto_discovery: true,
    ..Default::default()
};

let main_federation = McpFederation::new(FederationMode::Server, main_config);
main_federation.start().await?;

// Worker tower (Client mode)
let worker_config = FederationConfig {
    cluster_endpoints: vec!["192.168.1.9:8080".to_string()], // Main tower
    node_id: Some("hpc-worker-1".to_string()),
    cluster_id: Some("home-hpc".to_string()),
    ..Default::default()
};

let worker_federation = McpFederation::new(FederationMode::Client, worker_config);
worker_federation.start().await?;
```

### Federation Message Types

```rust
pub enum FederationMessageType {
    ServiceStatusUpdate,    // Service health updates
    NodeStatusUpdate,       // Node status changes
    ConfigurationChange,    // Configuration updates
    EmergencyAlert,         // Critical alerts
    LoadBalancingUpdate,    // Load balancing changes
    Announcement,           // General announcements
}
```

### Federation Request Types

```rust
pub enum FederationRequestType {
    ServiceDiscovery,       // Find services
    DataReplication,        // Replicate data
    HealthCheck,           // Health check
    ConfigUpdate,          // Update configuration
    LoadBalancing,         // Load balancing
    ResourceAllocation,    // Allocate resources
    NodeJoin,              // Node joining cluster
    NodeLeave,             // Node leaving cluster
}
```

## Working Federation Example

```bash
# Test the federation capabilities
cargo run --example federation_demo
```

This will demonstrate:
- Standalone federation mode
- Cluster federation mode  
- Multi-cluster federation mode
- Heartbeat and status monitoring

The federation system is **fully implemented and working** - perfect for your home HPC cluster of consumer towers!

---

For complete examples, see the [examples directory](https://github.com/your-org/songbird-orchestrator/tree/main/examples) and the [Getting Started Guide](GETTING_STARTED.md). 