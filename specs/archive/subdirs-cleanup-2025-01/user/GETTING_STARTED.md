# Getting Started with Songbird Universal Network Orchestrator

## What is Songbird?

Songbird is a **universal network orchestration platform** that provides enterprise-grade service orchestration, coordination, and management capabilities. It enables you to deploy, manage, and coordinate any type of service or application through a unified platform with auto-discovery, load balancing, and universal Primal coordination.

## Key Features

### 🌐 **Universal Orchestration Platform**
- **BYOB (Bring Your Own Biome)**: Deploy any service using YAML manifests
- **Universal Primal Coordination**: Native support for Toadstool, NestGate, BearDog, Squirrel, and future Primals
- **Gaming Bridge**: Specialized support for legacy LAN gaming and modern gaming infrastructure
- **Auto-Discovery**: Services automatically discover and coordinate with each other
- **Zero-Touch Deployment**: Minimal configuration required for most use cases

### 🚀 **Production-Ready Performance**
- **High Performance**: 2.5M+ HashMap operations/sec, <1ms coordination latency
- **Scalability**: Handles thousands of concurrent services and connections
- **Reliability**: Comprehensive health monitoring and automatic recovery
- **Security**: Built-in authentication, authorization, and audit trails

### 🔧 **Enterprise-Grade Capabilities**
- **Service Management**: Automatic registration, lifecycle management, health monitoring
- **Load Balancing**: Multiple algorithms (round-robin, health-aware, least-connections)
- **Communication**: WebSocket and REST APIs for service coordination
- **Monitoring**: Comprehensive metrics and observability
- **Configuration**: Environment-based configuration with dynamic updates

## Installation

### Option 1: Binary Installation (Recommended)

```bash
# Download latest release
curl -L https://github.com/your-org/songbird/releases/latest/download/songbird-linux-x64.tar.gz | tar -xz

# Install to system path
sudo mv songbird /usr/local/bin/
sudo chmod +x /usr/local/bin/songbird

# Verify installation
songbird --version
```

### Option 2: Build from Source

```bash
# Prerequisites: Rust 1.70+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/your-org/songbird.git
cd songbird
cargo build --release

# Install locally
cargo install --path .
```

### Option 3: Container Deployment

```bash
# Run with Docker
docker run -d \
  --name songbird \
  -p 8080:8080 \
  -v /path/to/config:/config \
  songbird/orchestrator:latest

# Or with Docker Compose
curl -o docker-compose.yml https://raw.githubusercontent.com/your-org/songbird/main/docker-compose.yml
docker-compose up -d
```

## Quick Start Guide

### Step 1: Start the Orchestrator

```bash
# Start with default configuration
songbird orchestrator start

# Or with custom configuration
songbird orchestrator start --config /path/to/config.toml
```

The orchestrator will start on `http://localhost:8080` by default.

### Step 2: Deploy Your First Service with BYOB

Create a BYOB manifest file (`my-service.yaml`):

```yaml
# my-service.yaml - BYOB (Bring Your Own Biome) Manifest
apiVersion: v1
kind: Service
metadata:
  name: my-web-service
  description: "My example web service"
  version: "1.0.0"
  
spec:
  # Service configuration
  type: web-api
  port: 3000
  healthCheck:
    path: /health
    interval: 30s
    timeout: 10s
    
  # Deployment configuration
  deployment:
    replicas: 2
    resources:
      cpu: "500m"
      memory: "512Mi"
    
  # Service discovery configuration
  discovery:
    tags:
      - "api"
      - "web"
      - "production"
    capabilities:
      - "http"
      - "websocket"
      
  # Load balancing configuration
  loadBalancer:
    algorithm: "health-aware"
    healthRequired: true
    
  # Communication configuration
  communication:
    protocols:
      - "http"
      - "websocket"
    endpoints:
      - path: "/api/v1"
        methods: ["GET", "POST"]
      - path: "/ws"
        protocol: "websocket"
```

Deploy the service:

```bash
# Deploy using BYOB manifest
songbird deploy my-service.yaml

# Check deployment status
songbird services list
songbird services status my-web-service
```

### Step 3: Primal Coordination Integration

Songbird supports universal Primal coordination. Configure your Primal endpoints:

```yaml
# primal-config.yaml
primals:
  toadstool:
    endpoint: "https://toadstool.example.com"
    apiKey: "your-toadstool-api-key"
    capabilities: ["data-processing", "analytics"]
    
  nestgate:
    endpoint: "https://nestgate.example.com"
    apiKey: "your-nestgate-api-key"
    capabilities: ["communication", "messaging"]
    
  beardog:
    endpoint: "https://beardog.example.com"
    apiKey: "your-beardog-api-key"
    capabilities: ["validation", "security"]
    
  squirrel:
    endpoint: "https://squirrel.example.com"
    apiKey: "your-squirrel-api-key"
    capabilities: ["storage", "caching"]
```

Apply the Primal configuration:

```bash
# Configure Primal endpoints
songbird primal configure primal-config.yaml

# Test Primal connectivity
songbird primal test-all

# List available Primals
songbird primal list
```

### Step 4: Gaming Bridge Configuration (Optional)

If you need gaming infrastructure support:

```yaml
# gaming-config.yaml
gaming:
  enabled: true
  
  # Legacy LAN gaming support
  lan_bridge:
    enabled: true
    discovery_port: 47624
    supported_protocols:
      - "IPX"
      - "TCP"
      - "UDP"
    
  # Modern gaming infrastructure
  matchmaking:
    enabled: true
    region: "us-east-1"
    max_players_per_session: 32
    
  # Gaming-specific networking
  networking:
    low_latency_mode: true
    packet_prioritization: true
    nat_traversal: true
```

Enable gaming bridge:

```bash
# Configure gaming bridge
songbird gaming configure gaming-config.yaml

# Start gaming discovery
songbird gaming discovery start

# List gaming sessions
songbird gaming sessions list
```

### Step 5: Monitor and Manage

Use the Web UI or CLI to monitor your services:

```bash
# Web UI (default: http://localhost:8080)
open http://localhost:8080

# CLI monitoring
songbird services list
songbird services metrics
songbird services health

# Real-time monitoring
songbird services watch
songbird logs tail --service my-web-service
```

## REST API Usage

Songbird provides comprehensive REST APIs for programmatic integration:

### Service Management API

```bash
# List all services
curl -X GET http://localhost:8080/api/v1/services

# Get specific service
curl -X GET http://localhost:8080/api/v1/services/my-web-service

# Deploy service via API
curl -X POST http://localhost:8080/api/v1/services \
  -H "Content-Type: application/json" \
  -d @service-manifest.json

# Update service configuration
curl -X PUT http://localhost:8080/api/v1/services/my-web-service \
  -H "Content-Type: application/json" \
  -d '{"replicas": 3}'
```

### Health and Monitoring API

```bash
# System health
curl -X GET http://localhost:8080/api/v1/health

# Service health
curl -X GET http://localhost:8080/api/v1/services/my-web-service/health

# Metrics
curl -X GET http://localhost:8080/api/v1/metrics
curl -X GET http://localhost:8080/api/v1/services/my-web-service/metrics
```

### Primal Coordination API

```bash
# List available Primals
curl -X GET http://localhost:8080/api/v1/primals

# Route request to specific Primal
curl -X POST http://localhost:8080/api/v1/primals/toadstool/process \
  -H "Content-Type: application/json" \
  -d '{"data": "process-this"}'

# Coordinate across multiple Primals
curl -X POST http://localhost:8080/api/v1/primals/coordinate \
  -H "Content-Type: application/json" \
  -d '{"primals": ["toadstool", "nestgate"], "operation": "sync"}'
```

## WebSocket API Usage

For real-time communication:

```javascript
// Connect to WebSocket API
const ws = new WebSocket('ws://localhost:8080/ws');

// Subscribe to service events
ws.send(JSON.stringify({
  type: 'subscribe',
  channel: 'services',
  filters: { service: 'my-web-service' }
}));

// Listen for events
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Service event:', data);
};

// Send coordination request
ws.send(JSON.stringify({
  type: 'coordinate',
  primal: 'toadstool',
  operation: 'process',
  data: { input: 'data-to-process' }
}));
```

## Configuration Examples

### Basic Configuration

```toml
# config/songbird.toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4

[orchestrator]
name = "songbird-orchestrator"
max_services = 1000
auto_discovery = true
health_check_interval = "30s"

[communication]
protocols = ["http", "websocket"]
max_connections = 10000
heartbeat_interval = "30s"

[load_balancer]
default_algorithm = "health-aware"
health_required = true
max_retries = 3
```

### Advanced Configuration

```toml
# config/songbird-advanced.toml
[server]
host = "0.0.0.0"
port = 8080
workers = 8
tls_enabled = true
tls_cert = "/path/to/cert.pem"
tls_key = "/path/to/key.pem"

[orchestrator]
name = "songbird-production"
max_services = 10000
auto_discovery = true
health_check_interval = "15s"
service_timeout = "60s"

[primals]
coordination_enabled = true
discovery_timeout = "30s"
max_concurrent_requests = 1000

[gaming]
enabled = true
low_latency_mode = true
discovery_port = 47624

[security]
authentication_enabled = true
authorization_enabled = true
audit_enabled = true
rate_limit_per_minute = 10000

[monitoring]
metrics_enabled = true
metrics_port = 9090
tracing_enabled = true
log_level = "info"
```

## Next Steps

1. **Read the [Architecture Guide](ARCHITECTURE.md)** to understand the platform design
2. **Check the [API Reference](API_REFERENCE.md)** for detailed API documentation
3. **Review the [Production Guide](PRODUCTION_GUIDE.md)** for deployment best practices
4. **Explore the [Installation Guide](INSTALLATION.md)** for advanced installation options

## Support

- **Documentation**: [https://docs.songbird.dev](https://docs.songbird.dev)
- **GitHub Issues**: [https://github.com/your-org/songbird/issues](https://github.com/your-org/songbird/issues)
- **Community**: [https://discord.gg/songbird](https://discord.gg/songbird)
- **Support**: support@songbird.dev

## Examples

Explore working examples in the `examples/` directory:

```bash
# BYOB deployment examples
songbird deploy examples/web-api-service.yaml
songbird deploy examples/microservice-cluster.yaml

# Primal coordination examples
songbird primal demo --type toadstool
songbird primal demo --type multi-primal

# Gaming bridge examples
songbird gaming demo --type lan-bridge
songbird gaming demo --type matchmaking
``` 