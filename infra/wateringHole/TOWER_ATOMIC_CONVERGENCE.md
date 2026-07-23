# Tower Atomic — Parity Convergence Brief

**Date**: July 23, 2026  
**Wave**: 150v  
**Status**: Phase 2 — ALL BLOCKERS RESOLVED, Parity Benchmark Ready to Execute  
**Target**: WireGuard replacement on LAN+WAN mesh

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

Before Tower Atomic can replace WireGuard on the LAN mesh, we need to demonstrate:

| Metric | WireGuard Baseline | Tower Target | How to Measure |
|--------|-------------------|--------------|----------------|
| Throughput | ~900 Mbps (LAN) | ≥80% of WG | `iperf3` through each stack |
| Latency (RTT) | ~0.3ms (LAN) | ≤2x WG | `ping` vs Tower RPC round-trip |
| Connection setup | ~50ms (handshake) | ≤500ms | Time from connect to first byte |
| Reconnect | Instant (stateless) | ≤2s | Mesh re-discovery after link drop |
| CPU (idle) | ~0% | ≤1% | `top` with mesh active, no traffic |
| CPU (saturated) | ~5% | ≤20% | `top` during throughput test |

### Benchmark Harness — SHIPPED (Wave 150v)

`songbird benchmark` CLI is live (`src/benchmark.rs`, 366 lines). 3-phase measurement:

1. **Setup**: TCP connection establishment time (10 attempts)
2. **Latency**: JSON-RPC `health.ping` round-trip (configurable probes, p50/p95/p99)
3. **Throughput**: Sustained 64KiB chunk stream (configurable duration, Mbps)

```bash
# LAN benchmark: eastGate ↔ sporeGate (same backbone)
songbird benchmark --mode tower-atomic --peer 10.13.37.2:7700 --output json
songbird benchmark --mode wireguard   --peer 10.13.37.2:7700 --output json

# WAN benchmark: sporeGate → flockGate via TURN on golgiBody
songbird benchmark --mode tower-atomic --peer 10.13.37.6:7700 --output json
songbird benchmark --mode wireguard   --peer 10.13.37.6:7700 --output json
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

| Item | Status | Detail |
|------|--------|--------|
| 5-tier NAT traversal | LIVE | direct → STUN → relay → TURN → tunnel |
| `mesh.enroll` with BTSP proof | LIVE | HMAC-SHA256 verification via bearDog |
| BTSP Phase 3 encrypted framing | LIVE | ChaCha20-Poly1305 on all 3 IPC paths |
| Cross-gate `capability.call` | LIVE | TCP direct + TURN relay fallback |
| TURN relay server (VPS) | **LIVE** | golgiBody:3478, PID 2140600, since Jul 12 |
| TURN client (data plane) | LIVE | `send()`/`recv()` + ChannelData framing, 26 tests |
| Shadow dual-path comparator | LIVE | TURN vs cloudflared setup time comparison |
| NAT field test harness | LIVE | CGNAT/double-NAT/symmetric scenarios |
| Latency measurement (`mesh.probe_latency`) | LIVE | TCP→`health.ping` RTT per peer |
| Drawbridge port solving | LIVE | `:7780` capability→URL resolution |
| Throughput benchmark | **SHIPPED** | 64KiB stream, configurable duration, Mbps output |
| `songbird benchmark` CLI | **SHIPPED** | 3-phase harness, JSON+text, p50/p95/p99 |

### Deployment/Ops Team

| Need | Priority | Detail |
|------|----------|--------|
| ~~TURN relay deployment~~ | **LIVE** | golgiBody VPS, LIVE since Jul 12, deployment guide at `deployment/relay/README.md` |
| `SONGBIRD_DRAWBRIDGE_ADDR=0.0.0.0:7780` | P2 | Set on gates where cross-WG drawbridge access needed |
| Parity benchmark environment | P2 | sporeGate↔ironGate LAN pair with both WG and Tower active |

## Existing Measurement Infrastructure

songBird already has the building blocks for parity assessment:

| Component | Crate | What It Measures |
|-----------|-------|-----------------|
| `mesh.probe_latency` | `songbird-universal-ipc` | TCP RTT to each peer via `health.ping` JSON-RPC |
| `shadow_comparator::compare_paths()` | `songbird-lineage-relay` | TURN vs cloudflared setup time (parallel) |
| `nat_field_test::probe_turn_path()` | `songbird-lineage-relay` | TURN allocation + relay setup per NAT scenario |
| `LineageRelayCoordinator::probe_turn_relay()` | `songbird-lineage-relay` | Relay addr + setup duration measurement |
| `TurnRelayStats` | `songbird-stun` | Live stats: packets/bytes relayed, allocations, uptime |
| `TurnSession::send()/recv()` | `songbird-turn-client` | Raw data plane (for throughput measurement) |

**SHIPPED**: `songbird benchmark` CLI (366 lines, `src/benchmark.rs`). All 3 items above are live.

## Convergence Timeline

| Phase | Milestone | Status |
|-------|-----------|--------|
| **Phase 0** | All Tower components live independently | **COMPLETE** |
| **Phase 1** (current) | Parity benchmark: measure Tower vs WG on LAN+WAN | **UNBLOCKED — harness + relay live** |
| **Phase 2** | Shadow mode: Tower runs alongside WG, metrics compared | Pending Phase 1 results |
| **Phase 3** | Cutover: Tower replaces WG for inter-gate traffic | Pending Phase 2 validation |

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
