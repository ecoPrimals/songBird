# Westgate HTTPS Server Diagnostic

**Date:** December 20, 2025  
**Issue:** HTTPS server not responding despite active UDP discovery  
**Priority:** 🔥 **HIGH**

---

## 🔍 Current Status

### ✅ What's Working
- **UDP Discovery Broadcaster:** Active and transmitting on port 2300
- **Session ID Rotation:** Working correctly
- **Capability Advertisement:** Broadcasting `["orchestration", "federation"]`
- **Eastgate Discovery:** Successfully receiving westgate broadcasts

### ❌ What's NOT Working
- **HTTPS Server:** Not responding on port 8080
- **Federation API:** Unreachable (`/api/federation/heartbeat`)
- **Health Endpoint:** Timeout (`/health`)
- **TLS Handshake:** Not completing

---

## 📊 Evidence

### Discovery (Working)
```
INFO: 🔍 Discovered peer: f501a584... at https://192.168.1.123:8080
INFO: 🔍 Discovered peer: 91b8f7e0... at https://192.168.1.123:8080
INFO: 🔍 Discovered peer: 3f38b78e... at https://192.168.1.123:8080
INFO: 🔍 Discovered peer: 87110153... at https://192.168.1.123:8080
```

### Heartbeats (Failing)
```
WARN: ⚠️  Heartbeat error to peer-3f38b78e: operation timed out
WARN: ⚠️  Heartbeat error to peer-87110153: operation timed out
WARN: ⚠️  Heartbeat error to peer-91b8f7e0: operation timed out
```

### Direct Connection (Failing)
```bash
$ curl -k https://192.168.1.123:8080/health
# Result: TIMEOUT (no response)
```

### Federation Status
```
Total westgate entries in federation: 237
Active westgate peers: 6
Inactive westgate peers: 231
```

---

## 🔬 Root Cause Analysis

### Hypothesis 1: HTTPS Server Crashed
**Symptoms:**
- UDP broadcaster still running
- No response from TCP port 8080

**Check:**
```bash
# On westgate
ps aux | grep songbird-orchestrator
# Should show ONE process

lsof -i :8080
# Should show songbird-orchestrator listening on TCP

tail -100 logs/westgate-*.log | grep -i "panic\|error\|crash"
# Look for crash messages
```

### Hypothesis 2: Firewall Blocking TCP 8080
**Symptoms:**
- UDP works (port 2300)
- TCP fails (port 8080)

**Check:**
```bash
# On westgate
sudo ufw status
# Should show 8080 ALLOW or firewall disabled

sudo iptables -L -n | grep 8080
# Should not block port 8080
```

### Hypothesis 3: IPv6 Binding Issue
**Symptoms:**
- Server listening on IPv6 only (`::`)
- Eastgate trying to connect via IPv4

**Check:**
```bash
# On westgate
lsof -i :8080
# Should show: *:8080 (IPv4) or 0.0.0.0:8080
# NOT: [::]:8080 (IPv6 only)

# If IPv6 only, restart with:
SONGBIRD_BIND_ADDRESS="0.0.0.0" ./start-tower.sh
```

### Hypothesis 4: TLS Certificate Issue
**Symptoms:**
- Server listening but TLS handshake failing

**Check:**
```bash
# On westgate
ls -la certs/
# Should have songbird.crt and songbird.key

openssl s_client -connect localhost:8080 -showcerts
# Should show TLS handshake success
```

---

## 🛠️ Diagnostic Commands for Westgate Agent

Run these commands on westgate and report back:

### 1. Process Check
```bash
ps aux | grep songbird-orchestrator
```
**Expected:** One process running  
**If not:** Orchestrator crashed, check logs

### 2. Port Listening Check
```bash
lsof -i :8080
```
**Expected:** `songbird-orchestrator` listening on `*:8080` or `0.0.0.0:8080`  
**If empty:** HTTPS server not started  
**If `[::]:8080`:** IPv6 only binding (PROBLEM)

### 3. Localhost Connection Test
```bash
curl -k -v https://localhost:8080/health 2>&1 | head -30
```
**Expected:** `{"status":"healthy",...}`  
**If timeout:** Server not responding  
**If connection refused:** Server not listening

### 4. IPv4 Connection Test
```bash
curl -k -v https://127.0.0.1:8080/health 2>&1 | head -30
```
**Expected:** `{"status":"healthy",...}`  
**If timeout:** Server not responding on IPv4

### 5. TLS Test
```bash
timeout 5 openssl s_client -connect localhost:8080 -showcerts 2>&1 | head -40
```
**Expected:** TLS handshake complete, certificate shown  
**If timeout:** TLS not working

### 6. Firewall Check
```bash
sudo ufw status
```
**Expected:** `Status: inactive` or `8080 ALLOW`  
**If active and no 8080 rule:** Firewall blocking

### 7. Recent Logs
```bash
tail -100 logs/westgate-*.log | grep -E "HTTPS|server|panic|error|crash"
```
**Look for:**
- "HTTPS Server started"
- Any panic/error messages
- Server crash indicators

### 8. Binding Address Check
```bash
tail -100 logs/westgate-*.log | grep -i "bind\|listening"
```
**Expected:** "Listening on 0.0.0.0:8080" or similar  
**If ":::":** IPv6 only (PROBLEM)

---

## 🎯 Quick Fixes

### Fix 1: If HTTPS Server Not Started
```bash
cd ~/songbird
./stop-tower.sh
./start-tower.sh
# Wait 10 seconds
curl -k https://localhost:8080/health
```

### Fix 2: If IPv6 Only Binding
```bash
cd ~/songbird
./stop-tower.sh
# Edit start-tower.sh or run:
SONGBIRD_BIND_ADDRESS="0.0.0.0" ./start-tower.sh
# Wait 10 seconds
lsof -i :8080  # Should show 0.0.0.0:8080
```

### Fix 3: If Firewall Blocking
```bash
sudo ufw allow 8080/tcp
# Or disable firewall:
sudo ufw disable
```

### Fix 4: If TLS Certs Missing
```bash
cd ~/songbird
ls -la certs/
# If missing or broken:
rm -rf certs/
./start-tower.sh
# Certs will auto-generate
```

---

## 📋 Diagnostic Checklist

Run each command and report results:

- [ ] `ps aux | grep songbird-orchestrator` - Process running?
- [ ] `lsof -i :8080` - Listening on port?
- [ ] `curl -k https://localhost:8080/health` - Localhost works?
- [ ] `curl -k https://127.0.0.1:8080/health` - IPv4 works?
- [ ] `sudo ufw status` - Firewall status?
- [ ] `tail -100 logs/westgate-*.log | grep HTTPS` - Server started?
- [ ] `ls -la certs/` - TLS certs present?

---

## 🎯 Expected Results After Fix

After fixing, you should see:

### From Westgate
```bash
$ curl -k https://localhost:8080/health
{"status":"healthy","uptime_seconds":45,...}
```

### From Eastgate
```bash
$ curl -k https://192.168.1.123:8080/health
{"status":"healthy",...}
```

### Federation Logs
```
INFO: 💓 Heartbeat sent to peer-3f38b78e
INFO: 💓 Heartbeat sent to peer-87110153
```

### Federation Status
```
Active Nodes: 2 (or more)
✅ Connected to federation!
```

---

## 📞 Next Steps

1. **Run diagnostic commands above**
2. **Report back with results**
3. **Apply appropriate fix**
4. **Verify with `curl -k https://localhost:8080/health`**
5. **Check federation: `./check-tower.sh`**

---

*Generated: December 20, 2025*  
*For: Westgate Agent*  
*From: Eastgate Federation Monitor*

