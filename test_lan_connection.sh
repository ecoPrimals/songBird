#!/bin/bash
# LAN Connection Test Script for Songbird Multi-Tower Setup

set -e

TOWER_A_IP="192.168.1.144"
TOWER_A_PORT="8080"

echo "🔗 Songbird LAN Connection Test"
echo "================================"
echo ""

# Get current machine's IP
MY_IP=$(hostname -I | awk '{print $1}')
echo "📍 This machine: $MY_IP"
echo "📍 Tower A: $TOWER_A_IP:$TOWER_A_PORT"
echo ""

# Test 1: Ping
echo "Test 1: Ping Tower A"
echo "-------------------"
if ping -c 2 -W 2 $TOWER_A_IP &>/dev/null; then
    echo "✅ PASS: Can reach Tower A"
else
    echo "❌ FAIL: Cannot reach Tower A"
    exit 1
fi
echo ""

# Test 2: Port connectivity
echo "Test 2: Port $TOWER_A_PORT connectivity"
echo "-------------------"
if timeout 3 bash -c "cat < /dev/null > /dev/tcp/$TOWER_A_IP/$TOWER_A_PORT" 2>/dev/null; then
    echo "✅ PASS: Port $TOWER_A_PORT is open"
else
    echo "❌ FAIL: Port $TOWER_A_PORT is not accessible"
    echo "   (Tower A may not be running yet)"
    exit 1
fi
echo ""

# Test 3: HTTP health check
echo "Test 3: HTTP Health Check"
echo "-------------------"
if curl -sf http://$TOWER_A_IP:$TOWER_A_PORT/health &>/dev/null; then
    echo "✅ PASS: Health endpoint responding"
    curl http://$TOWER_A_IP:$TOWER_A_PORT/health
else
    echo "❌ FAIL: Health endpoint not responding"
    exit 1
fi
echo ""

# Test 4: Discovery
echo "Test 4: Service Discovery"
echo "-------------------"
if curl -sf http://$TOWER_A_IP:$TOWER_A_PORT/discovery/peers &>/dev/null; then
    echo "✅ PASS: Discovery endpoint responding"
    curl -s http://$TOWER_A_IP:$TOWER_A_PORT/discovery/peers | jq '.' 2>/dev/null || curl -s http://$TOWER_A_IP:$TOWER_A_PORT/discovery/peers
else
    echo "⚠️  WARN: Discovery endpoint not responding (may be normal if no peers yet)"
fi
echo ""

echo "========================================="
echo "✅ LAN connectivity tests complete!"
echo "========================================="

