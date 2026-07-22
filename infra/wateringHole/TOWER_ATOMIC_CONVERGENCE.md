# Tower Atomic — Parity Convergence Brief

**Date**: July 21, 2026  
**Wave**: 150t  
**Status**: Phase 1 — Parity Benchmark Pending  
**Target**: WireGuard replacement on LAN mesh

---

## What Is Tower Atomic

Tower Atomic is the sovereign transport stack: **bearDog** (crypto) + **songBird**
(transport/routing) + **skunkBat** (protocol negotiation). Together they provide
encrypted peer-to-peer communication without WireGuard.

```
┌─────────────────────────────────────────────────────────┐
│ Tower Atomic Stack                                       │
├─────────────────────────────────────────────────────────┤
│ skunkBat  — Protocol negotiation, bond formation        │
│ songBird  — Transport routing, NAT traversal, mesh      │
│ bearDog   — Crypto: Ed25519, X25519, ChaCha20-Poly1305  │
└─────────────────────────────────────────────────────────┘
```

## songBird's Role

songBird provides the **transport layer**:

| Capability | Implementation | Status |
|-----------|---------------|--------|
| IPC routing | JSON-RPC dispatch, capability→port resolution | LIVE |
| Drawbridge | HTTP bridge (:7780), path→capability mapping | LIVE |
| NAT traversal | 5-tier: direct → STUN → relay → TURN → tunnel | LIVE |
| Mesh networking | BeaconMesh: peer discovery, topology, health | LIVE |
| Cross-gate dispatch | `capability.call` → local UDS or remote TCP/TURN | LIVE |
| BTSP Phase 3 | ChaCha20-Poly1305 encrypted framing on IPC | LIVE |
| Enrollment | `mesh.enroll` with HMAC proof verification | LIVE |
| TURN relay server | RFC 5766 sovereign relay (VPS) | CODE COMPLETE |

## Parity Benchmark Requirements

**Philosophy**: Initial goal is **WireGuard parity** — any tractable first solution
that matches WG performance. WireGuard has years of development time on us; we
leverage what we learn from benchmarking to evolve past parity. Parity is the
floor, not the ceiling. Targets below are relative to WG baseline, not absolute
thresholds (since physical path characteristics vary by topology).

Before Tower Atomic can replace WireGuard on the LAN mesh, we need to demonstrate:

| Metric | WireGuard Baseline | Tower Target | How to Measure |
|--------|-------------------|--------------|----------------|
| Throughput | ~900 Mbps (LAN) | ≥80% of WG | `iperf3` through each stack |
| Latency (RTT) | ~0.3ms (LAN) | ≤2x WG | `ping` vs Tower RPC round-trip |
| Connection setup | ~50ms (handshake) | ≤500ms | Time from connect to first byte |
| Reconnect | Instant (stateless) | ≤2s | Mesh re-discovery after link drop |
| CPU (idle) | ~0% | ≤1% | `top` with mesh active, no traffic |
| CPU (saturated) | ~5% | ≤20% | `top` during throughput test |

### Benchmark Harness (TODO — needs implementation)

```bash
# Proposed: run on sporeGate↔ironGate (same LAN, WireGuard peers)
songbird benchmark --mode tower-atomic --peer ironGate --duration 30s
songbird benchmark --mode wireguard   --peer ironGate --duration 30s
songbird benchmark --compare          --output /tmp/parity-report.json
```

## What Each Team Needs To Do

### bearDog Team

| Need | Priority | Detail |
|------|----------|--------|
| `enrollment.verify` endpoint | **P1** | songBird calls this to verify HMAC proofs during `mesh.enroll`. Params: `{node_id, public_key, timestamp, proof}`. Returns `{verified: bool, reason?: string}` |
| ChaCha20-Poly1305 session keys | P2 | `btsp.server.export_keys` — derive per-session c2s/s2c keys via HKDF-SHA256 |
| Throughput crypto benchmark | P2 | Measure raw encrypt/decrypt throughput for parity assessment |

### skunkBat Team

| Need | Priority | Detail |
|------|----------|--------|
| Bond negotiation protocol | P2 | `btsp.negotiate` selects cipher suite; skunkBat defines bond formation rules |
| Protocol version exchange | P2 | Agree on frame format before encrypted channel activates |
| Fallback behavior | P2 | What happens when cipher negotiation fails (NULL cipher? reject?) |

### songBird (self — already done or in progress)

| Item | Status |
|------|--------|
| 5-tier NAT traversal | LIVE |
| `mesh.enroll` with BTSP proof | LIVE |
| BTSP Phase 3 encrypted framing | LIVE |
| Cross-gate `capability.call` | LIVE |
| TURN relay server (VPS) | CODE COMPLETE (deployment = ops) |
| Drawbridge port solving | LIVE |
| Benchmark harness | TODO (P2 — needs throughput measurement tooling) |

### Deployment/Ops Team

| Need | Priority | Detail |
|------|----------|--------|
| TURN relay deployment | P2 | Deploy `songbird relay` on golgiBody VPS (systemd unit ready) |
| `SONGBIRD_DRAWBRIDGE_ADDR=0.0.0.0:7780` | P2 | Set on gates where cross-WG drawbridge access needed |
| Parity benchmark environment | P2 | sporeGate↔ironGate LAN pair with both WG and Tower active |

## Convergence Timeline

| Phase | Milestone | Depends On |
|-------|-----------|-----------|
| **Phase 0** (current) | All Tower components live independently | — |
| **Phase 1** | Parity benchmark: measure Tower vs WG on LAN | Benchmark harness |
| **Phase 2** | Shadow mode: Tower runs alongside WG, metrics compared | Phase 1 pass |
| **Phase 3** | Cutover: Tower replaces WG for inter-gate traffic | Phase 2 validated |

## Protocol: HMAC Enrollment (mesh.enroll)

```
Enrolling gate computes:
  proof = HMAC-SHA256(family_seed, node_id || "|" || public_key || "|" || timestamp)

Sends to hub gate:
  {"jsonrpc":"2.0","method":"mesh.enroll","params":{
    "node_id": "southGate",
    "public_key": "<wg-pubkey>",
    "timestamp": 1753128000,
    "proof": "<base64-hmac>",
    "address": "10.13.37.9:7700"
  },"id":1}

Hub verifies via bearDog:
  songBird → enrollment.verify({node_id, public_key, timestamp, proof}) → bearDog
  bearDog checks HMAC against family_seed
  Returns {verified: true} or {verified: false, reason: "..."}

On success:
  - Node persisted to peers.toml
  - Added to live mesh (if mesh active)
  - Response: {"enrolled": true, "node_id": "southGate", "mesh_active": true}
```

## Contacts

| Role | Team | Socket/Endpoint |
|------|------|-----------------|
| Transport + routing | songBird | `songbird.sock` / `:7780` drawbridge |
| Crypto + enrollment verify | bearDog | `security.sock` |
| Protocol negotiation | skunkBat | (via songBird BTSP dispatch) |
| Deployment | ops (sporeGate team) | golgiBody VPS |
