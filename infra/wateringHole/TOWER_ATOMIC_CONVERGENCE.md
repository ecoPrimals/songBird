# Tower Atomic — Parity Convergence Brief

**Date**: July 23, 2026  
**Wave**: 150w  
**Status**: Phase 2 — Shadow Deploy All Live Topo + Exploration  
**Target**: WireGuard replacement → sovereign compute mesh

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
| **Phase 1** | Parity benchmark: measure Tower vs WG on LAN+WAN | **PASS** (Jul 23 — 0.99x latency, 1.07x throughput LAN) |
| **Phase 2** (current) | Shadow deploy all live topo + exploration | **DEPLOYING** |
| **Phase 3** | Cutover: Tower replaces WG for inter-gate traffic | Pending Phase 2 validation |

---

## Phase 2: Shadow Deployment + Exploration

### Deployment Scope

Deploy Tower Atomic alongside WireGuard on all currently live gates. Both stacks
run simultaneously — WireGuard carries production traffic, Tower carries mirrored
traffic with continuous metrics.

| Gate | Mesh IP | Status | Shadow Role |
|------|---------|--------|-------------|
| golgiBody | 10.13.37.1 | **LIVE** | Hub + TURN relay, multi-stack routing target |
| sporeGate | 10.13.37.2 | **LIVE** | Build authority, benchmark driver |
| eastGate | 10.13.37.5 | **LIVE** | Code hub, LAN peer to sporeGate |
| flockGate | 10.13.37.6 | **LIVE** | WAN peer, remote covalent mesh |
| ironGate | 10.13.37.7 | DOWN | Enroll when restored |
| northGate | 10.13.37.8 | Enrolled | Windows — enroll when Linux dual-boot active |

**Shadow mode per gate**: `membrane tower.shadow --enable` activates Tower
transport on the songBird mesh port (7700) while WireGuard continues on wg0.
All inter-gate RPC is duplicated: primary on WG, shadow on Tower. Metrics
collected for latency, throughput, jitter, and error rate.

### Exploration Domains — Where Tower Exceeds WireGuard

WireGuard is a general-purpose kernel VPN. Tower Atomic is a userspace
capability-routed mesh. The specialization opens exploration space that WG
fundamentally cannot address:

#### 1. Capability-Aware Routing

WireGuard: All packets traverse the same tunnel. No application awareness.

Tower Atomic: JSON-RPC dispatch routes by capability. songBird knows *what*
the traffic is — `nestgate.blob_put`, `beardog.sign`, `toadstool.dispatch` —
and can route, prioritize, and shape accordingly.

**primalSpring exploration**: `s_tower_capability_routing` — measure latency
and throughput per-capability class. Does RPC routing overhead amortize when
mixed workloads compete for the same tunnel?

#### 2. Multi-Stack Routing on golgiBody

WireGuard: One tunnel, all traffic.

Tower Atomic: golgiBody can run N Tower Atomic stacks, each tuned for a
different traffic pattern. A TURN relay stack for WAN peers. A low-latency
stack for real-time RPC. A high-throughput stack for blob sync.

**primalSpring exploration**: `s_tower_multi_stack` — deploy 2–3 songBird
instances on golgiBody with different tuning profiles. Measure whether
per-purpose routing outperforms a single fat tunnel.

#### 3. Large Data Transfer (nestGate CAS, ZFS replication)

WireGuard: Fixed MTU (1420 bytes typically), no content awareness.

Tower Atomic: Content-addressed blob routing via nestGate CAS. songBird
can negotiate payload-optimal framing (jumbo frames on 10G backbone, chunked
streaming on WAN). Blobs route to nearest cached copy rather than through
a fixed tunnel endpoint.

**primalSpring exploration**: `s_tower_large_data` — transfer 100MB, 1GB,
10GB blobs through both stacks on LAN (10G when cabled) and WAN.
Measure: throughput, CPU overhead, memory pressure, content-addressed
deduplication benefit.

**Springs science integration**:
- `wetSpring`: Bioinformatics datasets (Kraken2 DB, alignment indices) — strandGate → eastGate
- `hotSpring`: HBM2 benchmark results, compiler artifacts — biomeGate → gates
- `neuralSpring`: Model weights, training checkpoints

#### 4. Secure Compute Mesh

WireGuard: Encrypted tunnel, but no application-layer crypto policy.

Tower Atomic: bearDog provides end-to-end encryption *per capability*.
Different trust levels per workload — `PostPrimordial` primals get stronger
attestation than general data. CredentialStore integration means keys live
in HSM/TEE where available (grapheneGate, SoloKeys).

**primalSpring exploration**: `s_tower_secure_compute` — measure crypto
overhead of bearDog per-session keys vs WireGuard's tunnel-level encryption.
End-to-end: can a computation request be cryptographically bound to a
specific gate's hardware attestation?

#### 5. Distributed Compute Coordination

WireGuard: Just a pipe. No compute awareness.

Tower Atomic: songBird's mesh topology knows which gate has which hardware
(GPU VRAM, NPU, CPU cores). Combined with biomeOS workload dispatch,
Tower Atomic becomes a compute-aware mesh — workloads route to the
gate with the right substrate.

**primalSpring exploration**: `s_tower_compute_mesh` — toadStool dispatches
a parallel workload across 2+ gates. Compare coordination latency through
WG tunnel vs Tower Atomic's direct capability dispatch. Measure: task
dispatch latency, intermediate result transfer, aggregation time.

**Hardware targets**:
- GPU compute: strandGate (RTX 3090 + RX 6950 XT) ↔ biomeGate (Titan V HBM2)
- NPU: eastGate (Akida) ↔ strandGate (Akida) ↔ biomeGate (Akida)
- CPU-parallel: strandGate (64-core EPYC) ↔ biomeGate (32-core Threadripper)

#### 6. Edge + SFF Mesh (NUCs, NucBox)

WireGuard: Same overhead regardless of hardware. Kernel module helps but
NUC Celerons are CPU-constrained.

Tower Atomic: Userspace, so overhead is tunable. songBird on a Celeron NUC
can run a minimal relay/beacon profile — less crypto, fewer probes, lighter
mesh heartbeat — that WireGuard cannot specialize.

**primalSpring exploration**: `s_tower_edge_profile` — songBird on NUC
with minimal config vs WireGuard. Measure: idle CPU, memory footprint,
relay throughput, discovery latency.

### Metrics Collection

Shadow mode continuously collects for each gate pair:

```toml
[shadow_metrics]
collection_interval_sec = 60
export_format = "json"
export_path = "benchScale/tower_shadow/"

[shadow_metrics.dimensions]
latency_p50_ms = true
latency_p95_ms = true
latency_p99_ms = true
throughput_mbps = true
jitter_ms = true
setup_time_ms = true
error_rate = true
cpu_percent = true
memory_mb = true
```

Reports land in `benchScale/tower_shadow/` per gate, per day. The
`s_tower_atomic_parity_live` scenario reads these for validation.

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
