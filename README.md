# 🎵 Songbird - Port-Free P2P Orchestrator

**Version**: v3.18.2  
**Status**: Production Ready ✅  
**Architecture**: Modern Idiomatic Rust  

---

## 🎯 What is Songbird?

**Songbird** is a **port-free, encrypted P2P orchestrator** for distributed systems. It discovers peers via UDP multicast, establishes trust through genetic lineage, and creates encrypted tunnels (BTSP) for communication - all without requiring TCP port exposure.

### Key Features

- 🔐 **Port-Free**: Only UDP multicast (239.255.42.99:4242) - no TCP ports required
- 🔒 **Encrypted by Default**: BTSP tunnels provide end-to-end encryption
- 🌐 **NAT Traversal**: Automatic via BirdSong genetic lineage
- 📡 **Zero Configuration**: Auto-discovery via UDP multicast
- 🧬 **Genetic Trust**: Progressive trust levels based on cryptographic lineage
- 🦅 **Fractal Coordination**: Albatross (HPC) → Songbird (Tower) → Sparrow (Edge)

---

## 🚀 Quick Start

### Installation

```bash
# Clone repository
git clone https://github.com/ecoPrimals/songBird.git
cd songbird

# Build release binary
cargo build --release

# Binary location
./target/release/songbird-orchestrator
```

### Configuration

Songbird is configured via environment variables:

```bash
# Required
export SONGBIRD_NODE_ID="tower-001"
export SONGBIRD_FAMILY_ID="family-abc123"

# Optional
export SONGBIRD_DISCOVERY_PORT="4242"
export SONGBIRD_HTTP_PORT="8080"
export SECURITY_PROVIDER_SOCKET="unix:///var/run/beardog.sock"
```

### Running

```bash
# Start Songbird
songbird-orchestrator

# Or with systemd
systemctl start songbird
```

---

## 📚 Documentation

### Getting Started
- [00_START_HERE.md](./00_START_HERE.md) - Start here for new users
- [CHANGELOG.md](./CHANGELOG.md) - Version history
- [CONTRIBUTING.md](./CONTRIBUTING.md) - How to contribute

### Architecture
- [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) - System architecture
- [MULTI_PRIMAL_INTERACTION_ARCHITECTURE.md](./MULTI_PRIMAL_INTERACTION_ARCHITECTURE.md) - Multi-primal communication
- [specs/LIFECYCLE_ORCHESTRATION_EVOLUTION.md](./specs/LIFECYCLE_ORCHESTRATION_EVOLUTION.md) - Lifecycle vision

### Integration Guides
- [IPC_INTEGRATION_GUIDE.md](./IPC_INTEGRATION_GUIDE.md) - IPC integration
- [NESTGATE_INTEGRATION_GUIDE.md](./NESTGATE_INTEGRATION_GUIDE.md) - NestGate integration
- [NEURALAPI_INTEGRATION_PROGRESS.md](./NEURALAPI_INTEGRATION_PROGRESS.md) - NeuralAPI integration

### Latest Evolution (v3.18.x)
- [BTSP_CONNECTION_COMPLETE_V3_18_0.md](./BTSP_CONNECTION_COMPLETE_V3_18_0.md) - BTSP implementation complete
- [DEEP_DEBT_FIX_V3_18_2.md](./DEEP_DEBT_FIX_V3_18_2.md) - Architectural refactoring
- [BIOMEOS_HANDOFF_V3_17_0.md](./BIOMEOS_HANDOFF_V3_17_0.md) - biomeOS integration

### Archived Documentation
- [docs/archive/](./docs/archive/) - Historical documentation

---

## 🏗️ Architecture

### Components

```
┌─────────────────────────────────────────────┐
│ Songbird Orchestrator                       │
│                                             │
│  ┌─────────────┐      ┌──────────────┐    │
│  │  Discovery  │◄────►│  Connection  │    │
│  │  (UDP 4242) │      │   Manager    │    │
│  └─────────────┘      └──────────────┘    │
│         │                      │           │
│         ▼                      ▼           │
│  ┌─────────────┐      ┌──────────────┐    │
│  │   Trust     │      │     BTSP     │    │
│  │  Evaluation │      │   Tunnels    │    │
│  └─────────────┘      └──────────────┘    │
│         │                      │           │
│         └──────────┬───────────┘           │
│                    ▼                        │
│          ┌──────────────────┐              │
│          │   Federation     │              │
│          │      State       │              │
│          └──────────────────┘              │
└─────────────────────────────────────────────┘
```

### Connection Flow

1. **Discovery**: Peer broadcasts UDP multicast on 239.255.42.99:4242
2. **Trust Evaluation**: Security provider (e.g., BearDog) validates genetic lineage
3. **Connection**: 
   - **BTSP preferred**: Encrypted tunnel (port-free, NAT traversal)
   - **HTTPS fallback**: If BTSP unavailable
4. **Federation**: Peers join federation with progressive trust levels

---

## 🔐 Security

### Trust Levels

| Level | Name | Description | Allowed Operations |
|-------|------|-------------|-------------------|
| 0 | None | Untrusted | Nothing |
| 1 | Limited | Same genetic family | BirdSong coordination, health checks |
| 2 | Elevated | Human-approved | Federation, read-only data access |
| 3 | Highest | Human entropy (USB seed) | All operations |

### Security Architecture

- **Genetic Lineage**: Cryptographic family relationships
- **BTSP Encryption**: End-to-end encrypted tunnels
- **Progressive Trust**: Automatic escalation based on behavior
- **Zero Hardcoding**: No vendor names, runtime discovery only

---

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific package tests
cargo test --package songbird-orchestrator

# Run with coverage
cargo tarpaulin --out Html

# Run lints
cargo clippy --all-targets --all-features
```

### Test Coverage

- **Unit Tests**: 568 tests (100% passing)
- **Integration Tests**: Full discovery and federation flows
- **E2E Tests**: Multi-tower scenarios

---

## 🔧 Development

### Prerequisites

- Rust 1.75+ (stable)
- Linux (primary), macOS (supported), Windows (partial)
- Security provider (BearDog v0.15.0+) for BTSP

### Project Structure

```
songbird/
├── crates/
│   ├── songbird-orchestrator/   # Main orchestrator
│   ├── songbird-discovery/      # UDP multicast discovery
│   ├── songbird-universal/      # Universal adapter (BTSP, RPC)
│   ├── songbird-types/          # Shared types
│   ├── songbird-network-federation/  # Federation logic
│   └── songbird-cli/            # CLI tool
├── docs/                        # Documentation
├── specs/                       # Specifications
├── tests/                       # Integration tests
└── examples/                    # Usage examples
```

### Build Options

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# With all features
cargo build --all-features

# Specific package
cargo build --package songbird-orchestrator
```

---

## 🎯 Roadmap

### Current (v3.18.x)

- ✅ BTSP-first connection strategy
- ✅ Port-free architecture (UDP only)
- ✅ Graceful shutdown (zombie detection)
- ✅ Modern idiomatic Rust

### Next (v3.19.0)

- 🔄 Bidirectional BTSP communication
- 🔄 E2E BTSP testing with BearDog
- 🔄 Performance optimization
- 🔄 Enhanced observability

### Future

- 📋 Albatross mitosis (HPC scaling)
- 📋 Cloud-like migration
- 📋 Nested fractal coordination
- 📋 Enhanced UI/UX for monitoring

See [ROADMAP.md](./ROADMAP.md) for details.

---

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

### Quick Guide

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Run lints (`cargo clippy`)
6. Commit (`git commit -m 'feat: add amazing feature'`)
7. Push (`git push origin feature/amazing-feature`)
8. Open a Pull Request

---

## 📊 Status

### Build Status

| Component | Status | Tests | Coverage |
|-----------|--------|-------|----------|
| songbird-orchestrator | ✅ Passing | 466/466 | 85% |
| songbird-discovery | ✅ Passing | 52/52 | 90% |
| songbird-universal | ✅ Passing | 34/34 | 88% |
| songbird-types | ✅ Passing | 16/16 | 95% |

### Production Readiness

- ✅ All tests passing (568/568)
- ✅ Zero unsafe code
- ✅ Comprehensive error handling
- ✅ Graceful shutdown
- ✅ Production deployed (biomeOS)

---

## 📖 Key Concepts

### BirdSong Protocol

BirdSong is the P2P discovery and communication protocol:
- **Discovery**: UDP multicast on 239.255.42.99:4242
- **Contact Exchange**: Via genetic lineage
- **BTSP**: Encrypted tunnels for data transfer

### Genetic Lineage

Cryptographic family relationships enable:
- Trust evaluation
- NAT traversal (ask family for contact info)
- Progressive trust escalation

### Fractal Coordination

Three tiers of the same binary:
- **Albatross**: HPC clusters (high-capacity)
- **Songbird**: Tower orchestration (mid-tier)
- **Sparrow**: Edge devices (lightweight)

---

## 🐛 Troubleshooting

### Common Issues

**Issue**: Songbird exits immediately  
**Solution**: Check logs in `/tmp/primals/` - see [DEEP_DEBT_FIX_V3_18_2.md](./DEEP_DEBT_FIX_V3_18_2.md)

**Issue**: No peers discovered  
**Solution**: Verify UDP multicast: `ss -u -a | grep 4242`

**Issue**: BTSP tunnels fail  
**Solution**: Ensure security provider (BearDog) is running: `ss -x | grep beardog`

**Issue**: Port already in use  
**Solution**: Check for zombie processes: `ps aux | grep songbird`

---

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- **biomeOS Team** - Integration testing and feedback
- **BearDog Team** - Security provider implementation
- **ecoPrimals Community** - Vision and support

---

## 📞 Contact

- **Repository**: https://github.com/ecoPrimals/songBird
- **Issues**: https://github.com/ecoPrimals/songBird/issues
- **Discussions**: https://github.com/ecoPrimals/songBird/discussions

---

**Songbird** - *Port-free P2P orchestration for distributed systems*

🎵 **Security from cryptography, not port obscurity** 🎵
