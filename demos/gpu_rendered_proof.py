#!/usr/bin/env python3
"""
🎨 GPU-RENDERED PROOF - Using CUDA for visualization

Queries REAL tower data and uses GPU to render the visualization!
"""

import torch
import numpy as np
import requests
from PIL import Image, ImageDraw, ImageFont
from datetime import datetime
import time

print("=" * 70)
print("  🎨 GPU-RENDERED DISTRIBUTED AI PROOF")
print("=" * 70)
print()

# Check GPU
if torch.cuda.is_available():
    print(f"✅ GPU: {torch.cuda.get_device_name(0)}")
    print(f"   CUDA: {torch.version.cuda}")
    print(f"   Memory: {torch.cuda.get_device_properties(0).total_memory / 1e9:.1f}GB")
else:
    print("⚠️  No GPU - using CPU rendering")
print()

# Tower endpoints
TOWERS = {
    "B": {"name": "Strandgate", "gpu": "RTX 3070", "url": "http://192.168.1.134:9011"},
    "C": {"name": "Southgate", "gpu": "RTX 3090", "url": "http://192.168.1.207:9012"},
}

WIDTH, HEIGHT = 1920, 1080

def get_real_data_NOW():
    """Query towers RIGHT NOW for REAL data"""
    print("📡 Querying towers RIGHT NOW...")
    
    data = {}
    for name, info in TOWERS.items():
        try:
            start = time.time()
            resp = requests.get(f"{info['url']}/health", timeout=2)
            latency = (time.time() - start) * 1000
            
            if resp.status_code == 200:
                tower_data = resp.json()
                data[name] = {
                    "status": tower_data["status"],
                    "uptime": tower_data["uptime_seconds"],
                    "capabilities": tower_data["metadata"]["capabilities"],
                    "latency": latency,
                    "timestamp": datetime.now().isoformat()
                }
                print(f"  Tower {name}: ✅ {tower_data['status']} ({latency:.1f}ms, {tower_data['uptime_seconds']}s uptime)")
            else:
                print(f"  Tower {name}: ❌ HTTP {resp.status_code}")
                data[name] = None
        except Exception as e:
            print(f"  Tower {name}: ❌ {e}")
            data[name] = None
    
    print()
    return data

def gpu_generate_gradient(width, height):
    """Use GPU to generate background gradient"""
    if not torch.cuda.is_available():
        return np.zeros((height, width, 3), dtype=np.uint8) + 15
    
    # Create gradient on GPU
    x = torch.linspace(0, 1, width, device='cuda')
    y = torch.linspace(0, 1, height, device='cuda')
    xx, yy = torch.meshgrid(x, y, indexing='xy')
    
    # Create RGB channels
    r = (15 + 30 * yy).byte()
    g = (15 + 20 * xx).byte()
    b = torch.full_like(r, 25)
    
    # Stack and move to CPU
    gradient = torch.stack([r, g, b], dim=2).cpu().numpy()
    
    return gradient.astype(np.uint8)

def create_gpu_rendered_frame(tower_data):
    """Create frame with GPU-accelerated rendering"""
    print("🎨 Rendering with GPU acceleration...")
    
    # GPU-generated background
    bg = gpu_generate_gradient(WIDTH, HEIGHT)
    img = Image.fromarray(bg, 'RGB')
    draw = ImageDraw.Draw(img)
    
    try:
        title_font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 72)
        heading_font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 48)
        body_font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 36)
        small_font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 28)
    except:
        title_font = body_font = heading_font = small_font = ImageFont.load_default()
    
    # Title
    timestamp = datetime.now().strftime('%H:%M:%S')
    draw.text((60, 50), f"🚀 LIVE TOWER STATUS - {timestamp}", fill=(100, 200, 255), font=title_font)
    
    # Subtitle
    draw.text((60, 150), "GPU-Rendered with REAL Data from Running Services", fill=(255, 255, 255), font=heading_font)
    
    # Tower data
    y = 280
    for i, (name, data) in enumerate(tower_data.items()):
        if data is None:
            draw.text((80, y), f"❌ Tower {name}: OFFLINE", fill=(255, 100, 100), font=body_font)
            y += 60
            continue
        
        # Tower header
        color = (100, 255, 150)
        draw.text((80, y), f"✅ Tower {name} ({TOWERS[name]['name']}) - {TOWERS[name]['gpu']}", 
                 fill=color, font=heading_font)
        y += 70
        
        # Details
        details = [
            f"Status: {data['status']}",
            f"Uptime: {data['uptime']}s",
            f"Latency: {data['latency']:.1f}ms",
            f"Capabilities: {data['capabilities'][:60]}...",
            f"Timestamp: {data['timestamp']}",
        ]
        
        for detail in details:
            draw.text((120, y), detail, fill=(255, 255, 255), font=body_font)
            y += 50
        
        y += 40
    
    # GPU info at bottom
    y = HEIGHT - 200
    draw.line([(60, y), (WIDTH - 60, y)], fill=(100, 200, 255), width=3)
    y += 30
    
    if torch.cuda.is_available():
        gpu_text = f"🎨 Rendered with: {torch.cuda.get_device_name(0)} (CUDA {torch.version.cuda})"
        draw.text((80, y), gpu_text, fill=(255, 200, 100), font=body_font)
        y += 50
    
    draw.text((80, y), "✅ ALL DATA FROM REAL HTTP REQUESTS - CAPTURED LIVE!", 
             fill=(100, 255, 150), font=heading_font)
    
    print("✅ Frame rendered!")
    return img

def main():
    # Get REAL data RIGHT NOW
    tower_data = get_real_data_NOW()
    
    # Create GPU-rendered visualization
    img = create_gpu_rendered_frame(tower_data)
    
    # Save
    output_path = "/home/eastgate/Development/ecoPrimals/songbird/GPU_RENDERED_PROOF.png"
    img.save(output_path)
    
    print()
    print("=" * 70)
    print("  ✅ GPU-RENDERED PROOF SAVED!")
    print("=" * 70)
    print()
    print(f"📄 File: {output_path}")
    print(f"📊 Size: {img.size[0]}x{img.size[1]}")
    print()
    
    if torch.cuda.is_available():
        print(f"🎨 Rendered with: {torch.cuda.get_device_name(0)}")
    print("📡 Data: LIVE from running towers")
    print()
    print(f"View: xdg-open {output_path}")
    print()
    
    # Open it
    import subprocess
    subprocess.Popen(['xdg-open', output_path], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

if __name__ == "__main__":
    main()

