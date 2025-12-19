#!/bin/bash
# Quick Start Script for Westgate Tower
# Copy this to westgate and run it

echo "🎵 Starting Songbird on Westgate (Cold Storage Tower)"
echo "======================================================"
echo ""

# Configuration
export SONGBIRD_BIND_ADDRESS="0.0.0.0"
export SONGBIRD_PORT="8080"
export SONGBIRD_HOST="westgate.local"
export SONGBIRD_ENABLE_DISCOVERY="true"
export DISCOVERY_PORT="8081"
export SONGBIRD_MDNS_ENABLED="true"
export TOWER_NAME="westgate"
export TOWER_ROLE="storage"
export SONGBIRD_BROADCAST_ADDRESSES="255.255.255.255:2300,192.168.1.255:2300"

# Optional: Point to other towers (auto-discovery will find them anyway)
export SONGBIRD_FEDERATION_ENDPOINTS="http://eastgate.local:8080,http://strandgate.local:8080"

echo "Configuration:"
echo "  Tower Name: westgate"
echo "  Role: Cold Storage"
echo "  Bind Address: 0.0.0.0:8080"
echo "  Discovery: Enabled (port 8081)"
echo "  mDNS: Enabled"
echo ""

# Check if binary exists
if [ ! -f "./target/release/songbird-orchestrator" ]; then
    echo "❌ Songbird orchestrator binary not found!"
    echo "   Building now..."
    cargo build --release --bin songbird-orchestrator
    if [ $? -ne 0 ]; then
        echo "❌ Build failed!"
        exit 1
    fi
fi

echo "✅ Songbird binary ready"
echo ""

# Check if firewall needs configuration
echo "Checking firewall..."
if command -v ufw &> /dev/null; then
    echo "   UFW detected - checking rules..."
    sudo ufw status | grep -q "8080/tcp" || {
        echo "   Opening port 8080..."
        sudo ufw allow 8080/tcp comment "Songbird Orchestrator"
    }
    sudo ufw status | grep -q "8081/tcp" || {
        echo "   Opening port 8081..."
        sudo ufw allow 8081/tcp comment "Songbird Discovery"
    }
    sudo ufw status | grep -q "2300/udp" || {
        echo "   Opening port 2300..."
        sudo ufw allow 2300/udp comment "Songbird mDNS"
    }
    echo "   ✅ Firewall configured"
elif command -v firewall-cmd &> /dev/null; then
    echo "   firewalld detected - opening ports..."
    sudo firewall-cmd --permanent --add-port=8080/tcp
    sudo firewall-cmd --permanent --add-port=8081/tcp  
    sudo firewall-cmd --permanent --add-port=2300/udp
    sudo firewall-cmd --reload
    echo "   ✅ Firewall configured"
else
    echo "   ⚠️  No firewall detected or already configured"
fi
echo ""

echo "Starting Songbird Orchestrator..."
echo "  Log file: /tmp/westgate-orchestrator.log"
echo ""

# Start orchestrator (in foreground for now, can background later)
./target/release/songbird-orchestrator \
    --bind-address 0.0.0.0:8080 \
    --tower-name westgate \
    --enable-federation \
    --enable-discovery \
    2>&1 | tee /tmp/westgate-orchestrator.log

# If you want to run in background instead:
# nohup ./target/release/songbird-orchestrator \
#     --bind-address 0.0.0.0:8080 \
#     --tower-name westgate \
#     --enable-federation \
#     --enable-discovery \
#     > /tmp/westgate-orchestrator.log 2>&1 &

# echo "✅ Orchestrator started (PID: $!)"
# echo ""
# echo "To check status:"
# echo "  curl localhost:8080/health"
# echo ""
# echo "To stop:"
# echo "  kill $!"

