#!/bin/bash
# Verify Westgate Federation Setup
# Run this on eastgate to check if westgate can join

echo "🌐 Verifying Westgate Federation Setup"
echo "========================================"
echo ""

WESTGATE_IP="192.168.1.123"
WESTGATE_HOST="westgate.local"

echo "1️⃣  Network Connectivity"
echo "------------------------"
if ping -c 2 $WESTGATE_HOST >/dev/null 2>&1; then
    echo "✅ Can ping westgate ($WESTGATE_HOST)"
    echo "   IP: $WESTGATE_IP"
else
    echo "❌ Cannot ping westgate"
    exit 1
fi
echo ""

echo "2️⃣  Songbird Ports on Westgate"
echo "-------------------------------"
PORTS=(8080 8081 2300)
for port in "${PORTS[@]}"; do
    if timeout 2 bash -c "echo >/dev/tcp/$WESTGATE_IP/$port" 2>/dev/null; then
        echo "✅ Port $port is open"
    else
        echo "⚠️  Port $port is closed or filtered"
    fi
done
echo ""

echo "3️⃣  Health Endpoint"
echo "-------------------"
if curl -s --connect-timeout 3 http://$WESTGATE_HOST:8080/health >/dev/null 2>&1; then
    echo "✅ Westgate orchestrator responding"
    HEALTH=$(curl -s http://$WESTGATE_HOST:8080/health)
    echo "   Response: $HEALTH"
else
    echo "⚠️  Westgate orchestrator not responding on port 8080"
    echo ""
    echo "   Possible causes:"
    echo "   - Songbird not running on westgate"
    echo "   - Bound to localhost only (should be 0.0.0.0)"
    echo "   - Firewall blocking port 8080"
    echo "   - Different port configured"
fi
echo ""

echo "4️⃣  Service Discovery"
echo "---------------------"
cd /home/eastgate/Development/ecoPrimals/songbird
if [ -f "./target/release/songbird-cli" ]; then
    echo "Running discovery scan..."
    ./target/release/songbird-cli discover --timeout 5
else
    echo "⚠️  songbird-cli not built"
fi
echo ""

echo "5️⃣  Federation Registry"
echo "-----------------------"
if curl -s http://localhost:8080/api/v1/federation/towers 2>/dev/null; then
    echo "✅ Federation registry accessible"
else
    echo "⚠️  Federation registry not accessible"
fi
echo ""

echo "6️⃣  Current Federation Status"
echo "------------------------------"
echo "Known towers from eastgate:"
ps aux | grep songbird-orchestrator | grep -v grep | head -1
if [ $? -eq 0 ]; then
    echo "✅ Eastgate orchestrator running"
else
    echo "⚠️  Eastgate orchestrator not running"
fi
echo ""

echo "📊 Summary"
echo "----------"
echo "Westgate IP: $WESTGATE_IP"
echo "Westgate Host: $WESTGATE_HOST"
echo ""
echo "Next steps to get westgate online:"
echo "1. SSH to westgate: ssh westgate"
echo "2. Navigate to Songbird directory"
echo "3. Set environment: export SONGBIRD_BIND_ADDRESS='0.0.0.0'"
echo "4. Start orchestrator: ./target/release/songbird-orchestrator"
echo "5. Open firewall: sudo ufw allow 8080/tcp"
echo "6. Verify health: curl localhost:8080/health"
echo ""
echo "See: WESTGATE_FEDERATION_SETUP.md for detailed instructions"

