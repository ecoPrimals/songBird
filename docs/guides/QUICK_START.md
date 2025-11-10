# 🚀 Songbird Quick Start Guide
**Get up and running in 5 minutes!**

---

## ⚡ TL;DR - Fastest Path

```bash
# 1. Build (24.5 seconds)
cargo build --workspace --release

# 2. Setup environment
export SERVICE_PORT=8080
export SERVICE_ID=songbird-quickstart

# 3. Run discovery demo
cargo run --example infant_discovery_demo --package songbird-config

# Done! 🎉
```

---

## 📋 Prerequisites

Before starting, ensure you have:

- **Rust 1.70+** installed (`rustc --version`)
- **4GB+ RAM** available
- **Linux** (Ubuntu 20.04+) or **macOS**
- **Network access** (for distributed features)

### Install Rust (if needed)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

---

## 🎯 Step-by-Step Guide

### **Step 1: Clone the Repository**

```bash
git clone https://github.com/ecoPrimals/songbird
cd songbird
```

### **Step 2: Build Songbird**

```bash
# Full workspace build (13 crates)
cargo build --workspace --release

# Expected output:
#   Compiling songbird-types v0.1.0
#   Compiling songbird-config v0.1.0
#   ...
#   Finished release [optimized] target(s) in 24.50s
```

**Build Time**: ~24.5 seconds on modern hardware

### **Step 3: Verify Build**

```bash
# Run tests to verify everything works
cargo test --workspace

# Expected: All tests pass ✅
```

### **Step 4: Run Your First Example**

```bash
# Setup environment
export SERVICE_PORT=8080
export SERVICE_ID=my-first-songbird
export SONGBIRD_HOST=127.0.0.1

# Run infant discovery demo
cargo run --example infant_discovery_demo --package songbird-config
```

**Expected Output:**
```
🍼 Infant Discovery Demo - Starting with Zero Knowledge
================================================

👶 Starting State: ZERO KNOWLEDGE

❌ We DON'T know:
   - Primal names (beardog, squirrel, toadstool, nestgate)
   - Port numbers
   - Endpoint URLs

✅ We ONLY know:
   - Our own identity (from environment)
   - What capabilities we need
   - How to discover

🔍 Discovered self identity: my-first-songbird
✅ Configuration created successfully!
```

---

## 🎮 Try More Examples

### **Example 1: Vendor-Agnostic Discovery**

```bash
export SERVICE_PORT=8081
cargo run --example vendor_agnostic_demo --package songbird-discovery
```

Shows capability-based service discovery patterns.

### **Example 2: Ecosystem Integration**

```bash
export SERVICE_PORT=8082
cargo run --example ecosystem_standalone_demo --package songbird-primal-sdk
```

Demonstrates primal SDK usage and integration.

### **Example 3: Run Demo Scripts**

```bash
# Federation coordination demo
./demos/federation-coordination-demo.sh

# BYOB coordination demo
./demos/byob-coordination-demo.sh
```

---

## 🏗️ Next Steps

### **Option 1: Local Development**

```bash
# Setup development environment
cp config/development.env .env
source .env

# Run orchestrator locally
cargo run --bin songbird-orchestrator --release
```

### **Option 2: Multi-Tower Setup (LAN)**

Set up multiple Songbird instances on your local network:

```bash
# Tower A (192.168.1.100)
export SERVICE_ID=tower-a
export SERVICE_PORT=8080
export SONGBIRD_HOST=192.168.1.100
cargo run --bin songbird-orchestrator --release

# Tower B (192.168.1.101) - different machine
export SERVICE_ID=tower-b
export SERVICE_PORT=8081
export SONGBIRD_HOST=192.168.1.101
cargo run --bin songbird-orchestrator --release
```

Towers will discover each other automatically!

### **Option 3: Integration with Primals**

```bash
# Setup with Toadstool (compute)
export TOADSTOOL_ENDPOINT=http://localhost:8081
cargo run --bin songbird-orchestrator --release

# Setup with BearDog (security)
export BEARDOG_ENDPOINT=http://localhost:8443
export BEARDOG_AUTH_ENABLED=true
cargo run --bin songbird-orchestrator --release
```

---

## 🔧 Configuration

### **Environment Variables (Minimal)**

```bash
# Required
export SERVICE_PORT=8080              # Port to listen on
export SERVICE_ID=my-service          # Unique service identifier

# Optional but recommended
export SONGBIRD_HOST=127.0.0.1        # Bind address
export SONGBIRD_ENV=development        # Environment (development/staging/production)

# Discovery (optional)
export DISCOVERY_METHODS=environment,dns,network
```

### **Configuration File (Optional)**

Create `config/local.toml`:

```toml
[service]
id = "songbird-local"
port = 8080
host = "127.0.0.1"
environment = "development"

[discovery]
methods = ["environment", "dns", "network"]
timeout_secs = 30

[network]
bind_address = "0.0.0.0"
max_connections = 1000
```

Then run:
```bash
cargo run --bin songbird-orchestrator --release -- --config config/local.toml
```

---

## 🧪 Verify Everything Works

### **Health Check**

```bash
# If orchestrator is running on port 8080:
curl http://localhost:8080/health

# Expected: {"status": "healthy"}
```

### **Metrics**

```bash
curl http://localhost:8080/metrics

# Expected: Prometheus-style metrics
```

### **Discovery Status**

```bash
curl http://localhost:8080/discovery/status

# Expected: List of discovered services
```

---

## 🎯 Common Use Cases

### **Use Case 1: Service Orchestration**

```bash
# Start orchestrator
cargo run --bin songbird-orchestrator --release

# In another terminal, submit a task
curl -X POST http://localhost:8080/orchestrate/task \
  -H "Content-Type: application/json" \
  -d '{"capability": "compute", "payload": {"task": "example"}}'
```

### **Use Case 2: Development Testing**

```bash
# Run with test configuration
export SONGBIRD_ENV=development
export LOG_LEVEL=debug
cargo run --bin songbird-orchestrator

# Logs will show detailed debug information
```

### **Use Case 3: Benchmarking**

```bash
# Run performance benchmarks
cargo bench --workspace

# Results in target/criterion/
```

---

## 📚 Learn More

### **Documentation**

- **[Architecture Overview](ARCHITECTURE_OVERVIEW.md)** - System design
- **[API Reference](docs/API_REFERENCE.md)** - Complete API docs
- **[Examples](examples/)** - 51 example files
- **[Configuration Guide](CONFIG_MIGRATION_GUIDE.md)** - Config options

### **Advanced Topics**

- **[Federation](docs/FEDERATION.md)** - Multi-tower coordination
- **[Security](docs/SECURITY.md)** - Security best practices
- **[Performance](docs/PERFORMANCE_GUIDE.md)** - Optimization guide
- **[Gaming](docs/GAMING.md)** - Gaming-specific features

### **Community**

- **GitHub Issues**: Report bugs or request features
- **Discussions**: Ask questions, share projects
- **Contributing**: See [CONTRIBUTING.md](CONTRIBUTING.md)

---

## ❓ Troubleshooting

### **Build Fails**

```bash
# Update Rust
rustup update stable

# Clean and rebuild
cargo clean
cargo build --workspace --release
```

### **Port Already in Use**

```bash
# Use a different port
export SERVICE_PORT=8090
```

### **Examples Don't Run**

```bash
# Ensure environment variables are set
export SERVICE_PORT=8080
export SERVICE_ID=test

# Run with explicit package
cargo run --example <example_name> --package <crate_name>
```

### **Can't Find Services**

```bash
# Check discovery configuration
export DISCOVERY_METHODS=environment,network

# Verify network connectivity
ping <other_tower_ip>
```

---

## 🎉 What's Next?

Now that you have Songbird running, explore:

1. **Multi-Tower Setup** - Deploy on multiple machines
2. **Primal Integration** - Connect Toadstool, BearDog, NestGate
3. **Custom Orchestration** - Build your own orchestration logic
4. **Gaming Applications** - Use low-latency features
5. **Production Deployment** - Deploy to your infrastructure

---

## 📊 Performance Expectations

**On Modern Hardware:**

| Metric | Value |
|--------|-------|
| **Build Time** | 24.5s (13 crates) |
| **Memory Usage** | 150-300MB idle |
| **Startup Time** | <10 seconds |
| **Request Latency** | <1ms (local) |
| **Discovery Time** | <5 seconds |
| **Throughput** | 10K+ req/s |

---

## ✅ Success Checklist

- [ ] Rust installed and up-to-date
- [ ] Songbird cloned and built successfully
- [ ] Tests passing (100%)
- [ ] Examples running
- [ ] Environment variables configured
- [ ] Health check responding
- [ ] Ready to deploy!

---

**🎊 Congratulations! You're now running Songbird!** 🚀

For production deployment, see [DEPLOYMENT.md](DEPLOYMENT_CHECKLIST.md)  
For multi-tower setup, see session docs in `docs/sessions/2025-11-08/`

---

*Questions? Check the [docs/](docs/) directory or open an issue on GitHub!*
