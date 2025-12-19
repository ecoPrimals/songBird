# 🌐 Westgate Deployment Instructions

**Simple one-touch deployment for westgate tower!**

---

## 🚀 Quick Deploy (On Westgate Machine)

```bash
# 1. Navigate to songbird directory (or clone if first time)
cd ~/songbird
# or if first time:
# git clone <repo-url> songbird && cd songbird

# 2. Pull latest changes
git pull

# 3. Start tower (that's it!)
./start-tower.sh

# 4. Verify (optional)
./check-tower.sh
```

**Done!** Westgate will automatically:
- ✅ Auto-select available port
- ✅ Start HTTPS server with TLS
- ✅ Start anonymous discovery
- ✅ Broadcast presence every 30 seconds
- ✅ Discover eastgate automatically
- ✅ Establish zero-trust connection
- ✅ Progressive trust escalation

---

## ⏱️ Expected Timeline

```
0-10s:  Westgate starts, services initialize
10-30s: First discovery broadcast
30-60s: Eastgate and westgate discover each other
60-90s: Trust established (Anonymous level)
90s+:   Progressive escalation begins
```

**Check status anytime:**
```bash
./check-tower.sh
```

**Expected output when connected:**
```
🌐 Federation Status:
  Active Nodes: 2  ✅
  ✅ Connected to federation!
```

---

## 🔍 Verification

### On Westgate
```bash
# Check if running
./check-tower.sh

# View logs
tail -f logs/westgate-*.log

# Check federation
curl -k https://localhost:PORT/api/federation/status | jq '.active_nodes'
# Should show: 2
```

### On Eastgate
```bash
# Check federation
./check-tower.sh

# Or via API
curl -k https://localhost:8080/api/federation/status | jq '.active_nodes'
# Should show: 2
```

---

## 🚫 What NOT to Do

- ❌ Don't manually configure ports
- ❌ Don't scan for ports
- ❌ Don't edit config files
- ❌ Don't use old scripts (deleted)

**Why?** Everything is automatic! Just run `./start-tower.sh`

---

## 🔧 Troubleshooting

### If Discovery Doesn't Work After 2 Minutes

**Check firewall:**
```bash
sudo ufw status
sudo ufw allow 2300/udp
sudo ufw reload
```

**Check connectivity:**
```bash
ping 192.168.1.144  # eastgate
```

**Check logs:**
```bash
tail -f logs/westgate-*.log | grep -i discovery
```

### If Port Conflict

Script will automatically try next available port. Check logs for actual port:
```bash
grep "HTTPS server listening" logs/westgate-*.log
```

---

## 📊 Success Indicators

When working correctly, you'll see:

**On Westgate:**
```bash
$ ./check-tower.sh
✅ Status: RUNNING
✅ HTTPS: Port XXXX (auto-selected)
✅ Discovery: UDP port 2300
✅ Broadcaster: UDP port XXXXX
🌐 Federation Status:
  Active Nodes: 2
  ✅ Connected to federation!
```

**On Eastgate:**
```bash
$ ./check-tower.sh
🌐 Federation Status:
  Active Nodes: 2
  Nodes: [eastgate, westgate]
  ✅ Connected to federation!
```

---

## 🎯 That's It!

**Three commands:**
```bash
cd ~/songbird && git pull
./start-tower.sh
./check-tower.sh
```

**Zero configuration!** Songbird handles everything automatically! 🚀

