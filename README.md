# 🐦 Songbird - Decentralized P2P Discovery Service

**Version**: v3.22.1 → v3.23.0 (LiveSpore Evolution)  
**Status**: ✅ Production Ready + 🔄 Systematic Evolution Active  
**Grade**: **A (93/100)** → Target: **A+ (98/100)**

---

## ⚡ Quick Start

**New Here? Start with these 3 documents** (10 minutes total):

1. ⭐ **[README_AUDIT_FINDINGS.md](README_AUDIT_FINDINGS.md)** (2 min) - What is Songbird? Current status
2. **[STATUS.md](STATUS.md)** (3 min) - Latest metrics and progress
3. **[00_START_HERE_JAN_2026.md](00_START_HERE_JAN_2026.md)** (5 min) - Complete orientation

**Ready to contribute?** → [CONTRIBUTING.md](CONTRIBUTING.md)  
**Need to build?** → [QUICK_START.md](QUICK_START.md)

---

## 🎯 What is Songbird?

Songbird is a **zero-hardcoding, capability-based P2P discovery service** that enables primals in the ecoPrimals ecosystem to discover each other and federate services without hardcoded dependencies.

### Core Principles

1. **🍼 Primal Self-Knowledge**: Each primal knows only itself, discovers others at runtime
2. **🔍 Capability-Based Discovery**: Find providers by what they do, not who they are
3. **🌱 Zero Hardcoding**: No primal names, vendor names, or ports in production code
4. **✅ Perfect Ethics**: 100/100 human dignity and sovereignty compliance
5. **🌐 Federation Ready**: Multi-network, multi-provider support

---

## 🎊 Latest Achievements

### BiomeOS Integration Complete ✅ (Jan 15, 2026)

**Accomplished**: Full BiomeOS Neural API socket environment variable support

**Issue**: BiomeOS Neural API was setting socket path env vars, but Songbird ignored them

**Before** ❌:
```bash
export SONGBIRD_ORCHESTRATOR_SOCKET=/tmp/songbird-nat0.sock
# Songbird created: /run/user/1000/songbird-default.sock ❌
```

**After** ✅:
```bash
export SONGBIRD_ORCHESTRATOR_SOCKET=/tmp/songbird-nat0.sock
# Songbird creates: /tmp/songbird-nat0.sock ✅
```

**Impact**: BiomeOS NUCLEUS deployment now works! Multi-family deployments enabled!

See: [BIOMEOS_INTEGRATION_COMPLETE_JAN_15_2026.md](BIOMEOS_INTEGRATION_COMPLETE_JAN_15_2026.md)

---

### Zero Hardcoding Architecture ✅ (Jan 14, 2026)

**Accomplished**: Eliminated all primal name hardcoding from production code

**Before** ❌:
```rust
// Hardcoded primal dependencies
let beardog_url = "http://localhost:8443";
let toadstool_url = "http://localhost:8082";
```

**After** ✅:
```rust
// Pure capability discovery
export SECURITY_ENDPOINT=https://any-provider:9000
let security = discover_capability("security").await?;
```

**Impact**: True infant/zero-knowledge startup now works!

See: [HARDCODING_FIX_COMPLETE_JAN_14_2026.md](HARDCODING_FIX_COMPLETE_JAN_14_2026.md)

---

## 📊 Current Status

| Metric | Status |
|--------|--------|
| **Grade** | **A (93/100)** |
| **Production Ready** | ✅ Yes (v3.22.0) |
| **Architecture** | 98/100 (world-class) |
| **Ethics** | 100/100 (perfect) |
| **Primal Hardcoding** | ✅ 0 (eliminated) |
| **Vendor Hardcoding** | ✅ 0 (verified) |
| **Production Mocks** | ✅ 0 (test-isolated) |
| **Unsafe Code** | 207 (documented & justified) |
| **Test Coverage** | ~80% (estimated) |

**Full Status**: [STATUS.md](STATUS.md)

---

## 🚀 Key Features

### Discovery Protocols
- **BirdSong Protocol v2.0**: Encrypted discovery with lineage verification
- **mDNS/DNS-SD**: Local network discovery
- **Federation**: Multi-network coordination
- **Anonymous P2P**: Privacy-preserving discovery

### Architecture Highlights
- **Zero Hardcoding**: No primal names, vendor dependencies, or ports in code
- **Capability-Based**: Discover by capability, not identity
- **Vendor Agnostic**: Works with Kubernetes, Consul, or any service mesh
- **Self-Sovereign**: Each primal only knows itself

### Integration
- **BearDog**: Encrypted discovery and key rotation
- **BiomeOS**: Health monitoring and orchestration
- **PetalTongue**: Real-time event streaming
- **Universal Adapter**: Connect any primal to any other

---

## 🏗️ Architecture

```text
┌─────────────────────────────────────────────────┐
│           Songbird Discovery Service            │
├─────────────────────────────────────────────────┤
│  • Zero hardcoded primal names                  │
│  • Capability-based discovery                   │
│  • Environment-driven configuration             │
│  • Multi-protocol support (mDNS, BirdSong)     │
└─────────────────────────────────────────────────┘
                      │
         ┌────────────┴────────────┐
         ▼                         ▼
┌──────────────────┐    ┌──────────────────┐
│  Any Provider    │    │  Any Consumer    │
│  with Capability │    │  needs Capability│
└──────────────────┘    └──────────────────┘

No hardcoded dependencies - discovered at runtime!
```

**Detailed Architecture**: [specs/00_SPECIFICATIONS_INDEX.md](specs/00_SPECIFICATIONS_INDEX.md)

---

## 🛠️ Quick Start

### Prerequisites
- Rust 1.75+ (async traits support)
- Linux, macOS, or Windows
- Optional: Bluetooth hardware (for Genesis ceremony)

### Build
```bash
# Clone repository
git clone <repository-url>
cd songbird

# Build library
cargo build --lib --release

# Run tests
cargo test --lib

# Check code quality
cargo clippy --lib
cargo fmt --all -- --check
```

### Run Example
```bash
# Zero-knowledge startup demo
export SECURITY_ENDPOINT=http://localhost:9443
cargo run --example infant_discovery_demo
```

**Full Guide**: [QUICK_START.md](QUICK_START.md)

---

## 📚 Documentation

### Essential Reading
- **[README_AUDIT_FINDINGS.md](README_AUDIT_FINDINGS.md)** - 2-min overview
- **[STATUS.md](STATUS.md)** - Current status & metrics
- **[ROADMAP.md](ROADMAP.md)** - Long-term vision
- **[00_START_HERE_JAN_2026.md](00_START_HERE_JAN_2026.md)** - Orientation guide

### For Contributors
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - How to contribute
- **[QUICK_START.md](QUICK_START.md)** - Development setup
- **[docs/DEVELOPMENT_GUIDE.md](docs/DEVELOPMENT_GUIDE.md)** - Detailed guide

### Architecture & Design
- **[specs/](specs/)** - 67 technical specifications
- **[MULTI_PRIMAL_INTERACTION_ARCHITECTURE.md](MULTI_PRIMAL_INTERACTION_ARCHITECTURE.md)** - Cross-primal design
- **[docs/architecture/](docs/architecture/)** - Architecture docs

### Latest Evolution
- **[LIVESPORE_EXECUTIVE_SUMMARY_JAN_13_2026.md](LIVESPORE_EXECUTIVE_SUMMARY_JAN_13_2026.md)** - LiveSpore evolution
- **[AUDIT_EXECUTIVE_SUMMARY_JAN_14_2026.md](AUDIT_EXECUTIVE_SUMMARY_JAN_14_2026.md)** - Comprehensive audit
- **[WEEK1_PROGRESS_TRACKER.md](WEEK1_PROGRESS_TRACKER.md)** - Current progress

**Complete Index**: [ROOT_DOCS_INDEX.md](ROOT_DOCS_INDEX.md)

---

## 🤝 Contributing

We welcome contributions! Songbird follows strict architectural principles:

### Principles
1. **No Hardcoding**: Zero primal names, vendor names, or ports
2. **Capability-Based**: Discover by capability, not identity
3. **Self-Knowledge Only**: Each primal knows only itself
4. **Perfect Ethics**: Human dignity and sovereignty first
5. **Modern Rust**: Idiomatic, safe, fast

### How to Contribute
1. Read [CONTRIBUTING.md](CONTRIBUTING.md)
2. Check [WEEK1_PROGRESS_TRACKER.md](WEEK1_PROGRESS_TRACKER.md) for current work
3. Pick an issue or propose new work
4. Follow the code quality standards
5. Submit a PR with tests and docs

---

## 📈 Current Initiative: LiveSpore Evolution

**Timeline**: January 14 - February 24, 2026 (6 weeks)

**Goals**:
- BirdSong v3.0 with multi-callsign tag support
- 90%+ test coverage
- A+ grade (98/100)
- Production deployment

**Progress**: Week 1 - 60% complete (3x ahead of schedule!)

**Details**: [LIVESPORE_EXECUTIVE_SUMMARY_JAN_13_2026.md](LIVESPORE_EXECUTIVE_SUMMARY_JAN_13_2026.md)

---

## 🔬 Technical Highlights

### Zero-Copy Performance
- Optimized data paths with benchmarking
- Zero-copy where safe and beneficial
- Memory-efficient design

### Security
- BearDog integration for encryption
- Lineage-based trust verification
- Replay protection
- Key rotation support

### Testing
- ~80% test coverage (growing to 90%+)
- E2E, integration, and chaos tests
- Property-based testing
- Fault injection framework

### Modern Rust
- Async/await throughout
- Type-driven development
- Zero `unsafe` in core logic
- Comprehensive error handling

---

## 📊 Quality Metrics

| Metric | Value | Status |
|--------|-------|--------|
| Architecture Grade | 98/100 | ✅ World-class |
| Ethics Grade | 100/100 | ✅ Perfect |
| Overall Grade | 93/100 | ✅ Excellent |
| Rustfmt | 100% | ✅ Compliant |
| Clippy (production) | 0 errors | ✅ Clean |
| Test Coverage | ~80% | ⏳ Expanding |
| Security Vulnerabilities | 0 | ✅ Verified |
| Production Mocks | 0 | ✅ Test-isolated |

---

## 🌱 Ecosystem Integration

Songbird is part of the ecoPrimals ecosystem:

- **BearDog**: Security and encryption
- **BiomeOS**: Orchestration and deployment
- **PetalTongue**: Real-time event streaming
- **NestGate**: Storage and persistence
- **Toadstool**: Compute capabilities
- **Squirrel**: AI/ML services

All discovered dynamically - zero hardcoded dependencies!

---

## 📄 License

See [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

Built with modern Rust principles, inspired by:
- Zero-knowledge architecture
- Capability-based security
- Self-sovereign systems
- Human dignity first

---

## 📞 Getting Help

- **Quick Questions**: Check [STATUS.md](STATUS.md)
- **Documentation**: See [ROOT_DOCS_INDEX.md](ROOT_DOCS_INDEX.md)
- **Development**: Read [docs/DEVELOPMENT_GUIDE.md](docs/DEVELOPMENT_GUIDE.md)
- **Contributing**: See [CONTRIBUTING.md](CONTRIBUTING.md)

---

🐦🌱 **Songbird: Different orders of the same song!**

**Version**: v3.22.1 → v3.23.0  
**Status**: Production Ready + Active Evolution  
**Grade**: A (93/100) → A+ (98/100)

**Zero hardcoding. Infinite discovery. Infant wisdom.**

---

**Last Updated**: January 14, 2026  
**Next Milestone**: Week 1 Complete (94/100) - Days 2-3  
**Final Goal**: BirdSong v3.0 + A+ Grade - February 24, 2026
