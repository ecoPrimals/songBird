# Windows Laptop Testing Guide

**Goal:** Deploy a Songbird node on your Windows laptop for students to connect to.

---

## Prerequisites

- Windows 10/11
- Python 3.8+ installed
- Administrator access (for firewall rules)
- Connected to same network as students

---

## Quick Deployment

### Step 1: Install Songbird

```powershell
# Navigate to Songbird directory
cd C:\path\to\songbird

# Install dependencies
pip install -r requirements.txt

# Build Songbird (if Rust-based)
cargo build --release
```

### Step 2: Configure for Local Network

Create `config/local-network.toml`:

```toml
[server]
host = "0.0.0.0"  # Listen on all interfaces
port = 8080
protocol = "ws"   # WebSocket (not WSS for local network)

[federation]
name = "MSU-EcoPrimals-Test"
node_name = "songbird-coordinator"

# Auto-discovery via federation registry (NO hardcoded IPs!)
discovery_enabled = true
discovery_interval_seconds = 30

# Registry endpoint where compute nodes register
# This should point to whichever tower is running the registry
registry_url = "http://192.168.1.144:8000/api/federation/registry"

[compute]
# Capability-based routing - nodes self-register
required_capabilities = ["gpu-compute", "ml-training"]
scheduling = "least-loaded"

# No hardcoded nodes! Discovery happens automatically.

[logging]
level = "info"
```

**Key Point:** No hardcoded compute node IPs! Songbird discovers them via the federation registry.

### Step 3: Open Firewall

```powershell
# Open port 8080 for inbound connections
New-NetFirewallRule -DisplayName "Songbird-Student-Access" -Direction Inbound -LocalPort 8080 -Protocol TCP -Action Allow
```

### Step 4: Get Your IP Address

```powershell
ipconfig
```

Look for your local IP (e.g., `192.168.1.XXX`). This is what students will connect to.

### Step 5: Start Songbird

```powershell
# Run Songbird with local config
cargo run --release -- --config config/local-network.toml

# Or if Python-based:
python -m songbird.server --config config/local-network.toml
```

You should see:
```
🎵 Songbird Coordinator starting...
   Listening on: 0.0.0.0:8080
   Federation: MSU-EcoPrimals-Test
   Compute nodes: 2 discovered
   
✅ Ready for student connections!
```

---

## Testing Student Connection

### On Your Laptop (Server Side)

Keep Songbird running and watch for connections:

```
🎵 New connection from: 192.168.1.XXX
   Client: student
   Version: 0.1.0
✅ Student connected!
```

### From Student Laptop

Have a student (or yourself on another device):

```bash
# Set URL to your laptop's IP
export SONGBIRD_URL="ws://192.168.1.XXX:8080"

# Test connection
cd showcase/07-student-onboarding/client
python -m ecoprimals_client.connect
```

If successful:
```
🎵 Connecting to Songbird...
✅ Connected to MSU-EcoPrimals-Test Federation
   Available nodes: 2
   Total GPUs: 2
```

---

## Testing Full Workflow

### 1. Student Submits Task

On student laptop:
```bash
cd projects/01-mnist-digits
python submit.py
```

### 2. Watch on Server

Your Songbird laptop shows:
```
📥 Task received: train.py
   From: 192.168.1.XXX
   Dataset: mnist
   GPU required: yes

🔍 Finding available GPU...
   Selected: Eastgate (RTX 3090)

📤 Routing task to ToadStool...
   Task ID: task-abc123
```

### 3. Student Gets Results

On student laptop:
```
✅ Task completed!
   Accuracy: 95.12%
   Training time: 3m 45s
   
📜 Receipt saved: receipt_task-abc123.json
```

---

## Troubleshooting

### "Can't bind to 0.0.0.0:8080"

**Problem:** Port already in use.

**Solution:**
```powershell
# Find what's using port 8080
netstat -ano | findstr :8080

# Kill the process (replace XXXX with PID)
taskkill /PID XXXX /F

# Or use a different port in config
```

### Students Can't Connect

**Problem:** Network/firewall blocking connections.

**Solutions:**

1. **Verify firewall rule:**
```powershell
Get-NetFirewallRule -DisplayName "Songbird-Student-Access"
```

2. **Check you're on same network:**
```powershell
# Your IP should be 192.168.1.XXX
ipconfig

# Have student ping you
ping 192.168.1.XXX
```

3. **Try disabling Windows Firewall temporarily (testing only!):**
```powershell
Set-NetFirewallProfile -Profile Domain,Public,Private -Enabled False
```

4. **Check antivirus isn't blocking:**
   - Windows Defender might block Python/Cargo executables
   - Add exception for Songbird directory

### Songbird Can't Reach Compute Nodes

**Problem:** Can't connect to Eastgate/Strandgate.

**Solutions:**

1. **Verify nodes are running:**
```bash
# On each compute node
curl http://localhost:8000/health
```

2. **Check you can reach them from Windows laptop:**
```powershell
curl http://192.168.1.144:8000/health
```

3. **Update config with correct IPs:**
   - Make sure node URLs in config match actual IPs
   - Use `http://` not `https://` for local testing

---

## Network Configuration

### Classroom Setup

```
Internet Router (192.168.1.1)
    │
    ├── Windows Laptop (192.168.1.50) ← Songbird running here
    │
    ├── Student Laptop 1 (192.168.1.101)
    ├── Student Laptop 2 (192.168.1.102)
    ├── Student Laptop 3 (192.168.1.103)
    │
    ├── Eastgate Tower (192.168.1.144) ← Compute nodes
    └── Strandgate Tower (192.168.1.134) ← Compute nodes
```

**Students connect to:** `ws://192.168.1.50:8080`

**Songbird routes tasks to:** Eastgate (144) or Strandgate (134)

---

## Production Deployment (Later)

For **Version 2** (internet access), you'll need:

1. **Public IP or Domain:**
   - `songbird.university.edu`
   - Or dynamic DNS service

2. **TLS Certificates:**
   - Let's Encrypt for HTTPS/WSS
   - BearDog for authentication

3. **Router Configuration:**
   - Port forwarding: 443 → Laptop:8443
   - Or VPN solution

4. **BearDog Integration:**
   - Student authentication tokens
   - Rate limiting
   - Access control

**For now, local network is perfect for testing!**

---

## Validation Checklist

Before bringing students in:

- [ ] Songbird starts without errors
- [ ] Can see your IP address
- [ ] Firewall port 8080 open
- [ ] Test connection from another device works
- [ ] Submit MNIST task completes successfully
- [ ] Receipt generated correctly
- [ ] Can handle 2-3 simultaneous tasks

Once validated:
- [ ] Write URL on whiteboard for students
- [ ] Have backup plan (your IP, config file, troubleshooting steps)
- [ ] Monitor Songbird logs during class

---

## Tips

1. **Keep Terminal Open:** Watch Songbird logs during class to see student connections and tasks.

2. **Have Your IP Ready:** Write it on whiteboard or share in chat.

3. **Test Before Class:** Submit a test task yourself from another device.

4. **Monitor Resources:** Keep Task Manager open to watch CPU/network.

5. **Backup Config:** Save working config file in case you need to restart.

---

## Next Steps

Once local network testing works:

1. **Multiple Students:** Test with 5-10 students simultaneously
2. **Different Projects:** Have students try various ML tasks
3. **Performance Metrics:** Monitor task completion times
4. **Student Feedback:** What works? What's confusing?
5. **Internet Access:** Plan BearDog integration for remote submission

---

**Ready to test?** Start Songbird and try connecting from another device! 🎵✨

