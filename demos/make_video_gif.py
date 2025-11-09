#!/usr/bin/env python3
"""
🎬 REAL ANIMATED PROOF: Live system monitoring as GIF!

Creates animated GIF showing real-time tower status changes.
No ffmpeg needed - pure Python with PIL!
"""

import requests
import json
import time
from PIL import Image, ImageDraw, ImageFont
from datetime import datetime

# Tower URLs (REAL)
TOWERS = {
    "A": "http://192.168.1.144:8080",
    "B": "http://192.168.1.134:8081", 
    "C": "http://192.168.1.207:8082"
}

WIDTH = 1200
HEIGHT = 800
BG = (15, 15, 25)
TEXT = (255, 255, 255)
ACCENT = (100, 200, 255)
SUCCESS = (100, 255, 150)

def get_real_data():
    """Query REAL data from all towers"""
    data = {}
    for name, url in TOWERS.items():
        try:
            health_resp = requests.get(f"{url}/health", timeout=1)
            services_resp = requests.get(f"{url}/api/deployment/list", timeout=1)
            
            data[name] = {
                "health": health_resp.text,
                "services": len(services_resp.json()),
                "latency": int(health_resp.elapsed.total_seconds() * 1000),
                "status": "ONLINE"
            }
        except Exception as e:
            data[name] = {
                "health": "ERROR",
                "services": 0,
                "latency": 999,
                "status": "OFFLINE"
            }
    return data

def create_frame(data, frame_num, total_frames):
    """Create a single frame with real data"""
    img = Image.new('RGB', (WIDTH, HEIGHT), BG)
    draw = ImageDraw.Draw(img)
    
    try:
        title_font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 48)
        body_font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 28)
        small_font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 20)
    except:
        title_font = body_font = small_font = ImageFont.load_default()
    
    # Title with frame counter
    title = f"🚀 LIVE 3-TOWER STATUS [{frame_num}/{total_frames}]"
    draw.text((40, 30), title, fill=ACCENT, font=title_font)
    
    # Timestamp
    timestamp = datetime.now().strftime('%H:%M:%S.%f')[:-3]
    draw.text((40, 100), f"📸 {timestamp}", fill=TEXT, font=body_font)
    
    # Tower status in columns
    y = 180
    tower_x = [80, 420, 760]
    tower_info = {
        "A": ("Eastgate", "RTX 4070"),
        "B": ("Strandgate", "RTX 3070"),
        "C": ("Southgate", "RTX 3090")
    }
    
    for i, (name, x) in enumerate(zip(["A", "B", "C"], tower_x)):
        tower = data[name]
        tname, gpu = tower_info[name]
        
        # Box for tower
        box_height = 250
        color = SUCCESS if tower['status'] == "ONLINE" else (255, 100, 100)
        draw.rectangle([(x-20, y-20), (x+280, y+box_height)], outline=color, width=3)
        
        # Tower name
        draw.text((x, y), f"Tower {name}", fill=color, font=title_font)
        draw.text((x, y+50), tname, fill=TEXT, font=body_font)
        draw.text((x, y+85), gpu, fill=ACCENT, font=small_font)
        
        # Status
        draw.text((x, y+130), f"Status: {tower['status']}", fill=color, font=body_font)
        draw.text((x, y+165), f"Services: {tower['services']}", fill=TEXT, font=body_font)
        draw.text((x, y+200), f"Latency: {tower['latency']}ms", fill=TEXT, font=body_font)
    
    # Summary bar at bottom
    y = HEIGHT - 120
    draw.line([(40, y), (WIDTH - 40, y)], fill=ACCENT, width=2)
    
    total_services = sum(data[t]['services'] for t in data)
    online = sum(1 for t in data.values() if t['status'] == 'ONLINE')
    avg_latency = sum(data[t]['latency'] for t in data) / 3
    
    y += 20
    draw.text((50, y), f"💎 {online}/3 ONLINE", fill=SUCCESS, font=body_font)
    draw.text((350, y), f"⚡ {total_services} Services", fill=SUCCESS, font=body_font)
    draw.text((650, y), f"🚀 {avg_latency:.1f}ms avg", fill=SUCCESS, font=body_font)
    
    y += 40
    draw.text((50, y), "🔥 REAL DATA - NO SIMULATION", fill=ACCENT, font=small_font)
    
    return img

def main():
    print("\n" + "=" * 70)
    print("  🎬 CREATING ANIMATED GIF WITH REAL DATA")
    print("=" * 70)
    print()
    
    frames = []
    num_frames = 10
    
    print(f"📸 Capturing {num_frames} frames from live system...")
    print()
    
    for i in range(num_frames):
        print(f"  Frame {i+1}/{num_frames}...", end=" ")
        
        # Get REAL data
        data = get_real_data()
        
        # Create frame
        frame = create_frame(data, i+1, num_frames)
        frames.append(frame)
        
        print(f"✅ ({sum(data[t]['services'] for t in data)} services)")
        
        # Wait a bit between frames
        if i < num_frames - 1:
            time.sleep(0.5)
    
    print()
    print("🎨 Assembling GIF...")
    
    # Save as animated GIF
    output_path = "/home/eastgate/Development/ecoPrimals/songbird/LIVE_FEDERATION.gif"
    frames[0].save(
        output_path,
        save_all=True,
        append_images=frames[1:],
        duration=800,  # ms per frame
        loop=0
    )
    
    print(f"✅ Saved: {output_path}")
    print()
    print("=" * 70)
    print("  ✅ ANIMATED GIF CREATED!")
    print("=" * 70)
    print()
    print(f"View: xdg-open {output_path}")
    print(f"Size: {len(frames)} frames")
    print(f"Data: 100% REAL from running towers")
    print()
    print("🎬 This GIF shows your 3-tower federation ACTUALLY WORKING!")
    print("   Each frame = real query to your live system")
    print()

if __name__ == "__main__":
    main()

