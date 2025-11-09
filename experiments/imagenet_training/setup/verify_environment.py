#!/usr/bin/env python3
"""
Environment Verification for Distributed ImageNet Training
Checks PyTorch, CUDA, NCCL, and distributed capabilities
"""

import sys
import subprocess

print("=" * 70)
print("  🔧 ENVIRONMENT VERIFICATION")
print("  Checking distributed training requirements")
print("=" * 70)
print()

# Check Python
print("📍 Python:")
print(f"   Version: {sys.version.split()[0]}")
print(f"   Path: {sys.executable}")
print()

# Check PyTorch
print("📦 PyTorch:")
try:
    import torch
    print(f"   ✅ Version: {torch.__version__}")
    print(f"   Path: {torch.__file__}")
except ImportError:
    print("   ❌ PyTorch not installed!")
    sys.exit(1)

# Check CUDA
print()
print("🎮 CUDA:")
if torch.cuda.is_available():
    print(f"   ✅ Available: Yes")
    print(f"   Version: {torch.version.cuda}")
    print(f"   Devices: {torch.cuda.device_count()}")
    for i in range(torch.cuda.device_count()):
        props = torch.cuda.get_device_properties(i)
        print(f"   GPU {i}: {props.name} ({props.total_memory / 1e9:.1f}GB)")
else:
    print("   ❌ CUDA not available!")
    sys.exit(1)

# Check NCCL
print()
print("📡 NCCL (Distributed Backend):")
try:
    if torch.cuda.nccl.is_available([0]):
        print(f"   ✅ Available: Yes")
        print(f"   Version: {torch.cuda.nccl.version()}")
    else:
        print("   ⚠️  NCCL not available (will use Gloo fallback)")
except:
    print("   ⚠️  Could not check NCCL")

# Check distributed package
print()
print("🌐 Torch Distributed:")
try:
    import torch.distributed as dist
    print(f"   ✅ Module available")
    print(f"   Backends: {dist.Backend.NCCL}, {dist.Backend.GLOO}")
    
    # Try to get available backends
    available = []
    if dist.is_nccl_available():
        available.append("NCCL")
    if dist.is_gloo_available():
        available.append("Gloo")
    print(f"   Available: {', '.join(available)}")
except ImportError:
    print("   ❌ Torch distributed not available!")

# Check torchvision
print()
print("🖼️  Torchvision:")
try:
    import torchvision
    print(f"   ✅ Version: {torchvision.__version__}")
except ImportError:
    print("   ⚠️  Not installed (needed for data loading)")
    print("   Install: pip install torchvision")

# Check disk space
print()
print("💾 Disk Space:")
try:
    import shutil
    stat = shutil.disk_usage("/home/eastgate/Development/ecoPrimals/songbird")
    free_gb = stat.free / (1024**3)
    total_gb = stat.total / (1024**3)
    print(f"   Free: {free_gb:.1f} GB / {total_gb:.1f} GB")
    if free_gb < 10:
        print("   ⚠️  Less than 10GB free! May need cleanup.")
    else:
        print("   ✅ Sufficient space for ImageNet-100")
except:
    print("   ⚠️  Could not check disk space")

# Check network tools
print()
print("🔧 Network Tools:")
tools = {
    "iperf3": "Bandwidth testing",
    "ssh": "Remote execution",
    "rsync": "Data transfer"
}
for tool, desc in tools.items():
    try:
        subprocess.run([tool, "--version"], capture_output=True, timeout=2)
        print(f"   ✅ {tool}: Available ({desc})")
    except:
        print(f"   ⚠️  {tool}: Not found ({desc})")

# Summary
print()
print("=" * 70)
print("  📊 SUMMARY")
print("=" * 70)
print()

ready = True
requirements = [
    ("PyTorch >= 1.10", torch.__version__ >= "1.10", True),
    ("CUDA available", torch.cuda.is_available(), True),
    ("GPU count >= 1", torch.cuda.device_count() >= 1, True),
    ("Distributed available", hasattr(torch, 'distributed'), True),
    ("Disk space > 10GB", free_gb > 10 if 'free_gb' in locals() else False, True),
]

for req, status, critical in requirements:
    symbol = "✅" if status else ("❌" if critical else "⚠️")
    print(f"{symbol} {req}")
    if critical and not status:
        ready = False

print()
if ready:
    print("🎉 Environment is READY for distributed training!")
    print()
    print("Next steps:")
    print("  1. Download ImageNet-100 dataset")
    print("  2. Shard data across towers")
    print("  3. Run baseline training")
    sys.exit(0)
else:
    print("❌ Environment has critical issues. Please fix before proceeding.")
    sys.exit(1)

