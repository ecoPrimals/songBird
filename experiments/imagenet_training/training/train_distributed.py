#!/usr/bin/env python3
"""
Distributed multi-GPU training for Tiny ImageNet using PyTorch DDP
Trains across multiple towers with data sharding (no duplication)
"""

import torch
import torch.nn as nn
import torch.optim as optim
import torch.distributed as dist
from torch.nn.parallel import DistributedDataParallel as DDP
from torch.utils.data import DataLoader
from torch.utils.tensorboard import SummaryWriter
import time
import argparse
from pathlib import Path
import json
import os

from model import create_resnet50, count_parameters
from data_loader import TinyImageNetDataset, get_transforms

def setup_distributed(rank, world_size):
    """
    Initialize distributed training
    
    Args:
        rank: Process rank (0, 1, 2 for 3 towers)
        world_size: Total number of processes
    """
    # Master address (Tower A)
    os.environ['MASTER_ADDR'] = os.environ.get('MASTER_ADDR', '192.168.1.144')
    os.environ['MASTER_PORT'] = os.environ.get('MASTER_PORT', '29500')
    
    # Initialize process group
    dist.init_process_group(
        backend='nccl',  # Use NCCL for GPU
        init_method='env://',
        world_size=world_size,
        rank=rank
    )
    
    # Set device
    torch.cuda.set_device(rank % torch.cuda.device_count())

def cleanup_distributed():
    """Clean up distributed training"""
    dist.destroy_process_group()

def create_distributed_loader(shard_dir, split='train', batch_size=32, 
                              num_workers=4, rank=0, world_size=1):
    """
    Create distributed data loader
    Each rank loads from its own shard
    
    Args:
        shard_dir: Path to this rank's shard
        split: 'train' or 'val'
        batch_size: Batch size per GPU
        num_workers: Data loader workers
        rank: Process rank
        world_size: Total processes
    
    Returns:
        DataLoader
    """
    dataset = TinyImageNetDataset(
        root_dir=shard_dir,
        split=split,
        transform=get_transforms(split)
    )
    
    # Note: Each rank already has its own shard, so we don't use DistributedSampler
    # The data is already partitioned by sharding
    loader = DataLoader(
        dataset,
        batch_size=batch_size,
        shuffle=(split == 'train'),
        num_workers=num_workers,
        pin_memory=True
    )
    
    return loader

def train_epoch(model, train_loader, criterion, optimizer, device, epoch, rank):
    """Train for one epoch with distributed synchronization"""
    model.train()
    running_loss = 0.0
    correct = 0
    total = 0
    batch_time = 0
    
    end = time.time()
    
    for batch_idx, (images, labels) in enumerate(train_loader):
        images, labels = images.to(device), labels.to(device)
        
        # Forward pass
        outputs = model(images)
        loss = criterion(outputs, labels)
        
        # Backward pass (gradients automatically synced by DDP)
        optimizer.zero_grad()
        loss.backward()
        optimizer.step()
        
        # Statistics
        running_loss += loss.item()
        _, predicted = outputs.max(1)
        total += labels.size(0)
        correct += predicted.eq(labels).sum().item()
        
        batch_time += time.time() - end
        end = time.time()
        
        # Print progress (only rank 0)
        if rank == 0 and batch_idx % 100 == 0:
            print(f'  Batch [{batch_idx}/{len(train_loader)}] '
                  f'Loss: {loss.item():.4f} '
                  f'Acc: {100.*correct/total:.2f}% '
                  f'Time: {batch_time/(batch_idx+1):.3f}s/batch')
    
    epoch_loss = running_loss / len(train_loader)
    epoch_acc = 100. * correct / total
    images_per_sec = total / batch_time if batch_time > 0 else 0
    
    # Gather metrics from all ranks
    if dist.is_initialized():
        # Convert to tensors for all_reduce
        loss_tensor = torch.tensor([epoch_loss], device=device)
        acc_tensor = torch.tensor([epoch_acc], device=device)
        throughput_tensor = torch.tensor([images_per_sec], device=device)
        
        # Average across all ranks
        dist.all_reduce(loss_tensor, op=dist.ReduceOp.SUM)
        dist.all_reduce(acc_tensor, op=dist.ReduceOp.SUM)
        dist.all_reduce(throughput_tensor, op=dist.ReduceOp.SUM)
        
        epoch_loss = loss_tensor.item() / dist.get_world_size()
        epoch_acc = acc_tensor.item() / dist.get_world_size()
        images_per_sec = throughput_tensor.item()  # Sum for total throughput
    
    return epoch_loss, epoch_acc, images_per_sec

def validate(model, val_loader, criterion, device, rank):
    """Validate the model with distributed synchronization"""
    model.eval()
    val_loss = 0.0
    correct_top1 = 0
    correct_top5 = 0
    total = 0
    
    with torch.no_grad():
        for images, labels in val_loader:
            images, labels = images.to(device), labels.to(device)
            
            outputs = model(images)
            loss = criterion(outputs, labels)
            
            val_loss += loss.item()
            
            # Top-1 accuracy
            _, pred = outputs.max(1)
            correct_top1 += pred.eq(labels).sum().item()
            
            # Top-5 accuracy
            _, pred_top5 = outputs.topk(5, 1, largest=True, sorted=True)
            correct_top5 += pred_top5.eq(labels.view(-1, 1).expand_as(pred_top5)).sum().item()
            
            total += labels.size(0)
    
    # Gather metrics from all ranks
    if dist.is_initialized():
        metrics = torch.tensor([val_loss, correct_top1, correct_top5, total], 
                              dtype=torch.float32, device=device)
        dist.all_reduce(metrics, op=dist.ReduceOp.SUM)
        val_loss, correct_top1, correct_top5, total = metrics.tolist()
    
    val_loss = val_loss / len(val_loader) if len(val_loader) > 0 else 0
    top1_acc = 100. * correct_top1 / total if total > 0 else 0
    top5_acc = 100. * correct_top5 / total if total > 0 else 0
    
    return val_loss, top1_acc, top5_acc

def main():
    parser = argparse.ArgumentParser(description='Distributed Tiny ImageNet Training')
    parser.add_argument('--rank', type=int, default=0, help='Process rank')
    parser.add_argument('--world-size', type=int, default=1, help='Total number of processes')
    parser.add_argument('--shard-base', type=str,
                       default='/home/eastgate/Development/ecoPrimals/songbird/experiments/data/imagenet100/sharded',
                       help='Base path to shards')
    parser.add_argument('--batch-size', type=int, default=64, help='Batch size per GPU')
    parser.add_argument('--epochs', type=int, default=2, help='Number of epochs')
    parser.add_argument('--lr', type=float, default=0.1, help='Learning rate')
    parser.add_argument('--momentum', type=float, default=0.9, help='SGD momentum')
    parser.add_argument('--weight-decay', type=float, default=1e-4, help='Weight decay')
    parser.add_argument('--num-workers', type=int, default=4, help='Data loader workers')
    parser.add_argument('--output-dir', type=str,
                       default='../results/distributed',
                       help='Output directory')
    
    args = parser.parse_args()
    
    # Setup distributed training
    if args.world_size > 1:
        setup_distributed(args.rank, args.world_size)
    
    rank = args.rank
    world_size = args.world_size
    is_master = (rank == 0)
    
    # Setup device
    device = torch.device(f'cuda:{rank % torch.cuda.device_count()}')
    torch.cuda.set_device(device)
    
    # Setup output directory (only master)
    if is_master:
        output_dir = Path(args.output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)
    
    if is_master:
        print("=" * 70)
        print("  🚀 DISTRIBUTED MULTI-GPU TRAINING")
        print("=" * 70)
        print()
        print(f"World size: {world_size} GPUs")
        print(f"Rank: {rank}")
        print(f"Device: {device}")
        print(f"GPU: {torch.cuda.get_device_name(device)}")
        print(f"VRAM: {torch.cuda.get_device_properties(device).total_memory / 1e9:.1f}GB")
        print(f"Batch size per GPU: {args.batch_size}")
        print(f"Effective batch size: {args.batch_size * world_size}")
        print(f"Epochs: {args.epochs}")
        print()
    
    # Determine shard for this rank
    shard_dir = Path(args.shard_base) / f"shard_{rank}"
    
    if is_master:
        print(f"📁 Loading data from shard_{rank}...")
    
    # Create model
    model = create_resnet50(num_classes=200).to(device)
    
    if is_master:
        print(f"📦 Model parameters: {count_parameters(model):,}")
    
    # Wrap with DDP
    if world_size > 1:
        model = DDP(model, device_ids=[device.index])
    
    # Create data loaders
    train_loader = create_distributed_loader(
        shard_dir,
        split='train',
        batch_size=args.batch_size,
        num_workers=args.num_workers,
        rank=rank,
        world_size=world_size
    )
    val_loader = create_distributed_loader(
        shard_dir,
        split='val',
        batch_size=args.batch_size,
        num_workers=args.num_workers,
        rank=rank,
        world_size=world_size
    )
    
    if is_master:
        print(f"Train batches (rank {rank}): {len(train_loader)}")
        print(f"Val batches (rank {rank}): {len(val_loader)}")
        print()
    
    # Loss and optimizer
    criterion = nn.CrossEntropyLoss()
    optimizer = optim.SGD(model.parameters(), lr=args.lr,
                         momentum=args.momentum, weight_decay=args.weight_decay)
    
    # TensorBoard (only master)
    writer = None
    if is_master:
        writer = SummaryWriter(output_dir / 'tensorboard')
    
    # Training loop
    if is_master:
        print("=" * 70)
        print("  🔥 DISTRIBUTED TRAINING START")
        print("=" * 70)
        print()
    
    best_acc = 0.0
    results = []
    
    for epoch in range(args.epochs):
        epoch_start = time.time()
        
        if is_master:
            print(f"Epoch [{epoch+1}/{args.epochs}]")
        
        # Train
        train_loss, train_acc, images_per_sec = train_epoch(
            model, train_loader, criterion, optimizer, device, epoch, rank
        )
        
        # Validate
        val_loss, val_top1, val_top5 = validate(model, val_loader, criterion, device, rank)
        
        epoch_time = time.time() - epoch_start
        
        # Log (only master)
        if is_master:
            print(f"  Train Loss: {train_loss:.4f}, Train Acc: {train_acc:.2f}%")
            print(f"  Val Loss: {val_loss:.4f}, Val Top-1: {val_top1:.2f}%, Val Top-5: {val_top5:.2f}%")
            print(f"  Total Throughput: {images_per_sec:.1f} images/sec")
            print(f"  Speedup vs baseline: {images_per_sec/166.7:.2f}x")
            print(f"  Epoch time: {epoch_time:.1f}s")
            print()
            
            # TensorBoard
            if writer:
                writer.add_scalar('Loss/train', train_loss, epoch)
                writer.add_scalar('Loss/val', val_loss, epoch)
                writer.add_scalar('Accuracy/train', train_acc, epoch)
                writer.add_scalar('Accuracy/val_top1', val_top1, epoch)
                writer.add_scalar('Accuracy/val_top5', val_top5, epoch)
                writer.add_scalar('Throughput/images_per_sec', images_per_sec, epoch)
                writer.add_scalar('Throughput/speedup', images_per_sec/166.7, epoch)
            
            # Save results
            results.append({
                'epoch': epoch + 1,
                'train_loss': train_loss,
                'train_acc': train_acc,
                'val_loss': val_loss,
                'val_top1': val_top1,
                'val_top5': val_top5,
                'images_per_sec': images_per_sec,
                'speedup': images_per_sec / 166.7,
                'epoch_time': epoch_time
            })
            
            # Save checkpoint
            if val_top1 > best_acc:
                best_acc = val_top1
                checkpoint = {
                    'epoch': epoch + 1,
                    'model_state_dict': model.module.state_dict() if hasattr(model, 'module') else model.state_dict(),
                    'optimizer_state_dict': optimizer.state_dict(),
                    'best_acc': best_acc,
                }
                torch.save(checkpoint, output_dir / 'best_model.pth')
                print(f"  ✅ Saved best model (Top-1: {best_acc:.2f}%)")
                print()
    
    if writer:
        writer.close()
    
    # Save final results (only master)
    if is_master:
        with open(output_dir / 'results.json', 'w') as f:
            json.dump({
                'config': vars(args),
                'world_size': world_size,
                'results': results,
                'best_acc': best_acc,
                'final_speedup': results[-1]['speedup'] if results else 0
            }, f, indent=2)
        
        print("=" * 70)
        print("  ✅ DISTRIBUTED TRAINING COMPLETE!")
        print("=" * 70)
        print()
        print(f"Best validation Top-1 accuracy: {best_acc:.2f}%")
        print(f"Final throughput: {results[-1]['images_per_sec']:.1f} images/sec")
        print(f"Speedup vs baseline: {results[-1]['speedup']:.2f}x")
        print(f"Results saved to: {output_dir}")
        print()
    
    # Cleanup
    if world_size > 1:
        cleanup_distributed()

if __name__ == "__main__":
    main()

