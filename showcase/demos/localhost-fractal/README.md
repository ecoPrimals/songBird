# 🎭 Localhost Fractal Demo

**Purpose**: Demonstrate Songbird's fractal scaling on a single machine  
**Time**: 5 minutes  
**Hardware**: Any laptop (Linux/macOS)

---

## 📋 What This Demonstrates

✅ **Multi-Instance Support**
- 14 separate Songbird processes on one machine
- Each with unique identity (NODE_ID-scoped)
- No conflicts (unique sockets per instance)

✅ **Fractal Hierarchy**
- 1 Albatross (high-capacity hub)
- 3 Songbirds (regional coordinators)
- 10 Sparrows (edge sensors)

✅ **P2P Discovery**
- All nodes discover each other automatically
- Encrypted multicast (BirdSong)
- Zero manual configuration

✅ **Capability Registry**
- Query by capability (e.g., "find all sensors")
- O(1) lookup
- Dynamic service discovery

---

## 🚀 Quick Start

### Run the Demo

```bash
./run-demo.sh
```

**What happens**:
1. Starts 1 Albatross on port 8080
2. Starts 3 Songbirds on ports 8081-8083
3. Starts 10 Sparrows on ports 8091-8100
4. Waits 15 seconds for mesh formation
5. Queries Albatross for discovered peers

**Expected output**:
```
✅ Albatross discovered: 14 peers
✅ Mesh formation successful!
```

### Query Mesh Status

```bash
./query-mesh.sh
```

**Shows**:
- Total discovered peers
- Breakdown by role (Albatross/Songbird/Sparrow)
- Peer details (ID + capabilities)

### Stop the Demo

```bash
./stop-demo.sh
```

**Cleans up**:
- Terminates all Songbird processes
- Removes Unix sockets
- Removes PID files
- Preserves logs

---

## 📊 Architecture

```
        Albatross-Main (8080)
       ↙      ↓      ↘
   Song-1  Song-2  Song-3
   (8081)  (8082)  (8083)
     ↓       ↓       ↓
  [🐦 🐦] [🐦 🐦] [🐦 🐦]
  Sparrow-001 to Sparrow-010
  (8091-8100)

All discovering each other via:
  Multicast: 239.255.42.99:4242
  Family: demo-fractal
  Encryption: BirdSong (family-specific keys)
```

---

## 🔍 Verification Commands

### Check Running Processes

```bash
ps aux | grep songbird-orchestrator | wc -l
# Should show 14 (+ grep itself)
```

### Check Unix Sockets

```bash
ls -la /tmp/songbird-demo-fractal-*.sock
# Should show 14 sockets
```

### Query Specific Node

```bash
# Query a Songbird
echo '{"jsonrpc":"2.0","method":"primal.list_all","id":1}' | \
  nc -U /tmp/songbird-demo-fractal-songbird-tower-1.sock | jq

# Query a Sparrow
echo '{"jsonrpc":"2.0","method":"primal.list_all","id":1}' | \
  nc -U /tmp/songbird-demo-fractal-sparrow-sensor-001.sock | jq
```

### Check Logs

```bash
tail -f /tmp/songbird-demo-logs/*.log

# Or specific node:
tail -f /tmp/songbird-demo-logs/albatross-main.log
```

---

## 🧪 Experiments

### 1. Add More Sparrows

Edit `run-demo.sh` and change:
```bash
for i in {1..10}; do
```
to:
```bash
for i in {1..20}; do
```

Re-run and watch 20 Sparrows join!

### 2. Simulate Node Failure

```bash
# Kill a Songbird
pkill -f "songbird-tower-2"

# Query mesh - should still show other peers
./query-mesh.sh

# Restart it
export SONGBIRD_FAMILY_ID="demo-fractal"
export SONGBIRD_NODE_ID="songbird-tower-2"
export SONGBIRD_PORT=8082
# ... (copy config from run-demo.sh)

# It will rejoin automatically!
```

### 3. Test Capability Query

```bash
# Find all sensors
echo '{
  "jsonrpc": "2.0",
  "method": "primal.list_providers",
  "params": {"capability": "sensor"},
  "id": 1
}' | nc -U /tmp/songbird-demo-fractal-albatross-main.sock | jq
```

### 4. Test Cross-Tier Discovery

```bash
# Ask a Sparrow who it sees
echo '{"jsonrpc":"2.0","method":"primal.list_all","id":1}' | \
  nc -U /tmp/songbird-demo-fractal-sparrow-sensor-001.sock | jq

# Should see: All 14 peers (Albatross + Songbirds + other Sparrows)
```

---

## 📈 Performance Observations

On a typical laptop (8 cores, 16GB RAM):

- **Startup time**: ~5 seconds (all 14 nodes)
- **Mesh formation**: ~10 seconds (full discovery)
- **Memory per node**: 
  - Albatross: ~50MB
  - Songbird: ~30MB
  - Sparrow: ~20MB
- **Total memory**: ~450MB for 14 nodes
- **CPU usage**: <5% idle, <20% during discovery

---

## 🐛 Troubleshooting

### "Binary not found"

**Problem**: `run-demo.sh` can't find the Songbird binary

**Solution**:
```bash
cd /home/eastgate/Development/ecoPrimals/phase1/songbird
cargo build --release
cp target/release/songbird-orchestrator /home/eastgate/Development/ecoPrimals/primalBins/songbird-orchestrator-v3.7.3-multiinstance
```

### "Address already in use"

**Problem**: Port conflict (8080-8100 range)

**Solution**:
```bash
# Check what's using the ports
lsof -i :8080-8100

# Stop conflicting processes or change ports in run-demo.sh
```

### "No peers discovered"

**Problem**: Multicast not working (firewall or network)

**Solution**:
```bash
# Allow multicast
sudo iptables -A INPUT -d 239.255.42.99 -j ACCEPT
sudo iptables -A OUTPUT -d 239.255.42.99 -j ACCEPT

# Check multicast traffic
sudo tcpdump -i any host 239.255.42.99
```

### "Socket not found"

**Problem**: Node didn't start properly

**Solution**:
```bash
# Check logs
tail /tmp/songbird-demo-logs/*.log

# Look for errors in startup
grep -i error /tmp/songbird-demo-logs/*.log
```

---

## 🎓 Learning Exercises

### Exercise 1: Understand Discovery

1. Run demo: `./run-demo.sh`
2. Watch multicast traffic: `sudo tcpdump -i any host 239.255.42.99`
3. Observe: Encrypted announcements every 5 seconds
4. Kill a node, watch others detect failure

### Exercise 2: Query Capabilities

1. Run demo
2. Query for different capabilities:
   - `"sensor"` → Should return 10 Sparrows
   - `"orchestrator"` → Should return 3 Songbirds
   - `"multiplexer"` → Should return 1 Albatross

### Exercise 3: Scale It Up

1. Modify `run-demo.sh` to create 50 Sparrows
2. Run and time how long mesh formation takes
3. Query mesh status
4. Observe: Still forms in ~15 seconds (linear scaling!)

---

## 🔗 Related Documentation

- **Vision**: `../../whitePaper/FRACTAL_COORDINATION_WHITEPAPER.md`
- **Technical**: `../../whitePaper/SPARROW_SWARM_NETWORKS_HPC.md`
- **Deployment**: `../../whitePaper/SPARROW_DEPLOYMENT_EXAMPLES.md`

---

## 🎊 What's Next?

After running this demo, try:

1. **Multi-Family Demo** (`../multi-family/`)
   - Shows cryptographic isolation
   - 3 families on same network

2. **Circuit Switching Demo** (`../circuit-switching/`)
   - Dedicated path establishment
   - Resource reservation

3. **Production Deployment**
   - Use `../../scripts/deploy-iot-mesh.sh` for real hardware

---

**Version**: 1.0  
**Last Updated**: January 4, 2026  
**Status**: ✅ Fully functional, ready to run

🎭 **Experience the fractal mesh!** 🦅🎵🐦

