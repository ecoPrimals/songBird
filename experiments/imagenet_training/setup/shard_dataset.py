#!/usr/bin/env python3
"""
Shard ImageNet dataset across multiple towers for distributed training
Each tower gets a non-overlapping subset of classes
"""

import os
import shutil
import argparse
from pathlib import Path
import json

def shard_tiny_imagenet(source_dir, output_base, num_shards=3):
    """
    Shard Tiny ImageNet across multiple towers by class
    
    Args:
        source_dir: Path to extracted tiny-imagenet-200
        output_base: Base path for output shards
        num_shards: Number of shards to create (default: 3 for 3 towers)
    """
    print("=" * 70)
    print("  📊 SHARDING TINY IMAGENET")
    print("=" * 70)
    print()
    
    source_path = Path(source_dir)
    train_dir = source_path / "train"
    val_dir = source_path / "val"
    
    if not train_dir.exists():
        print(f"❌ Training directory not found: {train_dir}")
        return False
    
    # Get all class directories
    class_dirs = sorted([d for d in train_dir.iterdir() if d.is_dir()])
    total_classes = len(class_dirs)
    
    print(f"📁 Source: {source_dir}")
    print(f"📂 Output: {output_base}")
    print(f"🏷️  Total classes: {total_classes}")
    print(f"🔀 Number of shards: {num_shards}")
    print()
    
    # Calculate classes per shard
    classes_per_shard = total_classes // num_shards
    extra_classes = total_classes % num_shards
    
    # Create shards
    shard_info = []
    class_idx = 0
    
    for shard_idx in range(num_shards):
        # Calculate how many classes for this shard
        shard_size = classes_per_shard + (1 if shard_idx < extra_classes else 0)
        shard_classes = class_dirs[class_idx:class_idx + shard_size]
        
        shard_dir = Path(output_base) / f"shard_{shard_idx}"
        shard_train = shard_dir / "train"
        shard_train.mkdir(parents=True, exist_ok=True)
        
        print(f"Creating Shard {shard_idx}:")
        print(f"  Classes: {class_idx} to {class_idx + shard_size - 1}")
        print(f"  Output: {shard_dir}")
        
        # Copy classes to shard
        image_count = 0
        for class_dir in shard_classes:
            dest_class = shard_train / class_dir.name
            if not dest_class.exists():
                shutil.copytree(class_dir, dest_class)
                # Count images (in images/ subdirectory for Tiny ImageNet)
                images_dir = dest_class / "images"
                if images_dir.exists():
                    image_count += len(list(images_dir.glob("*.JPEG")))
                else:
                    # Fallback: count directly in class dir
                    image_count += len(list(dest_class.glob("*.JPEG")))
        
        # Save shard metadata
        metadata = {
            "shard_id": shard_idx,
            "class_start": class_idx,
            "class_end": class_idx + shard_size - 1,
            "num_classes": shard_size,
            "num_images": image_count,
            "class_names": [d.name for d in shard_classes]
        }
        
        with open(shard_dir / "metadata.json", 'w') as f:
            json.dump(metadata, f, indent=2)
        
        print(f"  ✅ {shard_size} classes, {image_count} images")
        print()
        
        shard_info.append(metadata)
        class_idx += shard_size
    
    # Handle validation set - replicate to all shards for now
    print("📋 Processing validation set...")
    if val_dir.exists():
        for shard_idx in range(num_shards):
            shard_dir = Path(output_base) / f"shard_{shard_idx}"
            shard_val = shard_dir / "val"
            if not shard_val.exists():
                shutil.copytree(val_dir, shard_val)
        print(f"  ✅ Validation set replicated to all {num_shards} shards")
    else:
        print(f"  ⚠️  Validation directory not found: {val_dir}")
    
    print()
    
    # Save overall sharding info
    sharding_info = {
        "dataset": "tiny-imagenet-200",
        "total_classes": total_classes,
        "num_shards": num_shards,
        "shards": shard_info
    }
    
    info_path = Path(output_base) / "sharding_info.json"
    with open(info_path, 'w') as f:
        json.dump(sharding_info, f, indent=2)
    
    print("=" * 70)
    print("  ✅ SHARDING COMPLETE!")
    print("=" * 70)
    print()
    
    # Summary
    total_images = sum(s["num_images"] for s in shard_info)
    print("📊 Summary:")
    for shard in shard_info:
        print(f"  Shard {shard['shard_id']}: {shard['num_classes']} classes, {shard['num_images']} images")
    print(f"  Total: {total_images} training images across {num_shards} shards")
    print()
    print(f"📄 Sharding info saved: {info_path}")
    print()
    
    return True

def main():
    parser = argparse.ArgumentParser(description="Shard ImageNet dataset for distributed training")
    parser.add_argument(
        "--source",
        type=str,
        default="/home/eastgate/Development/ecoPrimals/songbird/experiments/data/imagenet100/extracted/tiny-imagenet-200",
        help="Path to extracted dataset"
    )
    parser.add_argument(
        "--output",
        type=str,
        default="/home/eastgate/Development/ecoPrimals/songbird/experiments/data/imagenet100/sharded",
        help="Output directory for shards"
    )
    parser.add_argument(
        "--num-shards",
        type=int,
        default=3,
        help="Number of shards (default: 3 for 3 towers)"
    )
    
    args = parser.parse_args()
    
    success = shard_tiny_imagenet(args.source, args.output, args.num_shards)
    
    if success:
        print("🚀 Next steps:")
        print(f"  1. Copy shards to towers:")
        print(f"     Tower A: shard_0/")
        print(f"     Tower B: shard_1/")
        print(f"     Tower C: shard_2/")
        print()
        print(f"  2. Or test locally with all shards available")
        print()
        print(f"  3. Run baseline training:")
        print(f"     cd ../training")
        print(f"     python3 train_single.py")
        print()

if __name__ == "__main__":
    main()

