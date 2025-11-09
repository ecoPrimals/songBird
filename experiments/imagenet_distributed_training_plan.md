# ImageNet-100 Distributed Training Experiment

**Goal:** Train ResNet-50 on ImageNet-100 using distributed data-parallel training across 3 basement towers, demonstrating real distributed ML at scale.

**Status:** 🟡 Planning → Implementation  
**Start Date:** 2025-11-09  
**Target Completion:** 3-4 weeks

---

## 🎯 Objectives

### Primary
- **Demonstrate distributed GPU training** across heterogeneous hardware
- **Show data sharding** (no duplication) with network-based gradient sync
- **Measure speedup** from single-GPU to multi-GPU training
- **Prove production ML capability** of basement HPC to Prof. Murillo & MSDS students

### Secondary
- Establish patterns for future distributed ML projects
- Benchmark network communication overhead
- Create reusable distributed training framework
- Generate impressive visualizations for demo

---

## 📊 Dataset: ImageNet-100

### Specifications
```
Dataset: ImageNet-100 (subset of ILSVRC2012)
Classes: 100 (selected from 1000)
Training Images: ~130,000 images
Validation Images: ~5,000 images
Total Size: ~15GB
Format: JPEG images, varying sizes
Target Size: 224x224 (after preprocessing)
```

### Data Distribution Strategy
```
Tower A (Eastgate - RTX 4070 12GB):
  - Classes: 0-33 (34 classes)
  - Images: ~44,000
  - Storage: ~5GB local

Tower B (Strandgate - RTX 3070 8GB):
  - Classes: 34-66 (33 classes)
  - Images: ~43,000
  - Storage: ~5GB local

Tower C (Southgate - RTX 3090 24GB):
  - Classes: 67-99 (33 classes)
  - Images: ~43,000
  - Storage: ~5GB local

Validation Set:
  - All classes
  - Replicated on each tower (small - 600MB)
  - Enables independent validation
```

### Rationale
- **No data duplication:** Each tower stores only its shard
- **Balanced distribution:** Similar image counts per tower
- **Class-based sharding:** Simplifies data loading
- **Local storage:** Fast I/O during training
- **Future-proof:** Ready for Nestgate data orchestration

---

## 🏗️ Architecture

### Distributed Training Framework
```
Framework: PyTorch DistributedDataParallel (DDP)
Backend: NCCL (NVIDIA Collective Communications Library)
Communication: All-Reduce for gradient synchronization
Process Group: 3 workers (1 per tower)
```

### Network Topology
```
Master Node: Tower A (Eastgate) - 192.168.1.144
Worker 1: Tower B (Strandgate) - 192.168.1.134  
Worker 2: Tower C (Southgate) - 192.168.1.207

Communication Pattern:
┌─────────────┐
│   Tower A   │ ← Master (rank 0)
│  (Eastgate) │
└──────┬──────┘
       │
   ┌───┴────┐
   │        │
┌──▼───┐ ┌─▼────┐
│Tower │ │Tower │
│  B   │ │  C   │
└──────┘ └──────┘
rank 1    rank 2

All-Reduce Ring:
A ↔ B ↔ C ↔ A
```

### Model Architecture
```
Model: ResNet-50
Parameters: 25.6 million
Output Classes: 100
Input Size: 224x224x3
Memory per GPU: ~4GB (batch size 32)
FLOPs: ~4.1 GFLOPs per image
```

### Training Configuration
```
Optimizer: SGD with momentum
  - Learning Rate: 0.1
  - Momentum: 0.9
  - Weight Decay: 1e-4

LR Schedule: Step decay
  - Decay by 0.1 at epochs 30, 60, 80

Batch Size: 32 per GPU
  - Effective batch: 96 (32 × 3 GPUs)

Epochs: 90
Data Augmentation:
  - RandomResizedCrop(224)
  - RandomHorizontalFlip()
  - Normalize (ImageNet stats)
```

---

## 📈 Expected Performance

### Training Time Estimates
```
Single GPU (RTX 4070):
  - Time per epoch: ~32 minutes
  - Total training: ~48 hours
  - Images/sec: ~68

3 GPUs Distributed:
  - Time per epoch: ~12 minutes (with sync overhead)
  - Total training: ~18 hours
  - Images/sec: ~180
  - Speedup: 2.67x (accounting for communication)

Theoretical 3x speedup reduced by:
  - Gradient synchronization: ~10-15%
  - Network latency: ~5-10%
  - Load imbalance: ~5%
```

### Target Metrics
```
Top-1 Accuracy: 75-77% (published baseline: 76.1%)
Top-5 Accuracy: 92-94% (published baseline: 92.9%)
Final Loss: <1.0
Convergence: By epoch 80-90
```

### Network Communication
```
Gradient Size: ~100MB per sync
Sync Frequency: Every mini-batch
Bandwidth Required: ~800 Mbps sustained
Total Data Transfer: ~15-20 GB over 90 epochs
```

---

## 🔧 Implementation Phases

### Phase 1: Infrastructure Setup (Days 1-3)
**Objective:** Prepare environment and data

**Tasks:**
- [ ] Test network bandwidth between towers (iperf3)
- [ ] Create data directories on each tower
- [ ] Download ImageNet-100 dataset
- [ ] Shard dataset by class ranges
- [ ] Verify NCCL/PyTorch distributed capabilities
- [ ] Set up shared checkpoint directory

**Deliverables:**
- Network benchmark results
- Data shards in place
- Environment verification script

**Success Criteria:**
- Inter-tower bandwidth >500 Mbps
- Data successfully sharded
- PyTorch can see all GPUs

---

### Phase 2: Single-Tower Baseline (Days 4-6)
**Objective:** Establish baseline performance

**Tasks:**
- [ ] Implement ResNet-50 training script
- [ ] Train on single GPU (Tower C - most powerful)
- [ ] Validate data loading pipeline
- [ ] Measure single-GPU throughput
- [ ] Track accuracy/loss curves
- [ ] Save baseline model checkpoints

**Deliverables:**
- Working training script
- Baseline metrics (1-2 epochs sufficient)
- Performance benchmarks

**Success Criteria:**
- Training runs without errors
- Achieves expected images/sec
- Loss decreases properly

---

### Phase 3: Distributed Training (Days 7-14)
**Objective:** Implement and test multi-tower training

**Tasks:**
- [ ] Implement PyTorch DDP wrapper
- [ ] Create distributed data loader
- [ ] Set up gradient synchronization
- [ ] Test 2-tower training first
- [ ] Scale to 3 towers
- [ ] Implement distributed validation
- [ ] Add checkpointing and recovery
- [ ] Monitor network utilization
- [ ] Full 90-epoch training run

**Deliverables:**
- Distributed training script
- Training logs from all towers
- Synchronized model checkpoints
- Performance metrics

**Success Criteria:**
- All 3 towers training successfully
- Gradients synchronized correctly
- 2-3x speedup vs single GPU
- Model converges to target accuracy

---

### Phase 4: Monitoring & Visualization (Days 15-18)
**Objective:** Create real-time monitoring and demo materials

**Tasks:**
- [ ] Build real-time training dashboard
- [ ] Visualize per-tower metrics
- [ ] Show gradient sync timing
- [ ] Create training curves (loss, accuracy)
- [ ] Generate speedup comparison charts
- [ ] Network utilization graphs
- [ ] GPU utilization timeline
- [ ] Create demo video/screenshots

**Deliverables:**
- Web dashboard (or Python viz)
- Training visualization notebook
- Comparison charts
- Demo materials

**Success Criteria:**
- Can monitor all towers simultaneously
- Clear visualization of distributed nature
- Professional-quality demo materials

---

### Phase 5: Documentation & Demo (Days 19-21)
**Objective:** Package for Prof. Murillo presentation

**Tasks:**
- [ ] Write comprehensive documentation
- [ ] Create "how to run" guide
- [ ] Prepare presentation slides
- [ ] Generate final results summary
- [ ] Create reusable distributed training template
- [ ] Document lessons learned
- [ ] Plan scaling to full ImageNet

**Deliverables:**
- Final report
- Presentation materials
- Reusable codebase
- Student onboarding guide

**Success Criteria:**
- Prof. Murillo can understand value
- Students can replicate experiment
- Code is production-ready

---

## 📁 File Structure

```
experiments/
├── imagenet_distributed_training_plan.md  (this file)
├── imagenet_training/
│   ├── README.md
│   ├── setup/
│   │   ├── download_imagenet100.sh
│   │   ├── shard_dataset.py
│   │   ├── verify_environment.py
│   │   └── test_network.sh
│   ├── training/
│   │   ├── train_single.py
│   │   ├── train_distributed.py
│   │   ├── model.py
│   │   ├── data_loader.py
│   │   └── utils.py
│   ├── configs/
│   │   ├── single_gpu.yaml
│   │   └── distributed_3gpu.yaml
│   ├── monitoring/
│   │   ├── dashboard.py
│   │   ├── collect_metrics.py
│   │   └── visualize.py
│   └── results/
│       ├── baseline/
│       ├── distributed/
│       └── analysis.ipynb
└── data/  (local on each tower)
    └── imagenet100/
        ├── train_shard_0/  (Tower A)
        ├── train_shard_1/  (Tower B)
        ├── train_shard_2/  (Tower C)
        └── val/            (all towers)
```

---

## 🔬 Experiments to Run

### Experiment 1: Baseline
```
Config: Single GPU (Tower C - RTX 3090)
Duration: 2-3 epochs
Purpose: Establish baseline performance
Metrics: images/sec, GPU util, accuracy
```

### Experiment 2: 2-Tower Scaling
```
Config: Tower A + Tower C
Duration: 5 epochs
Purpose: Test basic distribution
Metrics: Speedup, sync overhead, accuracy
```

### Experiment 3: 3-Tower Full Run
```
Config: All 3 towers
Duration: 90 epochs (full training)
Purpose: Final demonstration
Metrics: All metrics, final accuracy
```

### Experiment 4: Ablations (Optional)
```
- Different batch sizes
- Different learning rates
- Gradient accumulation
- Mixed precision training
```

---

## 📊 Metrics to Track

### Performance Metrics
- **Throughput:** Images/second (per tower and total)
- **GPU Utilization:** % active (per tower)
- **Memory Usage:** GB VRAM used
- **Training Time:** Seconds per epoch
- **Speedup Factor:** Distributed vs single-GPU

### Communication Metrics
- **Gradient Sync Time:** ms per mini-batch
- **Network Bandwidth:** Mbps utilized
- **Communication Overhead:** % of total time
- **Data Transfer:** GB over network

### Model Metrics
- **Training Loss:** Cross-entropy
- **Training Accuracy:** Top-1, Top-5
- **Validation Loss:** Every epoch
- **Validation Accuracy:** Top-1, Top-5
- **Learning Rate:** Current value

### System Metrics
- **CPU Usage:** % per tower
- **Disk I/O:** MB/s reads
- **Network Latency:** ms round-trip
- **Temperature:** GPU temp

---

## 🎯 Success Criteria

### Must Have
- ✅ Training runs successfully on all 3 towers
- ✅ Model converges to published accuracy (±2%)
- ✅ Achieves 2x+ speedup vs single GPU
- ✅ Data successfully sharded (no duplication)
- ✅ Gradient synchronization works correctly

### Should Have
- ✅ Real-time monitoring dashboard
- ✅ Professional visualization of results
- ✅ Comprehensive documentation
- ✅ Reusable training framework
- ✅ Network efficiency >80%

### Nice to Have
- ✅ Mixed precision training (FP16)
- ✅ Gradient accumulation support
- ✅ Automatic fault recovery
- ✅ Live demo for Prof. Murillo
- ✅ Scaling to 4 GPUs

---

## 🚧 Known Challenges

### Challenge 1: Network Bottleneck
**Risk:** Gradient sync slower than compute  
**Mitigation:** 
- Use NCCL optimized all-reduce
- Benchmark network first
- Consider gradient compression

### Challenge 2: Data Loading
**Risk:** I/O bottleneck on data loading  
**Mitigation:**
- Use multiple data loader workers
- Pre-cache frequently used samples
- Optimize JPEG decoding

### Challenge 3: Heterogeneous GPUs
**Risk:** Slowest GPU determines speed  
**Mitigation:**
- Adjust batch sizes per GPU
- Use async gradient updates (if needed)
- Monitor individual GPU performance

### Challenge 4: Synchronization Bugs
**Risk:** Deadlocks or gradient mismatch  
**Mitigation:**
- Thorough testing with 2 GPUs first
- Add distributed debugging
- Use PyTorch DDP (well-tested)

### Challenge 5: Storage Management
**Risk:** Running out of disk space  
**Mitigation:**
- Verify 5GB free per tower before starting
- Clean old checkpoints periodically
- Monitor disk usage

---

## 💰 Resource Requirements

### Compute
- **GPU Hours:** ~54 hours (3 GPUs × 18 hours)
- **Electricity:** ~1.5 kWh × 18 hours = 27 kWh
- **Cost Equivalent:** ~$50-100 on cloud

### Storage
- **Per Tower:** 5GB (data) + 500MB (checkpoints)
- **Total:** ~16.5GB across all towers
- **Shared:** 1-2GB for validation data

### Network
- **Bandwidth:** 500+ Mbps sustained
- **Total Transfer:** 15-20 GB
- **Latency:** <5ms between towers

---

## 📚 References

### ImageNet-100
- Paper: "ImageNet Large Scale Visual Recognition Challenge"
- Original: https://www.image-net.org/
- Subset: Community-created 100-class subset

### ResNet-50
- Paper: "Deep Residual Learning for Image Recognition" (He et al., 2015)
- Baseline Accuracy: 76.1% top-1 on full ImageNet
- Expected on ImageNet-100: 75-77%

### PyTorch DDP
- Docs: https://pytorch.org/tutorials/intermediate/ddp_tutorial.html
- NCCL: https://github.com/NVIDIA/nccl
- Best Practices: https://pytorch.org/tutorials/intermediate/dist_tuto.html

---

## 🎓 Educational Value

### For MSDS Students
This experiment demonstrates:

1. **Distributed ML:** Real multi-node training
2. **Data Sharding:** Handling datasets larger than single-machine RAM
3. **GPU Programming:** Parallel computing on real hardware
4. **Network Optimization:** Communication-efficient algorithms
5. **Production ML:** Checkpointing, monitoring, fault tolerance
6. **Cost Analysis:** Local vs cloud computing economics
7. **System Design:** Building scalable ML infrastructure

### Learning Outcomes
- Understand distributed gradient descent
- Experience with PyTorch DDP
- Network performance considerations
- GPU memory management
- Data pipeline optimization
- Production ML workflows

---

## 📞 Status & Updates

**Current Phase:** Planning Complete → Starting Phase 1  
**Next Milestone:** Network test and data download  
**Blockers:** None  
**ETA:** 3 weeks to full training completion

**Last Updated:** 2025-11-09  
**Owner:** eastgate  
**Reviewers:** Prof. Murillo (future)

---

## ✅ Next Actions

**Immediate (Today):**
1. Test network bandwidth between towers
2. Create data directories
3. Download ImageNet-100 dataset
4. Verify PyTorch distributed setup

**This Week:**
1. Shard dataset across towers
2. Implement baseline training script
3. Run single-GPU baseline
4. Begin distributed implementation

**Next Week:**
1. Test 2-tower training
2. Scale to 3 towers
3. Start full training run
4. Build monitoring dashboard

---

**Let's build something impressive! 🚀**

