# 📚 Songbird Documentation Index

**Version**: v3.20.0  
**Status**: ✅ **PRODUCTION READY**  
**Updated**: February 4, 2026

---

## ⚡ Quick Start

| Time | Document | Purpose |
|------|----------|---------|
| **30 sec** | [`DEPLOYMENT_READY_STATUS.md`](DEPLOYMENT_READY_STATUS.md) | Deploy now |
| **1 min** | [`EXECUTIVE_SUMMARY.md`](EXECUTIVE_SUMMARY.md) | Status overview |
| **5 min** | [`README.md`](README.md) | Project overview |

---

## 📖 Core Documentation

### Essential Files

| Document | Description |
|----------|-------------|
| [`README.md`](README.md) | Project overview, quick start, architecture |
| [`EXECUTIVE_SUMMARY.md`](EXECUTIVE_SUMMARY.md) | Current status, metrics, key features |
| [`DEPLOYMENT_READY_STATUS.md`](DEPLOYMENT_READY_STATUS.md) | Deployment checklist and guide |
| [`CHANGELOG.md`](CHANGELOG.md) | Version history and release notes |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Contribution guidelines |

### Configuration

| Document | Description |
|----------|-------------|
| [`SONGBIRD_CLI_SPEC_FOR_BIOMEOS.yaml`](SONGBIRD_CLI_SPEC_FOR_BIOMEOS.yaml) | CLI specification |
| [`config/`](config/) | Configuration templates and examples |

---

## 📐 Specifications

### Core Specs (`specs/`)

| Category | Key Files |
|----------|-----------|
| **Architecture** | `UNIBIN_ARCHITECTURE_SPEC.md`, `ECOBIN_COMPLIANCE.md` |
| **Protocols** | `TARPC_JSON_RPC_PROTOCOL_SPEC.md`, `IPC_PROTOCOL.md` |
| **Discovery** | `CAPABILITY_BASED_DISCOVERY_SPECIFICATION.md` |
| **Security** | `DARK_FOREST_BEACON_SPEC.md`, `TLS_SPECIFICATION.md` |
| **Testing** | `TESTING_STRATEGY.md`, `COVERAGE_REQUIREMENTS.md` |

### Implementation Guides (`docs/`)

| Category | Description |
|----------|-------------|
| `docs/architecture/` | System design documents |
| `docs/protocols/` | Protocol implementations |
| `docs/integration/` | Integration guides |

---

## 🗂️ Session Archives

Historical session documents are archived in `ecoPrimals/sessions/`:

| Period | Location | Content |
|--------|----------|---------|
| Feb 2026 | `ecoPrimals/sessions/feb-2026/` | Deep debt evolution, biomeOS integration |
| Jan 2026 | `ecoPrimals/sessions/jan-2026/` | TLS evolution, reqwest elimination |

### Key Archived Documents

| Document | Description |
|----------|-------------|
| `DEEP_DEBT_EVOLUTION_PHASE_5_COMPLETE_FEB_04_2026.md` | Phase 5 summary |
| `SONGBIRD_BIOMEOS_INTEGRATION_COMPLETE_FEB_04_2026.md` | biomeOS integration |
| `DARK_FOREST_COMPLETE_FEB_03_2026.md` | Dark Forest implementation |

---

## 🔧 Development Resources

### Code Structure

```
songbird/
├── crates/                 # 23 workspace crates
│   ├── songbird-orchestrator/  # Main orchestrator
│   ├── songbird-cli/           # CLI interface
│   ├── songbird-config/        # Configuration
│   ├── songbird-discovery/     # Service discovery
│   ├── songbird-http-client/   # TLS 1.3 HTTP
│   └── ...
├── specs/                  # Technical specifications
├── docs/                   # Implementation guides
├── examples/               # Usage examples
├── tests/                  # Integration tests
└── scripts/                # Build and utility scripts
```

### Build & Test

```bash
cargo build --workspace          # Build all
cargo test --workspace --lib     # Run tests
cargo clippy --workspace --lib   # Lint
cargo fmt --all                  # Format
cargo doc --workspace --no-deps  # Generate docs
```

---

## 📊 Project Metrics

| Metric | Value |
|--------|-------|
| **Deep Debt Score** | 99.4% |
| **Pure Rust** | 100% |
| **Safe Rust** | 100% |
| **Test Pass Rate** | 100% |
| **Clippy Errors** | 0 |

---

## 🔗 Related Projects

### ecoPrimals Ecosystem

| Project | Description |
|---------|-------------|
| **BearDog** | Security & crypto primal |
| **Squirrel** | AI & MCP primal |
| **ToadStool** | Storage primal |
| **NestGate** | Sovereign gateway |

### Standards (`wateringHole/`)

| Standard | Description |
|----------|-------------|
| `UNIBIN_ARCHITECTURE_STANDARD.md` | Single binary architecture |
| `ECOBIN_ARCHITECTURE_STANDARD.md` | Pure Rust requirements |
| `PRIMAL_IPC_PROTOCOL.md` | IPC protocol standard |
| `INTER_PRIMAL_INTERACTIONS.md` | Primal communication |

---

**Last Updated**: February 4, 2026
