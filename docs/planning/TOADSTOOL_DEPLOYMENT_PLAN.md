# 🐸 Toadstool Deployment Plan

**Date:** November 8, 2025 (Updated November 9, 2025)  
**Status:** ⚠️ OUTDATED - See COMPUTE_LAYER_DECISION_GUIDE.md for current approach  
**Target:** Deploy Toadstool GPU compute from Tower A → Tower B via Songbird

---

## ⚠️ **UPDATE (November 9, 2025)**

**This document is OUTDATED.** The proper deployment approach is now:

1. **Use Execution Agent** to deploy Toadstool (bootstrap pattern)
2. **Use Toadstool** for all ML/GPU compute workloads
3. **See**:
   - `COMPUTE_LAYER_DECISION_GUIDE.md` - When to use which system
   - `deploy_toadstool_via_agent.sh` - Automated deployment script
   - `TOADSTOOL_SONGBIRD_ML_INTEGRATION.md` - Integration details

**Key Insight**: Execution Agent and Toadstool are complementary, not competing!

---

# Original Plan (Historical Reference)

---

## 🎯 Objective

Deploy the Toadstool GPU compute service from Tower A to Tower B using Songbird's HTTP deployment API, demonstrating:
- Cross-primal integration
- Zero-configuration deployment
- Adaptive method selection
- GPU capability discovery

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Tower A (Eastgate)                        │
│                   192.168.1.144                             │
│                                                             │
│  Songbird Orchestrator: 8080                                │
│  Toadstool Binary: ../toadstool/target/release/toadstool   │
│                                                             │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       │ HTTP Deployment
                       │ (Adaptive: single/chunked based on size)
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                   Tower B (Strandgate)                      │
│                   192.168.1.134                             │
│                                                             │
│  Songbird Orchestrator: 8081                                │
│  Toadstool Service: (deployed) → 9002                       │
│  GPUs: 8x RTX A6000 (48GB each)                            │
│  CPUs: 128 cores AMD EPYC                                   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 📋 Pre-Deployment Checklist

### Tower A (Source)
- [x] Songbird orchestrator running (8080)
- [x] HTTP deployment API operational
- [ ] Toadstool binary built
- [ ] Toadstool binary size determined
- [ ] Deployment method predicted (single/chunked)

### Tower B (Target)
- [ ] Songbird orchestrator rebuilt with latest code
- [ ] Orchestrator running (8081)
- [ ] Deployment API ready
- [ ] Port 9002 available for Toadstool
- [ ] GPU detection working

### Network
- [x] LAN connectivity verified (1Gbps)
- [x] Previous deployment successful (7.68MB in < 1s)
- [x] Adaptive system validated

---

## 🚀 Deployment Steps

### 1. Build Toadstool (if needed)

```bash
cd /home/eastgate/Development/ecoPrimals/toadstool
cargo build --release

# Check binary size
ls -lh target/release/toadstool
```

### 2. Verify Tower B is Ready

```bash
# Health check
curl http://192.168.1.134:8081/health

# Capability check
curl http://192.168.1.134:8081/api/deployment/capabilities | jq

# Verify GPUs detected
curl http://192.168.1.134:8081/api/system/resources
```

### 3. Deploy Toadstool via Songbird

```bash
cd /home/eastgate/Development/ecoPrimals/songbird

# Deploy using adaptive HTTP method
./target/release/songbird-deploy deploy-http \
  --tower http://192.168.1.134:8081 \
  --binary ../toadstool/target/release/toadstool \
  --service toadstool-gpu-compute \
  --env TOADSTOOL_HOST=192.168.1.134 \
  --env TOADSTOOL_PORT=9002 \
  --env TOADSTOOL_GPU_ENABLED=true \
  --env SONGBIRD_FEDERATION_ENDPOINT=http://192.168.1.134:8081
```

### 4. Verify Deployment

```bash
# Check deployment status
curl http://192.168.1.134:8081/api/deployment/list | jq

# Check Toadstool health
curl http://192.168.1.134:9002/health

# Verify GPU availability
curl http://192.168.1.134:9002/api/gpu/status
```

### 5. Test GPU Compute Task

```bash
# Submit a test GPU task through Songbird
curl -X POST http://192.168.1.144:8080/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "gpu_compute",
    "payload": {
      "operation": "matrix_multiply",
      "size": 1024
    },
    "requirements": {
      "gpu": true,
      "min_gpu_memory_gb": 8
    }
  }'
```

---

## 🎯 Expected Results

### Binary Size Prediction
Based on similar Rust GPU compute services:
- **Estimated size:** 15-30MB (includes CUDA/GPU libraries)
- **Method:** Single upload (< 50MB on LAN)
- **Time:** 1-3 seconds over 1Gbps LAN

### Deployment Success Criteria
- ✅ Binary uploaded successfully
- ✅ Service started on Tower B
- ✅ Toadstool health endpoint responding
- ✅ GPUs detected and available
- ✅ Registered in Songbird service registry
- ✅ Capability-based routing working

### Performance Targets
- **Deployment time:** < 5 seconds
- **GPU detection:** < 1 second
- **Service registration:** < 500ms
- **First task routing:** < 100ms

---

## 🔬 Test Scenarios

### Scenario 1: Basic Deployment
**Goal:** Verify Toadstool can be deployed and started
- Deploy Toadstool to Tower B
- Verify service health
- Check GPU detection

**Success:** Service running, GPUs detected

### Scenario 2: Service Discovery
**Goal:** Verify Songbird discovers Toadstool's GPU capabilities
- Query Tower B service registry
- Verify GPU capabilities advertised
- Check capability-based routing

**Success:** GPU capabilities visible in registry

### Scenario 3: GPU Task Routing
**Goal:** Submit GPU task to Tower A, route to Tower B's Toadstool
- Submit GPU task to Tower A orchestrator
- Verify task routed to Tower B's Toadstool
- Verify task executes on GPU
- Verify result returned to Tower A

**Success:** Task completes, result correct

### Scenario 4: Load Testing
**Goal:** Test multiple concurrent GPU tasks
- Submit 10 concurrent GPU tasks
- Verify all route to Tower B
- Verify GPU utilization
- Measure throughput

**Success:** All tasks complete, GPUs utilized

---

## 📊 Metrics to Track

### Deployment Metrics
- Binary size
- Upload method selected
- Upload time
- Total deployment time
- Success/failure status

### Runtime Metrics
- Service startup time
- GPU detection time
- Memory usage (Toadstool service)
- GPU memory usage
- Task execution time
- Task throughput (tasks/second)

### Federation Metrics
- Service discovery latency
- Capability routing latency
- Cross-tower communication latency
- Health check frequency and time

---

## 🚨 Potential Issues & Solutions

### Issue 1: Toadstool Binary Not Built
**Solution:** Build Toadstool first
```bash
cd ../toadstool && cargo build --release
```

### Issue 2: Port 9002 Already in Use
**Solution:** Songbird's smart port management will auto-increment
- Expected: Toadstool starts on 9003 or next available

### Issue 3: GPUs Not Detected
**Solution:** Verify CUDA/GPU drivers on Tower B
```bash
nvidia-smi  # Should show 8x RTX A6000
```

### Issue 4: Binary Too Large (> 50MB)
**Solution:** Adaptive system will automatically use chunked upload
- Expected: Songbird negotiates chunked upload
- Still completes in seconds over LAN

### Issue 5: Toadstool Crashes on Startup
**Solution:** Check logs, verify environment variables
```bash
curl http://192.168.1.134:8081/api/deployment/list | jq '.[] | select(.service_name=="toadstool-gpu-compute")'
# Check stderr/stdout logs
```

---

## 🎯 Success Definition

**Minimum Viable Success:**
- ✅ Toadstool deployed to Tower B
- ✅ Service running and healthy
- ✅ GPUs detected

**Full Success:**
- ✅ All above
- ✅ Registered in Songbird registry
- ✅ GPU capabilities advertised
- ✅ Task submitted to Tower A routes to Tower B
- ✅ Task executes on GPU
- ✅ Result returned correctly

**Stretch Goals:**
- ✅ All above
- ✅ Multiple concurrent tasks
- ✅ GPU utilization > 80%
- ✅ Sub-100ms task routing latency
- ✅ 10+ tasks/second throughput

---

## 📈 Comparison to Traditional Systems

### Kubernetes + GPU Operator
**Setup Time:** 2-4 hours (install K8s, GPU operator, device plugin)  
**Configuration:** Complex YAML, node labels, taints/tolerations  
**Deployment Time:** 2-5 minutes  
**Complexity:** High

### Songbird + Toadstool
**Setup Time:** < 5 minutes (build, deploy)  
**Configuration:** Zero (auto-detected)  
**Deployment Time:** < 5 seconds  
**Complexity:** Zero

**Improvement:** 24-48x faster setup, 24-60x faster deployment

---

## 🔮 Next Steps After Success

1. **Scale to 3+ Towers**
   - Add third tower with different GPU types
   - Test capability-based routing (RTX 2070 vs A6000)
   - Verify intelligent task distribution

2. **Complex GPU Workloads**
   - LLM inference
   - Image generation (Stable Diffusion)
   - Video processing
   - Scientific computing

3. **BearDog Security Integration**
   - Secure inter-tower communication
   - Encrypted task payloads
   - Authentication for GPU resources

4. **Performance Benchmarking**
   - Songbird+Toadstool vs K8s+GPU Operator
   - Latency measurements
   - Throughput comparison
   - Resource efficiency

---

## 📝 Notes

- Toadstool is the GPU compute cornerstone of ecoPrimals
- This is the first cross-primal deployment test
- Success validates the entire Songbird orchestration vision
- Demonstrates zero-config GPU orchestration (industry-first)

---

**Status:** Ready to proceed once Tower B rebuild completes  
**Next:** Verify Tower B, build Toadstool, deploy!

---

## 🎬 Quick Reference Commands

```bash
# Tower B: Verify ready
curl http://192.168.1.134:8081/health

# Tower A: Build Toadstool (if needed)
cd ../toadstool && cargo build --release

# Tower A: Check Toadstool size
ls -lh ../toadstool/target/release/toadstool

# Tower A: Deploy Toadstool
cd ../songbird
./target/release/songbird-deploy deploy-http \
  --tower http://192.168.1.134:8081 \
  --binary ../toadstool/target/release/toadstool \
  --service toadstool-gpu-compute \
  --env TOADSTOOL_HOST=192.168.1.134 \
  --env TOADSTOOL_PORT=9002 \
  --env TOADSTOOL_GPU_ENABLED=true

# Tower B: Verify deployment
curl http://192.168.1.134:8081/api/deployment/list | jq
curl http://192.168.1.134:9002/health
```

Ready to make history! 🚀🐸

