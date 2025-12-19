# 🎵 Songbird Tower - Simple Startup Scripts

**Universal one-touch scripts for ANY tower - zero configuration!**

---

## 🚀 Quick Start

### Start a Tower
```bash
./start-tower.sh
```

That's it! Works on eastgate, westgate, strandgate, or any other tower.

### Check Status
```bash
./check-tower.sh
```

### Stop Tower
```bash
./stop-tower.sh
```

---

## 📋 What These Scripts Do

### `start-tower.sh` - One-Touch Startup ✅

**What it does automatically:**
- ✅ Detects tower name (from hostname or `SONGBIRD_TOWER_NAME`)
- ✅ Builds binary if needed
- ✅ Cleans up old processes
- ✅ Starts orchestrator with secure defaults
- ✅ Enables TLS (failsafe)
- ✅ Enables anonymous discovery
- ✅ Enables federation with zero-trust
- ✅ Auto-selects available port
- ✅ Creates timestamped log file
- ✅ Shows startup status

**What you need to configure:**
- Nothing! Just run it.

**Optional:**
```bash
# Use custom tower name
SONGBIRD_TOWER_NAME=mytown ./start-tower.sh
```

### `check-tower.sh` - Status Check ✅

**Shows:**
- ✅ Tower running status
- ✅ PID and resource usage
- ✅ HTTPS port (auto-detected)
- ✅ Discovery status
- ✅ Federation status
- ✅ Active nodes count
- ✅ Recent log activity

**No arguments needed** - just run it!

### `stop-tower.sh` - Clean Shutdown ✅

**What it does:**
- ✅ Finds all songbird processes
- ✅ Graceful shutdown (TERM signal)
- ✅ Force kill if needed (after 2s)
- ✅ Verifies all processes stopped

**No arguments needed** - just run it!

---

## 🌐 Federation Example

### Tower 1 (Eastgate)
```bash
# On eastgate machine
cd /home/eastgate/Development/ecoPrimals/songbird
./start-tower.sh

# Output:
# ✅ Tower Name: eastgate
# ✅ HTTPS Server: Port 8080
# ✅ Discovery: UDP port 2300
# ✅ Tower is ready!
```

### Tower 2 (Westgate)
```bash
# On westgate machine (same command!)
cd /path/to/songbird
./start-tower.sh

# Output:
# ✅ Tower Name: westgate
# ✅ HTTPS Server: Port 8443  # Different port, auto-selected!
# ✅ Discovery: UDP port 2300
# ✅ Tower is ready!
```

### Automatic Discovery (30-60 seconds)
```bash
# On either tower
./check-tower.sh

# Output:
# 🌐 Federation Status:
#   Federation ID: fd796e08...
#   Active Nodes: 2  # ✅ Connected!
#   ✅ Connected to federation!
```

**No manual configuration!** They find each other automatically.

---

## 📁 Log Files

Logs are automatically created in `logs/` directory:
```
logs/
  eastgate-20251219-164605.log
  westgate-20251219-164612.log
  strandgate-20251219-164620.log
```

### View Logs
```bash
# Latest log
tail -f logs/*.log

# Filter for discovery
tail -f logs/*.log | grep -i discovery

# Filter for federation
tail -f logs/*.log | grep -i federation
```

---

## 🔧 Configuration (Optional)

### Environment Variables

**Tower Name** (auto-detects hostname if not set):
```bash
SONGBIRD_TOWER_NAME=mytown ./start-tower.sh
```

**Custom Settings** (all have secure defaults):
```bash
# Override TLS (not recommended)
SONGBIRD_TLS_ENABLED=false ./start-tower.sh

# Override discovery port (not recommended)
SONGBIRD_DISCOVERY_PORT=2301 ./start-tower.sh

# Custom bind address
SONGBIRD_BIND_ADDRESS="0.0.0.0" ./start-tower.sh
```

**But you shouldn't need any of these!** Defaults are secure and work everywhere.

---

## 🚫 What These Scripts DON'T Do

- ❌ Manual port configuration
- ❌ Port scanning
- ❌ Hardcoded IPs
- ❌ Manual federation joins
- ❌ Complex setup procedures

**Why?** Because Songbird handles all of this automatically! Just start it and let it work.

---

## 🔍 Troubleshooting

### Tower Won't Start

**Check:**
```bash
# 1. Is binary built?
ls -lh target/release/songbird-orchestrator

# 2. Check logs
tail -50 logs/*.log

# 3. Port conflict?
sudo lsof -i :8080 -P -n  # or whatever port
```

### Discovery Not Working

**Check:**
```bash
# 1. Is UDP 2300 listening?
sudo lsof -i UDP:2300 -P -n

# 2. Firewall blocking?
sudo ufw status
sudo ufw allow 2300/udp

# 3. Same subnet?
hostname -I  # Check IPs are 192.168.1.x etc
```

### Federation Not Connecting

**Wait:**
- Discovery broadcasts every 30 seconds
- Give it 60-90 seconds
- Check with `./check-tower.sh`

**Verify:**
```bash
# Both towers broadcasting?
# On each tower:
./check-tower.sh | grep Discovery

# Network connectivity?
ping <other-tower-ip>

# Firewall?
sudo ufw allow 2300/udp
```

---

## 📊 Example Session

```bash
# Start eastgate
user@eastgate:~/songbird$ ./start-tower.sh
🎵 Songbird Tower - One-Touch Startup
========================================
Tower Name: eastgate
✅ Orchestrator started
✅ HTTPS Server: Port 8080
✅ Discovery: UDP port 2300
✅ Tower is ready!

# Check status
user@eastgate:~/songbird$ ./check-tower.sh
🔍 Songbird Tower Status Check
✅ Status: RUNNING
  Name: eastgate
  IP: 192.168.1.144
✅ HTTPS: Port 8080
✅ Discovery: UDP port 2300
🌐 Federation Status:
  Active Nodes: 1
  ⏳ Waiting for peers...

# Wait 60 seconds, start westgate on another machine...

# Check again
user@eastgate:~/songbird$ ./check-tower.sh
🌐 Federation Status:
  Active Nodes: 2
  ✅ Connected to federation!

# Stop when done
user@eastgate:~/songbird$ ./stop-tower.sh
✅ All songbird processes stopped
```

---

## 🎯 Design Philosophy

### Zero Configuration
- Just run the script
- Automatic port selection
- Automatic discovery
- Automatic connection

### Secure by Default
- TLS always enabled
- Anonymous discovery
- Zero-trust architecture
- Progressive escalation

### OpSec Conscious
- No port scanning
- No manual configuration
- No hardcoded secrets
- Capability-based design

### Universal
- Same script on ALL towers
- Works anywhere
- No customization needed
- Self-configuring

---

## 📦 Files Structure

```
songbird/
├── start-tower.sh          # ✅ One-touch startup
├── stop-tower.sh           # ✅ Clean shutdown
├── check-tower.sh          # ✅ Status check
├── TOWER_SCRIPTS_README.md # ✅ This file
└── logs/                   # Created automatically
    ├── eastgate-*.log
    ├── westgate-*.log
    └── ...
```

---

## 🎊 Summary

**Three simple scripts:**
1. `./start-tower.sh` - Start any tower
2. `./check-tower.sh` - Check status
3. `./stop-tower.sh` - Stop tower

**Zero configuration required!**

**Works on ALL towers identically!**

**Secure by default!**

---

**That's it! Start towers, they discover each other, federation happens automatically! 🚀**

