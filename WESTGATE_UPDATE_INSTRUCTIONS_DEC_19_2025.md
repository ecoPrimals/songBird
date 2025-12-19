# Westgate Update Instructions - Trust Integration

**Date:** December 19, 2025  
**Priority:** 🔥 **HIGH** - Required for full federation  
**Estimated Time:** 5 minutes

---

## 🎯 What's New

Eastgate has been updated with **trust establishment** integration! The discovery bridge now automatically:
1. Discovers peers via UDP broadcast
2. Establishes anonymous trust
3. Registers peers in federation
4. Sends heartbeats to maintain connection

**Your westgate tower needs this update to complete the federation!**

---

## 🚀 Quick Update (5 minutes)

### Step 1: Pull Latest Code
```bash
cd ~/songbird
git pull
```

### Step 2: Rebuild
```bash
cargo build --release
```

### Step 3: Restart Tower
```bash
./stop-tower.sh
./start-tower.sh
```

### Step 4: Verify
```bash
./check-tower.sh
```

**Expected output:**
```
🌐 Federation Status:
  Federation ID: [uuid]
  Active Nodes: 2 (or more)
  ✅ Connected to federation!
```

---

## 🔍 What You Should See

### Immediately After Start
```
✅ HTTPS Server: Port 8080
✅ Discovery: UDP port 2300 (broadcasting & listening)
🌐 Federation Status:
  Mode: Zero-trust with progressive escalation
  Discovery: Automatic (no manual configuration)
```

### After 10-30 Seconds (Discovery)
```
INFO: 🔍 Discovered peer: [session_id] at https://192.168.1.144:8080 (capabilities: ["orchestration", "federation"])
INFO: ✅ Trust established with [short_id] (level: Anonymous)
INFO: 🤝 Peer [short_id] joined federation (anonymous trust)
```

### After 30-60 Seconds (Heartbeats)
```
DEBUG: 💓 Sending heartbeats to 2 nodes
DEBUG: 💓 Heartbeat sent to peer-[id]
```

---

## 🧪 Testing the Connection

### From Westgate
```bash
# Check federation status
curl -k https://localhost:8080/api/federation/status | jq '.'

# Expected: Should show eastgate (192.168.1.144) in nodes list
```

### From Eastgate (we'll check)
```bash
# Check federation status
curl -k https://localhost:8080/api/federation/status | jq '.'

# Expected: Should show westgate (192.168.1.123) in nodes list
```

---

## 📊 What Changed

### 1. Discovery → Trust → Federation Bridge
- **New:** Automatic trust establishment for discovered peers
- **Impact:** Peers now automatically join federation (no manual registration)

### 2. Trust Escalation Manager
- **New:** Progressive trust levels (Anonymous → Capability → Identity → Hardware)
- **Impact:** Secure by default, can escalate as needed

### 3. Heartbeat URL Handling
- **Fixed:** Heartbeats now work with HTTPS endpoints
- **Impact:** Federation health monitoring operational

### 4. E2E Test Coverage
- **New:** 7 comprehensive tests for trust flow
- **Impact:** Verified reliability and correctness

---

## 🐛 Troubleshooting

### Issue: "No peers discovered"
**Solution:** Wait 30-60 seconds. Discovery polls every 10 seconds.

### Issue: "Connection refused" in heartbeats
**Solution:** Verify HTTPS server is running:
```bash
curl -k https://localhost:8080/health
```

### Issue: "Federation shows 0 nodes"
**Solution:** Check discovery logs:
```bash
tail -f logs/westgate-*.log | grep -i discovery
```

### Issue: "Trust establishment failed"
**Solution:** Check trust manager logs:
```bash
tail -f logs/westgate-*.log | grep -i trust
```

---

## 📈 Expected Metrics

### Discovery
- **Broadcast interval:** Every 30 seconds
- **Listen port:** UDP 2300
- **Session ID rotation:** Every 5 minutes

### Trust
- **Establishment time:** < 1ms per peer
- **Initial level:** Anonymous (Level 0)
- **Timeout:** 1 hour (anonymous)

### Federation
- **Registration time:** < 1ms per peer
- **Heartbeat interval:** Every 30 seconds
- **Health check:** Every 60 seconds

---

## 🎉 Success Indicators

After updating, you should see:

1. **Discovery Logs:**
   ```
   🔍 Discovered peer: [eastgate_session_id] at https://192.168.1.144:8080
   ```

2. **Trust Logs:**
   ```
   ✅ Trust established with [eastgate_short_id] (level: Anonymous)
   ```

3. **Federation Logs:**
   ```
   🤝 Peer [eastgate_short_id] joined federation (anonymous trust)
   ```

4. **Heartbeat Logs:**
   ```
   💓 Heartbeat sent to peer-[eastgate_short_id]
   ```

5. **Federation Status:**
   ```
   Active Nodes: 2 (or more)
   ✅ Connected to federation!
   ```

---

## 📞 Communication

### After Update, Report Back:
```bash
# Run this and share the output:
./check-tower.sh

# And this:
tail -30 logs/westgate-*.log | grep -E "Trust|federation|Discovered"
```

---

## 🔐 Security Notes

### Zero-Trust by Default
- **Anonymous Discovery:** No identity shared initially
- **Progressive Escalation:** Trust increases with verification
- **Encrypted Transport:** All connections use TLS
- **Session Rotation:** Session IDs change every 5 minutes

### Trust Levels
- **Level 0 (Anonymous):** Discovery only, no data shared
- **Level 1 (Capability):** Task coordination allowed
- **Level 2 (Role):** Registry access allowed
- **Level 3 (Identity):** Infrastructure details shared
- **Level 4 (Hardware):** Full admin access (BearDog required)

**Current:** All peers start at Level 0 (Anonymous)

---

## 🎯 Expected Timeline

```
T+0s:   Start westgate
T+10s:  Discovery finds eastgate
T+10s:  Trust established (Anonymous)
T+10s:  Peer registered in federation
T+30s:  First heartbeat sent
T+30s:  First heartbeat received
T+60s:  Federation fully operational
```

---

## 📚 Additional Resources

- **Trust Architecture:** See `TRUST_ESTABLISHMENT_COMPLETE_DEC_19_2025.md`
- **Secure Federation:** See `SECURE_FEDERATION_DESIGN_DEC_19_2025.md`
- **Discovery Protocol:** See `DISCOVERY_EVOLUTION_DEC_19_2025.md`
- **Universal Scripts:** See `TOWER_SCRIPTS_README.md`

---

## ✅ Checklist

- [ ] Pull latest code (`git pull`)
- [ ] Rebuild (`cargo build --release`)
- [ ] Stop old tower (`./stop-tower.sh`)
- [ ] Start new tower (`./start-tower.sh`)
- [ ] Verify status (`./check-tower.sh`)
- [ ] Check logs for trust establishment
- [ ] Confirm federation shows 2+ nodes
- [ ] Report back with status

---

## 🎊 What This Enables

With this update, westgate will:
- ✅ Automatically discover eastgate (and any other towers)
- ✅ Establish secure, anonymous trust
- ✅ Join the federation automatically
- ✅ Exchange heartbeats for health monitoring
- ✅ Be ready for progressive trust escalation
- ✅ Participate in distributed task coordination

**No manual configuration needed!** 🚀

---

*Generated: December 19, 2025*  
*For: Westgate Agent*  
*From: Eastgate (Trust Integration Complete)*

