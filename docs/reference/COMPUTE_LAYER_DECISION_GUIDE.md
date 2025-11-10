# 🎯 Songbird Compute Layer Decision Guide

**Date**: November 9, 2025  
**Purpose**: Clear guidance on when to use Execution Agent vs. Toadstool  
**Status**: ✅ Production Architecture

---

## 🏗️ **The Two-Layer Compute Architecture**

Songbird's compute layer has TWO complementary systems, each optimized for different use cases:

```
┌─────────────────────────────────────────────────────────────────┐
│                      SONGBIRD ORCHESTRATOR                       │
│         (Federation, Discovery, Load Balancing, Routing)        │
└────┬──────────────────────────────────────────────────────┬─────┘
     │                                                       │
     │ Lightweight Tasks                    Heavy Compute   │
     ▼                                                       ▼
┌──────────────────────┐                    ┌────────────────────────┐
│  EXECUTION AGENT     │                    │      TOADSTOOL         │
│  (Port 9020)         │                    │      (Port 9000)       │
│                      │                    │                        │
│  • Simple commands   │                    │  • Python/PyTorch RT   │
│  • System tasks      │                    │  • GPU orchestration   │
│  • Quick ops         │                    │  • Distributed ML      │
│  • SSH replacement   │                    │  • Resource mgmt       │
│  • Songbird admin    │                    │  • Container runtime   │
│                      │                    │  • Complex workloads   │
│  ~100 LOC            │                    │  Production-grade      │
│  Single binary       │                    │  97/100 quality        │
└──────────────────────┘                    └────────────────────────┘
```

### Key Insight:
**They're NOT competing - they're complementary!**

---

## 🔍 **When to Use: Decision Matrix**

| Use Case | Execution Agent | Toadstool | Why |
|----------|----------------|-----------|-----|
| **Update Songbird binary** | ✅ Perfect | ❌ Overkill | Simple file operation |
| **System health check** | ✅ Perfect | ❌ Overkill | Quick command |
| **Restart Songbird service** | ✅ Perfect | ❌ Overkill | System admin |
| **Quick diagnostics** | ✅ Perfect | ❌ Overkill | Simple commands |
| **Deploy Toadstool** | ✅ Perfect | N/A | Bootstrap operation |
| **ImageNet Training** | ❌ Too basic | ✅ Perfect | ML workload |
| **PyTorch DDP** | ❌ No GPU support | ✅ Perfect | Distributed ML |
| **GPU workloads** | ❌ Not GPU-aware | ✅ Perfect | GPU orchestration |
| **Container orchestration** | ❌ No runtime | ✅ Perfect | Toadstool's specialty |
| **Long-running compute** | ❌ Too simple | ✅ Perfect | Resource management |
| **WASM execution** | ❌ No runtime | ✅ Perfect | Toadstool feature |
| **Cross-platform workloads** | ❌ Basic only | ✅ Perfect | Universal substrates |

---

## ⚡ **Quick Decision Tree**

```
Is it an ML/AI workload?
  ├─ YES → Use Toadstool
  └─ NO → Is it GPU-intensive?
      ├─ YES → Use Toadstool
      └─ NO → Is it a long-running compute job?
          ├─ YES → Use Toadstool
          └─ NO → Is it a simple command/system task?
              ├─ YES → Use Execution Agent
              └─ NO → If in doubt → Use Toadstool
```

**Rule of Thumb**:
- **Execution Agent**: Things you'd do over SSH
- **Toadstool**: Things you'd submit to a compute cluster

---

## 📋 **Use Cases by Category**

### 🟢 **Execution Agent (Lightweight)**

**Perfect For:**
1. **System Administration**
   - Restarting services
   - Checking system status
   - File operations (cp, mv, chmod)
   - Log inspection

2. **Songbird Operations**
   - Deploying Songbird updates
   - Deploying Toadstool/Squirrel binaries
   - Federation health checks
   - Quick diagnostic commands

3. **Bootstrap Operations**
   - Initial tower setup
   - Installing dependencies
   - Environment configuration

**Examples:**
```bash
# Update Songbird
curl -X POST http://tower-b:9020/api/v1/execution/command \
  -d '{"command": "sudo systemctl restart songbird"}'

# Check disk space
curl -X POST http://tower-b:9020/api/v1/execution/command \
  -d '{"command": "df -h"}'

# Deploy Toadstool
curl -X POST http://tower-b:9020/api/deployment/binary \
  -F "binary=@toadstool" -F "service_name=toadstool"
```

---

### 🔵 **Toadstool (Heavy Compute)**

**Perfect For:**
1. **Machine Learning**
   - PyTorch distributed training
   - Model inference
   - Dataset preprocessing
   - Hyperparameter tuning

2. **GPU Workloads**
   - CUDA computations
   - Image generation (Stable Diffusion)
   - Video processing
   - Scientific computing

3. **Distributed Computing**
   - Multi-node coordination
   - Map-reduce operations
   - Parallel data processing
   - Complex orchestration

4. **Container Workloads**
   - Docker container execution
   - WASM runtime
   - Native binary execution
   - Python script execution

**Examples:**
```bash
# Submit distributed ML training
curl -X POST http://songbird:8080/api/compute/distributed \
  -d '{
    "job_type": "pytorch_ddp",
    "world_size": 3,
    "script": "train_distributed.py",
    "requirements": {"gpu": true}
  }'

# GPU inference task
curl -X POST http://songbird:8080/api/compute/gpu \
  -d '{
    "task": "stable_diffusion",
    "prompt": "mountain landscape",
    "requirements": {"min_gpu_memory_gb": 8}
  }'
```

---

## 🚀 **Deployment Status**

### Current State (Nov 9, 2025):

**Execution Agent**:
- ✅ Tower B (192.168.1.134:9020) - Running
- ✅ Tower C (192.168.1.207:9020) - Running
- ✅ Purpose: Songbird admin tasks, binary deployment

**Toadstool**:
- ❌ Tower B (192.168.1.134:9000) - Not running yet
- ❌ Tower C (192.168.1.207:9000) - Not running yet
- 📋 Purpose: ML/GPU compute (to be deployed)

**Next Step**: Deploy Toadstool to both towers for ML workloads!

---

## 📖 **Deployment Workflows**

### Workflow 1: Deploy Toadstool via Execution Agent

**This is the bootstrap pattern!**

```bash
# Step 1: Build Toadstool locally
cd /home/eastgate/Development/ecoPrimals/toadstool
cargo build --release

# Step 2: Deploy to Tower B via Execution Agent
curl -X POST http://192.168.1.134:9020/api/deployment/binary \
  -F "binary=@target/release/toadstool" \
  -F "service_name=toadstool-compute" \
  -F 'env_vars={"TOADSTOOL_HOST":"192.168.1.134","TOADSTOOL_PORT":"9000","SONGBIRD_ENDPOINT":"http://192.168.1.134:8081"}' \
  -F "auto_start=true"

# Step 3: Verify Toadstool is running
curl http://192.168.1.134:9000/health

# Step 4: Repeat for Tower C
curl -X POST http://192.168.1.207:9020/api/deployment/binary \
  -F "binary=@target/release/toadstool" \
  -F "service_name=toadstool-compute" \
  -F 'env_vars={"TOADSTOOL_HOST":"192.168.1.207","TOADSTOOL_PORT":"9000","SONGBIRD_ENDPOINT":"http://192.168.1.207:8082"}' \
  -F "auto_start=true"

curl http://192.168.1.207:9000/health
```

**This is the proper pattern**: Use Execution Agent to bootstrap Toadstool!

### Workflow 2: Submit ML Training via Songbird → Toadstool

**After Toadstool is deployed:**

```bash
# Submit distributed training job to Songbird
curl -X POST http://192.168.1.144:8080/api/compute/distributed \
  -H "Content-Type: application/json" \
  -d '{
    "job_type": "pytorch_ddp",
    "framework": "pytorch",
    "world_size": 3,
    "script_path": "/path/to/train_distributed.py",
    "requirements": {
      "gpu": true,
      "min_memory_gb": 8,
      "python_version": "3.10"
    },
    "env": {
      "MASTER_ADDR": "192.168.1.144",
      "MASTER_PORT": "29500"
    }
  }'

# Songbird:
#   1. Discovers Toadstool instances on all towers
#   2. Routes rank 0 to Tower A Toadstool
#   3. Routes rank 1 to Tower B Toadstool
#   4. Routes rank 2 to Tower C Toadstool
#
# Toadstool on each tower:
#   1. Sets up Python environment
#   2. Executes training script
#   3. Manages GPU resources
#   4. Coordinates with other ranks via PyTorch DDP
```

---

## 🎯 **Architecture Principles**

### 1. **Right Tool for the Job**
- Don't use Toadstool for simple commands
- Don't use Execution Agent for ML workloads

### 2. **Bootstrap Pattern**
- Use Execution Agent to deploy Toadstool
- Use Execution Agent for Songbird administration
- Use Toadstool for all compute workloads after deployment

### 3. **Primal Sovereignty**
- Each primal does what it's best at
- Songbird orchestrates
- Toadstool computes
- Execution Agent administers

### 4. **Avoid SSH**
- Execution Agent replaces most SSH needs
- Toadstool provides compute without manual login
- Only use SSH for emergency debugging

---

## 📊 **Performance Comparison**

| Metric | Execution Agent | Toadstool |
|--------|----------------|-----------|
| **Binary Size** | ~10MB | ~30MB |
| **Startup Time** | <100ms | ~500ms |
| **Memory Footprint** | ~10MB | ~50MB+ |
| **GPU Support** | ❌ No | ✅ Yes |
| **Python Runtime** | ❌ No | ✅ Yes |
| **Distributed Coord** | ❌ No | ✅ Yes |
| **Use Case** | Admin | Compute |

**Both are lightweight compared to alternatives!**

---

## 🔧 **Configuration**

### Execution Agent Config
```bash
# Simple environment variables
EXECUTION_AGENT_PORT=9020
EXECUTION_AGENT_MAX_JOBS=10
EXECUTION_AGENT_TIMEOUT=300
```

### Toadstool Config
```bash
# More complex configuration
TOADSTOOL_HOST=192.168.1.134
TOADSTOOL_PORT=9000
TOADSTOOL_GPU_ENABLED=true
SONGBIRD_ENDPOINT=http://192.168.1.134:8081
TOADSTOOL_PYTHON_RUNTIME=true
TOADSTOOL_MAX_CONCURRENT_JOBS=5
```

---

## 🚨 **Common Mistakes to Avoid**

### ❌ **WRONG: Using Execution Agent for ML**
```bash
# DON'T DO THIS!
curl -X POST http://tower-b:9020/api/v1/execution/command \
  -d '{"command": "cd /path && python train.py"}'
```
**Problem**: No GPU support, no resource management, no coordination

### ✅ **RIGHT: Using Toadstool for ML**
```bash
# DO THIS!
curl -X POST http://songbird:8080/api/compute/pytorch \
  -d '{"script": "/path/train.py", "requirements": {"gpu": true}}'
```
**Benefit**: GPU-aware, resource-managed, coordinated

---

### ❌ **WRONG: Using Toadstool for Simple Commands**
```bash
# DON'T DO THIS!
curl -X POST http://tower-b:9000/api/workload \
  -d '{"type": "native", "command": "hostname"}'
```
**Problem**: Overkill, slower, wastes resources

### ✅ **RIGHT: Using Execution Agent for Simple Commands**
```bash
# DO THIS!
curl -X POST http://tower-b:9020/api/v1/execution/command \
  -d '{"command": "hostname"}'
```
**Benefit**: Fast, lightweight, purpose-built

---

## 📚 **Documentation References**

### Execution Agent:
- **Implementation**: `crates/songbird-execution-agent/`
- **API Spec**: `specs/REMOTE_EXECUTION_API_SPEC.md`
- **Deployment Plan**: `REMOTE_EXECUTION_DEPLOYMENT_PLAN.md`

### Toadstool:
- **README**: `../toadstool/README.md`
- **Integration**: `../toadstool/docs/🚀_SONGBIRD_INTEGRATION_PLAN.md`
- **This Project**: `TOADSTOOL_DEPLOYMENT_PLAN.md`
- **ML Integration**: `TOADSTOOL_SONGBIRD_ML_INTEGRATION.md`

---

## 🎊 **Summary: The Proper Stack**

```
┌──────────────────────────────────────────────────────┐
│              USER / APPLICATION                      │
└────────────────────┬─────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────┐
│           🐦 SONGBIRD ORCHESTRATOR                   │
│  • Discovery & Federation                            │
│  • Load Balancing & Routing                          │
│  • Health Monitoring                                 │
└─────┬────────────────────────────────────────┬───────┘
      │                                        │
      │ "restart songbird"           "train ML model"
      │                                        │
      ▼                                        ▼
┌──────────────┐                    ┌──────────────────┐
│  EXECUTION   │                    │    TOADSTOOL     │
│    AGENT     │                    │                  │
│  Port 9020   │                    │    Port 9000     │
│              │                    │                  │
│  Admin/Ops   │                    │  Compute/ML/GPU  │
└──────────────┘                    └──────────────────┘
```

**Each layer does what it does best!**

---

## 🚀 **Next Actions**

1. ✅ Keep Execution Agent running (admin tasks)
2. 📋 Deploy Toadstool to Tower B & C (compute tasks)
3. 🎯 Use proper layer for each task
4. 📖 Update docs to reflect this architecture

**Status**: Ready to deploy Toadstool via Execution Agent! 🎊

---

**Remember**: 
- **Execution Agent** = SSH replacement (Songbird admin)
- **Toadstool** = Compute cluster (ML/GPU workloads)
- **Both** work together beautifully! 🐦🍄

