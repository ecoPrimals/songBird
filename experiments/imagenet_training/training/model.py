"""
ResNet-50 model for Tiny ImageNet (200 classes, 64x64 images)
"""

import torch
import torch.nn as nn
from torchvision import models

def create_resnet50(num_classes=200, pretrained=False):
    """
    Create ResNet-50 model adapted for Tiny ImageNet
    
    Args:
        num_classes: Number of output classes (default: 200 for Tiny ImageNet)
        pretrained: Use ImageNet pretrained weights (default: False)
    
    Returns:
        ResNet-50 model
    """
    # Load ResNet-50
    model = models.resnet50(pretrained=pretrained)
    
    # Modify first conv layer for 64x64 input (Tiny ImageNet)
    # Standard ImageNet uses 7x7 kernel, stride 2
    # For 64x64, use 3x3 kernel, stride 1 to preserve spatial info
    model.conv1 = nn.Conv2d(3, 64, kernel_size=3, stride=1, padding=1, bias=False)
    
    # Remove maxpool to preserve spatial dimensions
    model.maxpool = nn.Identity()
    
    # Modify final FC layer for num_classes
    model.fc = nn.Linear(model.fc.in_features, num_classes)
    
    return model

def count_parameters(model):
    """Count trainable parameters"""
    return sum(p.numel() for p in model.parameters() if p.requires_grad)

if __name__ == "__main__":
    # Test model creation
    model = create_resnet50(num_classes=200)
    print(f"ResNet-50 for Tiny ImageNet")
    print(f"Parameters: {count_parameters(model):,}")
    print(f"Input size: (batch, 3, 64, 64)")
    print(f"Output size: (batch, 200)")
    
    # Test forward pass
    x = torch.randn(4, 3, 64, 64)
    y = model(x)
    print(f"\nTest forward pass:")
    print(f"  Input shape: {x.shape}")
    print(f"  Output shape: {y.shape}")
    print(f"✅ Model ready!")

