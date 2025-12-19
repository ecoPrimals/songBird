# 🔍 Federation Debug Status - December 19, 2025

## ✅ Eastgate Status (Confirmed Working)

**Process:**
- PID: 2800575
- Status: Running ✅

**Network:**
- HTTPS: Port 8080 (IPv4: 0.0.0.0)
- UDP Discovery Listener: Port 2300 ✅
- UDP Discovery Broadcaster: Port 41214 ✅
- Firewall: Inactive (no blocking)

**Issue:**
- Log files are empty (0 bytes) - logs may be buffered
- No discovery messages visible yet

## ❓ Need from Westgate

To debug the connection, please run these commands on westgate and share the output:

### 1. Check Tower Status
```bash
cd ~/songbird
./check-tower.sh
```

**We need to see:**
- What port is HTTPS on?
- Is UDP 2300 listening?
- How many active nodes?

### 2. Check Process
```bash
ps aux | grep songbird-orchestrator | grep -v grep
```

**We need:** PID confirmation

### 3. Check UDP Ports
```bash
sudo lsof -i UDP -P -n | grep songbird
```

**Should show:**
- UDP *:2300 (listener)
- UDP *:XXXXX (broadcaster)

### 4. Check HTTPS Port
```bash
sudo lsof -i TCP -P -n | grep songbird | grep LISTEN
```

**Should show:** TCP *:XXXX or TCP 0.0.0.0:XXXX

### 5. Test UDP Reception
```bash
# In one terminal, listen for UDP packets:
sudo tcpdump -i any 'udp port 2300' -A -n

# Should see packets arriving from 192.168.1.144
```

### 6. Check Logs
```bash
ls -lh logs/*.log | head -3
tail -50 logs/*.log | grep -i discovery
```

**We need:** Are logs being written? Any discovery messages?

### 7. Test Eastgate Connectivity
```bash
# Can westgate reach eastgate?
ping -c 2 192.168.1.144

# Can westgate connect to eastgate's HTTPS?
curl -k -s https://192.168.1.144:8080/health
```

### 8. Check Firewall
```bash
sudo ufw status
```

**Should be:** Inactive, or allow UDP 2300

---

## 🔍 Quick Diagnostic

**Run this one command on westgate and share output:**
```bash
cd ~/songbird && echo "=== Tower Status ===" && ./check-tower.sh && echo "" && echo "=== UDP Ports ===" && sudo lsof -i UDP -P -n | grep songbird && echo "" && echo "=== HTTPS Port ===" && sudo lsof -i TCP -P -n | grep songbird | grep LISTEN && echo "" && echo "=== Firewall ===" && sudo ufw status && echo "" && echo "=== Eastgate Reachable? ===" && ping -c 2 192.168.1.144 && curl -k -s https://192.168.1.144:8080/health && echo " ✅"
```

---

## 🧪 What We're Looking For

### Success Indicators:
1. ✅ Both towers have UDP 2300 listening
2. ✅ Both towers are broadcasting on random UDP ports
3. ✅ Both towers can ping each other
4. ✅ HTTPS ports are accessible
5. ✅ Firewall allows UDP 2300 (or is inactive)
6. ✅ Logs show "Discovered peer" messages

### Common Issues:
- ❌ Firewall blocking UDP 2300
- ❌ Different subnet (UDP broadcast doesn't cross routers)
- ❌ Logs not being written (buffering issue)
- ❌ Process not actually starting discovery tasks

---

## 💡 Alternative: Manual Join

If auto-discovery isn't working yet, we can manually join westgate to the federation:

**On eastgate, run:**
```bash
# Get westgate's HTTPS port from westgate's check-tower.sh
WESTGATE_PORT=8080  # Replace with actual port

curl -k -X POST https://localhost:8080/api/v1/federation/join \
  -H "Content-Type: application/json" \
  -d '{
    "node_id": "westgate",
    "node_name": "westgate",
    "node_address": "https://192.168.1.123:'"$WESTGATE_PORT"'",
    "cpu_cores": 8,
    "memory_gb": 31,
    "gpu_model": "Intel Xeon E3-1200 v3",
    "storage_gb": 1828,
    "capabilities": ["orchestrator", "storage"],
    "status": "active"
  }'
```

This manually registers westgate in eastgate's federation, bypassing auto-discovery for now.

---

**Please share the output from westgate and we'll diagnose from there!** 🔍

