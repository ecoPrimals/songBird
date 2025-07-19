# Production Deployment Guide

## Overview

This guide covers best practices for deploying the Songbird Universal Network Orchestrator in production environments with high availability, security, and performance.

## Prerequisites

### System Requirements

#### Minimum Production Requirements
- **CPU**: 4 cores
- **Memory**: 8GB RAM  
- **Storage**: 50GB SSD
- **Network**: High-speed connection with redundancy
- **OS**: Linux (Ubuntu 20.04+, RHEL 8+, or equivalent)

#### Recommended Production Requirements
- **CPU**: 8+ cores
- **Memory**: 16GB+ RAM
- **Storage**: 100GB+ NVMe SSD
- **Network**: Gigabit ethernet with redundancy
- **OS**: Linux with container runtime (Docker/Podman)

### Software Dependencies

```bash
# Ubuntu/Debian
sudo apt update && sudo apt install -y \
    curl wget git htop \
    postgresql-client redis-tools \
    prometheus-node-exporter

# RHEL/CentOS  
sudo yum update && sudo yum install -y \
    curl wget git htop \
    postgresql redis \
    node_exporter
```

## 🚀 High-Availability Deployment

### Load Balancer Setup

```nginx
# /etc/nginx/sites-available/songbird
upstream songbird_backend {
    server 10.0.1.10:8080 max_fails=3 fail_timeout=30s;
    server 10.0.1.11:8080 max_fails=3 fail_timeout=30s;
    server 10.0.1.12:8080 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;
    listen 443 ssl http2;
    server_name orchestrator.example.com;
    
    ssl_certificate /etc/ssl/certs/songbird.crt;
    ssl_certificate_key /etc/ssl/private/songbird.key;
    
    location / {
        proxy_pass http://songbird_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # WebSocket support
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        
        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }
    
    location /health {
        access_log off;
        proxy_pass http://songbird_backend;
    }
}
```

### Database Configuration

```sql
-- PostgreSQL setup for persistent storage
CREATE DATABASE songbird_production;
CREATE USER songbird WITH PASSWORD 'secure_password';
GRANT ALL PRIVILEGES ON DATABASE songbird_production TO songbird;

-- Redis setup for caching
CONFIG SET maxmemory 2gb
CONFIG SET maxmemory-policy allkeys-lru
```

### Production Configuration

```toml
# /etc/songbird/production.toml
[server]
host = "0.0.0.0"
port = 8080
workers = 8
max_connections = 10000

[orchestrator]
name = "songbird-production"
max_services = 10000
auto_discovery = true
health_check_interval = "15s"
service_timeout = "60s"
data_directory = "/data/songbird"
backup_enabled = true
backup_interval = "1h"

[database]
enabled = true
url = "postgresql://songbird:secure_password@localhost:5432/songbird_production"
max_connections = 50
connection_timeout = "30s"

[cache]
enabled = true
redis_url = "redis://localhost:6379/0"
ttl = "1h"
max_memory = "2gb"

[security]
authentication_enabled = true
authorization_enabled = true
rate_limit_per_minute = 10000
session_timeout = "24h"
api_key_rotation_interval = "30d"
tls_enabled = true
tls_cert = "/etc/ssl/certs/songbird.crt"
tls_key = "/etc/ssl/private/songbird.key"

[primals]
coordination_enabled = true
discovery_timeout = "30s"
max_concurrent_requests = 1000
circuit_breaker_enabled = true
circuit_breaker_threshold = 50
circuit_breaker_timeout = "60s"

[gaming]
enabled = true
discovery_port = 47624
low_latency_mode = true
max_sessions = 1000
session_timeout = "2h"

[load_balancer]
default_algorithm = "health-aware"
health_required = true
max_retries = 3
retry_delay = "1s"
connection_pooling = true
pool_size = 100

[monitoring]
metrics_enabled = true
metrics_port = 9090
tracing_enabled = true
log_level = "info"
log_format = "json"
log_rotation = "daily"
log_retention = "30d"
audit_enabled = true

[communication]
protocols = ["http", "websocket"]
max_connections = 10000
heartbeat_interval = "30s"
message_buffer_size = 10000
compression_enabled = true
```

## 🐳 Container Orchestration

### Docker Compose Production

```yaml
# docker-compose.production.yml
version: '3.8'

services:
  songbird-1:
    image: songbird/orchestrator:latest
    hostname: songbird-1
    environment:
      - SONGBIRD_NODE_ID=songbird-1
      - SONGBIRD_CLUSTER_ID=production
      - SONGBIRD_CONFIG=/etc/songbird/production.toml
    volumes:
      - ./config:/etc/songbird
      - ./data/songbird-1:/data
      - ./logs:/var/log/songbird
    networks:
      - songbird-network
    depends_on:
      - postgres
      - redis
    restart: unless-stopped
    deploy:
      resources:
        limits:
          cpus: '4'
          memory: 8G
        reservations:
          cpus: '2'
          memory: 4G

  songbird-2:
    image: songbird/orchestrator:latest
    hostname: songbird-2
    environment:
      - SONGBIRD_NODE_ID=songbird-2
      - SONGBIRD_CLUSTER_ID=production
      - SONGBIRD_CONFIG=/etc/songbird/production.toml
    volumes:
      - ./config:/etc/songbird
      - ./data/songbird-2:/data
      - ./logs:/var/log/songbird
    networks:
      - songbird-network
    depends_on:
      - postgres
      - redis
    restart: unless-stopped
    deploy:
      resources:
        limits:
          cpus: '4'
          memory: 8G
        reservations:
          cpus: '2'
          memory: 4G

  songbird-3:
    image: songbird/orchestrator:latest
    hostname: songbird-3
    environment:
      - SONGBIRD_NODE_ID=songbird-3
      - SONGBIRD_CLUSTER_ID=production
      - SONGBIRD_CONFIG=/etc/songbird/production.toml
    volumes:
      - ./config:/etc/songbird
      - ./data/songbird-3:/data
      - ./logs:/var/log/songbird
    networks:
      - songbird-network
    depends_on:
      - postgres
      - redis
    restart: unless-stopped
    deploy:
      resources:
        limits:
          cpus: '4'
          memory: 8G
        reservations:
          cpus: '2'
          memory: 4G

  postgres:
    image: postgres:15
    environment:
      - POSTGRES_DB=songbird_production
      - POSTGRES_USER=songbird
      - POSTGRES_PASSWORD=secure_password
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./sql:/docker-entrypoint-initdb.d
    networks:
      - songbird-network
    restart: unless-stopped
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G

  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes --maxmemory 2gb --maxmemory-policy allkeys-lru
    volumes:
      - redis_data:/data
    networks:
      - songbird-network
    restart: unless-stopped
    deploy:
      resources:
        limits:
          cpus: '1'
          memory: 2G

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/ssl
    networks:
      - songbird-network
    depends_on:
      - songbird-1
      - songbird-2
      - songbird-3
    restart: unless-stopped

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    networks:
      - songbird-network
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--storage.tsdb.retention.time=30d'
    restart: unless-stopped

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    volumes:
      - grafana_data:/var/lib/grafana
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards
    networks:
      - songbird-network
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_USERS_ALLOW_SIGN_UP=false
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
  prometheus_data:
  grafana_data:

networks:
  songbird-network:
    driver: bridge
```

### Kubernetes Production

```yaml
# k8s-production.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: songbird-orchestrator
  namespace: songbird-production
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: songbird-orchestrator
  template:
    metadata:
      labels:
        app: songbird-orchestrator
    spec:
      affinity:
        podAntiAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
          - labelSelector:
              matchExpressions:
              - key: app
                operator: In
                values:
                - songbird-orchestrator
            topologyKey: kubernetes.io/hostname
      containers:
      - name: songbird
        image: songbird/orchestrator:latest
        ports:
        - containerPort: 8080
        - containerPort: 9090
        env:
        - name: SONGBIRD_NODE_ID
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: SONGBIRD_CLUSTER_ID
          value: "production"
        - name: SONGBIRD_CONFIG
          value: "/etc/songbird/production.toml"
        volumeMounts:
        - name: config
          mountPath: /etc/songbird
        - name: data
          mountPath: /data
        resources:
          requests:
            memory: "4Gi"
            cpu: "2"
          limits:
            memory: "8Gi"
            cpu: "4"
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
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 2
        startupProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 10
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
  namespace: songbird-production
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
  sessionAffinity: ClientIP

---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: songbird-ingress
  namespace: songbird-production
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/websocket-services: "songbird-service"
    nginx.ingress.kubernetes.io/proxy-read-timeout: "3600"
    nginx.ingress.kubernetes.io/proxy-send-timeout: "3600"
spec:
  tls:
  - hosts:
    - orchestrator.example.com
    secretName: songbird-tls
  rules:
  - host: orchestrator.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: songbird-service
            port:
              number: 8080

---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: songbird-data
  namespace: songbird-production
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 100Gi
  storageClassName: fast-ssd
```

## 🔒 Security Best Practices

### TLS/SSL Configuration

```bash
# Generate production certificates (use Let's Encrypt or CA)
certbot certonly --nginx -d orchestrator.example.com

# Or use internal CA
openssl req -x509 -newkey rsa:4096 -keyout /etc/ssl/private/songbird.key \
  -out /etc/ssl/certs/songbird.crt -days 365 -nodes \
  -subj "/C=US/ST=State/L=City/O=Organization/CN=orchestrator.example.com"
```

### Authentication & Authorization

```bash
# Create admin user
songbird auth create-user \
  --username admin \
  --password $(openssl rand -base64 32) \
  --role admin \
  --email admin@example.com

# Create service accounts
songbird auth create-service-account \
  --name monitoring \
  --permissions metrics:read,health:read

songbird auth create-service-account \
  --name ci-cd \
  --permissions service:read,service:write,service:delete

# Generate API keys
songbird auth generate-api-key --service-account monitoring
songbird auth generate-api-key --service-account ci-cd
```

### Firewall Rules

```bash
# UFW (Ubuntu)
sudo ufw allow 22/tcp          # SSH
sudo ufw allow 80/tcp          # HTTP
sudo ufw allow 443/tcp         # HTTPS
sudo ufw allow 8080/tcp        # Songbird (internal)
sudo ufw allow 9090/tcp        # Metrics (internal)
sudo ufw enable

# iptables
iptables -A INPUT -p tcp --dport 22 -j ACCEPT
iptables -A INPUT -p tcp --dport 80 -j ACCEPT
iptables -A INPUT -p tcp --dport 443 -j ACCEPT
iptables -A INPUT -p tcp --dport 8080 -s 10.0.0.0/8 -j ACCEPT
iptables -A INPUT -p tcp --dport 9090 -s 10.0.0.0/8 -j ACCEPT
```

## 📊 Monitoring & Observability

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "songbird_rules.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - localhost:9093

scrape_configs:
  - job_name: 'songbird'
    static_configs:
      - targets: ['songbird-1:9090', 'songbird-2:9090', 'songbird-3:9090']
    metrics_path: /metrics
    scrape_interval: 15s
    
  - job_name: 'node'
    static_configs:
      - targets: ['localhost:9100']
```

### Grafana Dashboard

```json
{
  "dashboard": {
    "title": "Songbird Production Dashboard",
    "panels": [
      {
        "title": "System Health",
        "type": "stat",
        "targets": [
          {
            "expr": "songbird_health_status",
            "legendFormat": "Health Status"
          }
        ]
      },
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(songbird_requests_total[5m])",
            "legendFormat": "Requests/sec"
          }
        ]
      },
      {
        "title": "Service Count",
        "type": "stat",
        "targets": [
          {
            "expr": "songbird_services_total",
            "legendFormat": "Total Services"
          }
        ]
      }
    ]
  }
}
```

### Alerting Rules

```yaml
# songbird_rules.yml
groups:
  - name: songbird
    rules:
    - alert: SongbirdDown
      expr: up{job="songbird"} == 0
      for: 1m
      labels:
        severity: critical
      annotations:
        summary: "Songbird orchestrator is down"
        description: "Songbird orchestrator {{ $labels.instance }} has been down for more than 1 minute."
        
    - alert: HighRequestRate
      expr: rate(songbird_requests_total[5m]) > 1000
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: "High request rate detected"
        description: "Request rate is {{ $value }} requests/sec"
        
    - alert: ServiceUnhealthy
      expr: songbird_services_unhealthy > 0
      for: 2m
      labels:
        severity: warning
      annotations:
        summary: "Unhealthy services detected"
        description: "{{ $value }} services are unhealthy"
```

## 🚀 Performance Optimization

### System Tuning

```bash
# Increase file descriptor limits
echo "songbird soft nofile 65536" >> /etc/security/limits.conf
echo "songbird hard nofile 65536" >> /etc/security/limits.conf

# Network tuning
echo "net.core.somaxconn = 65535" >> /etc/sysctl.conf
echo "net.ipv4.tcp_max_syn_backlog = 65535" >> /etc/sysctl.conf
echo "net.core.netdev_max_backlog = 5000" >> /etc/sysctl.conf

# Apply changes
sysctl -p
```

### Database Optimization

```sql
-- PostgreSQL tuning
ALTER SYSTEM SET shared_buffers = '2GB';
ALTER SYSTEM SET effective_cache_size = '6GB';
ALTER SYSTEM SET maintenance_work_mem = '512MB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET default_statistics_target = 100;

-- Restart PostgreSQL
SELECT pg_reload_conf();
```

### Redis Optimization

```conf
# /etc/redis/redis.conf
maxmemory 4gb
maxmemory-policy allkeys-lru
save 900 1
save 300 10
save 60 10000
tcp-keepalive 300
timeout 0
```

## 🔄 Backup & Recovery

### Automated Backups

```bash
#!/bin/bash
# /usr/local/bin/songbird-backup.sh

BACKUP_DIR="/backup/songbird"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/songbird_backup_$TIMESTAMP.tar.gz"

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Backup configuration
tar -czf "$BACKUP_FILE" \
  /etc/songbird \
  /data/songbird \
  /var/log/songbird

# Backup database
pg_dump -h localhost -U songbird songbird_production | gzip > "$BACKUP_DIR/db_backup_$TIMESTAMP.sql.gz"

# Cleanup old backups (keep 7 days)
find "$BACKUP_DIR" -name "*.tar.gz" -mtime +7 -delete
find "$BACKUP_DIR" -name "*.sql.gz" -mtime +7 -delete

# Upload to S3 (optional)
aws s3 cp "$BACKUP_FILE" s3://songbird-backups/
```

### Disaster Recovery

```bash
#!/bin/bash
# /usr/local/bin/songbird-restore.sh

BACKUP_FILE="$1"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup_file>"
    exit 1
fi

# Stop services
systemctl stop songbird

# Restore files
tar -xzf "$BACKUP_FILE" -C /

# Restore database
gunzip -c "$BACKUP_DIR/db_backup_latest.sql.gz" | psql -h localhost -U songbird songbird_production

# Start services
systemctl start songbird
```

## 🎯 Health Checks & Monitoring

### Health Check Script

```bash
#!/bin/bash
# /usr/local/bin/songbird-health.sh

HEALTH_URL="http://localhost:8080/health"
METRICS_URL="http://localhost:9090/metrics"

# Check main health endpoint
if curl -sf "$HEALTH_URL" > /dev/null; then
    echo "✅ Songbird orchestrator is healthy"
else
    echo "❌ Songbird orchestrator is unhealthy"
    exit 1
fi

# Check metrics endpoint
if curl -sf "$METRICS_URL" > /dev/null; then
    echo "✅ Metrics endpoint is accessible"
else
    echo "❌ Metrics endpoint is not accessible"
    exit 1
fi

# Check service count
SERVICE_COUNT=$(curl -sf "$HEALTH_URL/detailed" | jq -r '.data.services.total')
if [ "$SERVICE_COUNT" -gt 0 ]; then
    echo "✅ $SERVICE_COUNT services are registered"
else
    echo "⚠️ No services are registered"
fi
```

### Automated Health Monitoring

```bash
# Add to crontab
*/5 * * * * /usr/local/bin/songbird-health.sh || /usr/local/bin/alert-admin.sh
```

## 📈 Scaling & Performance

### Horizontal Scaling

```yaml
# autoscaling.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: songbird-hpa
  namespace: songbird-production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: songbird-orchestrator
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### Load Testing

```bash
# Install wrk
sudo apt install wrk

# Basic load test
wrk -t12 -c400 -d30s --timeout 5s http://localhost:8080/health

# Service creation test
wrk -t4 -c100 -d30s --timeout 10s -s create-service.lua http://localhost:8080/api/v1/services
```

## 🔧 Maintenance

### Regular Maintenance Tasks

```bash
#!/bin/bash
# /usr/local/bin/songbird-maintenance.sh

echo "Starting Songbird maintenance..."

# Update system packages
apt update && apt upgrade -y

# Clean up old logs
find /var/log/songbird -name "*.log" -mtime +30 -delete

# Restart services if needed
if systemctl is-active --quiet songbird; then
    echo "Songbird is running"
else
    echo "Restarting Songbird..."
    systemctl restart songbird
fi

# Database maintenance
sudo -u postgres psql -d songbird_production -c "VACUUM ANALYZE;"

# Check disk space
df -h

echo "Maintenance complete"
```

### Automated Updates

```bash
# Add to crontab for weekly maintenance
0 2 * * 0 /usr/local/bin/songbird-maintenance.sh
```

## 📋 Troubleshooting

### Common Production Issues

1. **High Memory Usage**
   ```bash
   # Check memory usage
   songbird orchestrator stats --memory
   
   # Adjust configuration
   [orchestrator]
   max_services = 5000  # Reduce if needed
   ```

2. **Database Connection Issues**
   ```bash
   # Check connections
   psql -h localhost -U songbird -c "SELECT * FROM pg_stat_activity;"
   
   # Increase connection pool
   [database]
   max_connections = 100
   ```

3. **Service Discovery Problems**
   ```bash
   # Check service registry
   songbird services list --detailed
   
   # Restart discovery
   songbird orchestrator restart-discovery
   ```

For detailed troubleshooting, monitoring, and maintenance procedures, see the [Operations Guide](../operations/TROUBLESHOOTING.md).

This production guide ensures your Songbird Universal Network Orchestrator deployment is secure, scalable, and maintainable in production environments. 