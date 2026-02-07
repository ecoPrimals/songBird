# 🐦 Songbird - Network Orchestration & Discovery Primal

**Version**: v3.34.0  
**Status**: ✅ **WORLD-CLASS** - Phase 2A Tor Protocol Complete  
**License**: AGPL-3.0  
**Deep Debt**: **S+ Tier** (100% BearDog Delegation + Pure Rust Tor)

Songbird is the universal network orchestrator for the ecoPrimals ecosystem, managing service discovery, connection management, and inter-primal communication.

## 🏆 Quality Achievements

| Metric | Status | Achievement |
|--------|--------|-------------|
| **Deep Debt Score** | ✅ **S Tier** | 100% BearDog delegation (TRUE PRIMAL) |
| **Safe Rust** | ✅ **100%** | Zero `unsafe` blocks in production |
| **Pure Rust** | ✅ **100%** | coturn eliminated - TRUE ecoBin compliance |
| **Crypto Delegation** | ✅ **100%** | Zero direct crypto - all via BearDog |
| **P2P Sovereign Onion** | ✅ **Complete** | Service + Connector + encrypted comms |
| **Pure Rust Tor (Phase 2A)** | ✅ **Complete** | Directory protocol + consensus parsing |
| **Production Mocks** | ✅ **0** | Perfect test isolation |
| **Tests Passing** | ✅ **1,763+** | 100% passing, comprehensive coverage |
| **Build Status** | ✅ **Clean** | Zero errors, minimal warnings |

## 🎯 Key Features

| Feature | Status | Description |
|---------|--------|-------------|
| **biomeOS Integration** | ✅ Complete | Unix socket IPC, standard methods, family_id passthrough |
| **Dark Forest Discovery** | ✅ Complete | Zero metadata leakage, encrypted beacons |
| **TLS 1.3** | ✅ Complete | RFC 8446 compliant, protocol detection (HTTP/HTTPS same port) |
| **STUN Server** | ✅ Complete | Pure Rust RFC 5389, NAT discovery |
| **Relay Server** | ✅ Complete | Pure Rust packet forwarding, lineage-based auth, coturn eliminated |
| **Capability Discovery** | ✅ Complete | 6-layer strategy, environment-first configuration |
| **Federation** | ✅ Complete | Zero-trust progressive escalation |
| **Smart Refactoring** | ✅ Complete | 8 phases, responsibility-based modules |
| **Sovereign Onion P2P** | ✅ Complete | Custom onion service + connector with BearDog crypto |

## 🚀 Quick Start

```bash
# Build
cargo build --workspace --release

# Run server
cargo run --bin songbird -- server

# Health check
cargo run --bin songbird -- doctor

# Configuration
cargo run --bin songbird -- config show
```

### Environment Variables

```bash
# Socket configuration (XDG-compliant)
export SONGBIRD_SOCKET=/run/user/$(id -u)/biomeos/songbird.sock

# Or use shared directory
export BIOMEOS_SOCKET_DIR=/run/user/$(id -u)/biomeos

# Port configuration
export SONGBIRD_ORCHESTRATOR_PORT=8080
export SONGBIRD_METRICS_PORT=9090
```

## 📦 Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Application Layer                        │
│          (biomeOS Neural API, Squirrel, Gorilla)            │
└─────────────────────────────┬───────────────────────────────┘
                              │ JSON-RPC 2.0 / tarpc
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Songbird Orchestrator                     │
├─────────────────────────────────────────────────────────────┤
│  • IPC Server (Unix sockets + TCP)                          │
│  • Discovery (mDNS, DNS-SD, capability-based)               │
│  • TLS 1.3 (Pure Rust, CryptoCapability trait)              │
│  • STUN Server (Pure Rust RFC 5389, NAT discovery)          │
│  • Relay Server (Pure Rust packet forwarding, lineage auth) │
│  • Dark Forest (encrypted beacons, zero leakage)            │
│  • Federation (peer-to-peer, multi-tier relay)              │
│  • Sovereign Onion (P2P service + connector)                │
└─────────────────────────────┬───────────────────────────────┘
                              │ 100% BearDog Delegation
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    BearDog Crypto Primal                     │
│  • Ed25519 (identity, signing)                              │
│  • X25519 (ECDH key exchange)                               │
│  • ChaCha20Poly1305 (authenticated encryption)              │
│  • SHA3-256 (onion address derivation)                      │
└─────────────────────────────────────────────────────────────┘
```

### Core Principles

1. **Self-Knowledge Only** - Each primal knows only itself
2. **Runtime Discovery** - All external services discovered at runtime
3. **Capability-Based** - Request by capability, not by name
4. **Zero Hardcoding** - Environment-first configuration
5. **Pure Rust** - TRUE ecoBin (zero C dependencies, coturn eliminated)
6. **Safe Rust** - Zero `unsafe` in production

## 📚 Documentation

| Document | Purpose |
|----------|---------|
| [`ROOT_DOCS_INDEX.md`](ROOT_DOCS_INDEX.md) | Complete documentation map ⭐ START HERE |
| [`P2P_IMPLEMENTATION_COMPLETE_FEB_06_2026.md`](P2P_IMPLEMENTATION_COMPLETE_FEB_06_2026.md) | P2P completion report ⭐ NEW |
| [`CONFIGURATION_PATTERNS.md`](CONFIGURATION_PATTERNS.md) | Zero hardcoding standard |
| [`EXECUTIVE_SUMMARY.md`](EXECUTIVE_SUMMARY.md) | High-level status overview |
| [`NAT_TRAVERSAL_VALIDATION_GUIDE.md`](NAT_TRAVERSAL_VALIDATION_GUIDE.md) | Validation procedures |
| [`DEPLOYMENT_READY_STATUS.md`](DEPLOYMENT_READY_STATUS.md) | Deploy now guide |
| [`CHANGELOG.md`](CHANGELOG.md) | Version history |

### Key Specifications

- [`specs/`](specs/) - Technical specifications (104+ files)
- [`docs/`](docs/) - Implementation guides
- [`examples/`](examples/) - Usage examples

## 📦 Crate Structure

### Core
- `songbird-orchestrator` - Main orchestration engine
- `songbird-cli` - Command-line interface
- `songbird-config` - Configuration management

### Networking
- `songbird-http-client` - TLS 1.3 HTTP client
- `songbird-network-federation` - Peer federation
- `songbird-discovery` - Service discovery
- `songbird-universal-ipc` - Platform-agnostic IPC

### Security & P2P
- `songbird-tls` - TLS implementation
- `songbird-genesis` - Trust ceremony
- `songbird-lineage-relay` - Lineage tracking
- `songbird-sovereign-onion` - P2P onion service (100% BearDog) ⭐ NEW

## 🧪 Testing

```bash
# All tests
cargo test --workspace

# Library tests only
cargo test --workspace --lib

# Specific crate
cargo test -p songbird-http-client

# Coverage (requires cargo-llvm-cov)
cargo llvm-cov --workspace --html
```

## 🔧 Development

```bash
# Format
cargo fmt --all

# Lint
cargo clippy --workspace --lib

# Build docs
cargo doc --workspace --no-deps --open
```

## 🤝 Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for guidelines:

- Use `Result<T, E>` in production (no `unwrap()`)
- Follow async-first patterns
- Keep files under 1,000 lines
- Write tests for new functionality
- No hardcoding (capability-based discovery)

## 📜 License

AGPL-3.0 - See [`LICENSE`](LICENSE)

---

**Built with 100% Pure Rust** | **Zero C Dependencies (coturn eliminated!)** | **Production Ready**
