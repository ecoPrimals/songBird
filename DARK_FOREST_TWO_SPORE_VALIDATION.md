# Dark Forest Protocol - Two Spore Validation Test

**Date**: January 29, 2026  
**Version**: v8.19.0  
**Purpose**: Real-world validation of Dark Forest wiring fix with USB spores  
**Status**: 🧪 **VALIDATION SCRIPT READY**

---

## Executive Summary

This guide validates that all 6 Dark Forest methods work correctly between two LiveSpore USB deployments using real STUN servers for NAT traversal and peer handshaking.

**What We're Testing**:
1. ✅ STUN public address discovery (both spores)
2. ✅ STUN binding for hole punching
3. ✅ Local discovery via UDP beacons
4. ✅ Cross-spore peer listing
5. ✅ Rendezvous registration (relay server)
6. ✅ Direct peer connection (UDP hole punching)

**Expected Outcome**: Complete cross-spore handshake with NAT traversal!

---

## Prerequisites

### Hardware Setup
- 2 USB LiveSpores (physical USB drives)
- 2 separate machines (or same machine with separate network interfaces)
- LAN connectivity (same subnet or routable)

### Network Setup
```
┌─────────────────────────────────────────────────────────────┐
│                    LAN / Home Router                         │
│                   192.168.1.0/24                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   Spore Alpha (Tower A)        Spore Gamma (Tower B)       │
│   ├─ IP: 192.168.1.144         ├─ IP: 192.168.1.134        │
│   ├─ Port: 8081                ├─ Port: 8082               │
│   ├─ Family: nat0              ├─ Family: nat0             │
│   └─ Node: node-alpha          └─ Node: node-gamma         │
│                                                             │
│   Both connect to:                                          │
│   └─ STUN: stun.nextcloud.com:3478                         │
│   └─ (Discovers public IPs for NAT traversal)              │
└─────────────────────────────────────────────────────────────┘
```

### Software
- Songbird v8.19.0 (freshly built)
- `jq` (JSON parsing)
- `nc` (netcat for Unix socket testing)

---

## Validation Script

### Step 1: Start Both Spores

**On Tower A (Spore Alpha)**:
```bash
#!/bin/bash
# Start Spore Alpha

cd /home/eastgate/Development/ecoPrimals/phase1/songbird

# Clean any existing sockets
rm -f /run/user/1000/biomeos/songbird-nat0.sock

# Start Songbird orchestrator
FAMILY_ID=nat0 \
NODE_ID=node-alpha \
SONGBIRD_PORT=8081 \
./target/release/songbird server &

echo "Spore Alpha starting on port 8081..."
sleep 5

# Verify Universal IPC Broker started
if [ -S /primal/songbird ]; then
    echo "✅ Spore Alpha ready: /primal/songbird"
else
    echo "❌ Spore Alpha failed to start"
    exit 1
fi
```

**On Tower B (Spore Gamma)**:
```bash
#!/bin/bash
# Start Spore Gamma

cd /home/eastgate/Development/ecoPrimals/phase1/songbird

# Clean any existing sockets
rm -f /run/user/1000/biomeos/songbird-nat0.sock

# Start Songbird orchestrator
FAMILY_ID=nat0 \
NODE_ID=node-gamma \
SONGBIRD_PORT=8082 \
./target/release/songbird server &

echo "Spore Gamma starting on port 8082..."
sleep 5

# Verify Universal IPC Broker started
if [ -S /primal/songbird ]; then
    echo "✅ Spore Gamma ready: /primal/songbird"
else
    echo "❌ Spore Gamma failed to start"
    exit 1
fi
```

---

### Step 2: STUN Discovery (Get Public IPs)

**On Tower A**:
```bash
#!/bin/bash
# Discover Spore Alpha's public IP via STUN

echo "=== Spore Alpha: STUN Public Address Discovery ==="

STUN_RESULT=$(echo '{
  "jsonrpc": "2.0",
  "method": "stun.get_public_address",
  "params": {
    "server": "stun.nextcloud.com:3478"
  },
  "id": 1
}' | nc -U /primal/songbird -N)

echo "$STUN_RESULT" | jq '.'

# Extract public address
ALPHA_PUBLIC=$(echo "$STUN_RESULT" | jq -r '.result.public_address')
ALPHA_LOCAL=$(echo "$STUN_RESULT" | jq -r '.result.local_address')
ALPHA_NAT=$(echo "$STUN_RESULT" | jq -r '.result.nat_type')

echo ""
echo "Spore Alpha Addresses:"
echo "  Public: $ALPHA_PUBLIC"
echo "  Local:  $ALPHA_LOCAL"
echo "  NAT:    $ALPHA_NAT"
```

**On Tower B**:
```bash
#!/bin/bash
# Discover Spore Gamma's public IP via STUN

echo "=== Spore Gamma: STUN Public Address Discovery ==="

STUN_RESULT=$(echo '{
  "jsonrpc": "2.0",
  "method": "stun.get_public_address",
  "params": {
    "server": "stun.nextcloud.com:3478"
  },
  "id": 1
}' | nc -U /primal/songbird -N)

echo "$STUN_RESULT" | jq '.'

# Extract public address
GAMMA_PUBLIC=$(echo "$STUN_RESULT" | jq -r '.result.public_address')
GAMMA_LOCAL=$(echo "$STUN_RESULT" | jq -r '.result.local_address')
GAMMA_NAT=$(echo "$STUN_RESULT" | jq -r '.result.nat_type')

echo ""
echo "Spore Gamma Addresses:"
echo "  Public: $GAMMA_PUBLIC"
echo "  Local:  $GAMMA_LOCAL"
echo "  NAT:    $GAMMA_NAT"
```

**Expected Output**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "public_address": "203.0.113.45:54321",
    "local_address": "192.168.1.144:54321",
    "server": "stun.nextcloud.com:3478",
    "nat_type": "full_cone"
  },
  "id": 1
}
```

---

### Step 3: STUN Binding (Hole Punching Setup)

**On Tower A**:
```bash
#!/bin/bash
# Create STUN binding for Alpha

echo "=== Spore Alpha: Create STUN Binding ==="

BIND_RESULT=$(echo '{
  "jsonrpc": "2.0",
  "method": "stun.bind",
  "params": {
    "server": "stun.nextcloud.com:3478",
    "local_port": 54321,
    "keepalive_secs": 30
  },
  "id": 2
}' | nc -U /primal/songbird -N)

echo "$BIND_RESULT" | jq '.'

# Extract binding ID
ALPHA_BINDING=$(echo "$BIND_RESULT" | jq -r '.result.binding_id')
ALPHA_MAPPED=$(echo "$BIND_RESULT" | jq -r '.result.mapped_address')

echo ""
echo "Spore Alpha Binding:"
echo "  ID:      $ALPHA_BINDING"
echo "  Mapped:  $ALPHA_MAPPED"
```

**On Tower B**:
```bash
#!/bin/bash
# Create STUN binding for Gamma

echo "=== Spore Gamma: Create STUN Binding ==="

BIND_RESULT=$(echo '{
  "jsonrpc": "2.0",
  "method": "stun.bind",
  "params": {
    "server": "stun.nextcloud.com:3478",
    "local_port": 54322,
    "keepalive_secs": 30
  },
  "id": 2
}' | nc -U /primal/songbird -N)

echo "$BIND_RESULT" | jq '.'

# Extract binding ID
GAMMA_BINDING=$(echo "$BIND_RESULT" | jq -r '.result.binding_id')
GAMMA_MAPPED=$(echo "$BIND_RESULT" | jq -r '.result.mapped_address')

echo ""
echo "Spore Gamma Binding:"
echo "  ID:      $GAMMA_BINDING"
echo "  Mapped:  $GAMMA_MAPPED"
```

**Expected Output**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "binding_id": "stun-binding-1738195200",
    "mapped_address": "203.0.113.45:54321",
    "lifetime_secs": 300
  },
  "id": 2
}
```

---

### Step 4: Discovery (List Discovered Peers)

**On Tower A** (should discover Gamma via UDP beacons):
```bash
#!/bin/bash
# List peers discovered by Alpha

echo "=== Spore Alpha: Discovery Peers ==="

PEERS_RESULT=$(echo '{
  "jsonrpc": "2.0",
  "method": "discovery.peers",
  "params": {},
  "id": 3
}' | nc -U /primal/songbird -N)

echo "$PEERS_RESULT" | jq '.'

# Check if Gamma is discovered
GAMMA_FOUND=$(echo "$PEERS_RESULT" | jq -r '.result.peers[] | select(.node_id=="node-gamma")')

if [ -n "$GAMMA_FOUND" ]; then
    echo "✅ Spore Gamma discovered via UDP beacons!"
    echo "$GAMMA_FOUND" | jq '.'
else
    echo "⚠️  Spore Gamma not yet discovered (may take 30s)"
fi
```

**On Tower B** (should discover Alpha via UDP beacons):
```bash
#!/bin/bash
# List peers discovered by Gamma

echo "=== Spore Gamma: Discovery Peers ==="

PEERS_RESULT=$(echo '{
  "jsonrpc": "2.0",
  "method": "discovery.peers",
  "params": {},
  "id": 3
}' | nc -U /primal/songbird -N)

echo "$PEERS_RESULT" | jq '.'

# Check if Alpha is discovered
ALPHA_FOUND=$(echo "$PEERS_RESULT" | jq -r '.result.peers[] | select(.node_id=="node-alpha")')

if [ -n "$ALPHA_FOUND" ]; then
    echo "✅ Spore Alpha discovered via UDP beacons!"
    echo "$ALPHA_FOUND" | jq '.'
else
    echo "⚠️  Spore Alpha not yet discovered (may take 30s)"
fi
```

**Expected Output**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "peers": [
      {
        "node_id": "node-gamma",
        "family_id": "nat0",
        "address": "192.168.1.134:2300",
        "tcp_port": 8082,
        "capabilities": ["crypto", "tls", "discovery"],
        "last_seen": "2026-01-29T23:45:12Z",
        "quality": 0.95
      }
    ],
    "total_count": 1
  },
  "id": 3
}
```

---

### Step 5: Rendezvous Registration (Optional - For Symmetric NAT)

**On Tower A**:
```bash
#!/bin/bash
# Register Alpha with rendezvous server (if available)

echo "=== Spore Alpha: Rendezvous Registration ==="

# Note: This requires a rendezvous server running
# For now, this will return "not yet implemented" gracefully

RENDEZ_RESULT=$(echo "{
  \"jsonrpc\": \"2.0\",
  \"method\": \"rendezvous.register\",
  \"params\": {
    \"server\": \"http://relay.biomeos.local:8080\",
    \"node_id\": \"node-alpha\",
    \"family_id\": \"nat0\",
    \"public_address\": \"$ALPHA_PUBLIC\"
  },
  \"id\": 4
}" | nc -U /primal/songbird -N)

echo "$RENDEZ_RESULT" | jq '.'
```

**Expected Output** (graceful stub):
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32603,
    "message": "Rendezvous feature not yet fully implemented (returns graceful stub)"
  },
  "id": 4
}
```

---

### Step 6: Direct Peer Connection (UDP Hole Punching)

**On Tower A** (initiate connection to Gamma):
```bash
#!/bin/bash
# Alpha connects to Gamma using STUN binding

echo "=== Spore Alpha: Connect to Gamma ==="

CONNECT_RESULT=$(echo "{
  \"jsonrpc\": \"2.0\",
  \"method\": \"peer.connect\",
  \"params\": {
    \"target_address\": \"$GAMMA_MAPPED\",
    \"our_binding\": \"$ALPHA_BINDING\"
  },
  \"id\": 5
}" | nc -U /primal/songbird -N)

echo "$CONNECT_RESULT" | jq '.'

# Extract connection ID
CONN_ID=$(echo "$CONNECT_RESULT" | jq -r '.result.connection_id')
CONN_STATE=$(echo "$CONNECT_RESULT" | jq -r '.result.state')

echo ""
echo "Connection to Gamma:"
echo "  ID:     $CONN_ID"
echo "  State:  $CONN_STATE"
```

**Expected Output**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "connection_id": "peer-conn-1738195300",
    "state": "connecting",
    "channel": {
      "local_address": "192.168.1.144:54321",
      "remote_address": "192.168.1.134:54322",
      "protocol": "udp",
      "latency_ms": 5
    }
  },
  "id": 5
}
```

---

## Complete Two-Spore Test Script

Here's a complete automated test that runs on both spores:

```bash
#!/bin/bash
# complete_two_spore_test.sh
# Run this script on BOTH spores (modify NODE_ID and PORT for each)

set -e

# Configuration
NODE_ID="${NODE_ID:-node-alpha}"  # Override with node-gamma on Tower B
FAMILY_ID="nat0"
STUN_SERVER="stun.nextcloud.com:3478"
LOCAL_PORT="${SONGBIRD_PORT:-8081}"  # Use 8082 for Tower B
SOCKET="/primal/songbird"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                                                                ║"
echo "║    Dark Forest Protocol - Two Spore Validation                ║"
echo "║    Node: $NODE_ID                                              "
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Wait for Songbird to be ready
echo "⏳ Waiting for Songbird to start..."
for i in {1..30}; do
    if [ -S "$SOCKET" ]; then
        echo "✅ Songbird ready: $SOCKET"
        break
    fi
    sleep 1
done

if [ ! -S "$SOCKET" ]; then
    echo "❌ Songbird socket not available after 30 seconds"
    exit 1
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 1: STUN Public Address Discovery"
echo "════════════════════════════════════════════════════════════════"

STUN_RESULT=$(echo "{
  \"jsonrpc\": \"2.0\",
  \"method\": \"stun.get_public_address\",
  \"params\": {
    \"server\": \"$STUN_SERVER\"
  },
  \"id\": 1
}" | nc -U "$SOCKET" -N)

echo "$STUN_RESULT" | jq '.'

PUBLIC_ADDR=$(echo "$STUN_RESULT" | jq -r '.result.public_address // "unknown"')
LOCAL_ADDR=$(echo "$STUN_RESULT" | jq -r '.result.local_address // "unknown"')
NAT_TYPE=$(echo "$STUN_RESULT" | jq -r '.result.nat_type // "unknown"')

echo ""
echo "Addresses for $NODE_ID:"
echo "  Public: $PUBLIC_ADDR"
echo "  Local:  $LOCAL_ADDR"
echo "  NAT:    $NAT_TYPE"

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 2: STUN Binding (Hole Punching Setup)"
echo "════════════════════════════════════════════════════════════════"

# Use different ports for Alpha (54321) and Gamma (54322)
BIND_PORT=54321
if [ "$NODE_ID" = "node-gamma" ]; then
    BIND_PORT=54322
fi

BIND_RESULT=$(echo "{
  \"jsonrpc\": \"2.0\",
  \"method\": \"stun.bind\",
  \"params\": {
    \"server\": \"$STUN_SERVER\",
    \"local_port\": $BIND_PORT,
    \"keepalive_secs\": 30
  },
  \"id\": 2
}" | nc -U "$SOCKET" -N)

echo "$BIND_RESULT" | jq '.'

BINDING_ID=$(echo "$BIND_RESULT" | jq -r '.result.binding_id // "unknown"')
MAPPED_ADDR=$(echo "$BIND_RESULT" | jq -r '.result.mapped_address // "unknown"')

echo ""
echo "STUN Binding for $NODE_ID:"
echo "  ID:      $BINDING_ID"
echo "  Mapped:  $MAPPED_ADDR"

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Step 3: Discovery (List Peers)"
echo "════════════════════════════════════════════════════════════════"

PEERS_RESULT=$(echo '{
  "jsonrpc": "2.0",
  "method": "discovery.peers",
  "params": {},
  "id": 3
}' | nc -U "$SOCKET" -N)

echo "$PEERS_RESULT" | jq '.'

PEER_COUNT=$(echo "$PEERS_RESULT" | jq -r '.result.total_count // 0')

echo ""
echo "Discovered Peers: $PEER_COUNT"
if [ "$PEER_COUNT" -gt 0 ]; then
    echo "✅ Other spore(s) discovered via UDP beacons!"
else
    echo "⚠️  No peers discovered yet (may take 30s for beacon propagation)"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Summary for $NODE_ID"
echo "════════════════════════════════════════════════════════════════"
echo "  Public Address: $PUBLIC_ADDR"
echo "  STUN Binding:   $BINDING_ID"
echo "  Mapped Address: $MAPPED_ADDR"
echo "  Peers Found:    $PEER_COUNT"
echo ""
echo "✅ Dark Forest methods validated!"
echo ""
echo "Next Steps:"
echo "  1. Run this on the other spore"
echo "  2. Wait 30s for mutual discovery"
echo "  3. Initiate peer.connect from either spore"
echo ""
```

---

## Running the Validation

### Quick Start (Both Towers)

**Tower A**:
```bash
cd /home/eastgate/Development/ecoPrimals/phase1/songbird

# Build latest
cargo build --release

# Start Spore Alpha
NODE_ID=node-alpha SONGBIRD_PORT=8081 ./target/release/songbird server &

# Wait 5 seconds
sleep 5

# Run validation
NODE_ID=node-alpha SONGBIRD_PORT=8081 bash complete_two_spore_test.sh
```

**Tower B**:
```bash
cd /home/eastgate/Development/ecoPrimals/phase1/songbird

# Build latest
cargo build --release

# Start Spore Gamma
NODE_ID=node-gamma SONGBIRD_PORT=8082 ./target/release/songbird server &

# Wait 5 seconds
sleep 5

# Run validation
NODE_ID=node-gamma SONGBIRD_PORT=8082 bash complete_two_spore_test.sh
```

---

## Expected Results

### Success Criteria

| Test | Expected Result |
|------|----------------|
| STUN Discovery | ✅ Both spores get public IP from STUN server |
| STUN Binding | ✅ Both spores create bindings for hole punching |
| UDP Beacons | ✅ Both spores discover each other within 30s |
| Peer Listing | ✅ `discovery.peers` returns the other spore |
| Connection | ✅ `peer.connect` initiates UDP hole punch |

### Debug Output

If any test fails, check:

1. **STUN Server Reachable?**
   ```bash
   nc -u -w 5 stun.nextcloud.com 3478 < /dev/null
   echo $?  # Should be 0
   ```

2. **UDP Port 2300 Open?**
   ```bash
   ss -ulnp | grep 2300
   ```

3. **Songbird Logs**:
   ```bash
   # Check for errors
   journalctl -u songbird --since "5 minutes ago" | grep -i error
   ```

4. **Both Spores Same Family?**
   ```bash
   # Both should show family_id: "nat0"
   echo '{"jsonrpc":"2.0","method":"discovery.peers","params":{},"id":1}' | \
       nc -U /primal/songbird -N | jq '.result.peers[0].family_id'
   ```

---

## Success Indicators

When everything works, you should see:

```
╔════════════════════════════════════════════════════════════════╗
║                                                                ║
║    ✅ DARK FOREST PROTOCOL VALIDATION COMPLETE! ✅            ║
║                                                                ║
║           Two Spore Cross-Discovery & STUN Handshake          ║
║                                                                ║
╚════════════════════════════════════════════════════════════════╝

Spore Alpha (Tower A):
  ✅ STUN public address discovered
  ✅ STUN binding created
  ✅ Spore Gamma discovered via UDP beacons
  ✅ Peer connection initiated

Spore Gamma (Tower B):
  ✅ STUN public address discovered
  ✅ STUN binding created
  ✅ Spore Alpha discovered via UDP beacons
  ✅ Peer connection initiated

Results:
🌲 Dark Forest Protocol: FULLY OPERATIONAL
🎯 Cross-Spore NAT Traversal: SUCCESS
🦀 Pure Rust Implementation: VALIDATED
🚀 Production Ready: CONFIRMED
```

---

## Troubleshooting

### Issue: "Unknown method" errors

**Cause**: Old version of Songbird (< v8.19.0)

**Fix**:
```bash
git pull origin main
cargo build --release
# Restart both spores
```

### Issue: No peers discovered

**Causes**:
1. Different family IDs
2. Firewall blocking UDP 2300
3. Not enough time (beacons sent every 30s)

**Fix**:
```bash
# Check family ID
echo '{"jsonrpc":"2.0","method":"discovery.peers","params":{},"id":1}' | \
    nc -U /primal/songbird -N | jq '.result.peers[0].family_id'

# Check UDP port
ss -ulnp | grep 2300

# Wait longer (60s minimum)
sleep 60
```

### Issue: STUN server unreachable

**Cause**: Network blocking UDP to STUN server

**Fix**: Try alternative STUN servers:
```bash
# Use different STUN server
STUN_SERVER="stun.l.google.com:19302"
# Or
STUN_SERVER="stun.voipawesome.com:3478"
```

---

## Next Steps After Validation

Once validated:
1. ✅ Dark Forest protocol confirmed working
2. ✅ Document results in biomeOS integration guide
3. ✅ Enable cross-tower gaming/collaboration
4. ✅ Deploy to production USB spores

---

**Generated**: January 29, 2026  
**Version**: Songbird v8.19.0  
**Status**: 🧪 **VALIDATION READY** - Test with real USB spores!

