# songBird — Upstream Handoff

**Primal**: songBird  
**Version**: v0.2.1-wave137c  
**Date**: July 13, 2026  
**Gate**: flockGate (eastGate)

## Current State

| Metric | Status |
|--------|--------|
| Clippy | Zero warnings (pedantic + nursery, `-D warnings`) |
| Tests | Full workspace pass, 0 failures |
| Unsafe | 0 (`forbid(unsafe_code)` all 31 crates) |
| Production stubs | 0 (Wave 137: last fake-data stub evolved to real probes) |
| Files >800L prod | 0 |
| Hardcoding | 0 in production (all env-driven, capability-based) |
| Mocks in prod | 0 (all `#[cfg(test)]` gated) |

## Recent Evolution (Wave 131–137)

| Wave | Summary |
|------|---------|
| 137c | FP-API COMPLETE: HTTPS outbound proxy (tokio-rustls + CA certs), drawbridge wired into server command, footPrint PROXY_PATH migrated to `/ext`, E2E verified |
| 137b | UDS-HTTP-PROTOCOL fix: peer.connect → BeaconMesh registration (CRITICAL) |
| 137a | FLOCKGATE-MESH fix (WG auto-discovery port 8080→7700); FP-API `?url=` compat layer in drawbridge |
| 137 | CLI fake metrics → real TCP RTT probes; dead code removal (-672L); dep diet (fastrand, rand removed from CLI) |
| 136b | Drawbridge external proxy allowlist (footPrint composition); domain-validated forwarding |
| 136a | Drawbridge auth-gate: bearer tokens, CIDR trusted peers, public paths, per-route `!public` |
| 134f | `http.request` path-handling bug (absolute-form→origin-form fix in HTTP client) |
| 132f | bearDog CryptoProvider + BindMode::Auto fixes pushed to both remotes |
| 131b | LAN peering bypass, health honesty, security fail-closed, dep diet |
| 131 | Allocation elimination, typed dispatch, hardcoding removal |

## Upstream Dependencies (consumed capabilities)

| Capability | Provider | Status |
|------------|----------|--------|
| `security.*` (Ed25519, X25519, ChaCha20) | bearDog | LIVE via IPC |
| `primal.announce` | biomeOS Neural API | LIVE (optional) |

## Provided Capabilities (15 tokens)

`network.discovery`, `network.federation`, `network.relay`, `network.stun`,
`network.igd`, `network.quic`, `network.tls`, `network.tor`, `network.onion`,
`network.btsp`, `ipc.jsonrpc`, `ipc.tarpc`, `crypto.delegate`, `nfc.genesis`, `bluetooth.pair`

## Blocking Items for Other Teams

| Item | Blocked Team | Status |
|------|-------------|--------|
| TOPO-VIS mesh heartbeat data | petalTongue + nestGate | Ready (mesh.status, mesh.peers expose data) |
| EXP-06-CADDY auth wiring | sporeGate | Auth-gate live, Caddy config pending sporeGate |
| Drawbridge external proxy | footPrint composition | **COMPLETE** (Wave 137c) — HTTPS outbound, client migrated, E2E verified |

## Known Gaps (not blocking)

- Coverage 73.41% → 90% target (I/O-heavy modules need mock infra)
- BTSP Phase 3: multi-frame stress tests pending
- Tor onion crypto: blocked on live security provider Ed25519/X25519 surface
- CLI interactive prompts: `songbird init` still prints placeholder message

## Fossil Record

Docs preserved as fossil record in `ecoPrimals/`. Archived specs in `specs/archived/`.
Wave history in `REMAINING_WORK.md` and `CHANGELOG.md`.
