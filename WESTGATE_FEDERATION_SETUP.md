# 🌐 Westgate Federation Setup Guide

**Tower:** Westgate (Cold Storage System)  
**Date:** December 19, 2025  
**Status:** Attempting to join federation  
**Network:** 192.168.1.123 (westgate.local)

---

## 🎯 Current Status

### Network Connectivity ✅
- **Ping:** Success (0.3ms latency)
- **IP:** 192.168.1.123
- **IPv6:** 2600:1700:b0b0:5b90:e2c2:f337:9774:209
- **DNS:** westgate.local resolves correctly

### Songbird Status ⚠️
- **Orchestrator:** Not responding on port 8080
- **Federation:** Not yet visible to eastgate

---

## 🚀 Setup Steps for Westgate

### 1. Verify Songbird is Running on Westgate

**On westgate, check:**
```bash
# Check if orchestrator is running
ps aux | grep songbird-orchestrator

# Check which ports are listening
sudo netstat -tlnp | grep songbird
# or
sudo ss -tlnp | grep songbird
```

### 2. Configuration for Federation

**Environment Variables (on westgate):**
```bash
# Basic Configuration
export SONGBIRD_BIND_ADDRESS="0.0.0.0"  # Bind to all interfaces
export SONGBIRD_PORT="8080"
export SONGBIRD_HOST="westgate.local"

# Federation Discovery
export SONGBIRD_ENABLE_DISCOVERY="true"
export DISCOVERY_PORT="8081"
export SONGBIRD_MDNS_ENABLED="true"

# Broadcast addresses for discovery
export SONGBIRD_BROADCAST_ADDRESSES="255.255.255.255:2300,192.168.1.255:2300"

# Federation endpoints (optional - will auto-discover)
export SONGBIRD_FEDERATION_ENDPOINTS="http://eastgate.local:8080,http://strandgate.local:8080"

# Tower Identity
export TOWER_NAME="westgate"
export TOWER_ROLE="storage"  # Cold storage system
```

### 3. Start Songbird Orchestrator

**On westgate:**
```bash
cd /path/to/songbird

# Build if needed
cargo build --release

# Start orchestrator
./target/release/songbird-orchestrator \
  --bind-address 0.0.0.0:8080 \
  --tower-name westgate \
  --enable-federation \
  --enable-discovery
```

**Or use systemd service:**
```bash
sudo systemctl start songbird-orchestrator
sudo systemctl status songbird-orchestrator
```

### 4. Verify Services are Listening

**On westgate:**
```bash
# Check ports
curl http://localhost:8080/health

# Should return:
# {"status":"ok","service":"westgate-orchestrator",...}
```

### 5. Verify Federation from Eastgate

**On eastgate (this tower):**
```bash
# Discover services
./target/release/songbird-cli discover

# Check registry
curl http://localhost:8080/api/v1/federation/towers

# Ping westgate orchestrator
curl http://westgate.local:8080/health
```

---

## 🔥 Firewall Configuration

### On Westgate (Allow Incoming)

**UFW (Ubuntu/Debian):**
```bash
# Allow Songbird ports
sudo ufw allow 8080/tcp comment "Songbird Orchestrator"
sudo ufw allow 8081/tcp comment "Songbird Discovery"
sudo ufw allow 2300/udp comment "Songbird mDNS/Broadcast"
sudo ufw reload
```

**firewalld (RHEL/CentOS):**
```bash
# Allow Songbird ports
sudo firewall-cmd --permanent --add-port=8080/tcp  # Orchestrator
sudo firewall-cmd --permanent --add-port=8081/tcp  # Discovery
sudo firewall-cmd --permanent --add-port=2300/udp  # mDNS
sudo firewall-cmd --reload
```

**iptables:**
```bash
# Allow Songbird ports
sudo iptables -A INPUT -p tcp --dport 8080 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 8081 -j ACCEPT
sudo iptables -A INPUT -p udp --dport 2300 -j ACCEPT
sudo iptables-save | sudo tee /etc/iptables/rules.v4
```

---

## 📋 Verification Checklist

### Westgate (Local) ✅
- [ ] Songbird binary built (`cargo build --release`)
- [ ] Environment variables configured
- [ ] Orchestrator running (`ps aux | grep songbird`)
- [ ] Port 8080 listening (`netstat -tlnp | grep 8080`)
- [ ] Health endpoint responds (`curl localhost:8080/health`)
- [ ] Firewall allows incoming connections

### From Eastgate (Remote) ✅
- [ ] Can ping westgate (`ping westgate.local`)
- [ ] Can reach health endpoint (`curl westgate.local:8080/health`)
- [ ] Westgate appears in discovery (`songbird-cli discover`)
- [ ] Westgate listed in registry (`curl localhost:8080/api/v1/federation/towers`)

---

## 🔍 Troubleshooting

### Issue: Orchestrator Not Responding

**Check 1: Is it running?**
```bash
# On westgate
ps aux | grep songbird-orchestrator
```

**Check 2: Which address is it bound to?**
```bash
# On westgate
sudo netstat -tlnp | grep songbird
# Look for 0.0.0.0:8080 (good) vs 127.0.0.1:8080 (bad - localhost only)
```

**Check 3: Firewall blocking?**
```bash
# On westgate
sudo ufw status
# or
sudo firewall-cmd --list-all
# or
sudo iptables -L -n
```

---

### Issue: Can't Join Federation

**Check 1: mDNS working?**
```bash
# On eastgate
avahi-browse -a -t
# Should show _songbird._tcp services
```

**Check 2: Broadcast discovery working?**
```bash
# On eastgate
sudo tcpdump -i any udp port 2300
# Should see discovery broadcasts
```

**Check 3: Service registry?**
```bash
# On eastgate
curl http://localhost:8080/api/v1/federation/towers
# Should list all known towers
```

---

## 🎯 Expected Federation Architecture

### Three-Tower Setup
```
eastgate (Orchestration)    ←→    strandgate (GPU Compute)
    ↑                                    ↑
    |                                    |
    └────────→    westgate (Cold Storage)
```

### Port Assignment
| Tower | Orchestrator | Discovery | Role |
|-------|-------------|-----------|------|
| **eastgate** | 8080 | 8081 | Primary orchestrator |
| **strandgate** | 8080 | 8081 | GPU compute |
| **westgate** | 8080 | 8081 | Cold storage |

### Auto-Discovery Flow
1. **westgate** starts → Broadcasts presence on UDP 2300
2. **eastgate** receives → Registers westgate in service registry
3. **strandgate** receives → Registers westgate in service registry
4. All towers can now discover each other via mDNS or registry

---

## 🚀 Quick Start Commands

### On Westgate
```bash
# 1. Navigate to songbird directory
cd /path/to/songbird

# 2. Set environment
export SONGBIRD_BIND_ADDRESS="0.0.0.0"
export SONGBIRD_ENABLE_DISCOVERY="true"
export TOWER_NAME="westgate"

# 3. Start orchestrator
./target/release/songbird-orchestrator

# 4. Verify
curl localhost:8080/health
```

### On Eastgate (Verify)
```bash
# 1. Discover westgate
./target/release/songbird-cli discover

# 2. Check health
curl http://westgate.local:8080/health

# 3. List federation towers
curl http://localhost:8080/api/v1/federation/towers
```

---

## 📊 Real-Time Verification

### Check Federation Status
```bash
# On any tower
./target/release/songbird-cli discover --continuous

# Should show:
# - eastgate (orchestration)
# - strandgate (gpu-compute)
# - westgate (storage) ← NEW!
```

### Monitor Federation Events
```bash
# Watch federation logs
tail -f /var/log/songbird/orchestrator.log

# Should see:
# "New tower discovered: westgate at 192.168.1.123"
# "Federation peer registered: westgate"
# "Health check: westgate OK"
```

---

## 🎉 Success Indicators

### When Westgate Successfully Joins:

1. **Discovery finds it:**
   ```bash
   $ ./target/release/songbird-cli discover
   Found 3 towers:
   - eastgate (orchestration) - 192.168.1.100
   - strandgate (gpu-compute) - 192.168.1.101
   - westgate (storage) - 192.168.1.123 ✓
   ```

2. **Health checks pass:**
   ```bash
   $ curl westgate.local:8080/health
   {"status":"ok","service":"westgate-orchestrator","towers_connected":2}
   ```

3. **Registry shows all three:**
   ```bash
   $ curl localhost:8080/api/v1/federation/towers
   {
     "towers": [
       {"name":"eastgate","status":"connected"},
       {"name":"strandgate","status":"connected"},
       {"name":"westgate","status":"connected"}
     ]
   }
   ```

4. **Can route tasks to westgate:**
   ```bash
   $ ./target/release/songbird-cli task submit \
     --tower westgate \
     --type storage \
     ./my-ml-model.pkl
   ✓ Task submitted to westgate
   ```

---

## 📞 Next Steps

### Immediate
1. ✅ SSH to westgate
2. ✅ Start Songbird orchestrator
3. ✅ Configure firewall
4. ✅ Verify health endpoint
5. ✅ Confirm federation from eastgate

### After Federation
6. Configure storage capabilities on westgate
7. Set up persistent storage paths
8. Configure backup/archival policies
9. Test ML model storage and retrieval
10. Monitor cold storage metrics

---

## 🔐 Security Notes

### For Cold Storage Role
```bash
# Westgate-specific security
export SONGBIRD_ROLE="storage"
export STORAGE_READ_ONLY="false"  # Allow writes for model storage
export STORAGE_ENCRYPTION="true"  # Encrypt stored models
export STORAGE_RETENTION_DAYS="365"  # Keep for 1 year

# Access control
export ALLOW_FEDERATION="true"
export REQUIRE_AUTH="true"  # Require JWT tokens
export STORAGE_QUOTA_GB="1000"  # 1TB quota
```

---

**Status:** ⚠️ **SETUP IN PROGRESS**  
**Network:** ✅ Reachable  
**Orchestrator:** ⚠️ Not yet responding  
**Federation:** 🔜 Pending westgate startup

**Next:** Start Songbird on westgate and verify!

