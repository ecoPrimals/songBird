# Eastgate Startup Issue - Root Cause & Solution

**Date:** December 20, 2025  
**Tower:** Eastgate  
**Status:** ❌ Not starting properly

---

## 🔍 Root Cause Identified

### Issue #1: Port Conflict
- **Port 8080** occupied by: `nestgate_bin-0e` (PID 2114601)
- This appears to be a **Cursor IDE** process (nestgate_bin)
- Songbird falls back to port 8082
- **Problem:** Discovery broadcasts port 8080, but server listens on 8082
- **Result:** Westgate can't connect to Eastgate

### Issue #2: Discovery Mismatch
- **Broadcasting:** "eastgate available at port 8080"
- **Reality:** Server listening on port 8082
- **Outcome:** Federation connection fails

---

## ✅ Solution: Use Alternative Port

Instead of fighting with Cursor IDE for port 8080, let's use a different port entirely.

### Option A: Use Port 8083 (Recommended)

```bash
# Set environment variable for custom port
export SONGBIRD_PORT=8083

# Start Songbird
cd /home/eastgate/Development/ecoPrimals/songbird
./target/release/songbird-orchestrator &

# This will:
# 1. Bind HTTPS to 8083
# 2. Broadcast 8083 in discovery
# 3. No conflict with Cursor IDE
```

### Option B: Kill Cursor IDE Process (Not Recommended)

```bash
# This might break Cursor IDE functionality
kill -9 2114601
```

---

## 🎯 Quick Fix Commands (Run on Eastgate)

```bash
# 1. Kill any running Songbird
pkill -9 songbird-orchestrator
sleep 2

# 2. Set custom port
export SONGBIRD_PORT=8083

# 3. Start Songbird
cd /home/eastgate/Development/ecoPrimals/songbird
./target/release/songbird-orchestrator &

# 4. Wait and verify
sleep 20
curl -sk https://localhost:8083/health

# 5. Check federation
curl -sk https://localhost:8083/api/federation/status | \
  jq -r '.nodes[] | "\(.node_name) (\(.node_id[:12])...)"'
```

---

## 📊 Expected Result

After using port 8083:
- ✅ Eastgate HTTPS responding on 8083
- ✅ Discovery broadcasting port 8083
- ✅ Westgate sees Eastgate in federation
- ✅ Identity routing working

---

## 🧹 Alternative: Persistent Configuration

Create a config file to always use port 8083:

```bash
# Create or edit ~/.bashrc
echo 'export SONGBIRD_PORT=8083' >> ~/.bashrc
source ~/.bashrc
```

Or create a systemd service with the custom port.

---

## 🐛 Why This Happened

Cursor IDE (nestgate_bin) starts an HTTP server on port 8080 for its internal functionality (possibly for language server protocol or debugging). This is normal behavior for IDEs.

Rather than fighting with it, we should:
1. Use a different port (8083)
2. Or configure Cursor to use a different port
3. Or use systemd to ensure Songbird starts first

---

**Recommendation:** Use `SONGBIRD_PORT=8083` as the quick fix.


