# 🚀 Westgate Quick Start - Join the Federation

**Status:** Westgate is trying to join the federation  
**Network:** ✅ Reachable at 192.168.1.123 (westgate.local)  
**Songbird:** ⚠️ Not running yet  
**Next:** Start Songbird on westgate

---

## ⚡ Quick Actions

### On Westgate (the cold storage tower):

```bash
# 1. Copy the startup script (if needed)
scp eastgate:/path/to/songbird/start_westgate_tower.sh ~/

# 2. Navigate to Songbird directory
cd /path/to/songbird

# 3. Run the startup script
./start_westgate_tower.sh

# Or manually:
export SONGBIRD_BIND_ADDRESS="0.0.0.0"
export TOWER_NAME="westgate"
./target/release/songbird-orchestrator
```

### Verify From Eastgate (this tower):

```bash
# 1. Check health
curl http://westgate.local:8080/health

# 2. Discover towers
./target/release/songbird-cli discover

# 3. Verify federation
./verify_westgate_federation.sh
```

---

## 📊 Current Status

**Network Connectivity:** ✅
```
✅ Ping: 0.3ms latency
✅ IP: 192.168.1.123
✅ DNS: westgate.local resolves
```

**Songbird Status:** ⚠️
```
⚠️  Port 8080: Closed (orchestrator)
⚠️  Port 8081: Closed (discovery)
⚠️  Port 2300: Closed (mDNS)
→ Songbird not running yet
```

**Federation:** 🔜 Pending
```
Current towers:
✅ eastgate (orchestration) - 192.168.1.100
✅ strandgate (gpu-compute) - 192.168.1.101
🔜 westgate (storage) - 192.168.1.123 ← Joining now
```

---

## 🎯 What Needs to Happen

### 1. On Westgate - Start Songbird ⚠️ (Action Required)

**Option A: Use the startup script (easiest)**
```bash
cd /path/to/songbird
./start_westgate_tower.sh
```

**Option B: Manual start**
```bash
# Set environment
export SONGBIRD_BIND_ADDRESS="0.0.0.0"
export TOWER_NAME="westgate"
export SONGBIRD_ENABLE_DISCOVERY="true"

# Start orchestrator
./target/release/songbird-orchestrator \
  --bind-address 0.0.0.0:8080 \
  --tower-name westgate \
  --enable-federation \
  --enable-discovery
```

**Option C: Background service**
```bash
# Start in background
nohup ./target/release/songbird-orchestrator \
  --bind-address 0.0.0.0:8080 \
  --tower-name westgate \
  > /tmp/westgate.log 2>&1 &

# Save PID
echo $! > /tmp/westgate.pid
```

### 2. On Westgate - Open Firewall

```bash
# UFW (Ubuntu/Debian)
sudo ufw allow 8080/tcp comment "Songbird"
sudo ufw allow 8081/tcp comment "Discovery"
sudo ufw allow 2300/udp comment "mDNS"

# Or firewalld (RHEL/CentOS)
sudo firewall-cmd --permanent --add-port=8080/tcp
sudo firewall-cmd --permanent --add-port=8081/tcp
sudo firewall-cmd --permanent --add-port=2300/udp
sudo firewall-cmd --reload
```

### 3. On Westgate - Verify

```bash
# Check if it's running
ps aux | grep songbird-orchestrator

# Check health
curl localhost:8080/health

# Should see:
# {"status":"ok","service":"westgate-orchestrator",...}
```

### 4. On Eastgate - Verify Federation

```bash
# Run verification
./verify_westgate_federation.sh

# Should show:
# ✅ Port 8080 is open
# ✅ Westgate orchestrator responding
# ✅ Discovered in federation
```

---

## ✅ Success Indicators

### When Westgate Successfully Joins:

**1. Health endpoint responds:**
```bash
$ curl westgate.local:8080/health
{"status":"ok","service":"westgate-orchestrator","towers_connected":2}
```

**2. Discovery finds it:**
```bash
$ ./target/release/songbird-cli discover
Found 3 towers:
- eastgate (orchestration)
- strandgate (gpu-compute)
- westgate (storage) ✓ NEW!
```

**3. Can route tasks to it:**
```bash
$ curl http://localhost:8080/api/v1/federation/towers
{
  "towers": [
    {"name":"eastgate","status":"connected"},
    {"name":"strandgate","status":"connected"},
    {"name":"westgate","status":"connected"}
  ]
}
```

---

## 🔧 Troubleshooting

### Westgate Orchestrator Won't Start

**Check 1: Binary built?**
```bash
ls -lh ./target/release/songbird-orchestrator
# If not: cargo build --release
```

**Check 2: Port already in use?**
```bash
sudo netstat -tlnp | grep 8080
# If occupied: use different port or kill process
```

**Check 3: Permissions?**
```bash
# Check if binary is executable
chmod +x ./target/release/songbird-orchestrator
```

---

### Firewall Still Blocking

**Check current rules:**
```bash
sudo ufw status verbose
# or
sudo firewall-cmd --list-all
# or
sudo iptables -L -n
```

**Temporarily disable to test:**
```bash
sudo ufw disable
# Test connection
curl westgate.local:8080/health
# Re-enable
sudo ufw enable
```

---

### Discovery Not Finding Westgate

**Check 1: mDNS working?**
```bash
# On eastgate
ping -c 2 westgate.local
# Should resolve to 192.168.1.123
```

**Check 2: Broadcast discovery?**
```bash
# On eastgate - listen for broadcasts
sudo tcpdump -i any udp port 2300

# On westgate - trigger broadcast
curl localhost:8080/api/v1/discovery/announce
```

**Check 3: Manual registration?**
```bash
# On eastgate - manually register westgate
curl -X POST http://localhost:8080/api/v1/federation/towers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "westgate",
    "endpoint": "http://192.168.1.123:8080",
    "role": "storage"
  }'
```

---

## 📁 Files Created

1. **WESTGATE_FEDERATION_SETUP.md** - Comprehensive setup guide
2. **start_westgate_tower.sh** - Quick startup script for westgate
3. **verify_westgate_federation.sh** - Diagnostic script (run from eastgate)
4. **WESTGATE_QUICK_START.md** - This file (quick reference)

---

## 🎯 Summary

**Current Situation:**
- Westgate is network-reachable ✅
- Songbird needs to be started on westgate ⚠️
- Once started, auto-discovery will handle federation ✨

**What You Need to Do:**
1. SSH to westgate
2. Navigate to Songbird directory
3. Run: `./start_westgate_tower.sh`
4. Verify: `curl localhost:8080/health`
5. Check from eastgate: `./verify_westgate_federation.sh`

**Expected Result:**
Three-tower federation running with automatic discovery! 🎉

---

**Next:** SSH to westgate and start Songbird!

```bash
ssh westgate
cd /path/to/songbird
./start_westgate_tower.sh
```

Then verify from eastgate:
```bash
./verify_westgate_federation.sh
```

---

**🎵 Capability-based discovery in action! Zero hardcoded IPs needed!** ✨

