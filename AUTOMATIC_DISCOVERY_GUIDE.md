# 🌐 Automatic Discovery Guide - No Manual Ports!

**Philosophy:** Songbird manages ports automatically. You should NEVER need to manually configure ports for federation!

---

## ✅ How It Should Work

### 1. Start Eastgate (Done!)
```bash
SONGBIRD_TLS_ENABLED=true \
SONGBIRD_FEDERATION_ENABLED=true \
SONGBIRD_ANONYMOUS_DISCOVERY=true \
./target/release/songbird-orchestrator
```

**What Happens:**
- ✅ Selects available port automatically (8080 in this case)
- ✅ Starts HTTPS server on that port
- ✅ Starts UDP discovery listener on port 2300
- ✅ Broadcasts presence every 30 seconds
- ✅ Listens for other towers

### 2. Start Westgate (Needed!)
```bash
# On westgate machine, same command:
SONGBIRD_TLS_ENABLED=true \
SONGBIRD_FEDERATION_ENABLED=true \
SONGBIRD_ANONYMOUS_DISCOVERY=true \
./target/release/songbird-orchestrator
```

**What Should Happen:**
- ✅ Westgate selects its own available port (could be 8080, 8443, anything)
- ✅ Westgate starts HTTPS server
- ✅ Westgate starts UDP discovery on port 2300
- ✅ Westgate broadcasts presence every 30 seconds
- ✅ Eastgate receives westgate's broadcast
- ✅ Westgate receives eastgate's broadcast
- ✅ They connect automatically!

---

## 🔍 Current Status

### Eastgate ✅
```
Status: RUNNING
HTTPS: Port 8080 (auto-selected)
Discovery: UDP 2300 (listening and broadcasting)
Federation ID: fd796e08-2ca0-4410-ada7-2ea8b4f55f23
```

### Westgate ❓
```
Status: Unknown (user says running)
HTTPS: Unknown port (auto-selected)
Discovery: Unknown (should be UDP 2300)
Problem: Not receiving broadcasts from eastgate OR not broadcasting
```

---

## 🚫 What You Should NOT Do

### ❌ Manual Port Configuration
```bash
# DON'T DO THIS:
./connect_to_westgate.sh 8080  # Manual port = OpSec risk!
```

**Why Not:**
- Exposes port information unnecessarily
- Defeats the purpose of automatic discovery
- Creates manual configuration burden
- Not sovereign/capability-based

### ❌ Port Scanning
```bash
# DON'T DO THIS:
nmap 192.168.1.123  # Scanning = OpSec risk!
```

**Why Not:**
- Reveals network topology
- Creates security logs
- Unnecessary if discovery works

---

## ✅ What You SHOULD Do

### 1. Verify Westgate is Running Correctly

**On westgate machine:**
```bash
# Check if orchestrator is running
ps aux | grep songbird-orchestrator

# Check if discovery is listening
sudo lsof -i UDP:2300 -P -n

# Check logs for discovery activity
tail -f /path/to/westgate.log | grep -i discovery
```

**Expected Output:**
```
songbird-orchestrator ... UDP *:2300 (LISTEN)
✅ Anonymous discovery listener initialized (port 2300)
✅ Anonymous discovery started (UDP port 2300)
🌐 Broadcasting presence...
```

### 2. Verify Network Connectivity

**On either machine:**
```bash
# Test UDP connectivity (both directions)
# On eastgate:
echo "test" | nc -u 192.168.1.123 2300

# On westgate:
echo "test" | nc -u 192.168.1.144 2300
```

### 3. Check Firewall Rules

**On both machines:**
```bash
# Check firewall status
sudo ufw status

# If needed, allow UDP 2300
sudo ufw allow 2300/udp comment "Songbird Discovery"
```

### 4. Wait for Automatic Discovery

**Timeline:**
- Broadcasts happen every 30 seconds
- Discovery should complete within 60 seconds
- Trust establishment: immediate (Anonymous level)
- Progressive escalation: over time

**Monitor:**
```bash
# On eastgate:
tail -f eastgate_secure_federation.log | grep -i "discovered\|peer\|westgate"

# Check federation status:
curl -k https://localhost:8080/api/federation/status | jq '.active_nodes'
# Should show: 2 (when connected)
```

---

## 🔧 Troubleshooting

### Issue: No Discovery After 2 Minutes

**Possible Causes:**
1. Westgate not running with discovery enabled
2. Firewall blocking UDP 2300
3. Different subnets (broadcast doesn't cross routers)
4. Westgate using wrong broadcast address

**Solutions:**

#### 1. Verify Westgate Configuration
```bash
# On westgate, check environment:
env | grep SONGBIRD

# Should show:
# SONGBIRD_FEDERATION_ENABLED=true
# SONGBIRD_ANONYMOUS_DISCOVERY=true
# SONGBIRD_TLS_ENABLED=true
```

#### 2. Check Firewall
```bash
# On both machines:
sudo ufw allow 2300/udp
sudo ufw reload
```

#### 3. Test UDP Connectivity
```bash
# On eastgate, listen:
sudo tcpdump -i any udp port 2300 -v

# On westgate, send:
echo "test" | nc -u 255.255.255.255 2300
```

#### 4. Check Broadcast Addresses
```bash
# On both machines:
ip addr show | grep "inet "
# Verify they're on same subnet (e.g., 192.168.1.x)
```

---

## 🎯 Expected Behavior

### Phase 1: Initial Broadcast (0-30s)
```
Eastgate: Broadcasting presence on 255.255.255.255:2300
Westgate: Broadcasting presence on 255.255.255.255:2300
```

### Phase 2: Discovery (30-60s)
```
Eastgate: Received discovery message from anonymous peer
Westgate: Received discovery message from anonymous peer
Both: Exchanging capabilities
```

### Phase 3: Trust Establishment (60-90s)
```
Both: Established Anonymous trust (Level 0)
Both: Exchanging session IDs
Both: Verifying capabilities
```

### Phase 4: Progressive Escalation (90s+)
```
Both: Escalating to CapabilityVerified (Level 1)
Both: Can now coordinate tasks
Both: Graduated disclosure active
```

### Phase 5: Connected (Complete)
```
Federation Status:
  Active Nodes: 2
  Nodes: [eastgate, westgate]
  Trust: CapabilityVerified or higher
```

---

## 📊 Verification Commands

### Check Federation Status
```bash
curl -k https://localhost:8080/api/federation/status | jq '.'
```

**Expected:**
```json
{
  "active_nodes": 2,
  "nodes": [
    {
      "node_id": "eastgate",
      "node_address": "192.168.1.144:8080",
      ...
    },
    {
      "node_id": "westgate",
      "node_address": "192.168.1.123:XXXX",
      ...
    }
  ]
}
```

### Check Discovered Peers
```bash
curl -k https://localhost:8080/api/federation/nodes | jq '.'
```

### Test Graduated Disclosure
```bash
# Anonymous level (no auth):
curl -k https://localhost:8080/api/federation/nodes/westgate | jq '.'
# Should show: node_id, capabilities only

# With trust (after escalation):
# Should show: + node_name, status, resources
```

---

## 🎊 Success Criteria

Federation is working when:

1. ✅ `active_nodes: 2` in federation status
2. ✅ Both towers visible in `/api/federation/nodes`
3. ✅ Trust level established (at least Anonymous)
4. ✅ Graduated disclosure working (different info at different trust levels)
5. ✅ No manual port configuration needed
6. ✅ Automatic discovery and connection

---

## 💡 Key Principles

### Songbird Philosophy
1. **Zero Configuration** - Just start it, it works
2. **Secure by Default** - TLS, anonymous, zero-trust
3. **Automatic Discovery** - No manual networking
4. **Progressive Trust** - Start anonymous, escalate as needed
5. **Graduated Disclosure** - Share only what's appropriate
6. **OpSec Conscious** - No unnecessary information exposure

### What This Means
- ❌ No manual port configuration
- ❌ No port scanning
- ❌ No hardcoded IPs
- ✅ Automatic discovery
- ✅ Automatic port selection
- ✅ Automatic trust establishment

---

## 📋 Westgate Startup Checklist

**On westgate machine, run:**

```bash
# 1. Navigate to songbird directory
cd /path/to/songbird

# 2. Build if needed
cargo build --release

# 3. Start with secure federation
SONGBIRD_TLS_ENABLED=true \
SONGBIRD_FEDERATION_ENABLED=true \
SONGBIRD_ANONYMOUS_DISCOVERY=true \
SONGBIRD_NODE_NAME="westgate" \
./target/release/songbird-orchestrator

# 4. Verify it started
ps aux | grep songbird-orchestrator
sudo lsof -i UDP:2300 -P -n

# 5. Wait 60 seconds for discovery

# 6. Check if connected to eastgate
curl -k https://localhost:PORT/api/federation/status
# (PORT will be auto-selected, check logs for actual port)
```

---

## 🎯 Bottom Line

**You're right!** Songbird SHOULD handle ports automatically. The discovery system is designed for exactly this - zero manual configuration.

**Current Status:**
- ✅ Eastgate: Ready and broadcasting
- ❓ Westgate: Needs to be started with same environment variables
- ⏳ Connection: Will happen automatically once westgate is broadcasting

**No manual port configuration needed!** Just start westgate with the right environment variables and let discovery do its job! 🚀

---

**Next:** Start westgate with `SONGBIRD_FEDERATION_ENABLED=true` and `SONGBIRD_ANONYMOUS_DISCOVERY=true`, then wait 60 seconds for automatic discovery!

