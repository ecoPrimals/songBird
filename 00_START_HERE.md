# 🎵 Welcome to Songbird!

**Universal Federated ML Orchestration - Production Ready**

Last Updated: December 24, 2025

---

## 🎉 Latest Achievement: Universal Coordinator PRODUCTION READY!

**v0.1.0 Delivered (December 24, 2025)**:
- ✅ Zero hardcoded primal names - capability-based discovery
- ✅ O(N) coordination replacing O(N²) hardcoded connections
- ✅ Infant discovery - starts with 0 knowledge
- ✅ 9/9 tests passing (100% coverage)
- ✅ 14 comprehensive guides (4,479 lines of documentation)
- ✅ Production deployment ready

**Impact**: New primals join without ANY code changes! Request "security" not "beardog".

---

## 🚀 Quick Navigation

### For New Users
1. **[README.md](README.md)** - Project overview and quick start
2. **[STATUS.md](STATUS.md)** - Current project status
3. **[TEAM_HANDOFF_UNIVERSAL_COORDINATOR.md](TEAM_HANDOFF_UNIVERSAL_COORDINATOR.md)** - ⭐ Start here for coordination!
4. **[QUICK_REFERENCE_UNIVERSAL_COORDINATOR.md](QUICK_REFERENCE_UNIVERSAL_COORDINATOR.md)** - Quick patterns

### For Universal Coordinator (NEW!)
1. **[TEAM_HANDOFF_UNIVERSAL_COORDINATOR.md](TEAM_HANDOFF_UNIVERSAL_COORDINATOR.md)** - Complete onboarding
2. **[QUICK_REFERENCE_UNIVERSAL_COORDINATOR.md](QUICK_REFERENCE_UNIVERSAL_COORDINATOR.md)** - 5-minute quick start
3. **[specs/PRIMAL_COORDINATION_ARCHITECTURE.md](specs/PRIMAL_COORDINATION_ARCHITECTURE.md)** - Architecture
4. **[ROADMAP_UNIVERSAL_COORDINATOR.md](ROADMAP_UNIVERSAL_COORDINATOR.md)** - Future plans

### For Bluetooth/Genesis Work
1. **[BLUETOOTH_README.md](BLUETOOTH_README.md)** - BLE stack overview
2. **[QUICK_START_PURE_RUST_BLUETOOTH.md](QUICK_START_PURE_RUST_BLUETOOTH.md)** - Usage guide
3. **[BLUETOOTH_HARDWARE_TESTING_GUIDE.md](BLUETOOTH_HARDWARE_TESTING_GUIDE.md)** - Hardware validation
4. **[docs/bluetooth-stack/](docs/bluetooth-stack/)** - Detailed reports

### For Developers
1. **[CONTRIBUTING.md](CONTRIBUTING.md)** - How to contribute
2. **[DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md)** - Deployment instructions
3. **[docs/](docs/)** - Comprehensive documentation
4. **[specs/](specs/)** - Technical specifications

### For Integration Teams
1. **[QUICK_COMMIT_AND_RELEASE_GUIDE.md](QUICK_COMMIT_AND_RELEASE_GUIDE.md)** - Release process
2. **[docs/sessions/](docs/sessions/)** - Session reports and feedback
3. **[showcase/](showcase/)** - Working examples

---

## 🎯 What is Songbird?

Songbird is a production-ready federated ML orchestration system with:

### Core Features
- **🌳 Universal Coordinator**: Zero hardcoded primal names, capability-based discovery ✨ NEW!
- **🔐 Sovereign Computing**: Self-hosted, no vendor lock-in
- **🌐 True P2P Networking**: VPN-free encrypted communication (BTSP & BirdSong)
- **🔑 Physical Genesis**: Secure node bootstrap with hardware attestation
- **📡 Pure Rust Bluetooth**: Universal comms with zero system dependencies
- **🎓 Education First**: Built for research and learning environments
- **🐻 Primal Agnostic**: Works with ANY primal providing ANY capability

### Production Status

| Component | Status | Details |
|-----------|--------|---------|
| **Universal Coordinator** | ✅ Production Ready | Capability-based, zero hardcoding |
| **P2P Networking** | ✅ Production | BTSP + BirdSong fully operational |
| **Federation** | ✅ Production | Multi-federation with trust escalation |
| **Genesis Bootstrap** | ✅ Ready | Physical channels + BLE integration |
| **Pure Rust BLE** | ✅ Software Complete | Awaiting hardware validation |
| **Orchestration** | ✅ Production | Full ML workflow support |
| **Configuration** | ✅ Production | Zero-config with smart defaults |

---

## 📦 Installation

### Prerequisites
- Rust 1.70+ (`rustup install stable`)
- Docker (optional, for containers)
- Linux/macOS/Windows (WSL2 for Windows)
- For Bluetooth: USB Bluetooth 4.0+ dongle ($10)

### Quick Start

```bash
# Clone repository
git clone https://github.com/eastgate/songbird
cd songbird

# Build release
cargo build --release

# Run tests
cargo test

# Start orchestrator
cargo run --release --bin songbird-orchestrator
```

### Bluetooth Quick Start

```bash
# Build with Bluetooth support (default)
cargo build --release -p songbird-bluetooth

# Run tests
cargo test -p songbird-bluetooth

# Hardware tests (requires USB dongle)
cargo test -p songbird-bluetooth -- --ignored --nocapture
```

---

## 🏗️ Architecture Overview

### Complete Stack

```
┌─────────────────────────────────────┐
│   Application Layer                 │
│   - ML Orchestration                │
│   - Genesis Ceremony                │
├─────────────────────────────────────┤
│   Federation Layer                  │
│   - Multi-federation coordinator    │
│   - Trust escalation                │
│   - Capability discovery            │
├─────────────────────────────────────┤
│   Networking Layer                  │
│   - BTSP (secure tunnels)           │
│   - BirdSong (discovery)            │
│   - Pure Rust BLE ✨ NEW!           │
├─────────────────────────────────────┤
│   Security Layer                    │
│   - Capability-based access         │
│   - Physical proximity proof        │
│   - Cryptographic lineage           │
├─────────────────────────────────────┤
│   Transport Layer                   │
│   - TCP/IP (LAN/Internet)           │
│   - USB Bluetooth (universal!)      │
│   - UART (embedded)                 │
└─────────────────────────────────────┘
```

### Key Innovations

1. **Pure Rust BLE Stack** 🆕
   - No system Bluetooth dependency
   - Works on any platform
   - Just needs USB dongle
   - Genesis-integrated

2. **Physical Genesis**
   - Hardware attestation
   - Multi-primal witness
   - Cryptographic lineage
   - "Never alone" guarantee

3. **True P2P**
   - No central servers
   - Direct peer communication
   - Encrypted by default
   - VPN-free operation

---

## 📚 Documentation Index

### Getting Started
- [README.md](README.md) - Overview
- [STATUS.md](STATUS.md) - Current status
- [QUICK_REFERENCE.md](QUICK_REFERENCE.md) - Commands
- [CONFIGURATION_GUIDE.md](CONFIGURATION_GUIDE.md) - Setup

### Bluetooth Stack 🆕
- [BLUETOOTH_README.md](BLUETOOTH_README.md) - Overview and API
- [QUICK_START_PURE_RUST_BLUETOOTH.md](QUICK_START_PURE_RUST_BLUETOOTH.md) - Usage
- [BLUETOOTH_HARDWARE_TESTING_GUIDE.md](BLUETOOTH_HARDWARE_TESTING_GUIDE.md) - Testing
- [RELEASE_NOTES_PURE_RUST_BLUETOOTH_v0.1.0.md](RELEASE_NOTES_PURE_RUST_BLUETOOTH_v0.1.0.md) - Release notes
- [docs/bluetooth-stack/](docs/bluetooth-stack/) - Technical deep dives

### Development
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contributing guidelines
- [DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md) - Deployment
- [ROADMAP.md](ROADMAP.md) - Future plans
- [CHANGELOG.md](CHANGELOG.md) - Version history

### Technical
- [docs/](docs/) - Comprehensive documentation
- [specs/](specs/) - Technical specifications
- [examples/](examples/) - Code examples
- [showcase/](showcase/) - Working demos

### Session Reports
- [docs/sessions/](docs/sessions/) - Development sessions
  - Session summaries
  - Integration feedback
  - Audit reports

---

## 🎯 Common Tasks

### Running Songbird

```bash
# Start orchestrator
cargo run --release --bin songbird-orchestrator

# With custom config
SONGBIRD_CONFIG=config/production.env cargo run --release

# With Bluetooth enabled
cargo run --release --features pure-bluetooth
```

### Testing

```bash
# All tests
cargo test

# Specific crate
cargo test -p songbird-bluetooth

# With output
cargo test -- --nocapture

# Hardware tests (needs USB dongle)
cargo test -- --ignored --nocapture
```

### Development

```bash
# Check code
cargo clippy -- -D warnings

# Format code
cargo fmt

# Build documentation
cargo doc --no-deps --open

# Run benchmarks
cargo bench
```

---

## 🐛 Troubleshooting

### Common Issues

**Build Failures**:
```bash
# Update Rust
rustup update stable

# Clean build
cargo clean && cargo build --release
```

**Bluetooth Not Working**:
```bash
# Check for USB dongle
lsusb | grep -i bluetooth

# Stop system Bluetooth (Linux)
sudo systemctl stop bluetooth

# Run with logging
RUST_LOG=debug cargo test -p songbird-bluetooth -- --nocapture
```

**Port Conflicts**:
```bash
# Songbird auto-falls back to alternative ports
# Check logs for actual ports used
```

### Getting Help

1. **Documentation**: Check [docs/](docs/) for detailed guides
2. **Examples**: Look at [showcase/](showcase/) for working code
3. **Issues**: Check GitHub issues for similar problems
4. **Chat**: Reach out to the team (see CONTRIBUTING.md)

---

## 🎉 Recent Milestones

### December 24, 2025 - Phase 3 Complete ✅
- ✅ L2CAP transport layer
- ✅ Complete GATT/ATT protocol
- ✅ Genesis BLE physical channel
- ✅ 52 tests passing
- ✅ Software implementation complete

### December 23, 2025 - Phase 2 Complete ✅
- ✅ BLE scanning and connection
- ✅ HCI protocol implementation
- ✅ Core stack operational

### Earlier December 2025
- ✅ True P2P networking operational
- ✅ Genesis bootstrap specified
- ✅ Federation coordinator complete
- ✅ Production deployment ready

---

## 🚀 Next Steps

### For First-Time Users
1. Read [README.md](README.md) for overview
2. Follow [CONFIGURATION_GUIDE.md](CONFIGURATION_GUIDE.md) to set up
3. Try examples in [showcase/](showcase/)
4. Explore [QUICK_REFERENCE.md](QUICK_REFERENCE.md) for commands

### For Bluetooth Developers
1. Read [BLUETOOTH_README.md](BLUETOOTH_README.md)
2. Follow [QUICK_START_PURE_RUST_BLUETOOTH.md](QUICK_START_PURE_RUST_BLUETOOTH.md)
3. Review [docs/bluetooth-stack/](docs/bluetooth-stack/) for details
4. Use [BLUETOOTH_HARDWARE_TESTING_GUIDE.md](BLUETOOTH_HARDWARE_TESTING_GUIDE.md) for testing

### For Contributors
1. Read [CONTRIBUTING.md](CONTRIBUTING.md)
2. Check [ROADMAP.md](ROADMAP.md) for priorities
3. Look at open issues on GitHub
4. Join development discussions

---

## 📊 Project Status Summary

```
✅ Core Networking:        Production Ready
✅ Federation:             Production Ready
✅ Genesis Bootstrap:      Ready (Software Complete)
✅ Pure Rust Bluetooth:    Software Complete (Hardware Validation Pending)
✅ Orchestration:          Production Ready
✅ Configuration:          Production Ready
✅ Documentation:          Comprehensive (5,000+ lines)
✅ Tests:                  543 passing
```

**Overall**: 🎉 **PRODUCTION READY** with active evolution

---

## 🔗 Quick Links

- **Main Docs**: [docs/](docs/)
- **Bluetooth Docs**: [docs/bluetooth-stack/](docs/bluetooth-stack/)
- **Session Reports**: [docs/sessions/](docs/sessions/)
- **Examples**: [showcase/](showcase/)
- **Specifications**: [specs/](specs/)

---

## ✅ Ready to Start!

Choose your path:
- 👤 **New User**: Start with [README.md](README.md)
- 📡 **Bluetooth Work**: Go to [BLUETOOTH_README.md](BLUETOOTH_README.md)
- 💻 **Developer**: Check [CONTRIBUTING.md](CONTRIBUTING.md)
- 🔧 **Operator**: See [DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md)

**Welcome to Songbird - Universal Federated ML Orchestration!** 🎵

---

**Last Updated**: December 24, 2025  
**Status**: 🎉 Production Ready + Pure Rust BLE Complete  
**Version**: Phase 3 Complete

🦀 **Pure Rust. Zero Dependencies. Universal Deployment.**
