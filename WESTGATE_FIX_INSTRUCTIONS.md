# 🔧 Westgate Fix - IPv4 Binding Issue

**Problem Found:** Both towers were binding to IPv6 only, preventing IPv4 federation!

**Solution:** Restart westgate with explicit IPv4 binding.

---

## 🚀 On Westgate - Run These Commands

```bash
# 1. Stop current instance
pkill -f songbird-orchestrator

# 2. Navigate to songbird
cd ~/songbird

# 3. Pull latest code (if needed)
git pull

# 4. Restart with IPv4 binding
SONGBIRD_BIND_ADDRESS="0.0.0.0" SONGBIRD_TOWER_NAME="westgate" ./start-tower.sh
```

---

## ✅ What This Does

- **SONGBIRD_BIND_ADDRESS="0.0.0.0"**: Forces IPv4 binding instead of IPv6
- Rest is automatic: TLS, discovery, federation all work

---

## 🔍 Verify It's Working

After restarting, run:

```bash
# Check tower status
./check-tower.sh

# You should see:
# ✅ HTTPS: Port XXXX
# ✅ Discovery: UDP port 2300
# ✅ Broadcaster: UDP port XXXXX
```

Then check if it's listening on IPv4:

```bash
# Find PID
REAL_PID=$(pgrep -f "target/release/songbird-orchestrator" | head -1)

# Check binding (should show IPv4, not IPv6)
sudo lsof -p $REAL_PID -i TCP -P -n | grep 8080

# Should show: TCP *:8080 (LISTEN)  or  TCP 0.0.0.0:8080 (LISTEN)
# NOT: TCP [::]:8080 (LISTEN)
```

---

## ⏱️ Expected Timeline After Fix

```
0-5s:   Westgate restarts with IPv4
10s:    Eastgate and westgate start broadcasting
30s:    First discovery cycle
60s:    Federation established!
```

Check federation on either tower:

```bash
./check-tower.sh

# Should show:
# 🌐 Federation Status:
#   Active Nodes: 2  ✅
#   ✅ Connected to federation!
```

---

## 🎯 Summary

**Eastgate:** ✅ Fixed and waiting
**Westgate:** Needs restart with `SONGBIRD_BIND_ADDRESS="0.0.0.0"`

Once westgate restarts, they'll discover each other automatically within 60 seconds!

