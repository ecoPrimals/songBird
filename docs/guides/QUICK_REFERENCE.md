# Songbird Quick Reference

**Last Updated**: December 21, 2025  
**Version**: 0.1.0 (Internet Deployment Foundation)

---

## 🎯 What is Songbird?

Songbird is a **sovereign, privacy-preserving, internet-ready P2P orchestration platform** that coordinates distributed compute, manages federation, and acts as the Universal Port Authority for the ecoPrimals ecosystem.

**Key Features**:
- 🌍 **LAN & Internet Federation** - Works on trusted LANs, scales to internet
- 🔒 **Privacy-First** - No hardcoded IPs, ephemeral session IDs, optional encryption
- 🎯 **Universal Port Authority** - Dynamic port allocation for all primals
- 🧬 **BearDog Integration** - Optional security layer for encrypted discovery
- ⚡ **Graceful Degradation** - Works standalone, enhanced when BearDog available

---

## 🚀 Quick Start

### Install & Build

```bash
# Clone repository
git clone <repository-url>
cd songbird

# Build all binaries
cargo build --release

# Binaries available at:
# - target/release/songbird-orchestrator
# - target/release/songbird-cli
```

### Run Locally

```bash
# Start orchestrator (default port 8080)
cargo run --bin songbird-orchestrator

# In another terminal, check health
curl -k https://localhost:8080/health

# Check federation status
curl -k https://localhost:8080/api/federation/status | jq

# List registered services (UPA)
curl -k https://localhost:8080/api/v1/services | jq
```

### Create a Federation (LAN)

```bash
# Tower 1 (bootstrap)
export SONGBIRD_NODE_NAME="eastgate"
export SONGBIRD_PORT=8080
cargo run --bin songbird-orchestrator

# Tower 2
export SONGBIRD_NODE_NAME="westgate"
export SONGBIRD_PORT=8080
export SONGBIRD_BOOTSTRAP_ADDRESS="eastgate:8080"
cargo run --bin songbird-orchestrator

# Tower 3
export SONGBIRD_NODE_NAME="strandgate"
export SONGBIRD_PORT=8080
export SONGBIRD_BOOTSTRAP_ADDRESS="eastgate:8080"
cargo run --bin songbird-orchestrator

# Verify federation (from any tower)
curl -k https://localhost:8080/api/federation/status | jq '.nodes | length'
# Should show: 3
```

---

## 📡 API Endpoints

### Health & Status

```bash
# Health check
GET /health
curl -k https://localhost:8080/health

# Federation status
GET /api/federation/status
curl -k https://localhost:8080/api/federation/status | jq

# Node information
GET /api/v1/info
curl -k https://localhost:8080/api/v1/info | jq
```

### Universal Port Authority (UPA)

```bash
# List all registered services
GET /api/v1/services
curl -k https://localhost:8080/api/v1/services | jq

# Register a service
POST /api/v1/services/register
curl -k -X POST https://localhost:8080/api/v1/services/register \
  -H "Content-Type: application/json" \
  -d '{
    "primal_name": "toadstool",
    "primal_version": "0.1.0",
    "capabilities": [{"name": "compute", "type": "compute"}],
    "endpoints": [{"protocol": "https", "address": "https://localhost:8100"}]
  }'

# Send heartbeat
POST /api/v1/services/{id}/heartbeat
curl -k -X POST https://localhost:8080/api/v1/services/{service-id}/heartbeat

# Query by capability
GET /api/v1/services/query/{capability}
curl -k https://localhost:8080/api/v1/services/query/compute | jq

# Deregister
DELETE /api/v1/services/{id}
curl -k -X DELETE https://localhost:8080/api/v1/services/{service-id}
```

### Compute API

```bash
# Submit task
POST /api/compute/tasks
curl -k -X POST https://localhost:8080/api/compute/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "python_script",
    "payload": {"script": "print(\"Hello from Songbird\")"}
  }'

# Check task status
GET /api/compute/tasks/{task_id}
curl -k https://localhost:8080/api/compute/tasks/{task-id} | jq
```

---

## 🔧 Configuration

### Environment Variables

```bash
# Node configuration
SONGBIRD_NODE_NAME="mynode"           # Node name (default: hostname)
SONGBIRD_PORT=8080                     # HTTP server port (default: 8080)
SONGBIRD_DISCOVERY_PORT=8081           # UDP discovery port (default: 8081)

# Federation
SONGBIRD_BOOTSTRAP_ADDRESS="host:port" # Bootstrap node for federation
SONGBIRD_RENDEZVOUS_URL="https://..."  # Rendezvous server for internet discovery

# BearDog integration (optional)
BEARDOG_URL="https://localhost:8200"   # BearDog service URL
BEARDOG_PORT=8200                      # BearDog service port

# Logging
RUST_LOG=info                          # Log level (trace, debug, info, warn, error)
```

### Discovery Modes

**Plaintext Mode** (default, no BearDog):
- For trusted LANs
- Fast, zero-config
- All information visible to network observers

**BirdSong Mode** (with BearDog):
- For untrusted networks (internet, public WiFi)
- Privacy-preserving
- Only family can decrypt broadcasts

**Auto-Detection**: Songbird automatically detects BearDog and switches modes.

---

## 🌍 Deployment Scenarios

### Scenario 1: Trusted LAN (University Lab)

**Setup**: 3-10 machines on same LAN  
**Mode**: Plaintext discovery  
**BearDog**: Not required

```bash
# On each machine
export SONGBIRD_NODE_NAME="unique-name"
export SONGBIRD_BOOTSTRAP_ADDRESS="first-machine:8080"
cargo run --release --bin songbird-orchestrator
```

### Scenario 2: Internet Federation

**Setup**: Nodes across internet  
**Mode**: BirdSong (requires BearDog)  
**Infrastructure**: Rendezvous server

```bash
# Deploy rendezvous server (cloud)
cd rendezvous
cargo run --release

# On each node
export SONGBIRD_RENDEZVOUS_URL="https://rendezvous.yourdomain.com:8888"
export BEARDOG_URL="https://localhost:8200"
cargo run --release --bin songbird-orchestrator
```

### Scenario 3: Hybrid (LAN + Internet)

**Setup**: Local cluster + remote nodes  
**Mode**: Auto-detect (plaintext on LAN, birdSong for internet)

```bash
# Configure both bootstrap (LAN) and rendezvous (internet)
export SONGBIRD_BOOTSTRAP_ADDRESS="local-cluster-head:8080"
export SONGBIRD_RENDEZVOUS_URL="https://rendezvous.yourdomain.com:8888"
cargo run --release --bin songbird-orchestrator
```

---

## 🧪 Testing & Validation

### Run Unit Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p songbird-network-federation
cargo test -p songbird-orchestrator

# With output
cargo test -- --nocapture
```

### Run Integration Tests

```bash
# BTSP integration
./showcase/10-inter-primal-foundation/04-btsp-integration.sh

# Graceful degradation (BearDog)
./showcase/13-beardog-integration/02-graceful-degradation-test.sh

# Rendezvous integration
./showcase/12-internet-deployment/02-rendezvous-integration-test.sh
```

### Validate Federation

```bash
# Start 3 nodes, then:
curl -k https://localhost:8080/api/federation/status | jq '.nodes | length'
# Expected: 3

# Check all nodes are active
curl -k https://localhost:8080/api/federation/status | jq '.nodes[] | select(.status == "active") | .node_name'
```

---

## 🐛 Troubleshooting

### Problem: "Port already in use"

**Solution**: Songbird has port fallback. If 8080 is busy, it will try 8081, 8082, etc.

```bash
# Check which port it's actually using
grep "HTTP server listening" /path/to/songbird.log

# Or specify a different starting port
export SONGBIRD_PORT=9000
```

### Problem: "Another instance already running"

**Solution**: Songbird uses singleton enforcement via PID file.

```bash
# Check if already running
cat /tmp/songbird.pid

# Kill existing instance
killall songbird-orchestrator

# Remove stale PID file if needed
rm /tmp/songbird.pid
```

### Problem: "Federation not forming"

**Checklist**:
1. ✅ Firewall allows UDP 8081 (discovery)
2. ✅ Firewall allows TCP 8080 (HTTPS)
3. ✅ Bootstrap address is correct
4. ✅ Nodes are on same network (for LAN) or using rendezvous (for internet)

```bash
# Debug discovery
RUST_LOG=debug cargo run --bin songbird-orchestrator

# Check logs for "Received discovery message" or "Registered node"
```

### Problem: "BearDog not detected"

**Expected**: BearDog is optional. Songbird will log:
```
ℹ️  BearDog not available - using plaintext discovery (trusted LAN only)
```

**To enable BearDog**:
1. Implement BearDog (see `BEARDOG_TEAM_BLURB.md`)
2. Register with UPA: `POST /api/v1/services/register`
3. Songbird will auto-detect and switch to birdSong mode

---

## 📚 Documentation Index

### Getting Started
- [README.md](README.md) - Project overview
- [QUICK_REFERENCE.md](QUICK_REFERENCE.md) - This document
- [STATUS.md](STATUS.md) - Current status & roadmap

### Architecture & Design
- [docs/PRIVACY_FIRST_FEDERATION.md](docs/PRIVACY_FIRST_FEDERATION.md) - Privacy architecture
- [docs/INTERNET_DEPLOYMENT_SECURITY.md](docs/INTERNET_DEPLOYMENT_SECURITY.md) - Security analysis
- [docs/PRIMAL_RESPONSIBILITY_SEPARATION.md](docs/PRIMAL_RESPONSIBILITY_SEPARATION.md) - Primal roles

### Specifications
- [specs/RENDEZVOUS_PROTOCOL_SPEC.md](specs/RENDEZVOUS_PROTOCOL_SPEC.md) - Internet discovery protocol
- [specs/BIRDSONG_PROTOCOL.md](specs/BIRDSONG_PROTOCOL.md) - Encrypted broadcast protocol
- [specs/LINEAGE_GATED_RELAY_PROTOCOL.md](specs/LINEAGE_GATED_RELAY_PROTOCOL.md) - Sovereign relay protocol
- [specs/SONGBIRD_BEARDOG_INTEGRATION.md](specs/SONGBIRD_BEARDOG_INTEGRATION.md) - BearDog integration spec

### For Other Primals
- [BEARDOG_TEAM_BLURB.md](BEARDOG_TEAM_BLURB.md) - BearDog integration overview
- [crates/songbird-primal-sdk/](crates/songbird-primal-sdk/) - SDK for primal integration

### Session Summaries
- [EXTENDED_SESSION_FINAL_DEC_21_2025.md](EXTENDED_SESSION_FINAL_DEC_21_2025.md) - Complete session summary
- [DOCS_INDEX.md](DOCS_INDEX.md) - Full documentation index

---

## 🎯 Common Tasks

### Task: Register a Primal with UPA

```bash
# Your primal starts up and discovers Songbird
ORCHESTRATOR=$(curl -s http://localhost:8080/api/v1/info | jq -r '.endpoints[0]')

# Register capabilities
curl -k -X POST $ORCHESTRATOR/api/v1/services/register \
  -H "Content-Type: application/json" \
  -d '{
    "primal_name": "your-primal",
    "primal_version": "0.1.0",
    "capabilities": [
      {"name": "your-capability", "type": "compute"}
    ],
    "endpoints": [
      {"protocol": "https", "address": "https://localhost:YOUR_ASSIGNED_PORT"}
    ]
  }'

# Start heartbeat loop (every 30 seconds)
while true; do
  curl -k -X POST $ORCHESTRATOR/api/v1/services/$SERVICE_ID/heartbeat
  sleep 30
done
```

### Task: Query Federation Status

```bash
# Get all nodes
curl -k https://localhost:8080/api/federation/status | jq '.nodes'

# Get active nodes only
curl -k https://localhost:8080/api/federation/status | jq '.nodes[] | select(.status == "active")'

# Get specific node by name
curl -k https://localhost:8080/api/federation/status | jq '.nodes[] | select(.node_name == "eastgate")'

# Count total nodes
curl -k https://localhost:8080/api/federation/status | jq '.nodes | length'
```

### Task: Deploy to Production

```bash
# 1. Build release binary
cargo build --release

# 2. Create systemd service (example)
cat > /etc/systemd/system/songbird.service << 'EOF'
[Unit]
Description=Songbird Orchestrator
After=network.target

[Service]
Type=simple
User=songbird
Environment="RUST_LOG=info"
Environment="SONGBIRD_NODE_NAME=production-node"
Environment="SONGBIRD_RENDEZVOUS_URL=https://rendezvous.yourdomain.com:8888"
ExecStart=/usr/local/bin/songbird-orchestrator
Restart=always

[Install]
WantedBy=multi-user.target
EOF

# 3. Enable and start
systemctl enable songbird
systemctl start songbird

# 4. Verify
systemctl status songbird
curl -k https://localhost:8080/health
```

---

## 🏆 Key Principles

### Universal Port Authority
> "Once other primals understand how to interact with Songbird, they will never set another port themselves."

**Meaning**: All port allocation is managed by Songbird. Primals register with UPA and receive a dynamically assigned port.

### Privacy-First Federation
> "IPs are like SSNs or phone numbers - private by nature and should be masked at all times."

**Meaning**: No hardcoded IPs. All discovery is name-based or capability-based. IPs are ephemeral, discovered dynamically.

### Graceful Degradation
> "Works without BearDog, enhanced when BearDog is available."

**Meaning**: Songbird is fully functional standalone. Adding BearDog unlocks privacy-preserving, internet-wide capabilities.

### Primal Self-Knowledge
> "Each primal knows only itself and builds for its own sovereignty."

**Meaning**: No compile-time dependencies between primals. Integration happens at runtime via capability-based discovery.

---

## 📊 Current Capabilities

### Deployed ✅
- LAN federation (3+ tower verified)
- Universal Port Authority
- Privacy-first discovery (no hardcoded IPs)
- Process lifecycle management
- Multi-interface coalescence
- Enhanced capability router
- Federation TTL cleanup

### Cloud-Ready ✅
- Rendezvous server (standalone)
- Rendezvous client (integrated)
- Internet discovery foundation

### Integration-Ready ✅
- BearDog trait interfaces
- Mock BearDog provider
- Complete specifications (5,700+ lines)
- Graceful degradation

### In Progress 🔜
- Rendezvous cloud deployment (Phase 2.4)
- STUN implementation (Phase 3)
- Lineage-gated relay (Phase 3, requires BearDog)
- Connection migration (Phase 4)

---

## 🤝 Contributing

### For Songbird Development
1. Review [DOCS_INDEX.md](DOCS_INDEX.md)
2. Check [STATUS.md](STATUS.md) for current work
3. See `showcase/` for examples
4. Follow modern Rust patterns (zero unsafe, async/await, trait-based)

### For Primal Integration
1. Review [crates/songbird-primal-sdk/](crates/songbird-primal-sdk/)
2. See `showcase/10-inter-primal-foundation/` for examples
3. Register with UPA on startup
4. Send heartbeats every 30 seconds
5. Never hardcode ports (use UPA assigned port)

### For BearDog Team
1. Review [BEARDOG_TEAM_BLURB.md](BEARDOG_TEAM_BLURB.md)
2. Review [specs/SONGBIRD_BEARDOG_INTEGRATION.md](specs/SONGBIRD_BEARDOG_INTEGRATION.md)
3. Implement trait interfaces in `crates/songbird-network-federation/src/beardog/`
4. Use mock providers as reference
5. Register with UPA as "security" capability

---

## 💬 Support

**Documentation**: See [DOCS_INDEX.md](DOCS_INDEX.md) for complete index  
**Issues**: Check logs with `RUST_LOG=debug`  
**Architecture Questions**: Review `specs/` directory  
**Integration Questions**: See `showcase/` examples

---

**Last Updated**: December 21, 2025  
**Status**: Production-ready (LAN), Internet foundation ready  
**Quality**: A+ (Zero technical debt)

*Songbird: Sovereign, Privacy-Preserving, Internet-Ready Orchestration* 🎵🌍🔒✨

