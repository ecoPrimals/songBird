#!/bin/bash
# Download and prepare ImageNet-100 dataset
# This is a subset of ImageNet with 100 classes

set -e

echo "========================================================================"
echo "  📥 IMAGENET-100 DOWNLOAD"
echo "========================================================================"
echo ""

# Configuration
DATA_DIR="/home/eastgate/Development/ecoPrimals/songbird/experiments/data/imagenet100"
DOWNLOAD_DIR="${DATA_DIR}/download"
EXTRACT_DIR="${DATA_DIR}/extracted"

echo "Configuration:"
echo "  Data directory: $DATA_DIR"
echo "  Download directory: $DOWNLOAD_DIR"
echo "  Extract directory: $EXTRACT_DIR"
echo ""

# Create directories
mkdir -p "$DOWNLOAD_DIR"
mkdir -p "$EXTRACT_DIR"

echo "========================================================================"
echo "  ℹ️  IMAGENET-100 INFORMATION"
echo "========================================================================"
echo ""
echo "ImageNet-100 is a 100-class subset of the full ImageNet dataset."
echo ""
echo "Options to obtain ImageNet-100:"
echo ""
echo "Option 1: Kaggle ImageNet Mini (1000 classes, ~30GB)"
echo "  - Smaller than full ImageNet"
echo "  - Can subset to 100 classes after download"
echo "  - Requires Kaggle API key"
echo "  - Command: kaggle datasets download -d ifigotin/imagenetmini-1000"
echo ""
echo "Option 2: ImageNet-100 from Academic Source"
echo "  - Pre-selected 100 classes"
echo "  - ~15GB total"
echo "  - May require academic access"
echo ""
echo "Option 3: Use CIFAR-100 for Testing (94MB)"
echo "  - Much smaller dataset"
echo "  - Good for testing pipeline"
echo "  - 100 classes, 50k train, 10k test"
echo "  - Built into torchvision"
echo ""
echo "Option 4: Tiny ImageNet (200 classes, 250MB)"
echo "  - Very small, quick to download"
echo "  - Good for initial testing"
echo "  - Can subset to 100 classes"
echo "  - URL: http://cs231n.stanford.edu/tiny-imagenet-200.zip"
echo ""

echo "========================================================================"
echo "  🎯 RECOMMENDATION: Start with Tiny ImageNet for testing"
echo "========================================================================"
echo ""
echo "Tiny ImageNet is perfect for:"
echo "  ✅ Testing distributed training pipeline"
echo "  ✅ Verifying data sharding works"
echo "  ✅ Quick iteration (minutes not hours)"
echo "  ✅ Then scale to full ImageNet-100"
echo ""

read -p "Download Tiny ImageNet for testing? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo "📥 Downloading Tiny ImageNet..."
    cd "$DOWNLOAD_DIR"
    
    if [ ! -f "tiny-imagenet-200.zip" ]; then
        wget http://cs231n.stanford.edu/tiny-imagenet-200.zip
        echo "✅ Downloaded"
    else
        echo "✅ Already downloaded"
    fi
    
    echo ""
    echo "📦 Extracting..."
    if [ ! -d "$EXTRACT_DIR/tiny-imagenet-200" ]; then
        unzip -q tiny-imagenet-200.zip -d "$EXTRACT_DIR"
        echo "✅ Extracted"
    else
        echo "✅ Already extracted"
    fi
    
    echo ""
    echo "📊 Dataset info:"
    echo "  Location: $EXTRACT_DIR/tiny-imagenet-200"
    echo "  Classes: 200"
    echo "  Training: 100,000 images (500 per class)"
    echo "  Validation: 10,000 images (50 per class)"
    echo "  Image size: 64x64"
    echo ""
    echo "✅ Ready for sharding!"
    echo ""
    echo "Next step:"
    echo "  python3 shard_dataset.py --dataset tiny-imagenet --num-shards 3"
else
    echo ""
    echo "ℹ️  For full ImageNet-100, you'll need:"
    echo ""
    echo "1. Kaggle API (if using Kaggle):"
    echo "   pip install kaggle"
    echo "   # Place kaggle.json in ~/.kaggle/"
    echo "   kaggle datasets download -d ifigotin/imagenetmini-1000"
    echo ""
    echo "2. Or obtain from academic source with ImageNet access"
    echo ""
    echo "3. Then run this script again to extract and prepare"
fi

echo ""
echo "========================================================================"
echo "  📁 DATA DIRECTORY STRUCTURE"
echo "========================================================================"
echo ""
tree -L 2 "$DATA_DIR" 2>/dev/null || find "$DATA_DIR" -maxdepth 2 -type d 2>/dev/null || echo "  (Use 'tree' or 'find' to view structure)"
echo ""

