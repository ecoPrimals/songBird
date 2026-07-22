# Songbird TURN Relay — VPS Deployment Guide

**Purpose**: Deploy a sovereign TURN relay server on a VPS for NAT traversal  
**Replaces**: `cloudflared` tunnels (H2-3c sovereignty)  
**Binary**: `songbird relay`

---

## Quick Deploy (5 minutes)

```bash
# 1. Copy binary to VPS (from plasmidBin depot)
scp plasmidBin/songbird user@vps:/usr/local/bin/

# 2. Copy systemd service
scp deployment/systemd/songbird-relay.service user@vps:/etc/systemd/system/

# 3. Create credentials (on VPS)
ssh user@vps
sudo mkdir -p /etc/songbird
sudo bash -c 'cat > /etc/songbird/relay-credentials << EOF
# Format: username:hex_key (beacon-tier HMAC material)
# Generate keys via: openssl rand -hex 32
nucleus-relay:$(openssl rand -hex 32)
EOF'
sudo chmod 640 /etc/songbird/relay-credentials

# 4. Open firewall
sudo ufw allow 3478/udp comment "TURN relay"

# 5. Enable and start
sudo systemctl daemon-reload
sudo systemctl enable --now songbird-relay

# 6. Verify
systemctl status songbird-relay
journalctl -u songbird-relay --no-pager -n 20
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  VPS (public IP, e.g. 203.0.113.10)                     │
│                                                         │
│  songbird relay --port 3478                             │
│    ├── UDP :3478 — STUN/TURN signaling                  │
│    ├── Allocate → ephemeral relay sockets                │
│    ├── Send Indication → client→peer via relay           │
│    ├── ChannelData → framed client↔peer via binding      │
│    └── Data Indication → peer→client via relay           │
└─────────────────────────────────────────────────────────┘
         ↑                              ↑
    TURN client                    Permitted peer
   (behind NAT)                   (behind NAT)
```

---

## Credential Provisioning

### Format

`/etc/songbird/relay-credentials`:
```
# One credential per line: username:hex_encoded_key
# Lines starting with # are comments
nucleus-relay:a3b4c5d6e7f8091011121314151617181920212223242526272829303132
family-alpha:deadbeefcafebabed00dfacefeedface0123456789abcdef0123456789ab
```

### Key Sources

**BearDog beacon-tier keys** (production):
```bash
# Request key from BearDog via IPC
echo '{"jsonrpc":"2.0","method":"auth.public_key","params":{"scope":"relay"},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/security.sock
```

**Manual generation** (testing/bootstrap):
```bash
openssl rand -hex 32
```

### Environment Variable Alternative

Instead of a file, set `SONGBIRD_RELAY_CREDENTIALS` with newline-separated entries:
```bash
export SONGBIRD_RELAY_CREDENTIALS="user1:abcdef0123456789
user2:fedcba9876543210"
```

---

## Firewall Configuration

```bash
# UFW (Ubuntu/Debian)
sudo ufw allow 3478/udp comment "TURN relay signaling"
sudo ufw allow 49152:65535/udp comment "TURN relay ephemeral ports"

# iptables
sudo iptables -A INPUT -p udp --dport 3478 -j ACCEPT
sudo iptables -A INPUT -p udp --dport 49152:65535 -j ACCEPT

# nftables
sudo nft add rule inet filter input udp dport 3478 accept
sudo nft add rule inet filter input udp dport 49152-65535 accept
```

The relay binds ephemeral ports (OS-assigned, typically 49152-65535) for each
allocation's relay socket. These must be reachable from peers.

---

## Monitoring

The relay emits structured stats every 60 seconds via `tracing::info!`:

```bash
# Live logs
journalctl -u songbird-relay -f

# Structured stats (every 60s — uptime, active allocations, bytes relayed)
journalctl -u songbird-relay | grep "TURN relay stats"

# Allocation activity
journalctl -u songbird-relay | grep "TURN: allocation"

# Auth failures
journalctl -u songbird-relay | grep "auth_failures\|Unauthorized"

# Resource usage
systemd-cgtop -1 | grep songbird-relay
```

Stats fields emitted per interval:
- `uptime_s` — seconds since server start
- `allocations_active` — currently held allocations
- `allocations_total` — lifetime allocation count
- `packets_relayed` — total data packets forwarded
- `bytes_relayed` — total bytes forwarded
- `auth_failures` — failed authentication attempts

---

## projectNUCLEUS NAT Shadow Run

Once deployed, configure the NAT shadow run by pointing Songbird clients
at the VPS relay address:

```bash
export SONGBIRD_TURN_SERVER=203.0.113.10:3478
export SONGBIRD_TURN_USERNAME=nucleus-relay
export SONGBIRD_TURN_KEY=<hex_key_from_credentials_file>
```

The `ConnectionFallbackChain` (Tier 4) will automatically attempt TURN
allocation through this server when direct and STUN-assisted paths fail.

Client code can also resolve config from env via `TurnSessionConfig::from_env(peer_addr)`
(songbird-turn-client crate). Shadow probe available via
`LineageRelayCoordinator::probe_turn_relay(peer_addr)` — returns `(relay_addr, setup_duration)`.

---

## golgiBody Deployment (10.13.37.1 / 157.230.3.183)

golgiBody is the WireGuard hub and public-facing VPS — the natural home for the TURN relay.

```bash
# From eastGate (build + deploy)
cd ~/Development/ecoPrimals/primals/songBird
cargo build --release -p songbird
scp target/release/songbird golgi:/usr/local/bin/songbird-new
ssh golgi 'sudo mv /usr/local/bin/songbird-new /usr/local/bin/songbird && sudo chmod 755 /usr/local/bin/songbird'

# On golgiBody
sudo cp deployment/systemd/songbird-relay.service /etc/systemd/system/
sudo mkdir -p /etc/songbird
sudo bash -c 'echo "tower-relay:$(openssl rand -hex 32)" > /etc/songbird/relay-credentials'
sudo chmod 640 /etc/songbird/relay-credentials
sudo ufw allow 3478/udp comment "TURN relay"
sudo ufw allow 49152:65535/udp comment "TURN ephemeral"
sudo systemctl daemon-reload
sudo systemctl enable --now songbird-relay
```

**Client env (set on all gates)**:
```bash
export SONGBIRD_TURN_SERVER=10.13.37.1:3478
export SONGBIRD_TURN_USERNAME=tower-relay
export SONGBIRD_TURN_KEY=<key from /etc/songbird/relay-credentials>
```

**Benchmark** (after relay is live):
```bash
# LAN benchmark: eastGate → sporeGate (same backbone)
songbird benchmark --mode tower-atomic --peer 10.13.37.2:7700 --output json

# WG baseline for comparison
songbird benchmark --mode wireguard --peer 10.13.37.2:7700 --output json

# WAN benchmark: sporeGate → flockGate via TURN on golgiBody
songbird benchmark --mode tower-atomic --peer 10.13.37.6:7700 --output json
```

---

## Production Checklist

- [ ] Binary deployed at `/usr/local/bin/songbird`
- [ ] Credentials file at `/etc/songbird/relay-credentials` (mode 640)
- [ ] systemd unit installed and enabled
- [ ] Firewall: UDP 3478 + ephemeral range open
- [ ] DNS A record pointing to VPS public IP
- [ ] Service starts successfully (`systemctl status songbird-relay`)
- [ ] Client can allocate (test with `TurnClient` from another machine)
- [ ] Bidirectional data flows through relay
- [ ] Monitoring/alerting configured
- [ ] projectNUCLEUS NAT shadow run validated
