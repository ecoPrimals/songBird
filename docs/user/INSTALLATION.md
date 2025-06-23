# Installation Guide

This guide covers how to integrate and use the Songbird Orchestrator library in your Rust projects.

## 🚀 Quick Start

### Prerequisites

- **Rust**: 1.70 or later
- **Operating System**: Linux, macOS, or Windows
- **Memory**: 512MB minimum, 2GB recommended for your application
- **Network**: HTTP/HTTPS connectivity for service communication

## 📦 Library Integration (Primary Use Case)

The Songbird Orchestrator is a **Rust library** that you integrate into your existing Rust projects to add service orchestration capabilities.

### Add to Your Project

Add to your `Cargo.toml`:

```toml
[dependencies]
songbird-orchestrator = "0.1.0"

# Optional: Enable specific features
songbird-orchestrator = { version = "0.1.0", features = ["full"] }
```

### Basic Integration

```rust
use songbird_orchestrator::{
    Orchestrator, 
    OrchestratorConfig,
    traits::service::UniversalService,
    ServiceInfo, ServiceRequest, ServiceResponse
};

// Your service implementation
struct MyService {
    id: String,
}

#[async_trait::async_trait]
impl UniversalService for MyService {
    type Config = ();
    type Health = bool;
    type Error = songbird_orchestrator::errors::SongbirdError;

    async fn initialize(&mut self, _config: Self::Config) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn start(&mut self) -> Result<(), Self::Error> {
        println!("Service {} starting", self.id);
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Self::Error> {
        println!("Service {} stopping", self.id);
        Ok(())
    }

    async fn health_check(&self) -> Result<Self::Health, Self::Error> {
        Ok(true)
    }

    async fn handle_request(&self, request: ServiceRequest) -> Result<ServiceResponse, Self::Error> {
        Ok(ServiceResponse::success(
            request.id,
            serde_json::json!({"service": self.id, "status": "ok"})
        ))
    }

    // ... implement other required methods
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create and configure orchestrator
    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await?;
    
    // Register your service
    let my_service = MyService { id: "my-service".to_string() };
    orchestrator.register_service(my_service, ()).await?;
    
    // Start the orchestrator
    orchestrator.start().await?;
    
    println!("Orchestrator running on http://localhost:8080");
    
    // Keep running
    tokio::signal::ctrl_c().await?;
    orchestrator.stop().await?;
    
    Ok(())
}
```

## 🔧 Development Setup

### Clone for Development

If you want to contribute or examine the source:

```bash
# Clone the repository
git clone https://github.com/your-org/songbird-orchestrator.git
cd songbird-orchestrator

# Build the library
cargo build

# Run tests
cargo test

# Run examples
cargo run --example api_demo
cargo run --example websocket_demo
```

### Available Examples

The repository includes working examples:

```bash
# API Demo - REST API orchestration
cargo run --example api_demo

# WebSocket Demo - Real-time communication
cargo run --example websocket_demo

# Federation Demo - Multi-node orchestration
cargo run --example federation_demo

# Proxy Demo - Service proxying
cargo run --example proxy_demo

# NestGate Integration - Legacy integration example
cargo run --example nestgate_integration
```

## 🐳 Docker Integration

### Using Songbird in Docker

Create a `Dockerfile` for your application that uses Songbird:

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build your application that uses songbird-orchestrator
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/your-app ./

EXPOSE 8080
CMD ["./your-app"]
```

### Example Docker Compose

```yaml
version: '3.8'
services:
  your-app:
    build: .
    ports:
      - "8080:8080"
    environment:
      - SONGBIRD_LOG_LEVEL=info
      - SONGBIRD_HOST=0.0.0.0
      - SONGBIRD_PORT=8080
    volumes:
      - ./config:/config
```

## ☸️ Kubernetes Deployment

### Deploy Your Songbird-Enabled Application

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: songbird-app
spec:
  replicas: 3
  selector:
    matchLabels:
      app: songbird-app
  template:
    metadata:
      labels:
        app: songbird-app
    spec:
      containers:
      - name: app
        image: your-org/your-songbird-app:latest
        ports:
        - containerPort: 8080
        env:
        - name: SONGBIRD_HOST
          value: "0.0.0.0"
        - name: SONGBIRD_PORT
          value: "8080"
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
---
apiVersion: v1
kind: Service
metadata:
  name: songbird-app-service
spec:
  selector:
    app: songbird-app
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer
```

### Using Helm

If you create a Helm chart for your application:

```bash
# Install your application with Songbird
helm install my-songbird-app ./your-helm-chart

# Upgrade
helm upgrade my-songbird-app ./your-helm-chart

# Uninstall
helm uninstall my-songbird-app
```

## ⚙️ Configuration

### Library Configuration

Configure the orchestrator in your code:

```rust
use songbird_orchestrator::{OrchestratorConfig, config::*};

let config = OrchestratorConfig {
    server: ServerConfig {
        host: "0.0.0.0".to_string(),
        port: 8080,
        workers: 4,
    },
    load_balancer: LoadBalancerConfig {
        strategy: LoadBalancingStrategy::RoundRobin,
        health_check_enabled: true,
        max_retries: 3,
        health_check_interval: std::time::Duration::from_secs(30),
        retry_delay: std::time::Duration::from_millis(100),
    },
    security: SecurityConfig {
        enable_authentication: false,
        enable_authorization: false,
        rate_limit_requests_per_minute: 1000,
    },
    monitoring: MonitoringConfig {
        enable_metrics: true,
        metrics_port: 9090,
        enable_tracing: true,
    },
    ..Default::default()
};

let orchestrator = Orchestrator::new(config).await?;
```

### Environment Variables

Your application can read configuration from environment variables:

```bash
export SONGBIRD_HOST=0.0.0.0
export SONGBIRD_PORT=8080
export SONGBIRD_LOG_LEVEL=debug
export SONGBIRD_WORKERS=4
```

## ✅ Verification

### Test Your Integration

```rust
#[tokio::test]
async fn test_orchestrator_integration() {
    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await.unwrap();
    
    // Test health endpoint
    let response = orchestrator.health_check().await.unwrap();
    assert_eq!(response.status, "healthy");
}
```

### API Testing

Once your application is running:

```bash
# Test health endpoint
curl http://localhost:8080/health

# List services
curl http://localhost:8080/api/v1/services

# Get orchestrator status
curl http://localhost:8080/api/v1/status
```

Expected health response:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime": "5m30s",
  "services": 3
}
```

## 🚨 Troubleshooting

### Common Integration Issues

#### Dependency Conflicts

```bash
# Update dependencies
cargo update

# Check for conflicts
cargo tree
```

#### Feature Conflicts

Make sure you're using compatible features:

```toml
[dependencies]
songbird-orchestrator = { version = "0.1.0", features = ["http-communication", "file-config"] }
```

#### Runtime Issues

```rust
// Enable logging to debug issues
use tracing_subscriber;

tracing_subscriber::fmt::init();
```

### Build Issues

```bash
# Clean build
cargo clean
cargo build

# Check compilation
cargo check --all-features

# Run tests
cargo test --lib
```

### Getting Help

- **Documentation**: [User Guide](README.md)
- **Examples**: Check the `examples/` directory
- **Issues**: [GitHub Issues](https://github.com/your-org/songbird-orchestrator/issues)
- **Community**: [Discussions](https://github.com/your-org/songbird-orchestrator/discussions)

## 🔄 Upgrading

### Update Library Version

```toml
[dependencies]
songbird-orchestrator = "0.2.0"  # Update version
```

Then:

```bash
cargo update
cargo build
```

### Migration Between Versions

Check the [Migration Guide](MIGRATION.md) for version-specific upgrade instructions.

## 🗑️ Removal

### Remove from Project

Remove from `Cargo.toml`:

```toml
[dependencies]
# songbird-orchestrator = "0.1.0"  # Comment out or remove
```

Remove imports and usage from your code, then:

```bash
cargo build  # Will fail if there are remaining references
```

### Clean Dependencies

```bash
# Remove unused dependencies
cargo clean
```

---

**Next Steps**: After integration, check out the [Getting Started Guide](GETTING_STARTED.md) to learn how to use Songbird Orchestrator effectively in your project. 