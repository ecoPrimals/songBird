# 🎭 Songbird Showcase Index

**Last Updated:** December 17, 2025

---

## 📚 Available Showcases

### 1. **Isolated Mode** (`01-isolated/`)
**Status:** ✅ Operational  
**Description:** Single tower demos showing core orchestration capabilities

- Service discovery and registration
- Health monitoring
- Resource management
- Local API access

### 2. **Federation Mode** (`02-federation/`)
**Status:** ✅ Operational  
**Description:** Multi-tower LAN federation with sovereign security

- Tower-to-tower discovery
- Cross-tower communication
- Federation heartbeats
- Sub-millisecond latency verified

**Notable:** `SOVEREIGN_SECURITY_READY.md` - Security architecture documented

### 3. **Inter-Primal Integration** (`03-inter-primal/`)
**Status:** 🚧 In Progress  
**Description:** Songbird + Toadstool collaboration

- Distributed ML training
- Friend joins LAN (zero-config)
- Compute bridge
- GPU-aware task routing

### 4. **Multi-Protocol Federation** (`04-multi-protocol/`) 🆕
**Status:** ✅ Ready for Testing  
**Description:** Protocol escalation and performance comparison

**Quick Start:** `QUICK_START.md` (5-minute demo)

**Features:**
- 7 protocols (HTTP, HTTPS, JSON-RPC, tarpc, WebSocket, WSS, BTSP)
- Protocol discovery and negotiation
- HTTP → JSON-RPC → tarpc escalation
- 100x performance improvement (tarpc vs HTTP)
- Tower-to-tower with multiple protocols
- Real latency measurements

**Scripts:**
- `start_tower_a.sh` - Launch primary tower
- `start_tower_b.sh` - Launch secondary tower (optional)
- `demo_protocol_escalation.sh` - Interactive demo
- `test_protocol_escalation.sh` - Automated test

**Expected Results:**
- HTTP: 5-10ms latency
- JSON-RPC: 2-3ms latency (2-3x speedup)
- tarpc: ~0.05ms latency (100x speedup)

---

## 🚀 Quick Test

### Fastest Demo (5 minutes)
```bash
# Terminal 1: Start tower
./showcase/04-multi-protocol/start_tower_a.sh

# Terminal 2: Run demo
./showcase/04-multi-protocol/demo_protocol_escalation.sh
```

### Federation Demo (10 minutes)
```bash
# Terminal 1: Tower A
./showcase/02-federation/start_tower_a.sh

# Terminal 2: Tower B
./showcase/02-federation/start_tower_b.sh

# Terminal 3: Test
./showcase/02-federation/test_federation.sh
```

---

## 📊 Showcase Progression

```
Isolated (01) → Federation (02) → Inter-Primal (03) → Multi-Protocol (04)
     ↓               ↓                   ↓                    ↓
Single Tower    Multi-Tower        + Toadstool       + Protocol Escalation
                  (LAN)            (ML Training)     (100x Performance)
```

---

## 🎯 Capabilities Matrix

| Feature                   | 01 | 02 | 03 | 04 |
|---------------------------|----|----|----|----|
| Single Tower              | ✅ | ✅ | ✅ | ✅ |
| Multi-Tower Federation    | ❌ | ✅ | ✅ | ✅ |
| Inter-Primal (Toadstool)  | ❌ | ❌ | ✅ | ✅ |
| Multi-Protocol            | ❌ | ❌ | ❌ | ✅ |
| TLS/HTTPS                 | ❌ | ❌ | ❌ | ✅ |
| JSON-RPC API              | ❌ | ❌ | ❌ | ✅ |
| tarpc (High-Perf)         | ❌ | ❌ | ❌ | ✅ |
| Protocol Negotiation      | ❌ | ❌ | ❌ | ✅ |
| BTSP Interface            | ❌ | ❌ | ❌ | ✅ |

---

## 🔮 Coming Soon

### 5. **BearDog Integration** (Planned)
- BTSP genetic cryptography
- Key lineage tracking
- Multi-party key renewal
- VPN-free encryption

### 6. **Cross-Internet Federation** (Planned)
- WAN connectivity
- Tailscale/WireGuard demos
- Global tower network
- Latency optimization

### 7. **Production Deployment** (Planned)
- Kubernetes manifests
- Docker Compose stacks
- Terraform infrastructure
- CI/CD pipelines

---

## 📚 Related Documentation

- **Architecture:** `../docs/MULTI_PROTOCOL_FEDERATION_PLAN.md`
- **APIs:** `../docs/JSONRPC_GUIDE.md`, `../docs/BTSP_INTERFACE_GUIDE.md`
- **Deployment:** `../docs/DEPLOYMENT_GUIDE_MULTI_PROTOCOL.md`
- **Security:** `02-federation/SOVEREIGN_SECURITY_READY.md`

---

## 🆘 Troubleshooting

### Port Conflicts
```bash
# Check what's using ports
lsof -i :8080
lsof -i :8081

# Use different ports
export SONGBIRD_PORT=7080
export SONGBIRD_TARPC_PORT=7081
```

### Scripts Not Executable
```bash
chmod +x showcase/**/*.sh
```

### Dependencies Missing
```bash
# Ubuntu/Debian
sudo apt install curl jq

# macOS
brew install curl jq
```

---

**Status:** ✅ 4 showcases ready, 3 more planned  
**Latest:** Multi-Protocol Federation (04) - December 17, 2025  
**Next:** BearDog Integration (05)

