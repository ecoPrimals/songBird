# 🚀 Songbird Deployment Guide

**Status**: ✅ Production Ready (A Grade: 93/100)  
**Last Updated**: November 18, 2025  
**Version**: v0.1.0

---

## Quick Start

### Prerequisites
```bash
# Rust toolchain
rustc --version  # Should be 1.75+

# Required tools
cargo --version
cargo-llvm-cov --version  # For coverage
```

### Verify Everything Works
```bash
# Clone/navigate to project
cd /path/to/songbird

# Run full test suite (should show 599/599 passing)
cargo test --workspace

# Build release binary
cargo build --workspace --release

# Binaries available at:
ls -lh target/release/songbird*
```

---

## 🏗️ Build Commands

### Development Build
```bash
cargo build --workspace
```

### Release Build (Optimized)
```bash
cargo build --workspace --release
```

### Specific Crate
```bash
cargo build -p songbird-orchestrator --release
```

---

## 🧪 Testing

### All Tests (599 tests)
```bash
cargo test --workspace
```

### Library Tests Only (544 tests)
```bash
cargo test --workspace --lib
```

### Integration Tests (55 tests)
```bash
cargo test -p songbird-universal --test security_adapter_integration_tests
cargo test -p songbird-universal --test security_adapter_http_tests
cargo test -p songbird-discovery --test discovery_comprehensive_real_tests
```

### Coverage Report
```bash
cargo llvm-cov --workspace --lib --summary-only
```

---

## 📦 Deployment Options

### Option 1: Local Deployment
```bash
# Build release binary
cargo build --workspace --release

# Run orchestrator
./target/release/songbird-orchestrator

# Or use cargo run
cargo run --release -p songbird-orchestrator
```

### Option 2: Docker Deployment
```bash
# Build Docker image
docker build -t songbird:latest -f docker/Dockerfile .

# Run container
docker run -p 8080:8080 songbird:latest

# Or use docker-compose
docker-compose up -d
```

### Option 3: Systemd Service
```bash
# Copy binary to system location
sudo cp target/release/songbird-orchestrator /usr/local/bin/

# Create systemd service
sudo cp config/songbird.service /etc/systemd/system/

# Enable and start
sudo systemctl enable songbird
sudo systemctl start songbird
sudo systemctl status songbird
```

---

## 🔧 Configuration

### Environment Variables
```bash
# Required
export SONGBIRD_ORCHESTRATOR_PORT=8080
export SONGBIRD_DISCOVERY_ENABLED=true

# Optional
export SONGBIRD_LOG_LEVEL=info
export SONGBIRD_SECURITY_ENDPOINT=http://security:8081
export RUST_BACKTRACE=1
```

### Configuration File
```bash
# Use example config
cp config.env.example config.env

# Edit as needed
vim config.env

# Load config
source config.env
```

---

## 📊 Health Checks

### Orchestrator Health
```bash
curl http://localhost:8080/health
```

### Discovery Health
```bash
curl http://localhost:8080/discovery/health
```

### Metrics
```bash
curl http://localhost:8080/metrics
```

---

## 🔍 Monitoring

### Logs
```bash
# Follow logs
journalctl -u songbird -f

# Or if running directly
RUST_LOG=info cargo run --release
```

### Key Metrics to Monitor
- Request latency
- Circuit breaker trips
- Service discovery updates
- Health check failures
- Security events

---

## 🚨 Troubleshooting

### Build Fails
```bash
# Clean and rebuild
cargo clean
cargo build --workspace --release
```

### Tests Fail
```bash
# Run specific failing test
cargo test <test_name> -- --nocapture

# Check for environment issues
env | grep SONGBIRD
```

### Runtime Issues
```bash
# Check logs
tail -f /var/log/songbird/songbird.log

# Verify configuration
cargo run --release -p songbird-orchestrator -- --validate-config
```

---

## 🔐 Security Checklist

Before deploying to production:

- [ ] Change default credentials
- [ ] Enable TLS/HTTPS
- [ ] Configure firewall rules
- [ ] Set up monitoring/alerting
- [ ] Review security configuration
- [ ] Test authentication flows
- [ ] Verify circuit breakers work
- [ ] Test failure scenarios

---

## 📈 Performance Tuning

### Resource Limits
```toml
# In config
[performance]
max_connections = 1000
max_memory_mb = 2048
max_cpu_cores = 4
```

### Connection Pooling
```toml
[network]
connection_pool_size = 100
request_timeout_secs = 30
```

### Caching
```toml
[cache]
enabled = true
ttl_secs = 300
max_size_mb = 512
```

---

## 🎯 Production Deployment Steps

### 1. Pre-Deployment
```bash
# Verify tests pass
cargo test --workspace

# Build release
cargo build --workspace --release

# Run security scan (optional)
cargo audit

# Check dependencies
cargo outdated
```

### 2. Staging Deployment
```bash
# Deploy to staging
./deploy-staging.sh

# Run smoke tests
./scripts/smoke-tests.sh staging

# Monitor for 24 hours
```

### 3. Production Deployment
```bash
# Create backup
./scripts/backup-config.sh

# Deploy to production
./deploy-production.sh

# Verify health
curl https://api.production.com/health

# Monitor closely for first hour
```

### 4. Post-Deployment
```bash
# Verify all services healthy
./scripts/health-check-all.sh

# Check metrics
./scripts/check-metrics.sh

# Review logs for errors
./scripts/log-review.sh
```

---

## 🔄 Rollback Plan

If issues occur:

```bash
# Quick rollback
./scripts/rollback.sh

# Or manual rollback
systemctl stop songbird
cp /backup/songbird-orchestrator /usr/local/bin/
systemctl start songbird
```

---

## 📞 Getting Help

### Documentation
- Main docs: `/docs/`
- API docs: `cargo doc --open`
- Specs: `/specs/`

### Commands
```bash
# Run with help
songbird-orchestrator --help

# Validate config
songbird-orchestrator --validate-config

# Check version
songbird-orchestrator --version
```

---

## ✅ Deployment Checklist

### Before First Deploy
- [ ] Review `/READY_FOR_PRODUCTION.md`
- [ ] Run all 599 tests
- [ ] Build release binary
- [ ] Configure environment
- [ ] Set up monitoring
- [ ] Test locally

### For Each Deploy
- [ ] Tests passing
- [ ] Configuration reviewed
- [ ] Backup created
- [ ] Deployment window scheduled
- [ ] Team notified
- [ ] Monitoring ready

### After Deploy
- [ ] Health checks passing
- [ ] Metrics normal
- [ ] No error spikes
- [ ] Performance acceptable
- [ ] Team updated

---

## 🎓 Best Practices

1. **Always test before deploying**
   ```bash
   cargo test --workspace
   ```

2. **Use release builds in production**
   ```bash
   cargo build --workspace --release
   ```

3. **Monitor health continuously**
   ```bash
   watch -n 5 'curl -s http://localhost:8080/health'
   ```

4. **Keep configurations in version control**
   ```bash
   git add config/
   git commit -m "Update production config"
   ```

5. **Have rollback plan ready**
   - Keep previous binary
   - Test rollback procedure
   - Document steps

---

## 📊 Current Status

**As of November 18, 2025:**

- ✅ **Build**: PASSING (all crates)
- ✅ **Tests**: 599/599 (100%)
- ✅ **Grade**: A (93/100)
- ✅ **Coverage**: 61.85% + 55 integration tests
- ✅ **Security**: 55 comprehensive tests
- ✅ **Documentation**: Complete
- ✅ **Status**: PRODUCTION READY

**Confidence Level**: HIGH ⭐  
**Ready For**: Immediate staging deployment

---

## 🚀 Quick Deploy Commands

```bash
# Full deployment workflow
cargo test --workspace && \
cargo build --workspace --release && \
sudo systemctl restart songbird && \
curl http://localhost:8080/health

# One-liner for staging
./scripts/deploy-staging.sh && ./scripts/smoke-tests.sh

# One-liner for production
./scripts/deploy-production.sh && ./scripts/monitor.sh
```

---

**Need Help?** See:
- Production Certificate: `/reports/nov_18_2025_session/READY_FOR_PRODUCTION.md`
- Full Documentation: `/00_START_HERE.md`
- Project Status: `/STATUS.md`

---

*Last Verified: November 18, 2025*  
*Next Review: After first production deployment*

