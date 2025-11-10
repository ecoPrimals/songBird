# 🎯 Remote Execution Deployment Plan

**Date**: November 9, 2025  
**Goal**: Deploy execution agents to Towers B & C, enabling SSH-free remote execution  
**Status**: Ready to Deploy

---

## 💡 **You're Absolutely Right!**

### Current Situation
- ✅ **Tower A (Eastgate)**: Has full remote execution capabilities
- ❌ **Tower B (Strandgate)**: Needs execution agent
- ❌ **Tower C (Southgate)**: Needs execution agent
- 🔄 **Using SSH**: As temporary workaround

### After Deployment
- ✅ **All towers** have execution agents
- ✅ **HTTP-based** remote execution
- ✅ **SSH becomes redundant** for execution tasks
- ✅ **Broadcast execution** works across all towers

---

## 🏗️ Architecture Overview

```
┌────────────────────────────────────────────────────────────────┐
│                     TOWER A (Eastgate)                         │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │    Songbird Orchestrator with Execution Manager          │ │
│  │    - Broadcast execution to all towers                   │ │
│  │    - Job lifecycle management                            │ │
│  │    - Federation-aware routing                            │ │
│  └──────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────┘
                              │
                              │ HTTP (no SSH!)
                ┌─────────────┴─────────────┐
                │                           │
        ┌───────▼──────┐            ┌──────▼───────┐
        │  TOWER B     │            │  TOWER C     │
        │ (Strandgate) │            │ (Southgate)  │
        ├──────────────┤            ├──────────────┤
        │ Execution    │            │ Execution    │
        │ Agent        │            │ Agent        │
        │ Port: 9020   │            │ Port: 9020   │
        └──────────────┘            └──────────────┘
```

---

## 🚀 Deployment Steps

### Phase 1: Deploy Execution Agents via HTTP

#### Step 1: Build the agent
```bash
cd /home/eastgate/Development/ecoPrimals/songbird
cargo build --release -p songbird-execution-agent
```

#### Step 2: Deploy to Tower B
```bash
curl -X POST http://192.168.1.134:8080/api/deployment/binary \
  -F "binary=@./target/release/agent" \
  -F "service_name=execution-agent" \
  -F 'env_vars={"AGENT_PORT":"9020","AGENT_HOST":"0.0.0.0","SECURITY_TIER":"sovereign"}' \
  -F "auto_start=true"
```

#### Step 3: Deploy to Tower C
```bash
curl -X POST http://192.168.1.207:8080/api/deployment/binary \
  -F "binary=@./target/release/agent" \
  -F "service_name=execution-agent" \
  -F 'env_vars={"AGENT_PORT":"9020","AGENT_HOST":"0.0.0.0","SECURITY_TIER":"sovereign"}' \
  -F "auto_start=true"
```

#### Step 4: Verify agents are running
```bash
# Tower B
curl http://192.168.1.134:9020/health

# Tower C  
curl http://192.168.1.207:9020/health
```

**Expected Response**:
```json
{
  "status": "healthy",
  "agent_version": "1.0.0",
  "uptime_secs": 42
}
```

---

### Phase 2: Test Remote Execution

#### Test single command on Tower B
```bash
curl -X POST http://192.168.1.144:8080/api/execution/run \
  -H "Content-Type: application/json" \
  -d '{
    "target": "tower-b-strandgate",
    "command": "hostname",
    "args": []
  }'
```

#### Test broadcast to all towers
```bash
curl -X POST http://192.168.1.144:8080/api/execution/broadcast \
  -H "Content-Type: application/json" \
  -d '{
    "command": "uname -a",
    "args": [],
    "targets": ["tower-b-strandgate", "tower-c-southgate"]
  }'
```

**Expected Response**:
```json
{
  "broadcast_id": "broadcast-12345",
  "targets": 2,
  "results": [
    {
      "target": "tower-b-strandgate",
      "status": "success",
      "stdout": "Linux strandgate 6.5.0 ...",
      "exit_code": 0
    },
    {
      "target": "tower-c-southgate", 
      "status": "success",
      "stdout": "Linux southgate 6.5.0 ...",
      "exit_code": 0
    }
  ]
}
```

---

## ✅ What This Enables (SSH-Free!)

### 1. Remote Command Execution
```bash
# Run commands on any tower via HTTP
curl -X POST http://192.168.1.144:8080/api/execution/run \
  -H "Content-Type: application/json" \
  -d '{
    "target": "tower-b-strandgate",
    "command": "nvidia-smi",
    "args": []
  }'
```

### 2. Broadcast Execution
```bash
# Run same command on all towers simultaneously
curl -X POST http://192.168.1.144:8080/api/execution/broadcast \
  -H "Content-Type: application/json" \
  -d '{
    "command": "df -h",
    "targets": ["tower-b-strandgate", "tower-c-southgate"]
  }'
```

### 3. Background Job Management
```bash
# Start long-running job
curl -X POST http://192.168.1.134:9020/api/jobs \
  -H "Content-Type: application/json" \
  -d '{
    "command": "python train_model.py",
    "background": true
  }'

# Check status
curl http://192.168.1.134:9020/api/jobs/{job_id}

# Get output
curl http://192.168.1.134:9020/api/jobs/{job_id}/output
```

### 4. Distributed ML Training (No SSH!)
```bash
# Launch training on all towers via broadcast
curl -X POST http://192.168.1.144:8080/api/execution/broadcast \
  -H "Content-Type: application/json" \
  -d '{
    "command": "python train_distributed.py",
    "args": ["--rank", "$RANK", "--world-size", "3"],
    "targets": ["tower-b-strandgate", "tower-c-southgate"]
  }'
```

---

## 📊 Comparison: SSH vs Remote Execution API

| Feature | SSH Method | Remote Execution API |
|---------|-----------|---------------------|
| **Setup** | ❌ SSH keys required | ✅ Zero config |
| **Security** | ⚠️ SSH credentials | ✅ BearDog-ready |
| **Federation** | ❌ External protocol | ✅ Native integration |
| **Broadcast** | ❌ Manual scripting | ✅ Built-in |
| **Job Management** | ❌ Manual tracking | ✅ Lifecycle API |
| **Monitoring** | ❌ No standard API | ✅ REST endpoints |
| **Internet-ready** | ⚠️ Port 22 often blocked | ✅ HTTPS (with BearDog) |
| **Audit Trail** | ⚠️ Scattered logs | ✅ Centralized |

---

## 🎯 Migration Path

### Current (Using SSH)
```bash
# Launch training on Tower B
ssh 192.168.1.134 "cd /path && python train.py --rank 1" &

# Launch training on Tower C
ssh 192.168.1.207 "cd /path && python train.py --rank 2" &
```

### After (Using Remote Execution)
```bash
# Launch training on all towers with one call
curl -X POST http://192.168.1.144:8080/api/execution/broadcast \
  -H "Content-Type: application/json" \
  -d '{
    "command": "python",
    "args": ["/path/train.py", "--rank", "$RANK"],
    "working_dir": "/path",
    "targets": ["tower-b-strandgate", "tower-c-southgate"]
  }'
```

**Benefits**:
- ✅ Single API call instead of multiple SSH commands
- ✅ Automatic error handling and retries
- ✅ Centralized job status
- ✅ No SSH key management

---

## 🔒 Security Tiers

The execution agent supports three security tiers:

### Tier 1: Sovereign (Current - LAN)
- ✅ Works standalone
- ✅ Basic validation
- ✅ No external dependencies
- ✅ Perfect for trusted LANs

### Tier 2: Network Effect (Future - with BearDog)
- 🔒 BearDog authentication
- 🔒 TLS/mTLS
- 🔒 Token-based access
- 🔒 Audit logging

### Tier 3: Federation (Future - full ecosystem)
- 🔒 All Tier 2 features
- 🔒 Cross-primal authorization
- 🔒 Threat detection
- 🔒 Compliance reporting

---

## 📋 Deployment Checklist

- [ ] Build execution agent binary
- [ ] Deploy to Tower B via HTTP
- [ ] Deploy to Tower C via HTTP
- [ ] Verify agents are running
- [ ] Test single execution
- [ ] Test broadcast execution
- [ ] Update training scripts to use API instead of SSH
- [ ] Document new execution patterns
- [ ] Archive SSH-based scripts (keep as backup)

---

## 🐛 Troubleshooting

### Agent won't start
```bash
# Check deployment logs
curl http://192.168.1.134:8080/api/deployment/list | jq '.[] | select(.service_name == "execution-agent")'

# Check if port 9020 is available
ssh 192.168.1.134 "netstat -tuln | grep 9020"
```

### Can't reach agent
```bash
# Test from Tower A
curl http://192.168.1.134:9020/health

# Check firewall (if needed)
ssh 192.168.1.134 "sudo ufw allow 9020/tcp"
```

### Broadcast fails
```bash
# Check orchestrator logs
curl http://192.168.1.144:8080/api/execution/logs

# Verify all agents are registered
curl http://192.168.1.144:8080/api/federation/services/type/execution
```

---

## 📚 Related Documentation

- **Remote Execution**: `README_REMOTE_EXECUTION.md`
- **API Spec**: `specs/REMOTE_EXECUTION_API_SPEC.md`
- **HTTP Deployment**: `HTTP_DEPLOYMENT_GUIDE.md`
- **Federation**: `crates/songbird-orchestrator/src/server/federation_api.rs`
- **Broadcast**: `crates/songbird-orchestrator/src/core/execution/broadcast.rs`

---

## 🎊 Summary

**Yes, you're absolutely right!**

1. ✅ Remote execution system is already built
2. ✅ Tower B & C just need the execution agent
3. ✅ Use HTTP deployment API to push the agent
4. ✅ After that, SSH becomes redundant for execution tasks
5. ✅ Everything works via federation-native HTTP APIs

**The only thing needed**: Deploy the agent binary to the other towers!

---

**Ready to deploy?** Run the commands above! 🚀

**Status**: Implementation complete, deployment pending  
**ETA**: 15 minutes to deploy and verify

