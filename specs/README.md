# Songbird Specifications

**Date**: March 2026
**Workspace**: 30-crate Rust workspace (see root `Cargo.toml`)
**Version**: v0.2.1

---

## Current Workspace Crates

Songbird is a 30-member workspace. There is **no** `songbird-core`, `songbird-network`,
`songbird-errors`, or `songbird-security` package — those names reflect the September 2025
layout and are historical. Current crate names:

```
songbird-types          songbird-config           songbird-canonical
songbird-orchestrator   songbird-universal        songbird-universal-ipc
songbird-discovery      songbird-network-federation  songbird-http-client
songbird-tls            songbird-stun             songbird-igd
songbird-onion-relay    songbird-tor-protocol     songbird-sovereign-onion
songbird-lineage-relay  songbird-compute-bridge   songbird-execution-agent
songbird-registry       songbird-observability    songbird-cli
songbird-nfc            songbird-bluetooth        songbird-crypto-provider
songbird-test-utils     songbird-process-env      songbird-primal-coordination
songbird-remote-deploy  songbird-genesis          (root: songbird)
```

## Active Specifications

### Core Architecture
- [UNIVERSAL_CAPABILITY_ADAPTER_IMPLEMENTATION_SPEC.md](./UNIVERSAL_CAPABILITY_ADAPTER_IMPLEMENTATION_SPEC.md) — Capability adapter implementation
- [CAPABILITY_BASED_DISCOVERY_SPECIFICATION.md](./CAPABILITY_BASED_DISCOVERY_SPECIFICATION.md) — Discovery system specification
- [ECOSYSTEM_DELEGATION_SPECIFICATION.md](./ECOSYSTEM_DELEGATION_SPECIFICATION.md) — Primal delegation boundaries
- [CAPABILITY_BASED_CLIENT_ANALYSIS.md](./CAPABILITY_BASED_CLIENT_ANALYSIS.md) — Client capability analysis

### Implementation
- [SONGBIRD_NATIVE_RPC_SPECIFICATION.md](./SONGBIRD_NATIVE_RPC_SPECIFICATION.md) — Native JSON-RPC + tarpc protocol

### Archived (fossil record)
- [UNIVERSAL_ECOSYSTEM_INTEGRATION_SPEC.md](./archive/UNIVERSAL_ECOSYSTEM_INTEGRATION_SPEC.md) — Universal integration patterns (archived)
- [UNIFIED_ERROR_HANDLING_SPECIFICATION.md](./archive/UNIFIED_ERROR_HANDLING_SPECIFICATION.md) — Error handling patterns (archived)
- [COMPREHENSIVE_TESTING_INFRASTRUCTURE_SPECIFICATION.md](./archive/COMPREHENSIVE_TESTING_INFRASTRUCTURE_SPECIFICATION.md) — Testing infrastructure (archived)

### Protocol
- [SOVEREIGN_MULTIPATH_PROTOCOL.md](./SOVEREIGN_MULTIPATH_PROTOCOL.md) — Sovereign multipath protocol
- [SOVEREIGN_FEDERATION_IMPLEMENTATION_PLAN.md](./SOVEREIGN_FEDERATION_IMPLEMENTATION_PLAN.md) — Federation implementation
- [SONGBIRD_NEURALAPI_ALIGNMENT_V3_12_1.md](./SONGBIRD_NEURALAPI_ALIGNMENT_V3_12_1.md) — Neural API alignment
- [SONGBIRD_ROLE_CLARIFICATION_SPEC.md](./SONGBIRD_ROLE_CLARIFICATION_SPEC.md) — Songbird vs other primals

## Archived Specifications

Specs in [archive/](./archive/) reference the historical September 2025 crate layout
(`songbird-core`, `songbird-network`, `songbird-errors`, `songbird-security`). They are
retained as fossil record but should not be used for current integration.

See [00_SPECIFICATIONS_INDEX.md](./00_SPECIFICATIONS_INDEX.md) for the full historical index.
