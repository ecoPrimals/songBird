# 🔍 Federation Monitoring Guide

## Real-Time Monitoring Tools

### 1. Watch for Westgate Connection
```bash
./watch-for-westgate.sh
```

**What it does**:
- Monitors eastgate logs in real-time
- Filters for westgate-related events
- Shows discovery, trust, and federation join
- Auto-exits when federation succeeds

**Output**:
```
🔍 Watching for Westgate Connection...
========================================

Monitoring eastgate logs for:
  - Westgate discovery
  - Trust establishment
  - Federation join

Watching logs...

🎯 [14:53:22] DISCOVERY: Discovered peer: westgate at 192.168.1.123:8080
🤝 [14:53:25] TRUST: Trust established with westgate (anonymous level)
🎊 [14:53:28] FEDERATION: Federation node joined: westgate
🌐 [14:53:30] SUCCESS: Federation status: Active nodes: 2

✅ Westgate has joined the federation!
```

### 2. Check Current Status
```bash
./check-tower.sh
```

**Quick status check**:
- Shows if eastgate is running
- Current federation node count
- Recent activity

### 3. Monitor Discovery Broadcasts
```bash
sudo tcpdump -i any 'udp port 2300' -n -A | grep -E "session_id|node_id|capabilities"
```

**What it shows**:
- Real UDP discovery packets
- Session IDs (rotating for anonymity)
- Node IDs and capabilities
- Confirms both towers are broadcasting

### 4. Monitor Federation API
```bash
watch -n 5 'curl -sk https://localhost:8080/api/federation/status | jq ".active_nodes"'
```

**What it does**:
- Polls federation status every 5 seconds
- Shows active node count
- Updates automatically when westgate joins

---

## Logging Strategy

### Log Levels
- **INFO**: Normal operation (discovery, federation events)
- **DEBUG**: Detailed discovery bridge activity
- **WARN**: Potential issues (timeouts, retries)
- **ERROR**: Actual problems (connection failures)

### Key Log Patterns to Watch

#### Discovery Phase
```
[INFO] Starting anonymous discovery broadcaster on port 2300
[INFO] Starting anonymous discovery listener on port 2300
[INFO] Broadcasting discovery message with session_id: [rotating-id]
```

#### Peer Discovery
```
[INFO] Discovered peer: westgate at 192.168.1.123:8080
[INFO] Peer capabilities: ["compute", "storage"]
[DEBUG] Discovery bridge: Processing 1 discovered peers
```

#### Trust Establishment
```
[INFO] Establishing trust with peer: westgate (192.168.1.123:8080)
[INFO] Trust level: Anonymous
[DEBUG] Trust handshake initiated with westgate
```

#### Federation Join
```
[INFO] Federation node joined: westgate
[INFO] Federation status: Active nodes: 2
[INFO] Node westgate capabilities: ["compute", "storage"]
```

---

## Monitoring Commands Reference

### Continuous Monitoring
```bash
# Watch for westgate (filtered, auto-exit on success)
./watch-for-westgate.sh

# Watch all eastgate logs (full detail)
tail -f logs/eastgate-*.log

# Watch only discovery events
tail -f logs/eastgate-*.log | grep Discovery

# Watch only federation events
tail -f logs/eastgate-*.log | grep Federation

# Watch for errors
tail -f logs/eastgate-*.log | grep -i error
```

### One-Time Checks
```bash
# Current status
./check-tower.sh

# Federation API (raw JSON)
curl -sk https://localhost:8080/api/federation/status | jq '.'

# Active nodes count
curl -sk https://localhost:8080/api/federation/status | jq '.active_nodes'

# List all nodes
curl -sk https://localhost:8080/api/federation/nodes | jq '.'

# Check specific node
curl -sk https://localhost:8080/api/federation/nodes/westgate | jq '.'
```

### Network Verification
```bash
# Check UDP discovery broadcasts
sudo tcpdump -i any 'udp port 2300' -n -c 10

# Check if westgate's HTTPS port is reachable
curl -sk https://192.168.1.123:8080/api/health

# Check network connectivity
ping -c 3 192.168.1.123

# Check if discovery port is listening
ss -ulnp | grep 2300

# Check if HTTPS port is listening
ss -tlnp | grep 8080
```

---

## Expected Timeline

### After Westgate Starts

**T+0 seconds** (Westgate starts)
```
[Westgate] Starting Songbird Tower...
[Westgate] HTTPS server started on port 8080
[Westgate] Starting discovery broadcaster
```

**T+5-10 seconds** (First broadcasts)
```
[Westgate] Broadcasting discovery message
[Eastgate] No log entry yet (broadcasts are async)
```

**T+30 seconds** (Eastgate discovery cycle)
```
[Eastgate] Discovered peer: westgate at 192.168.1.123:8080
[Eastgate] Peer capabilities: ["compute", "storage"]
```

**T+40 seconds** (Discovery bridge processes peer)
```
[Eastgate] Discovery bridge: Processing 1 discovered peers
[Eastgate] Bridge found peer: westgate (192.168.1.123:8080)
```

**T+50-60 seconds** (Trust establishment - WHEN IMPLEMENTED)
```
[Eastgate] Establishing trust with westgate
[Eastgate] Trust level: Anonymous
[Westgate] Trust request from eastgate accepted
```

**T+60-90 seconds** (Federation join - WHEN IMPLEMENTED)
```
[Eastgate] Federation node joined: westgate
[Eastgate] Federation status: Active nodes: 2
[Westgate] Joined federation: 1bc50902...
[Westgate] Federation status: Active nodes: 2
```

---

## Current Status (Dec 19, 2025 - 15:00)

### Foundation Complete ✅
- ✅ Discovery protocol v2.1 (with port)
- ✅ Discovery bridge integrated
- ✅ Multi-federation architecture
- ✅ IPv4 binding fixed
- ✅ Monitoring tools created

### Integration Pending ⏳
- ⏳ Trust establishment call (foundation ready)
- ⏳ Auto-join logic (bridge polls, needs trust integration)
- ⏳ Federation API update (when nodes join)

### What You'll See Now
When westgate starts, eastgate logs will show:
```
[INFO] Discovered peer: westgate at 192.168.1.123:8080
[DEBUG] Discovery bridge: Processing 1 discovered peers
[DEBUG] Bridge found peer: westgate (192.168.1.123:8080)
```

**This confirms discovery is working!**

The bridge is logging discovered peers, which is the critical first step.

---

## Next Integration Steps

### To Complete Auto-Join

In `crates/songbird-orchestrator/src/app/mod.rs`, the bridge currently logs peers:
```rust
debug!("Bridge found peer: {} ({})", peer.node_id, peer.address);
```

**Next**: Add trust establishment call:
```rust
// Establish anonymous trust
let trust = self.trust_escalation_manager
    .establish_trust(
        peer.node_id.clone(),
        peer.address.clone(),
        TrustLevel::Anonymous,
    ).await?;

// Join to federation via router
let federation_id = self.discovery_router
    .route_peer(&peer, &self.multi_federation_state)
    .await?;

info!("Peer {} joined federation {}", peer.node_id, federation_id);
```

**Estimated**: 20-30 minutes to integrate

---

## Troubleshooting with Monitoring

### Problem: No "Discovered peer" log entries
**Check**:
```bash
# Is discovery listener running?
ss -ulnp | grep 2300

# Are broadcasts arriving?
sudo tcpdump -i any 'udp port 2300' -n -c 5
```

**Expected**: UDP packets every 30 seconds from 192.168.1.123

### Problem: "Discovered peer" but no bridge activity
**Check**:
```bash
# Is bridge running?
tail -f logs/eastgate-*.log | grep "Discovery bridge"
```

**Expected**: Log entry every 10 seconds showing peer count

### Problem: Bridge processes peer but no federation join
**Status**: Expected! Trust integration is next step (see above)

**Confirms**: Discovery and bridge are working correctly

---

## Success Indicators

### ✅ Discovery Working
- `ss -ulnp | grep 2300` shows listener
- `tcpdump` shows UDP packets
- Logs show "Broadcasting discovery message"

### ✅ Bridge Working
- Logs show "Discovery bridge: Processing N discovered peers"
- Logs show "Bridge found peer: westgate"
- Bridge polls every 10 seconds

### ✅ Foundation Complete
- Discovery v2.1 includes port
- Multi-federation code integrated
- Trust manager initialized
- Router ready

### ⏳ Integration Pending
- Trust establishment call needed
- Federation join logic needed
- API update needed

---

## Monitoring Best Practices

### During Deployment
1. Start `./watch-for-westgate.sh` before westgate starts
2. Watch for discovery confirmation within 30-60 seconds
3. Check `./check-tower.sh` after 2 minutes

### During Operation
1. Check logs daily: `tail -100 logs/eastgate-*.log`
2. Monitor federation status: `./check-tower.sh`
3. Watch for errors: `grep -i error logs/eastgate-*.log`

### For Debugging
1. Capture full logs: `cat logs/eastgate-*.log > debug.txt`
2. Capture network: `sudo tcpdump -i any 'udp port 2300' -w discovery.pcap`
3. Export federation state: `curl -sk https://localhost:8080/api/federation/status > federation.json`

---

**📊 Comprehensive monitoring is now in place!** 🎯

