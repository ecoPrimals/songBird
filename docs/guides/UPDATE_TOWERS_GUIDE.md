# 🚀 Update Songbird on Remote Towers (HTTP Method)

**Date**: November 9, 2025  
**Method**: HTTP Deployment API (Federation-Native)  
**Towers**: B (Strandgate - 192.168.1.134), C (Southgate - 192.168.1.207)

---

## 🎯 Quick Start (Copy-Paste)

### 1. Build latest Songbird
```bash
cd /home/eastgate/Development/ecoPrimals/songbird
cargo build --release --bin songbird
```

### 2. Run the update script
```bash
./update_towers_http.sh
```

That's it! The script will:
- ✅ Upload new binary to Tower B via HTTP
- ✅ Upload new binary to Tower C via HTTP
- ✅ Show you the deployment paths
- ⚠️ You'll need to manually restart (see below)

---

## 📋 Manual Steps (After Upload)

### Tower B (Strandgate)
```bash
# SSH to Tower B
ssh 192.168.1.134

# Stop Songbird
sudo systemctl stop songbird
# OR if running manually:
pkill songbird

# Replace binary (deployed to /tmp/songbird-deployments/deploy-XXXXX/service)
sudo cp /tmp/songbird-deployments/deploy-*/service /usr/local/bin/songbird
sudo chmod +x /usr/local/bin/songbird

# Start Songbird
sudo systemctl start songbird
# OR run manually:
/usr/local/bin/songbird &

# Verify
curl http://localhost:8080/health
```

### Tower C (Southgate)
```bash
# SSH to Tower C
ssh 192.168.1.207

# Same steps as Tower B
sudo systemctl stop songbird
sudo cp /tmp/songbird-deployments/deploy-*/service /usr/local/bin/songbird
sudo chmod +x /usr/local/bin/songbird
sudo systemctl start songbird

# Verify
curl http://localhost:8080/health
```

---

## ✅ Verification

### Check Federation Status
```bash
# From Tower A (Eastgate)
curl http://192.168.1.144:8080/api/federation/nodes | jq '.'
```

**Expected Output**:
```json
[
  {
    "node_id": "tower-a-eastgate",
    "node_name": "Eastgate",
    "node_address": "192.168.1.144:8080",
    "status": "active"
  },
  {
    "node_id": "tower-b-strandgate",
    "node_name": "Strandgate",
    "node_address": "192.168.1.134:8080",
    "status": "active"
  },
  {
    "node_id": "tower-c-southgate",
    "node_name": "Southgate",
    "node_address": "192.168.1.207:8080",
    "status": "active"
  }
]
```

### Check Each Tower Individually
```bash
# Tower A
curl http://192.168.1.144:8080/health

# Tower B
curl http://192.168.1.134:8080/health

# Tower C
curl http://192.168.1.207:8080/health
```

---

## 🔧 Alternative: Direct curl Method

If the script fails, use direct curl:

### Upload to Tower B
```bash
curl -X POST http://192.168.1.134:8080/api/deployment/binary \
  -F "binary=@./target/release/songbird" \
  -F "service_name=songbird-update" \
  -F 'env_vars={}' \
  -F "auto_start=false" | jq '.'
```

### Upload to Tower C
```bash
curl -X POST http://192.168.1.207:8080/api/deployment/binary \
  -F "binary=@./target/release/songbird" \
  -F "service_name=songbird-update" \
  -F 'env_vars={}' \
  -F "auto_start=false" | jq '.'
```

---

## 🏗️ Current Tower Configuration

| Tower | Name | IP | Port | Role | Status |
|-------|------|------------|------|------|--------|
| A | Eastgate | 192.168.1.144 | 8080 | Master | ✅ Local |
| B | Strandgate | 192.168.1.134 | 8080 | Worker | 🔄 Remote |
| C | Southgate | 192.168.1.207 | 8080 | Worker | 🔄 Remote |

---

## 🐛 Troubleshooting

### "Connection refused" on Tower B or C
```bash
# Check if Songbird is running
ssh 192.168.1.134 "pgrep -a songbird"

# Check if port is listening
ssh 192.168.1.134 "netstat -tuln | grep 8080"

# Start if not running
ssh 192.168.1.134 "cd /home/eastgate/Development/ecoPrimals/songbird && ./target/release/songbird &"
```

### "Deployment API not available"
The remote Songbird might not have the deployment API enabled. Check:
```bash
curl http://192.168.1.134:8080/api/deployment/capabilities
```

If it returns 404, the API needs to be enabled in the orchestrator config.

### Binary too large (>100MB)
The default single-upload limit is 100MB. If your binary is larger:
1. Use chunked upload (see `HTTP_DEPLOYMENT_GUIDE.md`)
2. Or compress first: `gzip -c ./target/release/songbird > songbird.gz`

---

## 🔒 Security Note

**Current Setup** (LAN):
- ✅ Plain HTTP is fine on trusted LAN
- ✅ No authentication needed
- ✅ Firewall provides isolation

**Future** (Internet with BearDog):
- 🔒 Add TLS/bearer tokens
- 🔒 Enable BearDog security primal
- 🔒 Use mTLS for tower-to-tower

---

## 📚 Related Files

- **Update Script**: `./update_towers_http.sh`
- **Detailed HTTP Guide**: `HTTP_DEPLOYMENT_GUIDE.md`
- **Tower Verification**: `./verify_tower_b.sh`
- **Federation API**: `crates/songbird-orchestrator/src/server/federation_api.rs`
- **Deployment API**: `crates/songbird-orchestrator/src/server/deployment_api.rs`

---

## ⚡ Quick Reference

```bash
# Build
cargo build --release --bin songbird

# Upload
./update_towers_http.sh

# Verify upload
curl http://192.168.1.134:8080/api/deployment/list | jq '.'

# SSH and replace (Tower B)
ssh 192.168.1.134 "sudo systemctl stop songbird && \
  sudo cp /tmp/songbird-deployments/deploy-*/service /usr/local/bin/songbird && \
  sudo systemctl start songbird"

# Check federation
curl http://192.168.1.144:8080/api/federation/nodes | jq '.'
```

---

**Status**: ✅ Ready to Use  
**Last Updated**: November 9, 2025

