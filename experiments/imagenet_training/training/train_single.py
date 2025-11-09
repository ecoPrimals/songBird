#!/usr/bin/env python3
"""
Baseline single-GPU training for Tiny ImageNet
This establishes baseline performance for comparison with distributed training
"""

import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.tensorboard import SummaryWriter
import time
import argparse
from pathlib import Path
import json

from model import create_resnet50, count_parameters
from data_loader import create_data_loader

def train_epoch(model, train_loader, criterion, optimizer, device, epoch):
    """Train for one epoch"""
    model.train()
    running_loss = 0.0
    correct = 0
    total = 0
    batch_time = 0
    data_time = 0
    
    end = time.time()
    
    for batch_idx, (images, labels) in enumerate(train_loader):
        # Measure data loading time
        data_time += time.time() - end
        
        images, labels = images.to(device), labels.to(device)
        
        # Forward pass
        outputs = model(images)
        loss = criterion(outputs, labels)
        
        # Backward pass
        optimizer.zero_grad()
        loss.backward()
        optimizer.step()
        
        # Statistics
        running_loss += loss.item()
        _, predicted = outputs.max(1)
        total += labels.size(0)
        correct += predicted.eq(labels).sum().item()
        
        # Measure batch time
        batch_time += time.time() - end
        end = time.time()
        
        # Print progress
        if batch_idx % 100 == 0:
            print(f'  Batch [{batch_idx}/{len(train_loader)}] '
                  f'Loss: {loss.item():.4f} '
                  f'Acc: {100.*correct/total:.2f}% '
                  f'Time: {batch_time/(batch_idx+1):.3f}s/batch')
    
    epoch_loss = running_loss / len(train_loader)
    epoch_acc = 100. * correct / total
    images_per_sec = total / batch_time if batch_time > 0 else 0
    
    return epoch_loss, epoch_acc, images_per_sec

def validate(model, val_loader, criterion, device):
    """Validate the model"""
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
    
    val_loss = val_loss / len(val_loader)
    top1_acc = 100. * correct_top1 / total
    top5_acc = 100. * correct_top5 / total
    
    return val_loss, top1_acc, top5_acc

def main():
    parser = argparse.ArgumentParser(description='Single-GPU Tiny ImageNet Training')
    parser.add_argument('--shard', type=str, 
                       default='/home/eastgate/Development/ecoPrimals/songbird/experiments/data/imagenet100/sharded/shard_0',
                       help='Path to data shard')
    parser.add_argument('--batch-size', type=int, default=32, help='Batch size')
    parser.add_argument('--epochs', type=int, default=2, help='Number of epochs (default: 2 for baseline)')
    parser.add_argument('--lr', type=float, default=0.1, help='Learning rate')
    parser.add_argument('--momentum', type=float, default=0.9, help='SGD momentum')
    parser.add_argument('--weight-decay', type=float, default=1e-4, help='Weight decay')
    parser.add_argument('--num-workers', type=int, default=4, help='Data loader workers')
    parser.add_argument('--output-dir', type=str, 
                       default='../results/baseline',
                       help='Output directory')
    
    args = parser.parse_args()
    
    # Setup
    device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    print("=" * 70)
    print("  🚀 SINGLE-GPU BASELINE TRAINING")
    print("=" * 70)
    print()
    print(f"Device: {device}")
    if torch.cuda.is_available():
        print(f"GPU: {torch.cuda.get_device_name(0)}")
        print(f"VRAM: {torch.cuda.get_device_properties(0).total_memory / 1e9:.1f}GB")
    print(f"Shard: {args.shard}")
    print(f"Batch size: {args.batch_size}")
    print(f"Epochs: {args.epochs}")
    print(f"Learning rate: {args.lr}")
    print()
    
    # Create model
    print("📦 Creating model...")
    model = create_resnet50(num_classes=200).to(device)
    print(f"Parameters: {count_parameters(model):,}")
    print()
    
    # Create data loaders
    print("📁 Loading data...")
    train_loader = create_data_loader(
        args.shard, 
        split='train',
        batch_size=args.batch_size,
        num_workers=args.num_workers
    )
    val_loader = create_data_loader(
        args.shard,
        split='val',
        batch_size=args.batch_size,
        num_workers=args.num_workers,
        shuffle=False
    )
    print(f"Train batches: {len(train_loader)}")
    print(f"Val batches: {len(val_loader)}")
    print()
    
    # Loss and optimizer
    criterion = nn.CrossEntropyLoss()
    optimizer = optim.SGD(model.parameters(), lr=args.lr, 
                         momentum=args.momentum, weight_decay=args.weight_decay)
    
    # TensorBoard
    writer = SummaryWriter(output_dir / 'tensorboard')
    
    # Training loop
    print("=" * 70)
    print("  🔥 TRAINING START")
    print("=" * 70)
    print()
    
    best_acc = 0.0
    results = []
    
    for epoch in range(args.epochs):
        epoch_start = time.time()
        
        print(f"Epoch [{epoch+1}/{args.epochs}]")
        
        # Train
        train_loss, train_acc, images_per_sec = train_epoch(
            model, train_loader, criterion, optimizer, device, epoch
        )
        
        # Validate
        val_loss, val_top1, val_top5 = validate(model, val_loader, criterion, device)
        
        epoch_time = time.time() - epoch_start
        
        # Log
        print(f"  Train Loss: {train_loss:.4f}, Train Acc: {train_acc:.2f}%")
        print(f"  Val Loss: {val_loss:.4f}, Val Top-1: {val_top1:.2f}%, Val Top-5: {val_top5:.2f}%")
        print(f"  Throughput: {images_per_sec:.1f} images/sec")
        print(f"  Epoch time: {epoch_time:.1f}s")
        print()
        
        # TensorBoard
        writer.add_scalar('Loss/train', train_loss, epoch)
        writer.add_scalar('Loss/val', val_loss, epoch)
        writer.add_scalar('Accuracy/train', train_acc, epoch)
        writer.add_scalar('Accuracy/val_top1', val_top1, epoch)
        writer.add_scalar('Accuracy/val_top5', val_top5, epoch)
        writer.add_scalar('Throughput/images_per_sec', images_per_sec, epoch)
        
        # Save results
        results.append({
            'epoch': epoch + 1,
            'train_loss': train_loss,
            'train_acc': train_acc,
            'val_loss': val_loss,
            'val_top1': val_top1,
            'val_top5': val_top5,
            'images_per_sec': images_per_sec,
            'epoch_time': epoch_time
        })
        
        # Save checkpoint
        if val_top1 > best_acc:
            best_acc = val_top1
            checkpoint = {
                'epoch': epoch + 1,
                'model_state_dict': model.state_dict(),
                'optimizer_state_dict': optimizer.state_dict(),
                'best_acc': best_acc,
            }
            torch.save(checkpoint, output_dir / 'best_model.pth')
            print(f"  ✅ Saved best model (Top-1: {best_acc:.2f}%)")
            print()
    
    writer.close()
    
    # Save final results
    with open(output_dir / 'results.json', 'w') as f:
        json.dump({
            'config': vars(args),
            'results': results,
            'best_acc': best_acc
        }, f, indent=2)
    
    print("=" * 70)
    print("  ✅ TRAINING COMPLETE!")
    print("=" * 70)
    print()
    print(f"Best validation Top-1 accuracy: {best_acc:.2f}%")
    print(f"Results saved to: {output_dir}")
    print()
    print("Next steps:")
    print("  1. Review results:")
    print(f"     cat {output_dir}/results.json")
    print("  2. View TensorBoard:")
    print(f"     tensorboard --logdir {output_dir}/tensorboard")
    print("  3. Run distributed training for comparison!")
    print()

if __name__ == "__main__":
    main()

