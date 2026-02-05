# 🐦 Songbird - Network Orchestration & Discovery Primal

**Version**: v3.21.0  
**Status**: ✅ **PRODUCTION READY** - Deep Debt Evolution Complete  
**License**: AGPL-3.0  

Songbird is the universal network orchestrator for the ecoPrimals ecosystem, managing service discovery, connection management, and inter-primal communication.

## 🎯 Key Features

| Feature | Status | Description |
|---------|--------|-------------|
| **Pure Rust** | ✅ 100% | Zero C dependencies (TRUE ecoBin) |
| **Safe Rust** | ✅ 100% | Zero `unsafe` in production |
| **Deep Debt** | ✅ 99.4% | Near-perfect code quality |
| **biomeOS** | ✅ Complete | 11 JSON-RPC methods (health, identity, beacon_exchange) |
| **Dark Forest** | ✅ Complete | TRUE privacy, zero metadata leakage |
| **TLS 1.3** | ✅ Complete | RFC 8446 + protocol detection (HTTP/HTTPS same port) |
| **IPC** | ✅ Complete | Unix sockets + TCP support |
| **Tests** | ✅ 1,663+ | Comprehensive coverage |

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
│  • Dark Forest (encrypted beacons, zero leakage)            │
│  • Federation (peer-to-peer, multi-tier relay)              │
└─────────────────────────────┬───────────────────────────────┘
                              │ Arc<dyn CryptoCapability>
                              ▼
┌─────────────────────────────────────────────────────────────┐
│               Crypto Providers (BearDog, etc.)               │
└─────────────────────────────────────────────────────────────┘
```

### Core Principles

1. **Self-Knowledge Only** - Each primal knows only itself
2. **Runtime Discovery** - All external services discovered at runtime
3. **Capability-Based** - Request by capability, not by name
4. **Zero Hardcoding** - Environment-first configuration
5. **Pure Rust** - TRUE ecoBin (zero C dependencies)
6. **Safe Rust** - Zero `unsafe` in production

## 📚 Documentation

| Document | Purpose |
|----------|---------|
| [`EXECUTIVE_SUMMARY.md`](EXECUTIVE_SUMMARY.md) | High-level status overview |
| [`ROOT_DOCS_INDEX.md`](ROOT_DOCS_INDEX.md) | Complete documentation map |
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

### Security
- `songbird-tls` - TLS implementation
- `songbird-genesis` - Trust ceremony
- `songbird-lineage-relay` - Lineage tracking

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

**Built with 100% Pure Rust** | **Zero C Dependencies** | **Production Ready**
