# 🎵 **Songbird Universal Network Orchestrator**

<div align="center">

**🌱 ecoPrimals | Universal Network Orchestration Platform**

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Build Status](https://img.shields.io/badge/Build-Passing-brightgreen.svg)](https://github.com/ecoPrimals/songbird)
[![Performance](https://img.shields.io/badge/Architecture-Universal%20%26%20Agnostic-blue.svg)](https://github.com/ecoPrimals/songbird)

*Universal orchestration platform for any network service, application, or biome*

[🚀 Quick Start](#-quick-start) • [🏗️ Architecture](#️-architecture) • [📊 API Reference](#-api-reference) • [🤝 Primals](#-universal-primal-coordination) • [🌐 BYOB](#-bring-your-own-biome)

</div>

---

## 🎯 **What is Songbird?**

Songbird is a **universal network orchestration platform** that coordinates and manages any type of networked service, application, or system. Built with a **future-proof, primal-agnostic architecture**, Songbird seamlessly integrates with any ecosystem while maintaining complete operational independence.

### ⚡ **Core Capabilities**

- 🌐 **Universal Orchestration**: Manages any service, anywhere, anytime
- 🤝 **Primal Coordination**: Works with Toadstool, NestGate, BearDog, Squirrel, and any future Primal
- 🎮 **Gaming Bridge**: Legacy LAN gaming over modern internet infrastructure  
- 📦 **BYOB (Bring Your Own Biome)**: Deploy complete environments from YAML manifests
- 🔄 **Auto-Discovery**: Intelligent network scanning and service detection
- 📊 **Real-time Monitoring**: Built-in health checks, metrics, and observability
- 🛡️ **Security-First**: End-to-end encryption with BearDog integration
- ⚡ **High Performance**: <1ms coordination latency, 100+ concurrent sessions

### 🏆 **Production Ready**

```
✅ Build Status: Clean compilation (0 errors)
✅ Test Coverage: Comprehensive integration tests
✅ Documentation: Complete API reference + guides
✅ Performance: Benchmarked and optimized
✅ Architecture: Universal and future-proof
```

---

## 🚀 **Quick Start**

### 🎯 **Universal Deployment (30 seconds)**

```bash
# Clone Songbird
git clone https://github.com/ecoPrimals/songbird.git
cd songbird

# Build and start orchestrator
cargo build --release
cargo run --bin songbird-orchestrator

# Deploy a biome from manifest
songbird compose deploy examples/basic-biome.yaml

# Check status
songbird status
```

### 🌐 **BYOB (Bring Your Own Biome)**

Create a `biome.yaml` manifest:

```yaml
metadata:
  name: "my-awesome-biome"
  version: "1.0.0"
  description: "Full-stack web application"

services:
  web-server:
    endpoint: "http://localhost:3000"
    health_check:
      endpoint: "/health"
      interval_secs: 30
    depends_on: ["database"]
  
  database:
    endpoint: "postgresql://localhost:5432"
    primal_managed: "nestgate"

networking:
  discovery:
    method: "mDNS"
  ports: [3000, 5432]

primals:
  toadstool:
    enabled: true
    capabilities: ["orchestration", "deployment"]
  nestgate:
    enabled: true  
    capabilities: ["storage", "data"]
  beardog:
    enabled: true
    capabilities: ["security", "authentication"]
```

Deploy instantly:
```bash
songbird compose deploy biome.yaml
```

---

## 🤝 **Universal Primal Coordination**

Songbird's **revolutionary universal architecture** works with any Primal without requiring code changes.

### 🔮 **Current Primal Support**

| Primal | Capabilities | Status | Integration |
|--------|-------------|--------|-------------|
| **Toadstool** | Orchestration, Deployment | ✅ Production | Auto-discovery |
| **NestGate** | Storage, Data Management | ✅ Production | Auto-discovery |
| **BearDog** | Security, Authentication | ✅ Production | Auto-discovery |
| **Squirrel** | AI, Machine Learning | ✅ Production | Auto-discovery |
| **Future Primals** | Any Capability | ✅ Ready | Zero Code Changes |

### ⚡ **Auto-Discovery Magic**

```bash
# Songbird automatically finds and coordinates with available Primals
curl -X POST http://localhost:8080/api/v1/coordinate/all

# Response shows all discovered Primals and their capabilities
{
  "coordination_results": {
    "toadstool": {
      "status": "success",
      "endpoint": "http://toadstool:8080",
      "capabilities": ["orchestration", "deployment"]
    },
    "nestgate": {
      "status": "success", 
      "endpoint": "http://nestgate-storage:8080",
      "capabilities": ["storage", "data"]
    }
  }
}
```

---

## 🎮 **Gaming Bridge Excellence**

### **Legacy Gaming Made Modern**

Songbird includes a specialized gaming bridge for bringing legacy LAN games to the modern internet:

```bash
# Create a gaming session
curl -X POST http://localhost:8080/api/v1/gaming/sessions \
  -d '{"game_name": "StarCraft", "max_players": 8}'

# Returns: {"session_id": "game-123", "join_code": "ZERG42"}
```

**Supported Games:**
- ✅ StarCraft (IPX/UDP)
- ✅ Age of Empires II (DirectPlay)
- ✅ Warcraft II (IPX) 
- ✅ Diablo (IPX)
- ✅ Command & Conquer (NetBIOS)
- ✅ Any LAN game (Universal UDP/TCP bridge)

---

## 📊 **API Reference**

### **REST API**

```bash
# Service Management
GET    /api/v1/services              # List all services
POST   /api/v1/services              # Register new service
GET    /api/v1/services/{id}         # Get service details
DELETE /api/v1/services/{id}         # Unregister service

# Universal Primal Coordination  
POST   /api/v1/coordinate            # Coordinate with specific Primal
POST   /api/v1/coordinate/all        # Coordinate with all Primals
GET    /api/v1/coordinate/primals    # List available Primals

# BYOB Deployment
POST   /api/v1/byob/deploy           # Deploy biome manifest
GET    /api/v1/byob/deployments      # List deployments
GET    /api/v1/byob/deployments/{id} # Get deployment status
DELETE /api/v1/byob/deployments/{id} # Stop deployment

# Gaming Bridge
POST   /api/v1/gaming/sessions       # Create gaming session
GET    /api/v1/gaming/sessions       # List active sessions
DELETE /api/v1/gaming/sessions/{id}  # Close session

# Health & Monitoring
GET    /api/v1/health                # System health
GET    /api/v1/metrics               # Prometheus metrics
GET    /api/v1/status                # Detailed status
```

### **WebSocket API**

```javascript
// Real-time updates
const ws = new WebSocket('ws://localhost:8080/api/v1/ws');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  
  switch(data.type) {
    case 'service_status_changed':
      console.log(`Service ${data.service_id} is now ${data.new_status}`);
      break;
    case 'primal_coordination_success':
      console.log(`Successfully coordinated with ${data.primal_name}`);
      break;
    case 'gaming_session_created':
      console.log(`New gaming session: ${data.session_id}`);
      break;
  }
};
```

---

## 🏗️ **Architecture**

### **Universal Design Principles**

```
Songbird Universal Orchestration Architecture

┌─────────────────────────────────────────────────┐
│                Songbird Core                    │
├─────────────────────────────────────────────────┤
│  Universal Coordination Engine                  │
│  ├─ Auto-Discovery System                       │
│  ├─ Primal-Agnostic Interface                   │
│  ├─ Future-Proof Protocol Adaptation            │
│  └─ Zero-Configuration Integration               │
├─────────────────────────────────────────────────┤
│              Service Registry                   │
│  ├─ BYOB Manifest Processing                    │
│  ├─ Service Lifecycle Management                │
│  ├─ Health Monitoring & Circuit Breakers        │
│  └─ Real-time Status & Metrics                  │
├─────────────────────────────────────────────────┤
│               Network Bridge                    │
│  ├─ Gaming Protocol Translation                 │
│  ├─ Universal NAT Traversal                     │
│  ├─ Security Layer (BearDog Integration)        │
│  └─ Performance Optimization                    │
└─────────────────────────────────────────────────┘
           │                    │
    ┌──────▼──────┐      ┌──────▼──────┐
    │  Any Primal │      │ Any Service │
    │             │      │             │
    │ Toadstool   │◄────►│ Gaming      │
    │ NestGate    │      │ Web Apps    │ 
    │ BearDog     │      │ Databases   │
    │ Squirrel    │      │ APIs        │
    │ Future...   │      │ Any Biome   │
    └─────────────┘      └─────────────┘
```

### **🌟 Key Architectural Features**

- **🔄 Universal Coordination**: Works with any Primal through capability-based routing
- **🎯 Auto-Discovery**: Finds services and Primals automatically via mDNS, network scanning, environment variables
- **📦 BYOB Support**: Complete biome deployment from YAML manifests  
- **🔌 Protocol Agnostic**: HTTP/REST, WebSocket, gaming protocols, custom protocols
- **⚡ High Performance**: Async-first Rust architecture with <1ms coordination latency
- **🛡️ Security Integration**: Native BearDog security with end-to-end encryption
- **📊 Observability**: Built-in metrics, health checks, and real-time monitoring

---

## 🛠 **Development**

### 🔧 **Build from Source**

```bash
# Prerequisites: Rust 1.75+
git clone https://github.com/ecoPrimals/songbird.git
cd songbird

# Build all components
cargo build --release --workspace

# Run comprehensive tests
cargo test --workspace

# Run performance benchmarks
cargo run --bin benchmark-suite
```

### 📈 **Performance Benchmarks**

```bash
# Run built-in benchmarks
cargo run --example performance_demo

# Expected results:
# 🚀 HashMap Operations: 2,500,000+ ops/sec
# ⚡ Async Task Spawning: 1,000+ concurrent tasks
# 📊 JSON Serialization: 500,000+ ops/sec
# 🤝 Primal Coordination: <1ms latency
```

### 🧪 **Integration Testing**

```bash
# Comprehensive integration tests
cargo test --workspace --test integration_tests

# Specific test categories:
cargo test federation_workflow      # Test federation functionality
cargo test primal_coordination     # Test universal Primal integration
cargo test gaming_bridge           # Test gaming bridge features
cargo test byob_deployment         # Test BYOB deployment system
```

---

## 🌍 **Configuration**

### **Environment Variables**

```bash
# Core Configuration
SONGBIRD_API_HOST=localhost
SONGBIRD_API_PORT=8080
SONGBIRD_LOG_LEVEL=info

# Discovery Configuration  
SONGBIRD_DISCOVERY_ENABLED=true
SONGBIRD_DISCOVERY_PORTS=8080,8081,8082
SONGBIRD_DISCOVERY_TIMEOUT_MS=500

# Primal Configuration
SONGBIRD_PRIMALS_AUTO_DISCOVER=true
TOADSTOOL_ENDPOINT=http://toadstool:8080
NESTGATE_ENDPOINT=http://nestgate:8080
BEARDOG_ENDPOINT=http://beardog:8080

# Gaming Configuration
SONGBIRD_GAMING_ENABLED=true
SONGBIRD_GAMING_PORT_RANGE=25565-25575
```

### **Configuration File (`songbird.toml`)**

```toml
[api]
host = "0.0.0.0"
port = 8080

[discovery]
enabled = true
ports = [8080, 8081, 8082, 8083, 8084, 8085]
timeout_ms = 500

[primals]
auto_discover = true

[primals.toadstool]
enabled = true
endpoint = "http://toadstool:8080"
capabilities = ["orchestration", "deployment"]

[primals.nestgate]
enabled = true
endpoint = "http://nestgate:8080"
capabilities = ["storage", "data"]

[gaming]
enabled = true
port_range = "25565-25575"
protocols = ["ipx", "directplay", "netbios", "udp"]
```

---

## 🤝 **Contributing**

We welcome contributions to the **universal orchestration platform**!

### 🎯 **Ways to Contribute**

- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/ecoPrimals/songbird/issues)
- 🌐 **New Primal Integration**: Add support for new Primals
- ⚡ **Performance**: Optimize coordination and orchestration
- 📝 **Documentation**: Improve guides and examples
- 🧪 **Testing**: Add test coverage for new scenarios
- 🎮 **Gaming Protocols**: Add support for more legacy games

### 📋 **Development Setup**

```bash
# Fork and clone
git clone https://github.com/YOUR-USERNAME/songbird.git
cd songbird

# Create feature branch
git checkout -b feature/amazing-feature

# Make changes, test, commit
cargo test --workspace
cargo fmt
cargo clippy

git commit -am "Add amazing universal feature"

# Push and create PR
git push origin feature/amazing-feature
```

---

## 📞 **Support & Community**

### 🌟 **Community Resources**

- **📖 Documentation**: [Complete API Reference](docs/AI_API_REFERENCE.md)
- **💬 Community**: [GitHub Discussions](https://github.com/ecoPrimals/songbird/discussions)
- **🐛 Issues**: [Bug Reports](https://github.com/ecoPrimals/songbird/issues)
- **📊 Status**: [Project Roadmap](specs/IMPLEMENTATION-ROADMAP.md)

### 🏢 **Enterprise & Primal Integration**

- **🤝 Primal Partnerships**: partnerships@ecoprimals.dev
- **🏢 Enterprise Support**: enterprise@ecoprimals.dev
- **🔧 Custom Integration**: integrations@ecoprimals.dev

---

## 📜 **License**

**AGPL 3.0** - Universal orchestration capabilities free forever.

- ✅ **Universal Orchestration**: Free for all uses
- ✅ **Primal Coordination**: Free for all Primals
- ✅ **BYOB Deployment**: Free for all biomes
- ✅ **Gaming Bridge**: Free for all games
- ✅ **API & Integration**: Free for all developers

See [LICENSE](LICENSE) for complete details.

---

<div align="center">

**🌐 Universal Orchestration for the Decentralized Future 🚀**

**Made with ❤️ by ecoPrimals | Universal Platform for All Primals**

*One orchestrator, infinite possibilities. Welcome to the future of network coordination.*

</div> 