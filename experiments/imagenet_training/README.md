# ImageNet-100 Distributed Training Experiment

**Status:** 🟢 Phase 1 - Infrastructure Setup  
**Last Updated:** 2025-11-09

## 🎯 Quick Start

```bash
# 1. Verify environment
cd experiments/imagenet_training/setup
python3 verify_environment.py

# 2. Test network
./test_network.sh

# 3. Download dataset (start with Tiny ImageNet for testing)
./download_imagenet100.sh

# 4. Shard data across towers
python3 shard_dataset.py --dataset tiny-imagenet --num-shards 3

# 5. Run baseline (single GPU)
cd ../training
python3 train_single.py --config ../configs/single_gpu.yaml

# 6. Run distributed (3 GPUs)
python3 train_distributed.py --config ../configs/distributed_3gpu.yaml
```

## 📊 Current Status

### ✅ Completed
- [x] Project structure created
- [x] Comprehensive plan documented
- [x] Network connectivity verified (< 1ms latency!)
- [x] Environment verification script created
- [x] Download script prepared

### 🔄 In Progress
- [ ] Environment verification
- [ ] Dataset download
- [ ] Data sharding implementation

### 📋 Todo
- [ ] Baseline training script
- [ ] Distributed training implementation
- [ ] Monitoring dashboard
- [ ] Full training run

## 🏗️ Architecture

```
3-Tower Distributed Training:
┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
│ Tower A (Master) │  │ Tower B (Worker) │  │ Tower C (Worker) │
│   Eastgate       │  │   Strandgate     │  │   Southgate      │
│   RTX 4070 12GB  │  │   RTX 3070 8GB   │  │   RTX 3090 24GB  │
│                  │  │                  │  │                  │
│ Classes: 0-33    │  │ Classes: 34-66   │  │ Classes: 67-99   │
│ ~5GB data        │  │ ~5GB data        │  │ ~5GB data        │
└────────┬─────────┘  └────────┬─────────┘  └────────┬─────────┘
         │                     │                     │
         └─────────────────────┴─────────────────────┘
                      Gradient Sync (NCCL)
                      ~100MB per mini-batch
```

## 📁 Directory Structure

```
imagenet_training/
├── README.md                 (this file)
├── setup/
│   ├── verify_environment.py (check PyTorch/CUDA/distributed)
│   ├── test_network.sh       (network bandwidth test)
│   ├── download_imagenet100.sh (dataset download)
│   └── shard_dataset.py      (split data across towers)
├── training/
│   ├── train_single.py       (baseline single-GPU)
│   ├── train_distributed.py  (multi-tower distributed)
│   ├── model.py              (ResNet-50 architecture)
│   ├── data_loader.py        (sharded data loading)
│   └── utils.py              (helpers)
├── configs/
│   ├── single_gpu.yaml       (baseline config)
│   └── distributed_3gpu.yaml (distributed config)
├── monitoring/
│   ├── dashboard.py          (real-time metrics)
│   ├── collect_metrics.py    (gather stats)
│   └── visualize.py          (plots and charts)
└── results/
    ├── baseline/             (single-GPU results)
    ├── distributed/          (multi-GPU results)
    └── analysis.ipynb        (Jupyter analysis)
```

## 🔧 Setup Instructions

### Prerequisites
- Python 3.8+
- PyTorch 1.10+ with CUDA
- 5GB free space per tower
- Network: <10ms latency, >500Mbps bandwidth

### Installation
```bash
# PyTorch with CUDA (if not installed)
pip3 install torch torchvision --index-url https://download.pytorch.org/whl/cu118

# Additional dependencies
pip3 install pyyaml tensorboard matplotlib pandas tqdm

# Optional: iperf3 for bandwidth testing
sudo apt install iperf3
```

## 📊 Dataset Options

### Option 1: Tiny ImageNet (Recommended for Testing)
- **Size:** 250MB
- **Classes:** 200 (can subset to 100)
- **Images:** 100K train, 10K val
- **Download time:** < 5 minutes
- **Training time:** ~2 hours (3 GPUs)
- **Best for:** Pipeline testing, quick iteration

### Option 2: ImageNet-100 (Production)
- **Size:** ~15GB
- **Classes:** 100
- **Images:** 130K train, 5K val
- **Download time:** 30-60 minutes
- **Training time:** ~18 hours (3 GPUs)
- **Best for:** Final demo, publication results

### Option 3: Full ImageNet-1K (Future)
- **Size:** ~150GB
- **Classes:** 1000
- **Images:** 1.2M train, 50K val
- **Download time:** 2-4 hours
- **Training time:** ~3 days (3 GPUs)
- **Best for:** Production benchmark

## 📈 Expected Results

### Single GPU Baseline (Tower C - RTX 3090)
```
Throughput: ~68 images/sec
Time per epoch: ~32 minutes
Total training: ~48 hours (90 epochs)
GPU Utilization: 85-95%
```

### Distributed 3-GPU
```
Throughput: ~180 images/sec
Time per epoch: ~12 minutes
Total training: ~18 hours (90 epochs)
Speedup: 2.67x
GPU Utilization: 80-90% (accounting for sync)
Network: ~100MB/sec gradient traffic
```

### Model Performance
```
Top-1 Accuracy: 75-77%
Top-5 Accuracy: 92-94%
Final Loss: < 1.0
```

## 🔍 Monitoring

### Real-time Metrics
- Training loss & accuracy (per tower)
- Validation performance
- Throughput (images/sec)
- GPU utilization & memory
- Network bandwidth usage
- Gradient sync time

### Logging
```bash
# View logs from all towers
tail -f results/distributed/tower_*.log

# TensorBoard
tensorboard --logdir results/distributed/tensorboard
```

## 🐛 Troubleshooting

### Network Issues
```bash
# Test connectivity
./setup/test_network.sh

# Test bandwidth
# On Tower B/C: iperf3 -s
# On Tower A: iperf3 -c 192.168.1.134 -t 10
```

### CUDA/GPU Issues
```bash
# Check NVIDIA driver
nvidia-smi

# Test CUDA
python3 -c "import torch; print(torch.cuda.is_available())"

# Check NCCL
python3 setup/verify_environment.py
```

### Data Loading Issues
```bash
# Verify data shards
ls -lh ../data/imagenet100/train_shard_*

# Test data loader
python3 -c "from training.data_loader import test_loader; test_loader()"
```

## 📚 References

- [Full Plan](../imagenet_distributed_training_plan.md)
- [PyTorch DDP Tutorial](https://pytorch.org/tutorials/intermediate/ddp_tutorial.html)
- [NCCL Documentation](https://docs.nvidia.com/deeplearning/nccl/)
- [ImageNet Dataset](https://www.image-net.org/)

## 📞 Status Updates

Track progress in the main plan document:
`experiments/imagenet_distributed_training_plan.md`

**Current Phase:** Infrastructure Setup  
**Next Milestone:** Data download complete  
**ETA:** 3 weeks to full training

