#!/usr/bin/env python3
"""
ImageNet-100 HPC Benchmark Training Script
==========================================

Demonstrates HPC-level performance on heterogeneous distributed hardware.

Features:
  • PyTorch Distributed Data Parallel (DDP)
  • Mixed precision training (FP16)
  • Proper data sharding
  • Comprehensive metrics
  • Production-grade checkpointing

Hardware:
  • Tower A (Eastgate):   RTX 2070 Super (8GB) - Coordinator
  • Tower B (Strandgate): RTX 3070 (8GB) + Dual CPU - Heavy worker
  • Tower C (Southgate):  RTX 3090 (24GB) - Heaviest worker

Usage:
  python train_imagenet100_hpc_benchmark.py --rank <rank> --world-size <size>
"""

import argparse
import os
import time
from datetime import datetime

import torch
import torch.nn as nn
import torch.optim as optim
import torch.distributed as dist
from torch.nn.parallel import DistributedDataParallel as DDP
from torch.utils.data import DataLoader, DistributedSampler
from torch.cuda.amp import autocast, GradScaler
import torchvision
import torchvision.transforms as transforms
from torchvision.models import resnet50

# ============================================================================
# Configuration
# ============================================================================

class HPC_BenchmarkConfig:
    """HPC Benchmark Configuration"""
    
    # Model
    model_name = "ResNet-50"
    num_classes = 100
    
    # Training
    epochs = 90  # Full HPC benchmark
    batch_size_per_gpu = {
        0: 32,   # RTX 2070 Super (8GB) - Coordinator
        1: 64,   # RTX 3070 (8GB) - Heavy worker
        2: 96,   # RTX 3090 (24GB) - Heaviest worker
    }
    
    # Optimization
    lr = 0.1
    momentum = 0.9
    weight_decay = 1e-4
    lr_milestones = [30, 60, 80]
    lr_gamma = 0.1
    
    # Mixed Precision
    use_amp = True
    
    # Data
    data_root = "/home/eastgate/Development/ecoPrimals/songbird/experiments/data/imagenet100"
    num_workers = 4
    
    # Checkpointing
    checkpoint_freq = 5  # Save every 5 epochs
    
    # Logging
    log_freq = 10  # Log every 10 batches
    
# ============================================================================
# Distributed Setup
# ============================================================================

def setup_distributed(rank, world_size):
    """Initialize distributed training"""
    os.environ['MASTER_ADDR'] = os.environ.get('MASTER_ADDR', '192.168.1.144')
    os.environ['MASTER_PORT'] = os.environ.get('MASTER_PORT', '29503')
    
    # Initialize process group with NCCL backend for GPU
    dist.init_process_group(
        backend='nccl',
        init_method='env://',
        world_size=world_size,
        rank=rank
    )
    
    # Set device
    torch.cuda.set_device(rank % torch.cuda.device_count())
    
    if rank == 0:
        print(f"✅ Distributed setup complete")
        print(f"   Master: {os.environ['MASTER_ADDR']}:{os.environ['MASTER_PORT']}")
        print(f"   World Size: {world_size}")
        print(f"   Backend: NCCL")

def cleanup_distributed():
    """Cleanup distributed training"""
    dist.destroy_process_group()

# ============================================================================
# Data Loading
# ============================================================================

def get_data_transforms():
    """Get data augmentation transforms"""
    train_transform = transforms.Compose([
        transforms.RandomResizedCrop(224),
        transforms.RandomHorizontalFlip(),
        transforms.ColorJitter(brightness=0.4, contrast=0.4, saturation=0.4),
        transforms.ToTensor(),
        transforms.Normalize(mean=[0.485, 0.456, 0.406],
                           std=[0.229, 0.224, 0.225])
    ])
    
    val_transform = transforms.Compose([
        transforms.Resize(256),
        transforms.CenterCrop(224),
        transforms.ToTensor(),
        transforms.Normalize(mean=[0.485, 0.456, 0.406],
                           std=[0.229, 0.224, 0.225])
    ])
    
    return train_transform, val_transform

def create_dataloaders(rank, world_size, batch_size, num_workers=4):
    """Create distributed data loaders"""
    config = HPC_BenchmarkConfig()
    train_transform, val_transform = get_data_transforms()
    
    # Load datasets
    train_dataset = torchvision.datasets.ImageFolder(
        root=f"{config.data_root}/train",
        transform=train_transform
    )
    
    val_dataset = torchvision.datasets.ImageFolder(
        root=f"{config.data_root}/val",
        transform=val_transform
    )
    
    # Create distributed samplers
    train_sampler = DistributedSampler(
        train_dataset,
        num_replicas=world_size,
        rank=rank,
        shuffle=True
    )
    
    val_sampler = DistributedSampler(
        val_dataset,
        num_replicas=world_size,
        rank=rank,
        shuffle=False
    )
    
    # Create data loaders
    train_loader = DataLoader(
        train_dataset,
        batch_size=batch_size,
        sampler=train_sampler,
        num_workers=num_workers,
        pin_memory=True,
        persistent_workers=True if num_workers > 0 else False
    )
    
    val_loader = DataLoader(
        val_dataset,
        batch_size=batch_size,
        sampler=val_sampler,
        num_workers=num_workers,
        pin_memory=True,
        persistent_workers=True if num_workers > 0 else False
    )
    
    return train_loader, val_loader, train_sampler

# ============================================================================
# Model
# ============================================================================

def create_model(num_classes=100):
    """Create ResNet-50 model"""
    model = resnet50(weights=None)
    # Modify final layer for 100 classes
    model.fc = nn.Linear(model.fc.in_features, num_classes)
    return model

# ============================================================================
# Training
# ============================================================================

class MetricsTracker:
    """Track training metrics"""
    
    def __init__(self, rank):
        self.rank = rank
        self.reset()
    
    def reset(self):
        self.total_loss = 0.0
        self.total_correct = 0
        self.total_samples = 0
        self.batch_times = []
        self.data_times = []
    
    def update(self, loss, correct, samples, batch_time, data_time):
        self.total_loss += loss
        self.total_correct += correct
        self.total_samples += samples
        self.batch_times.append(batch_time)
        self.data_times.append(data_time)
    
    def get_avg_loss(self):
        return self.total_loss / len(self.batch_times) if self.batch_times else 0.0
    
    def get_accuracy(self):
        return 100.0 * self.total_correct / self.total_samples if self.total_samples > 0 else 0.0
    
    def get_throughput(self):
        """Images per second"""
        total_time = sum(self.batch_times)
        return self.total_samples / total_time if total_time > 0 else 0.0

def train_epoch(model, train_loader, criterion, optimizer, scaler, rank, epoch, config):
    """Train for one epoch"""
    model.train()
    metrics = MetricsTracker(rank)
    
    data_time_start = time.time()
    
    for batch_idx, (images, targets) in enumerate(train_loader):
        data_time = time.time() - data_time_start
        batch_time_start = time.time()
        
        images = images.cuda(non_blocking=True)
        targets = targets.cuda(non_blocking=True)
        
        optimizer.zero_grad()
        
        # Mixed precision training
        if config.use_amp:
            with autocast():
                outputs = model(images)
                loss = criterion(outputs, targets)
            scaler.scale(loss).backward()
            scaler.step(optimizer)
            scaler.update()
        else:
            outputs = model(images)
            loss = criterion(outputs, targets)
            loss.backward()
            optimizer.step()
        
        # Calculate accuracy
        _, predicted = outputs.max(1)
        correct = predicted.eq(targets).sum().item()
        
        batch_time = time.time() - batch_time_start
        metrics.update(loss.item(), correct, images.size(0), batch_time, data_time)
        
        # Log progress
        if batch_idx % config.log_freq == 0 and rank == 0:
            throughput = metrics.get_throughput()
            print(f"Epoch [{epoch}/{config.epochs}] "
                  f"Batch [{batch_idx}/{len(train_loader)}] "
                  f"Loss: {loss.item():.4f} "
                  f"Acc: {metrics.get_accuracy():.2f}% "
                  f"Throughput: {throughput:.1f} img/s "
                  f"GPU: {torch.cuda.memory_allocated() / 1e9:.2f}GB")
        
        data_time_start = time.time()
    
    return metrics

@torch.no_grad()
def validate(model, val_loader, criterion, rank, config):
    """Validate the model"""
    model.eval()
    metrics = MetricsTracker(rank)
    
    for images, targets in val_loader:
        images = images.cuda(non_blocking=True)
        targets = targets.cuda(non_blocking=True)
        
        outputs = model(images)
        loss = criterion(outputs, targets)
        
        _, predicted = outputs.max(1)
        correct = predicted.eq(targets).sum().item()
        
        metrics.update(loss.item(), correct, images.size(0), 0.0, 0.0)
    
    # Aggregate metrics across all ranks
    total_loss = torch.tensor(metrics.total_loss).cuda()
    total_correct = torch.tensor(metrics.total_correct).cuda()
    total_samples = torch.tensor(metrics.total_samples).cuda()
    
    dist.all_reduce(total_loss, op=dist.ReduceOp.SUM)
    dist.all_reduce(total_correct, op=dist.ReduceOp.SUM)
    dist.all_reduce(total_samples, op=dist.ReduceOp.SUM)
    
    avg_loss = total_loss.item() / total_samples.item()
    accuracy = 100.0 * total_correct.item() / total_samples.item()
    
    return avg_loss, accuracy

def save_checkpoint(model, optimizer, epoch, rank, output_dir):
    """Save model checkpoint"""
    if rank == 0:
        os.makedirs(output_dir, exist_ok=True)
        checkpoint = {
            'epoch': epoch,
            'model_state_dict': model.module.state_dict(),
            'optimizer_state_dict': optimizer.state_dict(),
        }
        path = f"{output_dir}/checkpoint_epoch_{epoch}.pth"
        torch.save(checkpoint, path)
        print(f"✅ Saved checkpoint: {path}")

# ============================================================================
# Main Training Loop
# ============================================================================

def main():
    parser = argparse.ArgumentParser(description='ImageNet-100 HPC Benchmark')
    parser.add_argument('--rank', type=int, required=True, help='Rank of this process')
    parser.add_argument('--world-size', type=int, required=True, help='Total number of processes')
    parser.add_argument('--epochs', type=int, default=90, help='Number of epochs')
    parser.add_argument('--output-dir', type=str, default='/tmp/imagenet100_hpc_benchmark',
                       help='Output directory for checkpoints and logs')
    args = parser.parse_args()
    
    config = HPC_BenchmarkConfig()
    config.epochs = args.epochs
    
    # Setup distributed
    setup_distributed(args.rank, args.world_size)
    rank = args.rank
    world_size = args.world_size
    
    # Get batch size for this rank
    batch_size = config.batch_size_per_gpu.get(rank, 32)
    
    if rank == 0:
        print("━" * 80)
        print("🚀 ImageNet-100 HPC Benchmark Training")
        print("━" * 80)
        print(f"Model: {config.model_name}")
        print(f"Classes: {config.num_classes}")
        print(f"Epochs: {config.epochs}")
        print(f"Batch sizes: {config.batch_size_per_gpu}")
        print(f"Mixed Precision: {config.use_amp}")
        print(f"Output: {args.output_dir}")
        print("━" * 80)
    
    # Create model
    model = create_model(config.num_classes).cuda()
    model = DDP(model, device_ids=[rank % torch.cuda.device_count()])
    
    # Loss and optimizer
    criterion = nn.CrossEntropyLoss().cuda()
    optimizer = optim.SGD(
        model.parameters(),
        lr=config.lr,
        momentum=config.momentum,
        weight_decay=config.weight_decay
    )
    scheduler = optim.lr_scheduler.MultiStepLR(
        optimizer,
        milestones=config.lr_milestones,
        gamma=config.lr_gamma
    )
    
    # Mixed precision scaler
    scaler = GradScaler() if config.use_amp else None
    
    # Data loaders
    train_loader, val_loader, train_sampler = create_dataloaders(
        rank, world_size, batch_size, config.num_workers
    )
    
    if rank == 0:
        print(f"✅ Training samples: {len(train_loader.dataset)}")
        print(f"✅ Validation samples: {len(val_loader.dataset)}")
        print(f"✅ Batches per epoch: {len(train_loader)}")
    
    # Training loop
    start_time = time.time()
    best_acc = 0.0
    
    for epoch in range(1, config.epochs + 1):
        train_sampler.set_epoch(epoch)
        
        # Train
        train_metrics = train_epoch(
            model, train_loader, criterion, optimizer, scaler,
            rank, epoch, config
        )
        
        # Validate
        val_loss, val_acc = validate(model, val_loader, criterion, rank, config)
        
        # Step scheduler
        scheduler.step()
        
        # Log epoch summary
        if rank == 0:
            elapsed = time.time() - start_time
            print("━" * 80)
            print(f"Epoch {epoch}/{config.epochs} Summary:")
            print(f"  Train Loss: {train_metrics.get_avg_loss():.4f}")
            print(f"  Train Acc:  {train_metrics.get_accuracy():.2f}%")
            print(f"  Val Loss:   {val_loss:.4f}")
            print(f"  Val Acc:    {val_acc:.2f}%")
            print(f"  Throughput: {train_metrics.get_throughput():.1f} img/s")
            print(f"  Time: {elapsed:.1f}s")
            print("━" * 80)
        
        # Save checkpoint
        if epoch % config.checkpoint_freq == 0:
            save_checkpoint(model, optimizer, epoch, rank, args.output_dir)
        
        # Save best model
        if val_acc > best_acc and rank == 0:
            best_acc = val_acc
            checkpoint = {
                'epoch': epoch,
                'model_state_dict': model.module.state_dict(),
                'optimizer_state_dict': optimizer.state_dict(),
                'accuracy': best_acc,
            }
            torch.save(checkpoint, f"{args.output_dir}/best_model.pth")
            print(f"🏆 New best accuracy: {best_acc:.2f}%")
    
    # Final summary
    if rank == 0:
        total_time = time.time() - start_time
        print("━" * 80)
        print("🎊 Training Complete!")
        print("━" * 80)
        print(f"Total Time: {total_time / 3600:.2f} hours")
        print(f"Best Validation Accuracy: {best_acc:.2f}%")
        print(f"Average Throughput: {train_metrics.get_throughput():.1f} img/s")
        print("━" * 80)
    
    cleanup_distributed()

if __name__ == '__main__':
    main()

