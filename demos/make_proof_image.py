#!/usr/bin/env python3
"""
📸 REAL PROOF GENERATOR: Live System Status with Receipts!

Queries REAL data from all towers and creates visual proof.
"""

import requests
import json
from PIL import Image, ImageDraw, ImageFont
from datetime import datetime

# Tower URLs (REAL)
TOWERS = {
    "A": "http://192.168.1.144:8080",
    "B": "http://192.168.1.134:8081", 
    "C": "http://192.168.1.207:8082"
}

WIDTH = 1920
HEIGHT = 1080
BG = (15, 15, 25)
TEXT = (255, 255, 255)
ACCENT = (100, 200, 255)
SUCCESS = (100, 255, 150)

def get_real_data():
    """Query REAL data from all towers"""
    print("📡 Querying REAL data from all towers...")
    
    data = {}
    for name, url in TOWERS.items():
        try:
            # Health check
            health = requests.get(f"{url}/health", timeout=2).text
            
            # Services
            services = requests.get(f"{url}/api/deployment/list", timeout=2).json()
            
            data[name] = {
                "health": health,
                "services": services,
                "url": url,
                "status": "✅ ONLINE"
            }
            print(f"  Tower {name}: {len(services)} services, {health}")
            
        except Exception as e:
            data[name] = {
                "health": "ERROR",
                "services": [],
                "url": url,
                "status": "❌ OFFLINE"
            }
            print(f"  Tower {name}: Error - {e}")
    
    return data

def create_proof_image(data):
    """Create proof image with REAL data"""
    img = Image.new('RGB', (WIDTH, HEIGHT), BG)
    draw = ImageDraw.Draw(img)
    
    try:
        title_font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 72)
        heading_font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 48)
        body_font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 32)
        small_font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 24)
    except:
        title_font = body_font = heading_font = small_font = ImageFont.load_default()
    
    # Title
    title = "🚀 3-TOWER FEDERATION - LIVE STATUS"
    draw.text((100, 50), title, fill=ACCENT, font=title_font)
    
    # Timestamp
    timestamp = f"📸 Captured: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}"
    draw.text((100, 150), timestamp, fill=TEXT, font=body_font)
    
    # Tower details
    y = 250
    tower_names = {
        "A": "Tower A (Eastgate) - 192.168.1.144:8080",
        "B": "Tower B (Strandgate) - 192.168.1.134:8081",
        "C": "Tower C (Southgate) - 192.168.1.207:8082"
    }
    
    for name in ["A", "B", "C"]:
        tower = data[name]
        
        # Tower header
        header = f"{tower['status']} {tower_names[name]}"
        draw.text((100, y), header, fill=SUCCESS if "ONLINE" in tower['status'] else (255, 100, 100), font=heading_font)
        y += 60
        
        # Health
        health_text = f"  Health: {tower['health']}"
        draw.text((120, y), health_text, fill=TEXT, font=body_font)
        y += 45
        
        # Services
        services_text = f"  Services: {len(tower['services'])} running"
        draw.text((120, y), services_text, fill=TEXT, font=body_font)
        y += 45
        
        # List each service with PID (REAL DATA!)
        for svc in tower['services'][:3]:  # Show first 3
            svc_text = f"    • {svc.get('service_name', 'unknown')}"
            draw.text((140, y), svc_text, fill=ACCENT, font=small_font)
            y += 35
            
            pid_text = f"      PID: {svc.get('pid', 'N/A')} | Status: {svc.get('status', 'unknown')}"
            draw.text((160, y), pid_text, fill=TEXT, font=small_font)
            y += 35
        
        y += 20
    
    # Summary at bottom
    y = HEIGHT - 200
    draw.line([(100, y), (WIDTH - 100, y)], fill=ACCENT, width=2)
    y += 20
    
    total_services = sum(len(data[t]['services']) for t in data)
    online_towers = sum(1 for t in data.values() if "ONLINE" in t['status'])
    
    summary = [
        f"📊 TOTAL: {online_towers}/3 towers online",
        f"🎯 SERVICES: {total_services} running",
        f"⚡ STATUS: {'OPERATIONAL' if online_towers == 3 else 'PARTIAL'}",
        "🏆 PROOF: REAL LIVE DATA"
    ]
    
    x = 120
    for line in summary:
        draw.text((x, y), line, fill=SUCCESS, font=body_font)
        x += 450
    
    return img

def main():
    print("\n" + "=" * 70)
    print("  📸 CREATING REAL PROOF IMAGE")
    print("=" * 70)
    print()
    
    # Get REAL data
    data = get_real_data()
    
    print()
    print("🎨 Creating proof image with real data...")
    
    # Create image
    img = create_proof_image(data)
    
    # Save
    output_path = "/home/eastgate/Development/ecoPrimals/songbird/LIVE_PROOF.png"
    img.save(output_path)
    
    print(f"✅ Saved: {output_path}")
    print()
    print("=" * 70)
    print("  ✅ REAL PROOF CREATED!")
    print("=" * 70)
    print()
    print(f"View it: eog {output_path}")
    print(f"Or:      xdg-open {output_path}")
    print()
    print("This image shows REAL data from your running system!")
    print(f"Timestamp: {datetime.now()}")
    print()

if __name__ == "__main__":
    main()

