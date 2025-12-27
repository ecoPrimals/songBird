# 🚀 Songbird Deployment Guide

**Version**: 5-Week MVP Complete  
**Date**: December 18, 2025  
**Status**: Production Ready

---

## ✅ Pre-Deployment Checklist

### System Requirements
- **OS**: Linux (recommended), macOS, Windows (WSL2)
- **Rust**: 1.70+ (stable channel)
- **Database**: SQLite 3.35+ (bundled via sqlx)
- **Memory**: 512MB minimum, 2GB recommended
- **CPU**: 1 core minimum, 4+ cores recommended
- **Network**: Internet access for TLS certificate generation

### Dependencies
- `cargo` and `rustc` (via rustup)
- `libsqlite3-dev` (for SQLite)
- `pkg-config` (for build tools)
- `openssl-dev` (for TLS)

---

## 📦 Installation

### Option 1: Build from Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/ecoPrimals/songbird.git
cd songbird

# Build release binary
cargo build --release

# Binary location
./target/release/songbird-orchestrator
```

### Option 2: Install via Cargo

```bash
cargo install --path crates/songbird-orchestrator
```

---

## ⚙️ Configuration

### Environment Variables

```bash
# HTTP/HTTPS Server
export SONGBIRD_HTTP_PORT=8080
export SONGBIRD_HTTPS_PORT=8443
export SONGBIRD_TLS_ENABLED=true  # Default: true (fail-secure)

# Database
export SONGBIRD_DATABASE_URL="sqlite:///var/lib/songbird/tasks.db"

# Resource Limits (per-user defaults)
export SONGBIRD_CPU_QUOTA=8      # CPU cores
export SONGBIRD_MEMORY_QUOTA=16384  # MB
export SONGBIRD_GPU_QUOTA=2      # GPU devices

# Retry & Circuit Breaker
export SONGBIRD_MAX_RETRIES=3
export SONGBIRD_CIRCUIT_BREAKER_THRESHOLD=5
export SONGBIRD_CIRCUIT_BREAKER_TIMEOUT=60  # seconds

# Observability
export SONGBIRD_METRICS_ENABLED=true
export SONGBIRD_METRICS_RETENTION=604800  # 7 days in seconds

# Consent Management
export SONGBIRD_AUTO_APPROVE_THRESHOLD=10.0  # USD

# Federation
export SONGBIRD_NODE_ID="tower-$(hostname)"
export SONGBIRD_NODE_NAME="$(hostname)"
```

### Configuration File (Optional)

Create `config/songbird.toml`:

```toml
[server]
http_port = 8080
https_port = 8443
tls_enabled = true

[database]
url = "sqlite:///var/lib/songbird/tasks.db"

[resources]
cpu_quota = 8
memory_quota = 16384
gpu_quota = 2

[retry]
max_attempts = 3
initial_backoff_ms = 100
max_backoff_ms = 30000

[circuit_breaker]
failure_threshold = 5
success_threshold = 2
timeout_seconds = 60

[observability]
metrics_enabled = true
metrics_retention_seconds = 604800

[consent]
auto_approve_threshold = 10.0
```

---

## 🚀 Running Songbird

### Development Mode

```bash
cargo run --release
```

### Production Mode (Systemd)

Create `/etc/systemd/system/songbird.service`:

```ini
[Unit]
Description=Songbird Orchestrator
After=network.target

[Service]
Type=simple
User=songbird
Group=songbird
WorkingDirectory=/opt/songbird
ExecStart=/opt/songbird/bin/songbird-orchestrator
Restart=always
RestartSec=10

# Environment
Environment="SONGBIRD_HTTP_PORT=8080"
Environment="SONGBIRD_HTTPS_PORT=8443"
Environment="SONGBIRD_TLS_ENABLED=true"
Environment="SONGBIRD_DATABASE_URL=sqlite:///var/lib/songbird/tasks.db"

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/songbird

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable songbird
sudo systemctl start songbird
sudo systemctl status songbird
```

### Docker (Optional)

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libsqlite3-0 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/songbird-orchestrator /usr/local/bin/
EXPOSE 8080 8443
CMD ["songbird-orchestrator"]
```

Build and run:

```bash
docker build -t songbird:latest .
docker run -d -p 8080:8080 -p 8443:8443 \
  -v songbird-data:/var/lib/songbird \
  --name songbird \
  songbird:latest
```

---

## 🧪 Verification

### Health Check

```bash
# HTTP (should redirect to HTTPS)
curl -v http://localhost:8080/health

# HTTPS (with self-signed cert)
curl -k https://localhost:8443/health
```

Expected response:
```json
{"status":"ok","timestamp":"2025-12-18T..."}
```

### Protocol Capabilities

```bash
curl -k https://localhost:8443/api/protocol/capabilities | jq
```

Expected response:
```json
{
  "protocols": ["http", "https", "json-rpc", "tarpc", "websocket", "wss"],
  "preferred": "tarpc"
}
```

### Task Lifecycle

```bash
# Create a task
curl -k -X POST https://localhost:8443/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "owner": "test-user",
    "spec": {
      "task_type": "test-task",
      "config": {},
      "required_capabilities": ["compute"],
      "resources": {
        "cpu_cores": 2,
        "memory_mb": 4096
      },
      "priority": "Standard"
    }
  }' | jq

# List tasks
curl -k https://localhost:8443/api/tasks?owner=test-user | jq
```

---

## 📊 Monitoring

### Metrics Endpoint

```bash
curl -k https://localhost:8443/api/metrics | jq
```

### Logs

```bash
# Systemd
sudo journalctl -u songbird -f

# Docker
docker logs -f songbird

# File
tail -f /var/log/songbird/orchestrator.log
```

### Key Metrics to Monitor

- **Task Queue Length**: Should remain < 1000
- **Resource Utilization**: CPU/Memory/GPU usage per user
- **Circuit Breaker State**: Should be "Closed" most of the time
- **Retry Rate**: High retry rate indicates transient failures
- **Consent Approval Rate**: Track auto-approvals vs manual

---

## 🔒 Security Recommendations

### TLS Configuration

- ✅ **TLS enabled by default** (fail-secure)
- Use proper certificates in production (Let's Encrypt recommended)
- Rotate certificates regularly (90 days recommended)

### Resource Quotas

- Set appropriate per-user quotas based on workload
- Monitor quota usage to prevent abuse
- Implement cost tracking for billing

### Consent Management

- Set conservative auto-approval thresholds
- Audit consent decisions regularly
- Implement human override for critical operations

### Network Security

- Run behind reverse proxy (nginx/traefik)
- Enable rate limiting
- Use firewall rules to restrict access

---

## 🔧 Troubleshooting

### Common Issues

**Issue**: Tasks stuck in "Queued" state
**Solution**: Check admission control system load, may need to scale

**Issue**: Circuit breaker constantly "Open"
**Solution**: Check downstream service health, may need to increase timeout

**Issue**: High memory usage
**Solution**: Check checkpoint retention, may need cleanup

**Issue**: TLS certificate errors
**Solution**: Ensure crypto provider is initialized (should be automatic)

### Debug Mode

```bash
RUST_LOG=debug cargo run --release
```

### Database Maintenance

```bash
# Compact database
sqlite3 /var/lib/songbird/tasks.db "VACUUM;"

# Check database size
du -h /var/lib/songbird/tasks.db

# Backup database
cp /var/lib/songbird/tasks.db /backups/tasks-$(date +%Y%m%d).db
```

---

## 📈 Scaling Recommendations

### Single Node (Current)
- Good for: 1-10 towers, 1000s of tasks/day
- Limits: SQLite single-writer

### Multi-Node (Future)
- Replace SQLite with PostgreSQL/CockroachDB
- Add Redis for distributed locks
- Implement task queue with message broker

### Performance Optimization
- Enable tarpc for low-latency workloads
- Use JSON-RPC for cross-language interop
- Batch operations where possible
- Implement caching layer

---

## 📚 Additional Resources

- **Specifications**: See `specs/` directory
- **Architecture**: See `docs/` directory
- **Roadmap**: See `ROADMAP.md`
- **5-Week MVP**: See `FIVE_WEEK_MVP_COMPLETE.md`
- **Showcase**: See `showcase/` for live demos

---

## 🆘 Support

- **Issues**: https://github.com/ecoPrimals/songbird/issues
- **Discussions**: https://github.com/ecoPrimals/songbird/discussions
- **Documentation**: https://ecoprimals.dev/songbird

---

*Last Updated: December 18, 2025*  
*Version: 5-Week MVP Complete (Production Ready)*

