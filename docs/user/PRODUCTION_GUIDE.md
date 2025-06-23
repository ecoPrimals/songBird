# Production Deployment Guide

## Overview

This guide covers best practices for deploying applications that use the Songbird Orchestrator library in production environments.

## Prerequisites

### System Requirements

#### Minimum Requirements
- **CPU**: 2 cores
- **Memory**: 2GB RAM  
- **Storage**: 10GB available space
- **Network**: Reliable internet connection
- **OS**: Linux, macOS, or Windows

#### Recommended Production Requirements
- **CPU**: 4+ cores
- **Memory**: 8GB+ RAM
- **Storage**: 50GB+ SSD
- **Network**: High-speed connection with redundancy
- **OS**: Linux (Ubuntu 20.04+, RHEL 8+, or equivalent)

### Software Dependencies

```bash
# Ubuntu/Debian
sudo apt update && sudo apt install -y \
    curl wget git build-essential \
    pkg-config libssl-dev

# RHEL/CentOS  
sudo yum update && sudo yum install -y \
    curl wget git gcc \
    openssl-devel pkgconfig
```

## Application Setup

### 1. Rust Installation

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 2. Build Your Application

```bash
# Clone your application repository
git clone https://github.com/your-org/your-app.git
cd your-app

# Build for production
cargo build --release

# Run tests
cargo test

# Verify the binary
./target/release/your-app --version
```

### 3. Configuration

Create a production configuration file:

```toml
# config/production.toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4

[orchestrator]
max_services = 100
health_check_interval = "30s"
service_timeout = "60s"

[load_balancer]  
strategy = "health_aware"
health_check_enabled = true
max_retries = 3
retry_delay = "1s"

[security]
enable_authentication = true
enable_authorization = true
rate_limit_requests_per_minute = 1000

[monitoring]
enable_metrics = true
metrics_port = 9090
enable_tracing = true
log_level = "info"

[communication]
protocol = "websocket"
max_connections = 1000
heartbeat_interval = "30s"
message_buffer_size = 1000
```

Load the configuration in your application:

```rust
use songbird_orchestrator::OrchestratorConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load production configuration
    let config = OrchestratorConfig::from_file("config/production.toml")?;
    
    // Create orchestrator with production config
    let orchestrator = Orchestrator::new(config).await?;
    
    // ... rest of your application setup
    
    Ok(())
}
```

## Deployment Options

### Option 1: Direct Binary Deployment

```bash
# Copy binary to production server
scp target/release/your-app user@production-server:/opt/your-app/
scp config/production.toml user@production-server:/opt/your-app/config/

# On production server
chmod +x /opt/your-app/your-app

# Test run
cd /opt/your-app
./your-app --config config/production.toml
```

### Option 2: Systemd Service (Linux)

Create a systemd service file:

```ini
# /etc/systemd/system/your-app.service
[Unit]
Description=Your Application with Songbird Orchestrator
After=network.target

[Service]
Type=simple
User=your-app
WorkingDirectory=/opt/your-app
ExecStart=/opt/your-app/your-app --config config/production.toml
Restart=always
RestartSec=10
StandardOutput=syslog
StandardError=syslog
SyslogIdentifier=your-app

# Security settings
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/your-app/logs

[Install]
WantedBy=multi-user.target
```

Enable and start the service:

```bash
# Create user and directories
sudo useradd --system --no-create-home your-app
sudo mkdir -p /opt/your-app/logs
sudo chown -R your-app:your-app /opt/your-app

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable your-app
sudo systemctl start your-app

# Check status
sudo systemctl status your-app
sudo journalctl -u your-app -f
```

### Option 3: Container Deployment

Create a Dockerfile:

```dockerfile
# Dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/your-app /app/
COPY config/production.toml /app/config/

EXPOSE 8080 9090
USER 1000:1000

CMD ["./your-app", "--config", "config/production.toml"]
```

Build and run:

```bash
# Build image
docker build -t your-app:latest .

# Run container
docker run -d \
    --name your-app \
    -p 8080:8080 \
    -p 9090:9090 \
    -v /opt/your-app/logs:/app/logs \
    your-app:latest

# Check logs
docker logs -f your-app
```

### Option 4: Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  your-app:
    build: .
    ports:
      - "8080:8080"
      - "9090:9090"
    volumes:
      - ./logs:/app/logs
      - ./config:/app/config
    environment:
      - RUST_LOG=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9091:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
    restart: unless-stopped
```

## Monitoring & Observability

### Metrics Collection

The orchestrator provides Prometheus-compatible metrics:

```yaml
# monitoring/prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'your-app'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'
    scrape_interval: 10s
```

### Health Monitoring

Set up health check endpoints:

```rust
// In your application
use songbird_orchestrator::api::start_server;

// Start the API server for health checks
let api_addr = "0.0.0.0:8080".parse()?;
start_server(
    Arc::new(orchestrator),
    Arc::new(websocket),
    api_addr,
).await?;
```

Health check URLs:
- Basic health: `GET http://your-app:8080/health`
- Detailed health: `GET http://your-app:8080/health/detailed`
- System info: `GET http://your-app:8080/system/info`
- Metrics: `GET http://your-app:9090/metrics`

### Logging Configuration

```rust
// Set up structured logging
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    
    tracing::info!("Starting application");
    
    // ... rest of application
    
    Ok(())
}
```

## Security Considerations

### Basic Security

1. **Run as non-root user**
2. **Use TLS for external connections**
3. **Validate all input data**
4. **Implement rate limiting**
5. **Regular security updates**

### TLS Configuration

```rust
// Example TLS setup (if needed)
use rustls::{Certificate, PrivateKey, ServerConfig};
use std::fs::File;

async fn setup_tls() -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let cert_file = File::open("cert.pem")?;
    let key_file = File::open("key.pem")?;
    
    let cert_chain = rustls_pemfile::certs(cert_file)?
        .into_iter()
        .map(Certificate)
        .collect();
    
    let mut keys = rustls_pemfile::pkcs8_private_keys(key_file)?
        .into_iter()
        .map(PrivateKey)
        .collect::<Vec<_>>();
    
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, keys.pop().unwrap())?;
    
    Ok(config)
}
```

## Performance Optimization

### Configuration Tuning

```toml
# Optimized configuration
[server]
workers = 8  # Match CPU cores
max_concurrent_requests = 10000
keep_alive_timeout = "30s"

[load_balancer]
strategy = "least_connections"  # Better for uneven load
health_check_interval = "15s"   # More frequent health checks
max_retries = 2                 # Faster failover

[communication]  
max_connections = 5000
heartbeat_interval = "20s"
message_buffer_size = 2000      # Larger buffer for high throughput
```

### Resource Monitoring

Monitor these key metrics:
- CPU usage per service
- Memory consumption
- Request latency
- Error rates
- Active connections
- Health check status

## Troubleshooting

### Common Issues

1. **High Memory Usage**
   - Monitor service metrics
   - Check for memory leaks
   - Tune buffer sizes

2. **Connection Issues**
   - Verify network connectivity
   - Check firewall settings
   - Review connection limits

3. **Service Health Failures**
   - Check service logs
   - Verify health check endpoints
   - Review timeout settings

4. **Performance Degradation**
   - Monitor system resources
   - Check database connections
   - Review load balancer configuration

### Diagnostic Commands

```bash
# Check application status
sudo systemctl status your-app

# View logs
sudo journalctl -u your-app -f

# Check resource usage
htop
df -h
netstat -tulpn

# Test endpoints
curl http://localhost:8080/health
curl http://localhost:9090/metrics

# Container diagnostics (if using Docker)
docker stats your-app
docker logs -f your-app
```

## Backup and Recovery

### Application Data

```bash
# Backup configuration
tar -czf config-backup-$(date +%Y%m%d).tar.gz config/

# Backup logs (if needed)
tar -czf logs-backup-$(date +%Y%m%d).tar.gz logs/

# Backup application state (if applicable)
# This depends on your specific application requirements
```

### Recovery Process

1. **Stop the application**
2. **Restore configuration files**
3. **Restore application binary**
4. **Start the application**
5. **Verify health checks**

## Scaling Considerations

### Horizontal Scaling

For multiple instances:

```rust
// Configure each instance with unique identifiers
let config = OrchestratorConfig {
    instance_id: Some("instance-1".to_string()),
    // ... other configuration
};
```

### Load Balancing Multiple Instances

Use a reverse proxy (e.g., nginx, HAProxy):

```nginx
# nginx configuration
upstream your_app {
    server 10.0.1.10:8080;
    server 10.0.1.11:8080;
    server 10.0.1.12:8080;
}

server {
    listen 80;
    server_name your-app.example.com;
    
    location / {
        proxy_pass http://your_app;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
    
    location /health {
        proxy_pass http://your_app;
    }
}
```

## Best Practices

1. **Always use configuration files for production**
2. **Implement comprehensive logging**  
3. **Monitor all key metrics**
4. **Set up automated health checks**
5. **Use proper error handling**
6. **Regular backups of configuration**
7. **Test deployment process in staging**
8. **Keep dependencies updated**
9. **Use resource limits appropriately**
10. **Plan for graceful shutdowns**

---

This guide provides a foundation for production deployment. Adapt these patterns to your specific application requirements and infrastructure constraints. 