# Student Onboarding Deployment Guide

**Status:** Ready for Testing  
**Target:** Windows Laptop + Linux Federation  
**Network:** Local Campus WiFi

---

## Pre-Deployment Checklist

### Hardware Requirements

**Coordinator Node (Windows Laptop):**
- Windows 10/11
- 4GB RAM minimum
- Network connectivity (campus WiFi)
- Python 3.8+ installed
- WSL2 (optional, for Rust development)

**Federation Nodes (Your Towers):**
- ✅ Eastgate (RTX 3090 24GB)
- ✅ Strandgate (RTX 3070 8GB)
- ✅ Northgate (RTX 5090 32GB) - optional
- ✅ Southgate (RTX 3090 24GB) - optional

### Software Requirements

**Windows Laptop:**
```powershell
# Python 3.8+
python --version

# Rust (if building from source)
rustc --version

# Or download pre-built binary
# (coming soon)
```

**Students' Devices:**
```bash
# Python 3.8+
python3 --version

# pip for package installation
pip3 --version
```

---

## Deployment Steps

### Step 1: Configure Federation

**On Your Towers (Already Done):**
```bash
# Verify registry is running (Eastgate)
curl http://192.168.1.144:8000/api/federation/registry

# Verify compute nodes are registered
curl http://192.168.1.144:8000/api/federation/nodes
```

Expected output:
```json
{
  "nodes": [
    {
      "id": "eastgate",
      "capabilities": ["gpu-compute"],
      "available": true
    },
    {
      "id": "strandgate",
      "capabilities": ["gpu-compute"],
      "available": true
    }
  ]
}
```

### Step 2: Deploy Songbird on Windows Laptop

#### Option A: Pre-Built Binary (Recommended)

```powershell
# Download pre-built binary
Invoke-WebRequest -Uri "https://github.com/ecoPrimals/songbird/releases/download/v0.1.0/songbird-orchestrator-windows.zip" -OutFile "songbird.zip"

# Extract
Expand-Archive -Path songbird.zip -DestinationPath C:\songbird

# Create config
New-Item -Path C:\songbird\config -ItemType Directory
```

#### Option B: Build from Source (WSL2)

```bash
# In WSL2
cd /mnt/c/songbird
git clone https://github.com/ecoPrimals/songbird.git
cd songbird
cargo build --release --bin songbird-orchestrator

# Binary at: target/release/songbird-orchestrator
cp target/release/songbird-orchestrator /mnt/c/songbird/songbird-orchestrator.exe
```

### Step 3: Configure Songbird

**Create `C:\songbird\config\local-network.toml`:**

```toml
[server]
host = "0.0.0.0"
port = 8080
protocol = "ws"

[federation]
# Point to your federation registry
registry_url = "http://192.168.1.144:8000/api/federation/registry"
discovery_enabled = true

[compute]
# NO hardcoded IPs - discovery only!
required_capabilities = ["gpu-compute"]
scheduling = "least-loaded"

[auth]
# JWT standalone mode (BearDog in Q2)
mode = "standalone"
jwt_secret = "change-me-in-production"

[limits]
# Student resource quotas
max_concurrent_tasks_per_student = 3
max_gpu_hours_per_week = 10.0
max_storage_gb = 5
```

### Step 4: Start Songbird

**PowerShell:**
```powershell
cd C:\songbird
.\songbird-orchestrator.exe --config config\local-network.toml
```

**Expected Output:**
```
🎵 Songbird Orchestrator v0.1.0
   Mode: Standalone JWT
   Server: ws://0.0.0.0:8080
   Registry: http://192.168.1.144:8000/api/federation/registry

✅ Connected to federation
   Nodes discovered: 2
   Available GPUs: 2
   Total VRAM: 32GB

🚀 Ready for student connections!
   Students connect to: ws://YOUR.LAPTOP.IP:8080
```

### Step 5: Get Your Laptop's IP

**PowerShell:**
```powershell
# Find your WiFi IP
ipconfig | Select-String -Pattern "IPv4"
```

Example output:
```
IPv4 Address. . . . . . . . . . . : 192.168.1.50
```

**Important:** This is the IP students will use.

### Step 6: Test Connection

**From Your Laptop:**
```bash
# Test federation discovery
curl http://192.168.1.50:8080/health

# Expected:
# {"status": "healthy", "nodes": 2, "available_gpus": 2}
```

---

## Student Setup

### Step 1: Install Client

**On Student Device:**
```bash
# Install from GitHub
pip install git+https://github.com/ecoPrimals/songbird-client.git

# Or from local files (if on campus network)
cd showcase/07-student-onboarding/client
pip install -e .
```

### Step 2: Configure Connection

**Set environment variable:**
```bash
# Linux/Mac
export SONGBIRD_URL="ws://192.168.1.50:8080"

# Windows
set SONGBIRD_URL=ws://192.168.1.50:8080
```

### Step 3: Test Connection

```bash
python -m ecoprimals_client.connect
```

**Expected Output:**
```
🎵 Connecting to Songbird...
✅ Connected to MSU-EcoPrimals Federation
   Available nodes: 2
   Total GPUs: 2 (32GB VRAM)
   Your quota: 3 concurrent tasks, 10 GPU hours/week
```

---

## Testing Workflow

### Test 1: MNIST Example

**Student runs:**
```bash
cd showcase/07-student-onboarding/projects/01-mnist-digits
python submit.py
```

**Watch on coordinator (Windows laptop):**
```
📥 Incoming task: mnist-training
   User: student-test-123
   Capabilities required: gpu-compute

🔍 Discovering nodes...
   Found: eastgate, strandgate

📊 Scheduling...
   Selected: eastgate (lower load)

✅ Task dispatched
   Task ID: task-abc123
   Node: eastgate
   GPU: RTX 3090 24GB
```

**Student sees:**
```
🚀 Task submitted: task-abc123
⏳ Training in progress...
   Epoch 1/3: Loss=0.32, Accuracy=91.2%
   Epoch 2/3: Loss=0.18, Accuracy=94.1%
   Epoch 3/3: Loss=0.15, Accuracy=95.2%

✅ Complete! Accuracy: 95.2%
📜 Receipt: receipt_task-abc123.json
```

### Test 2: Information Disclosure

**Student view:**
```json
{
  "task_id": "task-abc123",
  "status": "completed",
  "public_info": {
    "status": "completed",
    "completion_time_sec": 187.3
  },
  "educational_info": {
    "sharding_strategy": "single_node",
    "node_topology": {
      "nodes": [
        {
          "node_id": "compute-node-alpha",
          "capabilities": ["gpu-compute"],
          "gpu_class": "high-memory-gpu"
        }
      ]
    },
    "learning_notes": [
      "Your task ran on a single high-memory GPU",
      "Training time: 3 minutes 7 seconds"
    ]
  }
}
```

**Key Points:**
- ✅ Student sees educational info (how distribution works)
- ✅ Student does NOT see your home network IPs
- ✅ Node names are anonymized

---

## Monitoring

### On Windows Laptop (Coordinator)

**View active tasks:**
```powershell
curl http://localhost:8080/api/tasks/active
```

**View node status:**
```powershell
curl http://localhost:8080/api/nodes/status
```

**View student usage:**
```powershell
curl http://localhost:8080/api/users/student-123/usage
```

### On Your Admin Machine (Eastgate)

**Full infrastructure view (requires admin token):**
```bash
# Generate admin token
ADMIN_TOKEN=$(curl -X POST http://192.168.1.50:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"user_id": "admin", "role": "admin"}' | jq -r '.token')

# View full node details (including IPs)
curl http://192.168.1.50:8080/api/admin/nodes \
  -H "Authorization: Bearer $ADMIN_TOKEN"
```

**Output (admin only):**
```json
{
  "nodes": [
    {
      "name": "Eastgate",
      "internal_ip": "192.168.1.144:8000",
      "gpu": "RTX 3090 24GB",
      "utilization": 0.73,
      "uptime_hours": 48.2,
      "temperature_c": 52.0
    }
  ]
}
```

---

## Troubleshooting

### Problem: Students can't connect

**Check 1: Firewall**
```powershell
# Allow port 8080
New-NetFirewallRule -DisplayName "Songbird" -Direction Inbound -LocalPort 8080 -Protocol TCP -Action Allow
```

**Check 2: Network**
```bash
# From student device
ping 192.168.1.50

# If fails, check WiFi network (must be same network)
```

**Check 3: Songbird running**
```powershell
# Check if Songbird is running
Get-Process | Select-String "songbird"
```

### Problem: Tasks failing

**Check 1: Federation connectivity**
```powershell
# Test registry
curl http://192.168.1.144:8000/api/federation/registry
```

**Check 2: Compute node health**
```bash
# On Eastgate
curl http://localhost:8000/health
```

**Check 3: Logs**
```powershell
# View Songbird logs
type C:\songbird\logs\songbird.log
```

### Problem: Quota exceeded

**Check student usage:**
```bash
curl http://192.168.1.50:8080/api/users/student-123/usage
```

**Reset quota (admin only):**
```bash
curl -X POST http://192.168.1.50:8080/api/admin/quotas/reset \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"user_id": "student-123"}'
```

---

## Security Notes

### What Students CAN See

- ✅ Their own tasks
- ✅ Educational info (sharding, distribution)
- ✅ Anonymized node topology
- ✅ Their usage statistics

### What Students CANNOT See

- ❌ Your home network IPs
- ❌ Other students' tasks
- ❌ Node configurations
- ❌ Infrastructure details

### What You CAN See (Admin)

- ✅ All tasks
- ✅ All student activity
- ✅ Full infrastructure (IPs, configs)
- ✅ System logs
- ✅ Performance metrics

---

## Production Readiness

### Before Class Use

- [ ] Change JWT secret in config
- [ ] Set up student accounts (or use anonymous)
- [ ] Test with 2-3 volunteer students
- [ ] Verify firewall rules
- [ ] Document your laptop IP
- [ ] Prepare backup laptop (if primary fails)

### During Class

- [ ] Arrive early, start Songbird
- [ ] Verify federation connectivity
- [ ] Share laptop IP with students
- [ ] Monitor dashboard for issues
- [ ] Have backup plan (local GPU if federation fails)

### After Class

- [ ] Collect student feedback
- [ ] Review task logs
- [ ] Check for anomalies
- [ ] Update documentation based on issues

---

## Next Steps

### Immediate (This Week)

1. Deploy on Windows laptop
2. Test with your own student account
3. Verify full workflow
4. Document any issues

### Short Term (January)

1. Test with 2-3 volunteer students
2. Refine based on feedback
3. Prepare demo for Prof. Murillo
4. Plan full class rollout

### Medium Term (Spring 2025)

1. Full class deployment
2. Collect metrics
3. Iterate based on student feedback
4. Begin drafting research paper

---

## Support

### Technical Issues

- Kevin Mok: mokkevin@msu.edu
- Available for setup assistance
- Can provide remote debugging

### Documentation

- Student Guide: `STUDENT_GUIDE.md`
- Discovery Architecture: `DISCOVERY_ARCHITECTURE.md`
- Demo Materials: `MURILLO_DEMO.md`
- Windows Testing: `WINDOWS_TESTING.md`

---

**You're ready to deploy!** Start with local testing, then expand to volunteers, then full class. 🎓🎵✨

