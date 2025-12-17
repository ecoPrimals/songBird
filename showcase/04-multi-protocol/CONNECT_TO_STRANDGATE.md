# 🔌 Connecting to Strandgate Tower

**Quick guide for connecting to your existing Strandgate tower**

---

## ✅ Current Status

- **Host reachable:** `192.168.1.100` ✓ (responds to ping)
- **Songbird status:** Unknown (need to check)

---

## 🔍 Step 1: Check if Songbird is Running

### SSH to Strandgate and check status:

```bash
# Check if Songbird service is running
ssh 192.168.1.100 'systemctl status songbird'

# Expected output if running:
# ● songbird.service - Songbird Orchestrator
#    Loaded: loaded
#    Active: active (running)
```

### If not running as a service, check for process:

```bash
# Check if Songbird process is running
ssh 192.168.1.100 'ps aux | grep songbird'
```

---

## 🔍 Step 2: Find the Port

### Check what ports Songbird is listening on:

```bash
# Method 1: Using ss (socket statistics)
ssh 192.168.1.100 'sudo ss -tlnp | grep songbird'

# Method 2: Using lsof
ssh 192.168.1.100 'sudo lsof -i -P | grep songbird'

# Method 3: Check logs
ssh 192.168.1.100 'journalctl -u songbird -n 50 | grep -i "listening\|port"'
```

Common ports to check:
- `8080` - Default HTTP
- `8443` - Default HTTPS
- `8000` - Alternative HTTP
- `3000` - Alternative HTTP

---

## 🔍 Step 3: Test Connection

Once you know the port, test the connection:

```bash
# Replace <PORT> with the actual port
curl http://192.168.1.100:<PORT>/health

# Expected response:
# {"status":"healthy","version":"..."}
```

---

## ✅ Step 4: Run Connectivity Check

Once you know the correct port:

```bash
export REMOTE_HOST=192.168.1.100
export REMOTE_PORT=<the-port>
./showcase/04-multi-protocol/check_strandgate.sh
```

This will:
- ✅ Verify connection
- ✅ Check Songbird version
- ✅ Check available protocols
- ✅ Show deployment commands

---

## 🚀 Step 5: Deploy Update

After successful connection:

```bash
# Set configuration
export REMOTE_HOST=192.168.1.100
export REMOTE_PORT=<the-port>
export COMPUTE_BRIDGE=http://192.168.1.100:<the-port>

# Deploy the update
./showcase/04-multi-protocol/deploy_to_remote_tower.sh
```

---

## 🐛 Common Issues

### Issue 1: "Connection refused"

**Cause:** Songbird not running or wrong port

**Solution:**
```bash
# Start Songbird on Strandgate
ssh 192.168.1.100 'sudo systemctl start songbird'

# Or if not using systemd
ssh 192.168.1.100 'cd /path/to/songbird && cargo run --release'
```

### Issue 2: "Connection timeout"

**Cause:** Firewall blocking ports

**Solution:**
```bash
# Check firewall status
ssh 192.168.1.100 'sudo ufw status'

# Allow Songbird ports
ssh 192.168.1.100 'sudo ufw allow 8080/tcp'
ssh 192.168.1.100 'sudo ufw allow 8443/tcp'
ssh 192.168.1.100 'sudo ufw allow 8081/tcp'
```

### Issue 3: "Cannot SSH to Strandgate"

**Cause:** SSH not configured or wrong IP

**Solution:**
```bash
# Try SSH with username
ssh <your-username>@192.168.1.100

# Check if SSH is running
ping 192.168.1.100  # Should respond
telnet 192.168.1.100 22  # Should connect to SSH
```

### Issue 4: "Host not found"

**Cause:** Wrong IP address

**Solution:**
```bash
# Scan your network for Strandgate
nmap -sn 192.168.1.0/24 | grep -B 2 "strandgate"

# Or use arp
arp -a | grep -i strandgate

# Or check your router's DHCP leases
```

---

## 📋 Quick Command Reference

```bash
# Find Strandgate on network
./showcase/04-multi-protocol/check_strandgate.sh

# SSH to Strandgate
ssh 192.168.1.100

# Check Songbird status on Strandgate
ssh 192.168.1.100 'systemctl status songbird'

# Check Songbird logs on Strandgate
ssh 192.168.1.100 'journalctl -u songbird -f'

# Test connection manually
curl http://192.168.1.100:8080/health

# Deploy update
export REMOTE_HOST=192.168.1.100
export COMPUTE_BRIDGE=http://192.168.1.100:8080
./showcase/04-multi-protocol/deploy_to_remote_tower.sh
```

---

## 🎯 Expected Network Setup

```
Your Dev Machine (localhost)
         │
         │ LAN (192.168.1.x)
         │
         ▼
Strandgate Tower (192.168.1.100)
  ├── Songbird Orchestrator
  │   ├── HTTP: 8080
  │   ├── HTTPS: 8443
  │   └── tarpc: 8081
  └── OS Services
```

---

## ✅ Success Indicators

When connection is working:

```bash
$ ./showcase/04-multi-protocol/check_strandgate.sh

✓✓✓ FOUND SONGBIRD!

Songbird Instance Details:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Status:   healthy
Version:  0.1.0
Endpoint: http://192.168.1.100:8080

✓ Multi-protocol support detected! (or needs update)

🚀 Ready to deploy!
```

---

## 🆘 Still Having Issues?

1. **Verify Strandgate is on your network:**
   ```bash
   ping 192.168.1.100
   ```

2. **Check if you can SSH:**
   ```bash
   ssh 192.168.1.100 'echo "Connection works!"'
   ```

3. **Manually check for Songbird:**
   ```bash
   ssh 192.168.1.100 'ps aux | grep -i songbird'
   ssh 192.168.1.100 'which songbird-orchestrator'
   ```

4. **Check network configuration:**
   ```bash
   ip addr  # On your machine
   ssh 192.168.1.100 'ip addr'  # On Strandgate
   ```

---

**Once connected, you're ready to deploy the multi-protocol update!** 🚀

