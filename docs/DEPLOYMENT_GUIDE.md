# 🚀 Songbird Universal Orchestrator - Production Deployment Guide

**Version**: 0.1.0  
**Last Updated**: 2025-01-18  
**Status**: Production Ready ✅

---

## 🎯 **Overview**

This guide provides comprehensive instructions for deploying the Songbird Universal Orchestrator in production environments, including Docker, Kubernetes, and bare metal deployments.

## 📋 **Table of Contents**

1. [Prerequisites](#prerequisites)
2. [Environment Configuration](#environment-configuration)
3. [Docker Deployment](#docker-deployment)
4. [Kubernetes Deployment](#kubernetes-deployment)
5. [Bare Metal Deployment](#bare-metal-deployment)
6. [Federation Setup](#federation-setup)
7. [Monitoring & Observability](#monitoring--observability)
8. [Security Configuration](#security-configuration)
9. [Performance Tuning](#performance-tuning)
10. [Troubleshooting](#troubleshooting)

---

## ✅ **Prerequisites**

### System Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| **CPU** | 2 cores | 4+ cores |
| **Memory** | 4GB RAM | 8GB+ RAM |
| **Storage** | 20GB | 50GB+ SSD |
| **Network** | 1Gbps | 10Gbps |

### Software Dependencies

- **Rust**: 1.70+ (for building from source)
- **Docker**: 20.10+ (for containerized deployment)
- **Kubernetes**: 1.25+ (for K8s deployment)
- **Linux Kernel**: 5.4+ (for optimal performance)

### Network Requirements

| Port | Protocol | Purpose | Required |
|------|----------|---------|----------|
| 8080 | HTTP/TCP | Orchestrator API | ✅ |
| 8081 | HTTP/TCP | Gaming Bridge | Optional |
| 8082 | HTTP/TCP | Federation | ✅ |
| 8443 | HTTPS/TCP | Security (BearDog) | ✅ |
| 9090 | HTTP/TCP | Metrics | Optional |

---

## ⚙️ **Environment Configuration**

### Production Environment Variables

Create a `.env` file for production configuration:

```bash
# Core Configuration
SONGBIRD_ENV=production
SONGBIRD_BIND_ADDRESS=0.0.0.0
SONGBIRD_ORCHESTRATOR_PORT=8080
SONGBIRD_FEDERATION_PORT=8082

# Federation Configuration
SONGBIRD_FEDERATION_NODE_ID=prod-node-1
SONGBIRD_HEARTBEAT_INTERVAL=30
SONGBIRD_DISCOVERY_PORTS=8080,8081,8082,8443,9090
SONGBIRD_MAX_NODES=1000

# Performance Configuration
SONGBIRD_LARGE_BUFFER_SIZE=16384
SONGBIRD_CONNECTION_TIMEOUT=30
SONGBIRD_REQUEST_TIMEOUT=60
SONGBIRD_HEALTH_CHECK_TIMEOUT=5

# Security Configuration
BEARDOG_ENDPOINT=https://beardog.internal:8443
NESTGATE_ENDPOINT=http://nestgate.internal:8082
TOADSTOOL_ENDPOINT=http://toadstool.internal:8081
SQUIRREL_ENDPOINT=http://squirrel.internal:8083

# Observability
RUST_LOG=info,songbird_core=debug,songbird_federation=debug
SONGBIRD_METRICS_ENABLED=true
SONGBIRD_METRICS_PORT=9090

# Database (if applicable)
DATABASE_URL=postgresql://songbird:password@postgres:5432/songbird_prod

# TLS Configuration
TLS_CERT_PATH=/etc/ssl/certs/songbird.crt
TLS_KEY_PATH=/etc/ssl/private/songbird.key
```

### Environment-Specific Configurations

#### Development
```bash
SONGBIRD_ENV=development
SONGBIRD_BIND_ADDRESS=127.0.0.1
RUST_LOG=debug
SONGBIRD_MAX_NODES=10
```

#### Staging
```bash
SONGBIRD_ENV=staging
SONGBIRD_BIND_ADDRESS=0.0.0.0
RUST_LOG=info
SONGBIRD_MAX_NODES=100
```

#### Production
```bash
SONGBIRD_ENV=production
SONGBIRD_BIND_ADDRESS=0.0.0.0
RUST_LOG=warn,songbird_core=info
SONGBIRD_MAX_NODES=1000
```

---

## 🐳 **Docker Deployment**

### Single Node Deployment

#### 1. Create Dockerfile

```dockerfile
# Production Dockerfile
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy source code
COPY . .

# Build release binary
RUN cargo build --release --bin songbird-orchestrator

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false songbird

# Copy binary
COPY --from=builder /app/target/release/songbird-orchestrator /usr/local/bin/

# Create directories
RUN mkdir -p /etc/songbird /var/log/songbird && \
    chown -R songbird:songbird /etc/songbird /var/log/songbird

# Switch to non-root user
USER songbird

# Expose ports
EXPOSE 8080 8082 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Start application
CMD ["songbird-orchestrator"]
```

#### 2. Build and Run

```bash
# Build image
docker build -t songbird-orchestrator:latest .

# Run container
docker run -d \
  --name songbird-orchestrator \
  --env-file .env \
  -p 8080:8080 \
  -p 8082:8082 \
  -p 9090:9090 \
  --restart unless-stopped \
  songbird-orchestrator:latest
```

### Multi-Node Docker Compose

#### docker-compose.yml

```yaml
version: '3.8'

services:
  songbird-node-1:
    image: songbird-orchestrator:latest
    container_name: songbird-node-1
    environment:
      - SONGBIRD_FEDERATION_NODE_ID=node-1
      - SONGBIRD_BIND_ADDRESS=0.0.0.0
      - SONGBIRD_CLUSTER_ENDPOINTS=http://songbird-node-2:8082,http://songbird-node-3:8082
    ports:
      - "8080:8080"
      - "8082:8082"
      - "9090:9090"
    networks:
      - songbird-network
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  songbird-node-2:
    image: songbird-orchestrator:latest
    container_name: songbird-node-2
    environment:
      - SONGBIRD_FEDERATION_NODE_ID=node-2
      - SONGBIRD_BIND_ADDRESS=0.0.0.0
      - SONGBIRD_ORCHESTRATOR_PORT=8081
      - SONGBIRD_FEDERATION_PORT=8083
      - SONGBIRD_CLUSTER_ENDPOINTS=http://songbird-node-1:8082,http://songbird-node-3:8082
    ports:
      - "8081:8081"
      - "8083:8083"
      - "9091:9090"
    networks:
      - songbird-network
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8081/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  songbird-node-3:
    image: songbird-orchestrator:latest
    container_name: songbird-node-3
    environment:
      - SONGBIRD_FEDERATION_NODE_ID=node-3
      - SONGBIRD_BIND_ADDRESS=0.0.0.0
      - SONGBIRD_ORCHESTRATOR_PORT=8082
      - SONGBIRD_FEDERATION_PORT=8084
      - SONGBIRD_CLUSTER_ENDPOINTS=http://songbird-node-1:8082,http://songbird-node-2:8083
    ports:
      - "8082:8082"
      - "8084:8084"
      - "9092:9090"
    networks:
      - songbird-network
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8082/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  # Load Balancer
  nginx:
    image: nginx:alpine
    container_name: songbird-lb
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/ssl:ro
    networks:
      - songbird-network
    depends_on:
      - songbird-node-1
      - songbird-node-2
      - songbird-node-3
    restart: unless-stopped

  # Monitoring
  prometheus:
    image: prom/prometheus:latest
    container_name: songbird-prometheus
    ports:
      - "9093:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml:ro
    networks:
      - songbird-network
    restart: unless-stopped

networks:
  songbird-network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16
```

#### Deploy Multi-Node Cluster

```bash
# Start the cluster
docker-compose up -d

# Check cluster status
docker-compose ps

# View logs
docker-compose logs -f songbird-node-1

# Scale horizontally
docker-compose up -d --scale songbird-node=5
```

---

## ☸️ **Kubernetes Deployment**

### Namespace and ConfigMap

#### namespace.yaml
```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: songbird
  labels:
    name: songbird
```

#### configmap.yaml
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: songbird-config
  namespace: songbird
data:
  SONGBIRD_ENV: "production"
  SONGBIRD_BIND_ADDRESS: "0.0.0.0"
  SONGBIRD_ORCHESTRATOR_PORT: "8080"
  SONGBIRD_FEDERATION_PORT: "8082"
  SONGBIRD_MAX_NODES: "1000"
  RUST_LOG: "info,songbird_core=debug"
```

### Deployment

#### deployment.yaml
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: songbird-orchestrator
  namespace: songbird
  labels:
    app: songbird-orchestrator
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
      - name: songbird-orchestrator
        image: songbird-orchestrator:latest
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 8082
          name: federation
        - containerPort: 9090
          name: metrics
        envFrom:
        - configMapRef:
            name: songbird-config
        env:
        - name: SONGBIRD_FEDERATION_NODE_ID
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3
        securityContext:
          runAsNonRoot: true
          runAsUser: 1000
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
        volumeMounts:
        - name: tmp
          mountPath: /tmp
        - name: logs
          mountPath: /var/log/songbird
      volumes:
      - name: tmp
        emptyDir: {}
      - name: logs
        emptyDir: {}
      securityContext:
        fsGroup: 1000
```

### Service and Ingress

#### service.yaml
```yaml
apiVersion: v1
kind: Service
metadata:
  name: songbird-orchestrator
  namespace: songbird
  labels:
    app: songbird-orchestrator
spec:
  selector:
    app: songbird-orchestrator
  ports:
  - name: http
    port: 80
    targetPort: 8080
    protocol: TCP
  - name: federation
    port: 8082
    targetPort: 8082
    protocol: TCP
  - name: metrics
    port: 9090
    targetPort: 9090
    protocol: TCP
  type: ClusterIP

---
apiVersion: v1
kind: Service
metadata:
  name: songbird-orchestrator-headless
  namespace: songbird
  labels:
    app: songbird-orchestrator
spec:
  selector:
    app: songbird-orchestrator
  ports:
  - name: federation
    port: 8082
    targetPort: 8082
    protocol: TCP
  clusterIP: None
```

#### ingress.yaml
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: songbird-orchestrator
  namespace: songbird
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/rate-limit-window: "1m"
spec:
  tls:
  - hosts:
    - songbird.example.com
    secretName: songbird-tls
  rules:
  - host: songbird.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: songbird-orchestrator
            port:
              number: 80
```

### Deploy to Kubernetes

```bash
# Apply all configurations
kubectl apply -f namespace.yaml
kubectl apply -f configmap.yaml
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml
kubectl apply -f ingress.yaml

# Check deployment status
kubectl get pods -n songbird
kubectl get services -n songbird
kubectl get ingress -n songbird

# View logs
kubectl logs -f deployment/songbird-orchestrator -n songbird

# Scale deployment
kubectl scale deployment songbird-orchestrator --replicas=5 -n songbird
```

---

## 🖥️ **Bare Metal Deployment**

### System Preparation

#### 1. Install Dependencies

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y \
    curl \
    wget \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates

# CentOS/RHEL
sudo yum update
sudo yum groupinstall -y "Development Tools"
sudo yum install -y \
    openssl-devel \
    ca-certificates
```

#### 2. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup update stable
```

#### 3. Build from Source

```bash
# Clone repository
git clone https://github.com/ecoPrimals/SongBird.git
cd SongBird

# Build release binary
cargo build --release --bin songbird-orchestrator

# Install binary
sudo cp target/release/songbird-orchestrator /usr/local/bin/
sudo chmod +x /usr/local/bin/songbird-orchestrator
```

### Service Configuration

#### 1. Create User and Directories

```bash
# Create service user
sudo useradd -r -s /bin/false songbird

# Create directories
sudo mkdir -p /etc/songbird /var/log/songbird /var/lib/songbird
sudo chown -R songbird:songbird /etc/songbird /var/log/songbird /var/lib/songbird
```

#### 2. Configuration File

Create `/etc/songbird/config.toml`:

```toml
[core]
node_id = "prod-node-1"
bind_address = "0.0.0.0"
orchestrator_port = 8080
federation_port = 8082
environment = "production"

[federation]
enabled = true
heartbeat_interval_seconds = 30
discovery_enabled = true
max_nodes = 1000
cluster_endpoints = [
    "http://node2.internal:8082",
    "http://node3.internal:8082"
]

[network]
connection_timeout_seconds = 30
request_timeout_seconds = 60
health_check_timeout_seconds = 5

[observability]
metrics_enabled = true
metrics_port = 9090
log_level = "info"

[security]
tls_enabled = true
cert_path = "/etc/ssl/certs/songbird.crt"
key_path = "/etc/ssl/private/songbird.key"
```

#### 3. Systemd Service

Create `/etc/systemd/system/songbird-orchestrator.service`:

```ini
[Unit]
Description=Songbird Universal Orchestrator
Documentation=https://github.com/ecoPrimals/SongBird
After=network.target
Wants=network.target

[Service]
Type=exec
User=songbird
Group=songbird
ExecStart=/usr/local/bin/songbird-orchestrator --config /etc/songbird/config.toml
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=songbird-orchestrator

# Security settings
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/var/log/songbird /var/lib/songbird
CapabilityBoundingSet=CAP_NET_BIND_SERVICE
AmbientCapabilities=CAP_NET_BIND_SERVICE

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
```

#### 4. Start Service

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable and start service
sudo systemctl enable songbird-orchestrator
sudo systemctl start songbird-orchestrator

# Check status
sudo systemctl status songbird-orchestrator

# View logs
sudo journalctl -u songbird-orchestrator -f
```

---

## 🌐 **Federation Setup**

### Multi-Node Federation

#### 1. Node Configuration

**Node 1** (`node1.internal`):
```bash
export SONGBIRD_FEDERATION_NODE_ID=node-1
export SONGBIRD_BIND_ADDRESS=0.0.0.0
export SONGBIRD_ORCHESTRATOR_PORT=8080
export SONGBIRD_FEDERATION_PORT=8082
export SONGBIRD_CLUSTER_ENDPOINTS="http://node2.internal:8082,http://node3.internal:8082"
```

**Node 2** (`node2.internal`):
```bash
export SONGBIRD_FEDERATION_NODE_ID=node-2
export SONGBIRD_BIND_ADDRESS=0.0.0.0
export SONGBIRD_ORCHESTRATOR_PORT=8080
export SONGBIRD_FEDERATION_PORT=8082
export SONGBIRD_CLUSTER_ENDPOINTS="http://node1.internal:8082,http://node3.internal:8082"
```

**Node 3** (`node3.internal`):
```bash
export SONGBIRD_FEDERATION_NODE_ID=node-3
export SONGBIRD_BIND_ADDRESS=0.0.0.0
export SONGBIRD_ORCHESTRATOR_PORT=8080
export SONGBIRD_FEDERATION_PORT=8082
export SONGBIRD_CLUSTER_ENDPOINTS="http://node1.internal:8082,http://node2.internal:8082"
```

#### 2. Network Discovery

```bash
# Enable network discovery
export SONGBIRD_AUTO_DISCOVERY_ENABLED=true
export SONGBIRD_DISCOVERY_NETWORK_RANGES="10.0.0.0/8,192.168.0.0/16"
export SONGBIRD_DISCOVERY_PORTS="8080,8081,8082,8443,9090"
```

#### 3. Health Monitoring

```bash
# Configure health monitoring
export SONGBIRD_HEARTBEAT_INTERVAL=30
export SONGBIRD_HEALTH_CHECK_INTERVAL=60
export SONGBIRD_NODE_TIMEOUT=300
```

### Federation Verification

```bash
# Check federation status
curl http://localhost:8080/federation/status

# List federation nodes
curl http://localhost:8080/federation/nodes

# Check node health
curl http://localhost:8080/federation/health

# Send test message
curl -X POST http://localhost:8080/federation/broadcast \
  -H "Content-Type: application/json" \
  -d '{"message": "test", "type": "ping"}'
```

---

## 📊 **Monitoring & Observability**

### Prometheus Configuration

#### prometheus.yml
```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "songbird_rules.yml"

scrape_configs:
  - job_name: 'songbird-orchestrator'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: /metrics
    scrape_interval: 10s

  - job_name: 'songbird-federation'
    kubernetes_sd_configs:
      - role: pod
        namespaces:
          names:
            - songbird
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app]
        action: keep
        regex: songbird-orchestrator
      - source_labels: [__meta_kubernetes_pod_container_port_name]
        action: keep
        regex: metrics

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
```

### Grafana Dashboard

#### Key Metrics to Monitor

1. **System Metrics**
   - CPU usage per node
   - Memory usage per node
   - Network I/O
   - Disk usage

2. **Federation Metrics**
   - Active nodes count
   - Heartbeat success rate
   - Message broadcast latency
   - Node discovery rate

3. **Application Metrics**
   - Request throughput
   - Response latency
   - Error rates
   - Service registration count

4. **Performance Metrics**
   - Zero-copy message efficiency
   - Buffer pool utilization
   - Memory allocation rate
   - GC pressure

### Log Aggregation

#### Fluentd Configuration

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: fluentd-config
data:
  fluent.conf: |
    <source>
      @type tail
      path /var/log/songbird/*.log
      pos_file /var/log/fluentd-songbird.log.pos
      tag songbird.*
      format json
      time_key timestamp
      time_format %Y-%m-%dT%H:%M:%S.%NZ
    </source>
    
    <match songbird.**>
      @type elasticsearch
      host elasticsearch
      port 9200
      index_name songbird
      type_name _doc
    </match>
```

---

## 🔒 **Security Configuration**

### TLS/SSL Setup

#### 1. Generate Certificates

```bash
# Generate private key
openssl genrsa -out songbird.key 2048

# Generate certificate signing request
openssl req -new -key songbird.key -out songbird.csr \
  -subj "/C=US/ST=State/L=City/O=Organization/CN=songbird.example.com"

# Generate self-signed certificate (for testing)
openssl x509 -req -in songbird.csr -signkey songbird.key -out songbird.crt -days 365

# Or use Let's Encrypt for production
certbot certonly --standalone -d songbird.example.com
```

#### 2. Configure TLS

```bash
export TLS_CERT_PATH=/etc/ssl/certs/songbird.crt
export TLS_KEY_PATH=/etc/ssl/private/songbird.key
export SONGBIRD_TLS_ENABLED=true
```

### Authentication

#### JWT Configuration

```bash
export JWT_SECRET=your-super-secret-jwt-key
export JWT_EXPIRATION=3600
export SONGBIRD_AUTH_ENABLED=true
```

#### API Key Authentication

```bash
export API_KEY_ENABLED=true
export API_KEYS="key1:read,key2:write,key3:admin"
```

### Network Security

#### Firewall Rules

```bash
# Allow orchestrator port
sudo ufw allow 8080/tcp

# Allow federation port
sudo ufw allow 8082/tcp

# Allow metrics port (restrict to monitoring network)
sudo ufw allow from 10.0.1.0/24 to any port 9090

# Allow SSH (if needed)
sudo ufw allow 22/tcp

# Enable firewall
sudo ufw enable
```

#### Network Policies (Kubernetes)

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: songbird-network-policy
  namespace: songbird
spec:
  podSelector:
    matchLabels:
      app: songbird-orchestrator
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: monitoring
    ports:
    - protocol: TCP
      port: 9090
  - from: []
    ports:
    - protocol: TCP
      port: 8080
    - protocol: TCP
      port: 8082
  egress:
  - {}
```

---

## ⚡ **Performance Tuning**

### System Optimization

#### 1. Kernel Parameters

```bash
# Add to /etc/sysctl.conf
net.core.rmem_max = 16777216
net.core.wmem_max = 16777216
net.ipv4.tcp_rmem = 4096 87380 16777216
net.ipv4.tcp_wmem = 4096 65536 16777216
net.core.netdev_max_backlog = 5000
net.ipv4.tcp_congestion_control = bbr

# Apply changes
sudo sysctl -p
```

#### 2. File Descriptor Limits

```bash
# Add to /etc/security/limits.conf
songbird soft nofile 65536
songbird hard nofile 65536

# For systemd services, add to service file:
LimitNOFILE=65536
```

### Application Tuning

#### 1. Memory Configuration

```bash
# Increase buffer sizes for high throughput
export SONGBIRD_LARGE_BUFFER_SIZE=32768
export SONGBIRD_SMALL_BUFFER_SIZE=4096

# Configure buffer pool
export SONGBIRD_BUFFER_POOL_SIZE=1000
```

#### 2. Concurrency Settings

```bash
# Adjust worker threads
export TOKIO_WORKER_THREADS=8

# Configure connection pools
export SONGBIRD_MAX_CONNECTIONS=1000
export SONGBIRD_CONNECTION_POOL_SIZE=100
```

#### 3. Federation Optimization

```bash
# Optimize heartbeat intervals for your network
export SONGBIRD_HEARTBEAT_INTERVAL=15  # For low-latency networks
export SONGBIRD_HEARTBEAT_INTERVAL=60  # For high-latency networks

# Adjust discovery intervals
export SONGBIRD_DISCOVERY_INTERVAL=120  # 2 minutes
export SONGBIRD_NODE_SCAN_INTERVAL=60   # 1 minute
```

### Benchmarking

#### Load Testing

```bash
# Install hey (HTTP load testing tool)
go install github.com/rakyll/hey@latest

# Test orchestrator API
hey -n 10000 -c 100 -m GET http://localhost:8080/health

# Test federation endpoints
hey -n 5000 -c 50 -m GET http://localhost:8082/federation/status
```

#### Performance Monitoring

```bash
# Monitor system resources
htop
iotop
nethogs

# Monitor application metrics
curl http://localhost:9090/metrics | grep songbird

# Check memory usage
cat /proc/$(pgrep songbird)/status | grep -E "VmRSS|VmSize"
```

---

## 🔧 **Troubleshooting**

### Common Issues

#### 1. Service Won't Start

**Symptoms**: Service fails to start or exits immediately

**Solutions**:
```bash
# Check configuration
songbird-orchestrator --config /etc/songbird/config.toml --validate

# Check logs
journalctl -u songbird-orchestrator -n 50

# Check port availability
netstat -tlnp | grep :8080

# Check permissions
ls -la /etc/songbird/
sudo -u songbird songbird-orchestrator --version
```

#### 2. Federation Issues

**Symptoms**: Nodes can't discover each other or heartbeats fail

**Solutions**:
```bash
# Check network connectivity
telnet node2.internal 8082

# Verify DNS resolution
nslookup node2.internal

# Check firewall rules
sudo ufw status
iptables -L

# Test federation endpoints
curl http://node2.internal:8082/federation/info
```

#### 3. High Memory Usage

**Symptoms**: Memory usage continuously grows

**Solutions**:
```bash
# Check for memory leaks
valgrind --tool=memcheck --leak-check=full songbird-orchestrator

# Monitor memory allocation
export RUST_LOG=debug
# Look for excessive allocations in logs

# Adjust buffer pool settings
export SONGBIRD_BUFFER_POOL_SIZE=500  # Reduce pool size
```

#### 4. Performance Issues

**Symptoms**: High latency or low throughput

**Solutions**:
```bash
# Check CPU usage
top -p $(pgrep songbird)

# Monitor network I/O
iftop -i eth0

# Check disk I/O
iostat -x 1

# Optimize configuration
export TOKIO_WORKER_THREADS=16  # Match CPU cores
export SONGBIRD_LARGE_BUFFER_SIZE=65536  # Increase buffer size
```

### Diagnostic Commands

```bash
# Health check
curl http://localhost:8080/health

# Federation status
curl http://localhost:8082/federation/status | jq

# Metrics snapshot
curl http://localhost:9090/metrics | grep -E "songbird_(requests|errors|duration)"

# Configuration dump
curl http://localhost:8080/debug/config

# Memory statistics
curl http://localhost:8080/debug/memory

# Thread dump
kill -USR1 $(pgrep songbird)  # If implemented
```

### Log Analysis

#### Important Log Patterns

```bash
# Federation issues
grep -i "federation\|heartbeat\|discovery" /var/log/songbird/orchestrator.log

# Network errors
grep -i "connection\|timeout\|refused" /var/log/songbird/orchestrator.log

# Performance issues
grep -i "slow\|timeout\|buffer\|memory" /var/log/songbird/orchestrator.log

# Security issues
grep -i "auth\|unauthorized\|forbidden" /var/log/songbird/orchestrator.log
```

---

## 📞 **Support and Maintenance**

### Regular Maintenance Tasks

1. **Daily**
   - Check service health
   - Monitor resource usage
   - Review error logs

2. **Weekly**
   - Update security patches
   - Rotate logs
   - Backup configuration

3. **Monthly**
   - Performance review
   - Capacity planning
   - Security audit

### Backup Strategy

```bash
# Configuration backup
tar -czf songbird-config-$(date +%Y%m%d).tar.gz /etc/songbird/

# Log backup (if not using log aggregation)
tar -czf songbird-logs-$(date +%Y%m%d).tar.gz /var/log/songbird/

# Database backup (if applicable)
pg_dump songbird_prod > songbird-db-$(date +%Y%m%d).sql
```

### Update Procedure

```bash
# 1. Backup current installation
sudo systemctl stop songbird-orchestrator
cp /usr/local/bin/songbird-orchestrator /usr/local/bin/songbird-orchestrator.backup

# 2. Download new version
wget https://github.com/ecoPrimals/SongBird/releases/download/v0.2.0/songbird-orchestrator
chmod +x songbird-orchestrator
sudo mv songbird-orchestrator /usr/local/bin/

# 3. Test configuration
songbird-orchestrator --config /etc/songbird/config.toml --validate

# 4. Start service
sudo systemctl start songbird-orchestrator

# 5. Verify operation
curl http://localhost:8080/health
```

---

*This deployment guide is continuously updated with best practices and lessons learned from production deployments.* 