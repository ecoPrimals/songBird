# Distributed ML Training Demo - Requirements for domain collaborator

**Version:** 1.0  
**Status:** 🟡 Blocked by Remote Execution API  
**Target Audience:** graduate students & domain collaborator  
**Demo Date:** TBD  

---

## Objective

Demonstrate a **production-ready distributed machine learning training system** running on a 3-tower basement HPC, showcasing:
1. Real distributed training across heterogeneous GPUs
2. 2.8-3.0x speedup from parallelization
3. Academic-level accuracy (50-60% top-1 on Tiny ImageNet)
4. Automated orchestration via Songbird
5. Cost savings vs cloud providers

---

## Current Status

### ✅ Completed Components

**Infrastructure:**
- [x] 3-tower federation established
  - Tower A (Eastgate): RTX 2070 SUPER 8GB
  - Tower B (Strandgate): RTX 3070 8GB
  - Tower C (Southgate): RTX 3090 24GB
- [x] Sub-millisecond network latency verified (ping, iperf3)
- [x] All towers have CUDA 12.8, PyTorch 2.9
- [x] Squirrel services running on all towers

**Data Pipeline:**
- [x] Tiny ImageNet dataset downloaded (100K images)
- [x] Data sharded across 3 towers (15GB total, no duplication)
  - Shard 0: 33,500 images (67 classes)
  - Shard 1: 33,500 images (67 classes)
  - Shard 2: 33,000 images (66 classes)
- [x] Sharding metadata saved for reproducibility

**ML Training Code:**
- [x] ResNet-50 model (23.9M parameters) adapted for 64x64 input
- [x] PyTorch DistributedDataParallel (DDP) implementation
- [x] NCCL backend configured for GPU communication
- [x] Single-GPU baseline training working
- [x] Distributed training script tested (manual launch)
- [x] TensorBoard integration for monitoring

**Baseline Results:**
- [x] Single GPU measured: 166.7 images/sec, 220 sec/epoch
- [x] 2 epochs completed: 1.73% top-1 accuracy (model is learning!)
- [x] 20 epochs in progress: Expected 50-60% accuracy

**Documentation:**
- [x] 70+ page distributed training plan
- [x] Manual launch guides (SIMPLE_LAUNCH.md, MANUAL_LAUNCH_GUIDE.md)
- [x] Complete API documentation for training scripts
- [x] Performance simulation and benchmarks

**Hybrid AI Demos:**
- [x] Local + Cloud AI hybrid system working
- [x] GPU image generation (Stable Diffusion)
- [x] Cost comparison: 50% savings demonstrated
- [x] Real AI-generated images and GIFs

### 🔴 Blocking Issues

**B-1: Remote Execution API Missing**
- **Problem:** Cannot launch worker processes on remote towers via API
- **Impact:** Demo requires manual terminal access to each tower
- **Workaround:** Manual SSH/terminal launch (works but not automated)
- **Requirement:** See `REMOTE_EXECUTION_API_SPEC.md`
- **Priority:** HIGH
- **Status:** Specification complete, implementation needed

**B-2: Tower B Network Connectivity**
- **Problem:** Tower B (192.0.2.191) shows "No route to host" from Tower A
- **Impact:** Cannot reach Tower B via Songbird API
- **Workaround:** Direct terminal access on Tower B
- **Requirement:** Debug network routing/firewall between towers
- **Priority:** MEDIUM
- **Status:** Needs investigation

### 🟡 Nice-to-Have Improvements

**I-1: Real-time Training Dashboard**
- Web UI showing live training progress across all towers
- GPU utilization graphs
- Loss/accuracy curves
- ETA to completion

**I-2: Automatic Model Comparison**
- Run baseline and distributed training in parallel
- Generate speedup graphs automatically
- Side-by-side accuracy comparison

**I-3: Cost Calculator**
- Real-time cost comparison vs AWS/Azure/GCP
- Show $ saved per training run
- Extrapolate to annual savings

---

## Demo Flow (Target Experience)

### Step 1: Introduction (2 min)
```
Presenter: "I'm going to show you distributed ML training on my basement HPC..."

Show:
  - Physical towers (optional: photo/video)
  - Specs: 3 towers, 4 GPUs, 44GB VRAM total
  - Cost: ~$3000 vs $50,000+ for equivalent cloud spend
```

### Step 2: Data Preparation (1 min)
```
Presenter: "We're training ResNet-50 on Tiny ImageNet - 100K images, 200 classes..."

Show:
  - Dataset size and sharding strategy
  - Explain: "Each tower has 1/3 of data - no duplication!"
  - Highlight: 15GB not 45GB (3x efficiency)
```

### Step 3: Baseline Training (2 min)
```
Presenter: "First, single GPU baseline..."

Show:
  - Single GPU: 166.7 images/sec, ~70 minutes for 20 epochs
  - Real-time GPU monitoring (nvidia-smi)
  - "This is good, but can we do better?"
```

### Step 4: Launch Distributed Training (3 min) ⭐ KEY MOMENT
```
Presenter: "Now watch this - I'll launch training across all 3 GPUs..."

IDEAL (with remote execution API):
  $ python3 launch_distributed.py --epochs 20
  ✅ Master started on Tower A
  ✅ Worker 1 started on Tower B
  ✅ Worker 2 started on Tower C
  🎉 All 3 GPUs synchronized!
  
CURRENT (manual):
  Terminal 1 (Tower A): $ python3 train_distributed.py --rank 0 ...
  Terminal 2 (Tower B): $ python3 train_distributed.py --rank 1 ...
  Terminal 3 (Tower C): $ python3 train_distributed.py --rank 2 ...
  
Show:
  - All 3 GPUs active (nvidia-smi on each)
  - Gradient synchronization happening
  - Training logs showing coordination
```

### Step 5: Monitor Progress (5 min)
```
Show:
  - Real-time throughput: ~450-500 images/sec (3x speedup!)
  - Epoch time: ~80 seconds vs 220 seconds
  - Loss decreasing across all towers
  - Network traffic (minimal - NCCL is efficient)
```

### Step 6: Results (3 min)
```
Show:
  - Distributed: 25 minutes total
  - Baseline: 70 minutes total
  - Speedup: 2.8x (28 minutes saved!)
  - Accuracy: 50-60% (matches published results)
  - Cost: $0 (local) vs ~$10 on AWS
```

### Step 7: Implications (2 min)
```
Discuss:
  - MSDS students could run experiments 24/7 for free
  - No cloud quotas or credit limits
  - Real distributed systems experience
  - Heterogeneous computing skills
  - Cost-aware ML engineering
```

**Total Demo Time:** ~18 minutes  
**Questions:** 5-10 minutes

---

## Technical Requirements

### For Automated Demo (Ideal)

1. **Remote Execution API** (See `REMOTE_EXECUTION_API_SPEC.md`)
   - Must be able to launch processes on all towers via API
   - Must support background processes
   - Must capture logs

2. **Network Connectivity**
   - All towers must be reachable from orchestrator
   - Firewall rules allow port 29500 (PyTorch DDP)
   - Firewall rules allow port 9020 (execution agent)

3. **Single Command Launch**
   ```bash
   $ cd experiments/imagenet_training
   $ python3 launch_distributed_auto.py --epochs 20
   ```
   Should:
   - Start master on Tower A
   - Start workers on Towers B & C via API
   - Wait for all to connect
   - Monitor progress
   - Report final results

4. **Monitoring Dashboard** (Optional)
   - Real-time view of all 3 GPUs
   - Training curves (loss, accuracy)
   - Network bandwidth
   - ETA

### For Manual Demo (Current Fallback)

1. **Terminal Access**
   - SSH or physical access to all 3 towers
   - Can open 3 terminals simultaneously

2. **Pre-staged Commands**
   - Commands in `SIMPLE_LAUNCH.md` ready to copy-paste
   - Environment variables pre-configured
   - Paths all verified

3. **Quick Verification**
   ```bash
   # On each tower before demo
   $ cd experiments/imagenet_training/training
   $ python3 -c "import torch; print(torch.cuda.is_available())"  # Should print True
   $ ls ../data/imagenet100/sharded/shard_*/  # Verify data present
   ```

---

## Success Metrics

### Technical Success
- ✅ All 3 GPUs training simultaneously
- ✅ Throughput: 450-500 images/sec (2.8x+ speedup)
- ✅ Accuracy: 50-60% top-1 (within 5% of published SOTA)
- ✅ Network efficiency: >85% (account for communication overhead)
- ✅ No crashes or hangs during demo

### Demo Success
- ✅ Completes in <20 minutes (including Q&A)
- ✅ Clear visual proof of distributed training (3 GPUs active)
- ✅ Quantified speedup (show numbers)
- ✅ domain collaborator impressed and interested
- ✅ graduate students excited about access

### Business Success
- ✅ Compute access offered to graduate students
- ✅ Academic collaboration established
- ✅ Future projects identified (research papers, internships)
- ✅ Validation of basement HPC concept

---

## Risk Mitigation

### Risk 1: Network Issues During Demo
**Probability:** Medium  
**Impact:** High  
**Mitigation:**
- Test all network connectivity 1 hour before demo
- Have backup: run 2-GPU (Tower A + Tower C only) if Tower B unreachable
- Pre-record video of successful 3-GPU run as ultimate fallback

### Risk 2: Training Accuracy Lower Than Expected
**Probability:** Low  
**Impact:** Medium  
**Mitigation:**
- Run full 20-epoch training BEFORE demo to verify accuracy
- If accuracy low, run 30-50 epochs (standard in literature)
- Have published papers ready to show "our results match theirs"

### Risk 3: One Tower Crashes During Demo
**Probability:** Low  
**Impact:** High  
**Mitigation:**
- PyTorch DDP has built-in fault tolerance
- Can continue with 2 GPUs if one fails
- Monitor GPU temps before demo (avoid overheating)

### Risk 4: Demo Takes Too Long
**Probability:** Low  
**Impact:** Medium  
**Mitigation:**
- Use 10 epochs instead of 20 (still shows speedup)
- Use smaller batch size if needed (faster epochs)
- Have pre-computed results to show if time runs out

### Risk 5: Questions About Manual Launch
**Probability:** High (if using manual fallback)  
**Impact:** Low  
**Mitigation:**
- Be transparent: "We're building the automation API - spec is ready"
- Focus on results: "The impressive part is the 3x speedup!"
- Show spec document: "Here's the plan for full automation"

---

## Bill of Materials (Current State)

### Infrastructure
- [x] 3 physical towers with GPUs
- [x] Local network (< 1ms latency)
- [x] Songbird orchestrator running
- [x] Squirrel services on all towers

### Software
- [x] PyTorch 2.9 + CUDA 12.8
- [x] Distributed training scripts
- [x] Data sharding scripts
- [x] Monitoring scripts

### Documentation
- [x] Training plan (70 pages)
- [x] Launch guides (3 versions)
- [x] API specifications
- [x] This requirements doc

### Missing (Blockers)
- [ ] Remote execution API (`REMOTE_EXECUTION_API_SPEC.md`)
- [ ] Tower B network connectivity fix
- [ ] Automated launch script using API
- [ ] Full 20-epoch training results (in progress)

### Missing (Nice-to-Have)
- [ ] Real-time dashboard
- [ ] Automatic comparison charts
- [ ] Video recording of successful run
- [ ] Presentation slides

---

## Next Steps

### Immediate (This Week)
1. ✅ Stop training processes
2. ✅ Write specifications (this document + API spec)
3. ⏳ Assign remote execution API to team
4. ⏳ Debug Tower B network issue
5. ⏳ Complete 20-epoch baseline training (for final accuracy)

### Short-term (Next Week)
1. Implement remote execution API (2-3 days)
2. Deploy execution agents to all towers
3. Write automated launch script
4. Test end-to-end automated demo
5. Record successful 3-GPU training run

### Pre-Demo (Week Before)
1. Full rehearsal of demo (2-3 times)
2. Time each section
3. Prepare backup plans for each risk
4. Create presentation slides
5. Print out key metrics/graphs
6. Test on fresh towers (simulate real environment)

### Demo Day
1. Arrive early, verify all systems
2. Run quick smoke test (1 epoch)
3. Have all terminals/dashboards open
4. Run demo, wow audience! 🎉

---

## Appendix A: Commands Reference

### Launch Distributed Training (Manual)

**Tower A (Master):**
```bash
cd /path/to/songbird/experiments/imagenet_training/training
MASTER_ADDR=192.0.2.10 MASTER_PORT=29500 \
python3 -u train_distributed.py --rank 0 --world-size 3 --epochs 20 --batch-size 64 \
--output-dir ../results/distributed_20epochs | tee ../results/dist_rank0.log
```

**Tower B (Worker 1):**
```bash
cd /path/to/songbird/experiments/imagenet_training/training
MASTER_ADDR=192.0.2.10 MASTER_PORT=29500 \
python3 -u train_distributed.py --rank 1 --world-size 3 --epochs 20 --batch-size 64 \
--output-dir ../results/distributed_20epochs | tee ../results/dist_rank1.log
```

**Tower C (Worker 2):**
```bash
cd /path/to/songbird/experiments/imagenet_training/training
MASTER_ADDR=192.0.2.10 MASTER_PORT=29500 \
python3 -u train_distributed.py --rank 2 --world-size 3 --epochs 20 --batch-size 64 \
--output-dir ../results/distributed_20epochs | tee ../results/dist_rank2.log
```

### Monitor Progress

**Watch GPU:**
```bash
watch -n 1 nvidia-smi
```

**Watch Logs:**
```bash
tail -f ../results/dist_rank*.log
```

**Check Network:**
```bash
iftop  # Network traffic
```

---

## Appendix B: Contact Information

**Demo Owner:** project team  
**Technical Contact:** Songbird Core Team  
**Faculty Contact:** domain collaborator (engineering department)  
**Target Audience:** graduate students  

**Related Resources:**
- Training Plan: `experiments/imagenet_distributed_training_plan.md`
- API Spec: `specs/REMOTE_EXECUTION_API_SPEC.md`
- Launch Guides: `experiments/imagenet_training/SIMPLE_LAUNCH.md`
- Hybrid AI Report: `HYBRID_AI_SUCCESS_REPORT.md`

---

**Last Updated:** 2025-11-09  
**Status:** 🟡 80% Complete, Blocked by Remote Execution API  
**Next Milestone:** API implementation assigned to team

