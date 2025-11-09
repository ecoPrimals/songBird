"""
Data loader for Tiny ImageNet with sharded support
"""

import torch
from torch.utils.data import DataLoader, Dataset
from torchvision import transforms
from pathlib import Path
from PIL import Image

class TinyImageNetDataset(Dataset):
    """
    Tiny ImageNet dataset loader
    Handles the images/ subdirectory structure
    """
    
    def __init__(self, root_dir, split='train', transform=None):
        """
        Args:
            root_dir: Path to shard directory (e.g., shard_0/)
            split: 'train' or 'val'
            transform: torchvision transforms
        """
        self.root = Path(root_dir) / split
        self.transform = transform
        self.samples = []
        self.class_to_idx = {}
        
        # Build dataset
        if split == 'train':
            self._build_train_dataset()
        else:
            self._build_val_dataset()
    
    def _build_train_dataset(self):
        """Build training dataset from class/images/ structure"""
        class_dirs = sorted([d for d in self.root.iterdir() if d.is_dir()])
        
        for idx, class_dir in enumerate(class_dirs):
            class_name = class_dir.name
            self.class_to_idx[class_name] = idx
            
            # Images are in class_dir/images/
            images_dir = class_dir / "images"
            if images_dir.exists():
                for img_path in images_dir.glob("*.JPEG"):
                    self.samples.append((img_path, idx))
    
    def _build_val_dataset(self):
        """Build validation dataset"""
        # Validation has different structure: val/images/*.JPEG
        # with val_annotations.txt mapping filenames to classes
        images_dir = self.root / "images"
        annotations_file = self.root / "val_annotations.txt"
        
        # Parse annotations
        filename_to_class = {}
        if annotations_file.exists():
            with open(annotations_file, 'r') as f:
                for line in f:
                    parts = line.strip().split('\t')
                    if len(parts) >= 2:
                        filename = parts[0]
                        class_name = parts[1]
                        filename_to_class[filename] = class_name
        
        # Build class to idx mapping
        unique_classes = sorted(set(filename_to_class.values()))
        self.class_to_idx = {cls: idx for idx, cls in enumerate(unique_classes)}
        
        # Build samples
        if images_dir.exists():
            for img_path in images_dir.glob("*.JPEG"):
                filename = img_path.name
                if filename in filename_to_class:
                    class_name = filename_to_class[filename]
                    class_idx = self.class_to_idx[class_name]
                    self.samples.append((img_path, class_idx))
    
    def __len__(self):
        return len(self.samples)
    
    def __getitem__(self, idx):
        img_path, label = self.samples[idx]
        
        # Load image
        image = Image.open(img_path).convert('RGB')
        
        if self.transform:
            image = self.transform(image)
        
        return image, label

def get_transforms(split='train'):
    """
    Get transforms for Tiny ImageNet (64x64 images)
    
    Args:
        split: 'train' or 'val'
    
    Returns:
        torchvision transforms
    """
    if split == 'train':
        return transforms.Compose([
            transforms.RandomHorizontalFlip(),
            transforms.RandomCrop(64, padding=8),
            transforms.ColorJitter(brightness=0.2, contrast=0.2, saturation=0.2),
            transforms.ToTensor(),
            transforms.Normalize(mean=[0.485, 0.456, 0.406],
                               std=[0.229, 0.224, 0.225])
        ])
    else:
        return transforms.Compose([
            transforms.ToTensor(),
            transforms.Normalize(mean=[0.485, 0.456, 0.406],
                               std=[0.229, 0.224, 0.225])
        ])

def create_data_loader(shard_dir, split='train', batch_size=32, 
                       num_workers=4, shuffle=True):
    """
    Create data loader for a shard
    
    Args:
        shard_dir: Path to shard (e.g., shard_0/)
        split: 'train' or 'val'
        batch_size: Batch size
        num_workers: Number of data loading workers
        shuffle: Shuffle data
    
    Returns:
        DataLoader
    """
    dataset = TinyImageNetDataset(
        root_dir=shard_dir,
        split=split,
        transform=get_transforms(split)
    )
    
    loader = DataLoader(
        dataset,
        batch_size=batch_size,
        shuffle=shuffle,
        num_workers=num_workers,
        pin_memory=True
    )
    
    return loader

if __name__ == "__main__":
    # Test data loader
    import sys
    
    shard_dir = "/home/eastgate/Development/ecoPrimals/songbird/experiments/data/imagenet100/sharded/shard_0"
    
    print("Testing data loader...")
    print(f"Shard: {shard_dir}")
    print()
    
    # Test train loader
    train_loader = create_data_loader(shard_dir, split='train', batch_size=4, num_workers=2)
    print(f"Train dataset size: {len(train_loader.dataset)}")
    
    # Get a batch
    images, labels = next(iter(train_loader))
    print(f"Batch shape: {images.shape}")
    print(f"Labels shape: {labels.shape}")
    print(f"Labels: {labels}")
    print()
    
    # Test val loader
    val_loader = create_data_loader(shard_dir, split='val', batch_size=4, num_workers=2)
    print(f"Val dataset size: {len(val_loader.dataset)}")
    
    images, labels = next(iter(val_loader))
    print(f"Batch shape: {images.shape}")
    print(f"Labels shape: {labels.shape}")
    print()
    print("✅ Data loader working!")

