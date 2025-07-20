# 🚀 **SONGBIRD DEPLOYMENT GUIDE**

**Version**: 0.1.0  
**Status**: Production Ready  
**Last Updated**: January 2025  

---

## 🎯 **QUICK DEPLOYMENT OPTIONS**

### **Choose Your Deployment Style**
- **🚀 [Quick Start](#quick-start)** - Get running in 5 minutes
- **🧪 [Development Setup](#development-setup)** - Local testing environment
- **🏭 [Production Deployment](#production-deployment)** - Full production setup
- **🌐 [Federation Cluster](#federation-cluster-setup)** - Multi-node distributed setup
- **🐳 [Docker Deployment](#docker-deployment)** - Containerized deployment
- **☸️ [Kubernetes](#kubernetes-deployment)** - Kubernetes orchestration

---

## 🚀 **QUICK START** (5 minutes)

### **Prerequisites**
- Rust 1.70+ installed
- 8GB+ RAM recommended
- Network connectivity for primal discovery

### **Step 1: Build & Run**
```bash
# Clone and build (if not already done)
cd /path/to/songbird
cargo build --release

# Start the orchestrator
./target/release/songbird-orchestrator

# In another terminal, start gaming demo
./target/release/gaming-demo
```

### **Step 2: Verify Installation**
```bash
# Check health
curl http://localhost:8080/api/health

# Expected response:
{
  "overall_health": "healthy",
  "components": {
    "gaming_bridge": {"status": "healthy"},
    "federation": {"status": "healthy"},
    "primals": {"status": "healthy"}
  }
}
```

### **Step 3: Test Gaming Setup**
```bash
# Test one-touch gaming setup
curl -X POST http://localhost:8080/api/gaming/setup \
  -H "Content-Type: application/json" \
  -d '{"setup_type": "one_touch"}'

# Expected response:
{
  "success": true,
  "message": "Gaming setup completed successfully",
  "next_steps": ["Gaming network ready"]
}
```

**🎉 Congratulations! Songbird is running locally.**

---

## 🧪 **DEVELOPMENT SETUP**

### **Complete Development Environment**

#### **1. Environment Setup**
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install development dependencies
cargo install cargo-watch cargo-tarpaulin

# Clone repository
git clone <your-repo-url>
cd songbird
```

#### **2. Development Configuration**
```bash
# Create development config
cat > songbird-dev.toml << EOF
[primal_registry]
auto_discovery = true
development_mode = true
mock_primals_enabled = true

[[primal_registry.primals]]
primal_type = "mock-beardog"
display_name = "Mock BearDog (Development)"
enabled = true
endpoint = { primary_url = "http://localhost:8443" }

[[primal_registry.primals]]
primal_type = "mock-toadstool"
display_name = "Mock Toadstool (Development)"
enabled = true
endpoint = { primary_url = "http://localhost:8080" }

[federation]
cluster_id = "dev-cluster"
node_id = "dev-node-1"
development_mode = true
heartbeat_interval = 10

[gaming]
development_mode = true
family_safe_mode = false
mock_games = ["StarCraft", "Diablo", "AgeOfEmpires"]
protocols = ["ipx", "directplay", "tcp", "udp"]

[security]
development_mode = true
encryption_enabled = false
authentication_required = false

[observability]
development_mode = true
detailed_logging = true
metrics_interval = 5
EOF
```

#### **3. Development Workflow**
```bash
# Watch for changes and auto-rebuild
cargo watch -x "build --release"

# Run tests continuously
cargo watch -x "test --workspace"

# Start with development config
SONGBIRD_CONFIG=songbird-dev.toml ./target/release/songbird-orchestrator

# Run individual components for testing
./target/release/gaming-demo --config=songbird-dev.toml
```

#### **4. Development Testing**
```bash
# Test all APIs
./scripts/test-dev-apis.sh

# Load test (if you create this script)
./scripts/load-test-dev.sh

# Test gaming protocols
curl -X POST http://localhost:8080/api/gaming/stress-test \
  -d '{"protocol": "ipx", "sessions": 10, "duration": 30}'
```

---

## 🏭 **PRODUCTION DEPLOYMENT**

### **Production-Ready Setup**

#### **1. System Requirements**
- **OS**: Linux (Ubuntu 20.04+ recommended)
- **CPU**: 4+ cores
- **RAM**: 16GB+ for full federation
- **Storage**: 100GB+ SSD
- **Network**: Stable internet connection, ports 8080, 8443 open

#### **2. Production Configuration**
```toml
# songbird-production.toml
[primal_registry]
auto_discovery = true
default_timeout = 30
health_check_interval = 60

[[primal_registry.primals]]
primal_type = "beardog"
display_name = "BearDog Security Production"
enabled = true
endpoint = { primary_url = "https://beardog.prod.example.com:8443", health_check_path = "/health" }
authentication = { method = "ApiKey", credentials = { api_key = "${BEARDOG_API_KEY}" } }

[[primal_registry.primals]]
primal_type = "toadstool"
display_name = "Toadstool Compute Production"
enabled = true
endpoint = { primary_url = "https://toadstool.prod.example.com:8080" }

[federation]
cluster_id = "production-cluster"
node_id = "${NODE_ID}"
cluster_endpoints = [
  "https://songbird-node-1.prod.example.com:8080",
  "https://songbird-node-2.prod.example.com:8080",
  "https://songbird-node-3.prod.example.com:8080"
]
auto_discovery = true
heartbeat_interval = 30
connection_timeout = 10
max_retries = 3

[gaming]
family_safe_mode = false
auto_detect_games = true
protocols = ["ipx", "directplay", "tcp", "udp"]
max_concurrent_sessions = 1000

[security]
encryption_enabled = true
authentication_required = true
audit_level = "comprehensive"
tls_cert_path = "/etc/songbird/certs/songbird.crt"
tls_key_path = "/etc/songbird/certs/songbird.key"

[observability]
metrics_enabled = true
metrics_port = 9090
logging_level = "info"
audit_logging = true
performance_monitoring = true

[performance]
max_connections = 10000
worker_threads = 8
request_timeout = 30
```

#### **3. Production Deployment Script**
```bash
#!/bin/bash
# deploy-production.sh

set -e

echo "🚀 Deploying Songbird to Production..."

# Build release version
cargo build --release --target x86_64-unknown-linux-gnu

# Create production directories
sudo mkdir -p /opt/songbird/{bin,config,logs,data,certs}
sudo mkdir -p /etc/songbird
sudo mkdir -p /var/log/songbird

# Install binaries
sudo cp target/release/songbird-orchestrator /opt/songbird/bin/
sudo cp target/release/gaming-demo /opt/songbird/bin/
sudo chmod +x /opt/songbird/bin/*

# Install configuration
sudo cp songbird-production.toml /etc/songbird/songbird.toml
sudo chown -R songbird:songbird /etc/songbird
sudo chmod 600 /etc/songbird/songbird.toml

# Install systemd service
sudo tee /etc/systemd/system/songbird.service > /dev/null << EOF
[Unit]
Description=Songbird Universal Orchestrator
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=songbird
Group=songbird
WorkingDirectory=/opt/songbird
ExecStart=/opt/songbird/bin/songbird-orchestrator --config=/etc/songbird/songbird.toml
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
Environment=NODE_ID=%H

# Security settings
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/var/log/songbird /opt/songbird/data

[Install]
WantedBy=multi-user.target
EOF

# Create songbird user
sudo useradd --system --shell /bin/false --home /opt/songbird songbird
sudo chown -R songbird:songbird /opt/songbird /var/log/songbird

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable songbird
sudo systemctl start songbird

echo "✅ Songbird deployed successfully!"
echo "📊 Status: sudo systemctl status songbird"
echo "📝 Logs: sudo journalctl -u songbird -f"
```

#### **4. Production Health Monitoring**
```bash
#!/bin/bash
# health-check.sh

# Basic health check
curl -f http://localhost:8080/api/health || exit 1

# Detailed system check
curl -s http://localhost:8080/api/metrics | jq '.system_metrics'

# Federation health
curl -s http://localhost:8080/api/federation/status | jq '.cluster_status'

# Gaming status
curl -s http://localhost:8080/api/gaming/sessions | jq '.'
```

---

## 🌐 **FEDERATION CLUSTER SETUP**

### **Multi-Node Distributed Deployment**

#### **Node 1 (Primary)**
```bash
# songbird-node1.toml
[federation]
cluster_id = "production-cluster"
node_id = "primary-node"
cluster_endpoints = [
  "https://node2.example.com:8080",
  "https://node3.example.com:8080"
]
auto_discovery = true
heartbeat_interval = 30

# Start primary node
NODE_ID=primary-node ./target/release/songbird-orchestrator \
  --config=songbird-node1.toml
```

#### **Node 2 (Secondary)**
```bash
# songbird-node2.toml
[federation]
cluster_id = "production-cluster"
node_id = "secondary-node"
cluster_endpoints = [
  "https://node1.example.com:8080",
  "https://node3.example.com:8080"
]
join_on_startup = true

# Start secondary node
NODE_ID=secondary-node ./target/release/songbird-orchestrator \
  --config=songbird-node2.toml
```

#### **Node 3 (Tertiary)**
```bash
# songbird-node3.toml
[federation]
cluster_id = "production-cluster"
node_id = "tertiary-node"
cluster_endpoints = [
  "https://node1.example.com:8080",
  "https://node2.example.com:8080"
]
join_on_startup = true

# Start tertiary node
NODE_ID=tertiary-node ./target/release/songbird-orchestrator \
  --config=songbird-node3.toml
```

#### **Federation Health Check**
```bash
# Check cluster status from any node
curl http://node1.example.com:8080/api/federation/status

# Expected response:
{
  "cluster_status": {
    "cluster_id": "production-cluster",
    "node_count": 3,
    "healthy_nodes": 3,
    "cluster_health": 1.0
  },
  "nodes": [
    {"node_id": "primary-node", "status": "online"},
    {"node_id": "secondary-node", "status": "online"}, 
    {"node_id": "tertiary-node", "status": "online"}
  ]
}
```

---

## 🐳 **DOCKER DEPLOYMENT**

### **Containerized Deployment**

#### **1. Dockerfile**
```dockerfile
# Dockerfile
FROM rust:1.70 as builder

WORKDIR /usr/src/songbird
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/src/songbird/target/release/songbird-orchestrator .
COPY --from=builder /usr/src/songbird/target/release/gaming-demo .
COPY docker/songbird-production.toml ./config/

EXPOSE 8080 8443 9090

ENTRYPOINT ["./songbird-orchestrator"]
CMD ["--config=config/songbird-production.toml"]
```

#### **2. Docker Compose**
```yaml
# docker-compose.yml
version: '3.8'

services:
  songbird-primary:
    build: .
    container_name: songbird-primary
    hostname: songbird-primary
    ports:
      - "8080:8080"
      - "8443:8443"
      - "9090:9090"
    environment:
      - NODE_ID=primary-docker
      - CLUSTER_ID=docker-cluster
    volumes:
      - ./config/primary.toml:/app/config/songbird.toml:ro
      - songbird-data:/app/data
      - ./logs:/app/logs
    networks:
      - songbird-network
    restart: unless-stopped

  songbird-secondary:
    build: .
    container_name: songbird-secondary
    hostname: songbird-secondary
    ports:
      - "8081:8080"
      - "8444:8443"
      - "9091:9090"
    environment:
      - NODE_ID=secondary-docker
      - CLUSTER_ID=docker-cluster
    volumes:
      - ./config/secondary.toml:/app/config/songbird.toml:ro
      - songbird-data-2:/app/data
    depends_on:
      - songbird-primary
    networks:
      - songbird-network
    restart: unless-stopped

volumes:
  songbird-data:
  songbird-data-2:

networks:
  songbird-network:
    driver: bridge
```

#### **3. Deploy with Docker**
```bash
# Build and deploy
docker-compose up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f songbird-primary

# Scale the cluster
docker-compose up -d --scale songbird-secondary=3

# Health check
curl http://localhost:8080/api/health
```

---

## ☸️ **KUBERNETES DEPLOYMENT**

### **K8s Manifests**

#### **1. Namespace & ConfigMap**
```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: songbird
---
# k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: songbird-config
  namespace: songbird
data:
  songbird.toml: |
    [primal_registry]
    auto_discovery = true
    
    [federation]
    cluster_id = "k8s-cluster"
    auto_discovery = true
    heartbeat_interval = 30
    
    [gaming]
    auto_detect_games = true
    protocols = ["ipx", "directplay", "tcp", "udp"]
    
    [security]
    encryption_enabled = true
    
    [observability]
    metrics_enabled = true
    metrics_port = 9090
```

#### **2. Deployment**
```yaml
# k8s/deployment.yaml
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
        image: songbird:latest
        ports:
        - containerPort: 8080
        - containerPort: 8443
        - containerPort: 9090
        env:
        - name: NODE_ID
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: CLUSTER_ID
          value: "k8s-cluster"
        volumeMounts:
        - name: config
          mountPath: /app/config
          readOnly: true
        livenessProbe:
          httpGet:
            path: /api/health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /api/health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
      volumes:
      - name: config
        configMap:
          name: songbird-config
```

#### **3. Service & Ingress**
```yaml
# k8s/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: songbird-service
  namespace: songbird
spec:
  selector:
    app: songbird-orchestrator
  ports:
  - name: api
    port: 8080
    targetPort: 8080
  - name: secure
    port: 8443
    targetPort: 8443
  - name: metrics
    port: 9090
    targetPort: 9090
  type: ClusterIP
---
# k8s/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: songbird-ingress
  namespace: songbird
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
  rules:
  - host: songbird.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: songbird-service
            port:
              number: 8080
```

#### **4. Deploy to Kubernetes**
```bash
# Apply all manifests
kubectl apply -f k8s/

# Check deployment
kubectl get pods -n songbird
kubectl get services -n songbird

# View logs
kubectl logs -f deployment/songbird-orchestrator -n songbird

# Port forward for testing
kubectl port-forward -n songbird svc/songbird-service 8080:8080

# Test API
curl http://localhost:8080/api/health
```

---

## 🔧 **CONFIGURATION MANAGEMENT**

### **Environment Variables**
```bash
# Core Configuration
export SONGBIRD_CONFIG="/path/to/songbird.toml"
export NODE_ID="unique-node-identifier"
export CLUSTER_ID="production-cluster"

# Security
export BEARDOG_API_KEY="your-beardog-api-key"
export TOADSTOOL_TOKEN="your-toadstool-token"

# TLS Certificates
export TLS_CERT_PATH="/etc/songbird/certs/songbird.crt"
export TLS_KEY_PATH="/etc/songbird/certs/songbird.key"

# Observability
export METRICS_ENABLED="true"
export LOG_LEVEL="info"
export AUDIT_ENABLED="true"
```

### **Configuration Validation**
```bash
# Validate configuration before deployment
./target/release/songbird-orchestrator --validate-config --config=songbird.toml

# Test configuration connectivity
./target/release/songbird-orchestrator --test-connections --config=songbird.toml
```

---

## 📊 **MONITORING & MAINTENANCE**

### **Production Monitoring**
```bash
# System metrics
curl http://localhost:9090/metrics | grep songbird_

# Federation health
watch -n 30 'curl -s http://localhost:8080/api/federation/status | jq ".cluster_status.cluster_health"'

# Gaming sessions
watch -n 10 'curl -s http://localhost:8080/api/gaming/sessions | jq "length"'

# Error rates
curl -s http://localhost:8080/api/metrics | jq '.system_metrics.error_rate'
```

### **Log Management**
```bash
# Centralized logging with journald
sudo journalctl -u songbird -f --since "1 hour ago"

# Log rotation
sudo tee /etc/logrotate.d/songbird << EOF
/var/log/songbird/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    create 644 songbird songbird
}
EOF
```

### **Backup & Recovery**
```bash
# Backup critical data
tar -czf songbird-backup-$(date +%Y%m%d).tar.gz \
  /etc/songbird \
  /opt/songbird/data \
  /var/log/songbird

# Database backup (if applicable)
# Application state is typically stateless, but federation state may need backing up
```

---

## 🚨 **TROUBLESHOOTING**

### **Common Issues & Solutions**

#### **Issue**: Service won't start
```bash
# Check configuration
./target/release/songbird-orchestrator --validate-config

# Check permissions
ls -la /etc/songbird/
sudo chown -R songbird:songbird /opt/songbird

# Check ports
netstat -tulpn | grep :8080
```

#### **Issue**: Federation connection failed
```bash
# Test network connectivity
curl -v http://remote-node:8080/api/health

# Check certificates
openssl s_client -connect remote-node:8443 -servername remote-node

# Verify cluster endpoints in config
grep -A 10 "\[federation\]" /etc/songbird/songbird.toml
```

#### **Issue**: Gaming setup failed
```bash
# Check primal connectivity
curl http://localhost:8080/api/primals/discover

# Test gaming endpoints
curl -X POST http://localhost:8080/api/gaming/setup \
  -d '{"setup_type": "one_touch"}'

# Check system capabilities
curl http://localhost:8080/api/gaming/capabilities
```

---

## 🎯 **DEPLOYMENT CHECKLIST**

### **Pre-Deployment**
- [ ] System requirements met
- [ ] Rust 1.70+ installed
- [ ] Configuration files prepared
- [ ] TLS certificates obtained (production)
- [ ] Firewall rules configured
- [ ] DNS records configured (production)

### **Deployment**
- [ ] Build successful (`cargo build --release`)
- [ ] Configuration validated
- [ ] Service user created
- [ ] Systemd service installed (Linux)
- [ ] Service started and enabled
- [ ] Health check passes

### **Post-Deployment**
- [ ] API endpoints responding
- [ ] Federation cluster joined (if multi-node)
- [ ] Primal discovery working
- [ ] Gaming setup functional
- [ ] Monitoring alerts configured
- [ ] Log rotation configured
- [ ] Backup procedures tested

---

## 🎉 **SUCCESS VALIDATION**

### **Complete Deployment Test**
```bash
#!/bin/bash
# deployment-test.sh

echo "🧪 Testing Songbird Deployment..."

# Health check
echo "1. Health Check..."
curl -f http://localhost:8080/api/health || { echo "❌ Health check failed"; exit 1; }
echo "✅ Health check passed"

# Gaming setup
echo "2. Gaming Setup Test..."
RESULT=$(curl -s -X POST http://localhost:8080/api/gaming/setup -d '{"setup_type":"one_touch"}' | jq -r '.success')
if [ "$RESULT" = "true" ]; then
    echo "✅ Gaming setup working"
else
    echo "❌ Gaming setup failed"
    exit 1
fi

# Primal discovery
echo "3. Primal Discovery Test..."
curl -s http://localhost:8080/api/primals/discover | jq -e '.discovered_primals | length > 0' || {
    echo "⚠️  No primals discovered (may be normal in test environment)"
}

# Metrics
echo "4. Metrics Test..."
curl -f http://localhost:9090/metrics > /dev/null && echo "✅ Metrics working" || echo "⚠️  Metrics endpoint not responding"

echo "🎉 Deployment test completed successfully!"
```

---

**🚀 Your Songbird Universal Orchestrator is now deployed and ready for production!**

**Next Steps**: 
- [Gaming Setup Guide](GAMING_SETUP_GUIDE.md) - Configure gaming networks
- [API Reference](API_REFERENCE.md) - Full API documentation  
- [Live Testing Guide](LIVE_TESTING_GUIDE.md) - Comprehensive testing procedures 