# Installation Guide

This guide covers how to install and deploy the Songbird Universal Network Orchestrator in various environments.

## 🚀 Quick Start

### Prerequisites

- **Operating System**: Linux, macOS, or Windows
- **Memory**: 1GB minimum, 4GB recommended
- **Storage**: 2GB available space
- **Network**: HTTP/HTTPS connectivity for service coordination
- **Ports**: 8080 (default), 9090 (metrics), configurable

## 📦 Installation Methods

### Option 1: Binary Installation (Recommended)

Download and install the latest release:

```bash
# Linux x64
curl -L https://github.com/songbird/orchestrator/releases/latest/download/songbird-linux-x64.tar.gz | tar -xz
sudo mv songbird /usr/local/bin/
sudo chmod +x /usr/local/bin/songbird

# macOS
curl -L https://github.com/songbird/orchestrator/releases/latest/download/songbird-macos-x64.tar.gz | tar -xz
sudo mv songbird /usr/local/bin/
sudo chmod +x /usr/local/bin/songbird

# Windows
# Download songbird-windows-x64.zip from releases page
# Extract and add to PATH

# Verify installation
songbird --version
```

### Option 2: Package Managers

```bash
# Ubuntu/Debian
wget -qO - https://packages.songbird.dev/gpg | sudo apt-key add -
echo "deb https://packages.songbird.dev/ubuntu $(lsb_release -cs) main" | sudo tee /etc/apt/sources.list.d/songbird.list
sudo apt update && sudo apt install songbird

# Red Hat/CentOS
sudo rpm --import https://packages.songbird.dev/gpg
sudo yum-config-manager --add-repo https://packages.songbird.dev/rhel/songbird.repo
sudo yum install songbird

# macOS with Homebrew
brew install songbird/tap/songbird

# Windows with Chocolatey
choco install songbird
```

### Option 3: Build from Source

```bash
# Prerequisites: Rust 1.70+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Clone and build
git clone https://github.com/songbird/orchestrator.git
cd orchestrator
cargo build --release

# Install locally
sudo cp target/release/songbird /usr/local/bin/
sudo chmod +x /usr/local/bin/songbird
```

### Option 4: Container Deployment

```bash
# Docker
docker run -d \
  --name songbird \
  -p 8080:8080 \
  -p 9090:9090 \
  -v /path/to/config:/config \
  -v /path/to/data:/data \
  songbird/orchestrator:latest

# Docker Compose
curl -o docker-compose.yml https://raw.githubusercontent.com/songbird/orchestrator/main/docker-compose.yml
docker-compose up -d
```

## 🔧 Configuration

### Basic Configuration

Create a configuration file:

```bash
# Create config directory
sudo mkdir -p /etc/songbird
sudo chown $USER:$USER /etc/songbird

# Create basic config
cat > /etc/songbird/config.toml << EOF
[server]
host = "0.0.0.0"
port = 8080
workers = 4

[orchestrator]
name = "songbird-orchestrator"
max_services = 1000
auto_discovery = true

[communication]
protocols = ["http", "websocket"]
max_connections = 1000

[monitoring]
metrics_enabled = true
metrics_port = 9090
EOF
```

### Advanced Configuration

```toml
# /etc/songbird/config.toml
[server]
host = "0.0.0.0"
port = 8080
workers = 8
tls_enabled = true
tls_cert = "/etc/songbird/cert.pem"
tls_key = "/etc/songbird/key.pem"

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
discovery_port = 47624
low_latency_mode = true

[security]
authentication_enabled = true
authorization_enabled = true
rate_limit_per_minute = 10000

[load_balancer]
default_algorithm = "health-aware"
health_required = true
max_retries = 3

[monitoring]
metrics_enabled = true
metrics_port = 9090
tracing_enabled = true
log_level = "info"
```

## 🐳 Container Deployment

### Docker

```dockerfile
# Dockerfile
FROM songbird/orchestrator:latest

# Copy configuration
COPY config.toml /etc/songbird/config.toml

# Copy service manifests
COPY manifests/ /etc/songbird/manifests/

# Expose ports
EXPOSE 8080 9090

# Start orchestrator
CMD ["songbird", "orchestrator", "start", "--config", "/etc/songbird/config.toml"]
```

### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'
services:
  songbird:
    image: songbird/orchestrator:latest
    ports:
      - "8080:8080"
      - "9090:9090"
    volumes:
      - ./config:/etc/songbird
      - ./data:/data
      - ./manifests:/manifests
    environment:
      - SONGBIRD_LOG_LEVEL=info
      - SONGBIRD_WORKERS=4
    restart: unless-stopped
    
  # Optional: Prometheus for metrics
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9091:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
    
  # Optional: Grafana for dashboards
  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    volumes:
      - grafana-storage:/var/lib/grafana
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin

volumes:
  grafana-storage:
```

## ☸️ Kubernetes Deployment

### Basic Deployment

```yaml
# songbird-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: songbird-orchestrator
  namespace: default
spec:
  replicas: 3
  selector:
    matchLabels:
      app: songbird-orchestrator
  template:
    metadata:
      labels:
        app: songbird-orchestrator
    spec:
      containers:
      - name: songbird
        image: songbird/orchestrator:latest
        ports:
        - containerPort: 8080
        - containerPort: 9090
        env:
        - name: SONGBIRD_LOG_LEVEL
          value: "info"
        - name: SONGBIRD_WORKERS
          value: "4"
        volumeMounts:
        - name: config
          mountPath: /etc/songbird
        - name: data
          mountPath: /data
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: config
        configMap:
          name: songbird-config
      - name: data
        persistentVolumeClaim:
          claimName: songbird-data

---
apiVersion: v1
kind: Service
metadata:
  name: songbird-service
spec:
  selector:
    app: songbird-orchestrator
  ports:
  - name: http
    port: 8080
    targetPort: 8080
  - name: metrics
    port: 9090
    targetPort: 9090
  type: LoadBalancer

---
apiVersion: v1
kind: ConfigMap
metadata:
  name: songbird-config
data:
  config.toml: |
    [server]
    host = "0.0.0.0"
    port = 8080
    workers = 4
    
    [orchestrator]
    name = "songbird-k8s"
    max_services = 1000
    auto_discovery = true
    
    [monitoring]
    metrics_enabled = true
    metrics_port = 9090

---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: songbird-data
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
```

Deploy to Kubernetes:

```bash
kubectl apply -f songbird-deployment.yaml
kubectl get pods -l app=songbird-orchestrator
kubectl get service songbird-service
```

### Helm Chart

```bash
# Add Songbird Helm repository
helm repo add songbird https://charts.songbird.dev
helm repo update

# Install with default values
helm install songbird songbird/orchestrator

# Install with custom values
helm install songbird songbird/orchestrator -f values.yaml
```

## 🔒 Security Configuration

### TLS/SSL Setup

```bash
# Generate self-signed certificate (development only)
openssl req -x509 -newkey rsa:4096 -keyout /etc/songbird/key.pem -out /etc/songbird/cert.pem -days 365 -nodes

# Update config for TLS
cat >> /etc/songbird/config.toml << EOF
[server]
tls_enabled = true
tls_cert = "/etc/songbird/cert.pem"
tls_key = "/etc/songbird/key.pem"
EOF
```

### Authentication

```bash
# Create admin user
songbird auth create-user --username admin --password your-secure-password --role admin

# Create service account
songbird auth create-service-account --name api-client --permissions service:read,service:write

# Generate API key
songbird auth generate-api-key --service-account api-client
```

## 🖥️ Service Management

### Systemd Service (Linux)

```ini
# /etc/systemd/system/songbird.service
[Unit]
Description=Songbird Universal Network Orchestrator
After=network.target

[Service]
Type=simple
User=songbird
Group=songbird
WorkingDirectory=/opt/songbird
ExecStart=/usr/local/bin/songbird orchestrator start --config /etc/songbird/config.toml
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# Security settings
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/songbird /var/log/songbird

[Install]
WantedBy=multi-user.target
```

```bash
# Create user and directories
sudo useradd --system --no-create-home --shell /bin/false songbird
sudo mkdir -p /opt/songbird /var/log/songbird
sudo chown songbird:songbird /opt/songbird /var/log/songbird

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable songbird
sudo systemctl start songbird

# Check status
sudo systemctl status songbird
sudo journalctl -u songbird -f
```

### Windows Service

```powershell
# Install as Windows Service
songbird service install --config C:\songbird\config.toml

# Start service
songbird service start

# Check status
songbird service status
```

## 📊 Monitoring Setup

### Prometheus Integration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'songbird'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: /metrics
    scrape_interval: 15s
```

### Grafana Dashboard

```bash
# Import Songbird dashboard
# Dashboard ID: 12345 (from grafana.com)
```

## 🎯 Verification

### Health Checks

```bash
# Basic health check
curl http://localhost:8080/health

# Detailed health check
curl http://localhost:8080/health/detailed

# System metrics
curl http://localhost:8080/system/metrics
```

### CLI Verification

```bash
# Check orchestrator status
songbird orchestrator status

# List services
songbird services list

# Check primals
songbird primals list

# Test gaming bridge
songbird gaming test
```

## 🔄 Upgrades

### Binary Upgrade

```bash
# Download new version
curl -L https://github.com/songbird/orchestrator/releases/latest/download/songbird-linux-x64.tar.gz | tar -xz

# Backup current version
sudo cp /usr/local/bin/songbird /usr/local/bin/songbird.backup

# Install new version
sudo mv songbird /usr/local/bin/
sudo chmod +x /usr/local/bin/songbird

# Restart service
sudo systemctl restart songbird
```

### Container Upgrade

```bash
# Update image
docker pull songbird/orchestrator:latest

# Restart container
docker-compose down
docker-compose up -d
```

## 🛠️ Troubleshooting

### Common Issues

1. **Port conflicts**: Check if ports 8080/9090 are available
2. **Permission issues**: Ensure proper user permissions
3. **Configuration errors**: Validate TOML syntax
4. **Network connectivity**: Check firewall rules

### Debug Mode

```bash
# Start with debug logging
songbird orchestrator start --config /etc/songbird/config.toml --log-level debug

# Check logs
tail -f /var/log/songbird/orchestrator.log
```

### Recovery

```bash
# Reset to factory defaults
songbird orchestrator reset --confirm

# Restore from backup
songbird orchestrator restore --backup-file /path/to/backup.tar.gz
```

## 📚 Next Steps

1. **Configure Services**: Create BYOB manifests and deploy your first service
2. **Set up Primals**: Configure Toadstool, NestGate, BearDog, and Squirrel endpoints
3. **Enable Gaming**: Configure gaming bridge for LAN gaming support
4. **Monitor**: Set up Prometheus and Grafana for monitoring
5. **Scale**: Deploy additional orchestrator instances for high availability

For detailed configuration and usage, see:
- [Getting Started Guide](GETTING_STARTED.md)
- [API Reference](API_REFERENCE.md)
- [Production Guide](PRODUCTION_GUIDE.md)
- [Architecture Guide](ARCHITECTURE.md) 