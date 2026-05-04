# Songbird Specifications

**Date**: May 2026
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
- [PRIMAL_SELF_KNOWLEDGE_EVOLUTION_SPEC.md](./PRIMAL_SELF_KNOWLEDGE_EVOLUTION_SPEC.md) — Self-knowledge standard

### Implementation
- [SONGBIRD_NATIVE_RPC_SPECIFICATION.md](./SONGBIRD_NATIVE_RPC_SPECIFICATION.md) — Native JSON-RPC + tarpc protocol
- [HYBRID_PROTOCOL_ARCHITECTURE_SPECIFICATION.md](./HYBRID_PROTOCOL_ARCHITECTURE_SPECIFICATION.md) — Multi-protocol architecture

### Protocol
- [SOVEREIGN_MULTIPATH_PROTOCOL.md](./SOVEREIGN_MULTIPATH_PROTOCOL.md) — Sovereign multipath protocol
- [SOVEREIGN_ONION_PROTOCOL.md](./SOVEREIGN_ONION_PROTOCOL.md) — Onion routing
- [BIRDSONG_PROTOCOL.md](./BIRDSONG_PROTOCOL.md) — Encrypted discovery multicast
- [SONGBIRD_ROLE_CLARIFICATION_SPEC.md](./SONGBIRD_ROLE_CLARIFICATION_SPEC.md) — Songbird vs other primals

## Archived Specifications (`specs/archived/`)

22 specs archived (May 2–4, 2026). These reference crates that were never created
(`songbird-client`, `songbird-primal-sdk`, `songbird-universal-primals`,
`songbird-federation`, `songbird-rpc`) or file paths that no longer exist.
Retained as design fossil record.

See also the [consolidated fossil record](../../../infra/wateringHole/fossilRecord/consolidated-apr2026/) for
September 2025 crate layout specs (`songbird-core`, `songbird-network`, etc.).

See [00_SPECIFICATIONS_INDEX.md](./00_SPECIFICATIONS_INDEX.md) for the full historical index.
