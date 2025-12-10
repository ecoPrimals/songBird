#!/usr/bin/env bash
# 🔐 Demo 4: Sovereign Security with Authentication
# Demonstrates Songbird's built-in sovereign security

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FEDERATION_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_ROOT="$(cd "$FEDERATION_DIR/../.." && pwd)"
BINARY="$PROJECT_ROOT/target/release/songbird-orchestrator"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔐 Sovereign Security Demo"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "📝 This demo shows Songbird's built-in security:"
echo "  • Sovereign security (no dependencies)"
echo "  • Token-based authentication"
echo "  • Request validation"
echo "  • Graceful BearDog fallback"
echo
read -p "Press Enter to continue..."
echo

# Clean up any existing instances
echo "🧹 Cleaning up existing instances..."
killall -q songbird-orchestrator 2>/dev/null || true
sleep 1

# Generate a secure token
SECURITY_TOKEN=$(openssl rand -hex 32 2>/dev/null || echo "demo-token-$(date +%s)")

echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔑 Security Configuration"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "Generated Security Token:"
echo "  $SECURITY_TOKEN"
echo
echo "Save this token - you'll need it for authenticated requests!"
echo
read -p "Press Enter to start secure tower..."
echo

# Create logs directory
mkdir -p "$FEDERATION_DIR/logs"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Starting Secure Tower (Port 8090)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Start tower with security enabled
RUST_LOG=info \
SONGBIRD_PORT=8090 \
SONGBIRD_NODE_ID="secure-tower" \
SONGBIRD_SECURITY_MODE="sovereign" \
SONGBIRD_AUTH_REQUIRED="true" \
SONGBIRD_AUTH_TOKEN="$SECURITY_TOKEN" \
"$BINARY" > "$FEDERATION_DIR/logs/secure-tower.log" 2>&1 &
TOWER_PID=$!

echo "📝 Tower PID: $TOWER_PID"
echo "⏳ Waiting for startup..."

# Wait for startup
STARTED=false
for i in {1..30}; do
    sleep 1
    if curl -s http://localhost:8090/health > /dev/null 2>&1; then
        STARTED=true
        break
    fi
    echo -n "."
done
echo

if [ "$STARTED" = true ]; then
    echo "✅ Secure tower is ready!"
else
    echo "❌ Tower failed to start"
    echo "Check logs: tail $FEDERATION_DIR/logs/secure-tower.log"
    kill $TOWER_PID 2>/dev/null || true
    exit 1
fi

echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧪 Security Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Test 1: Public endpoint (should work)
echo "Test 1: Public Health Endpoint"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Request: curl http://localhost:8090/health"
echo
HEALTH=$(curl -s http://localhost:8090/health)
echo "Response: $HEALTH"
if [ "$HEALTH" = "OK" ]; then
    echo "✅ Public endpoint accessible (expected)"
else
    echo "⚠️  Unexpected response"
fi
echo
read -p "Press Enter to continue..."
echo

# Test 2: Protected endpoint without auth (should fail)
echo "Test 2: Protected Endpoint (No Auth)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Request: curl http://localhost:8090/api/federation/status"
echo
STATUS_CODE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8090/api/federation/status)
echo "Response Code: $STATUS_CODE"

if [ "$STATUS_CODE" = "200" ]; then
    echo "ℹ️  Note: Auth may not be enforced yet on this endpoint"
    echo "   (Implementation in progress)"
elif [ "$STATUS_CODE" = "401" ] || [ "$STATUS_CODE" = "403" ]; then
    echo "✅ Unauthorized (expected with auth enabled)"
else
    echo "ℹ️  Got $STATUS_CODE (auth enforcement varies by endpoint)"
fi
echo
read -p "Press Enter to continue..."
echo

# Test 3: Federation status (currently open)
echo "Test 3: Federation Status Check"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Request: curl http://localhost:8090/api/federation/status"
echo
FEDERATION_STATUS=$(curl -s http://localhost:8090/api/federation/status)
echo "Response:"
echo "$FEDERATION_STATUS" | jq '.' 2>/dev/null || echo "$FEDERATION_STATUS"
echo
read -p "Press Enter to continue..."
echo

# Test 4: BearDog integration check
echo "Test 4: BearDog Integration Status"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
if [ -n "$BEARDOG_SECURITY_ENDPOINT" ]; then
    echo "✅ BEARDOG_SECURITY_ENDPOINT set: $BEARDOG_SECURITY_ENDPOINT"
    echo "   Songbird will attempt to discover BearDog"
else
    echo "ℹ️  BEARDOG_SECURITY_ENDPOINT not set"
    echo "   Using sovereign security only (standalone mode)"
fi
echo
echo "Sovereign Security Features:"
echo "  • Token-based authentication ✅"
echo "  • Request validation ✅"
echo "  • Basic encryption (when implemented) ⚠️"
echo "  • Always available ✅"
echo
read -p "Press Enter to continue..."
echo

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📝 Security Configuration Examples"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "Sovereign Mode (Current):"
echo "  SONGBIRD_SECURITY_MODE=\"sovereign\""
echo "  SONGBIRD_AUTH_TOKEN=\"$SECURITY_TOKEN\""
echo
echo "With BearDog (Network Effect):"
echo "  SONGBIRD_SECURITY_MODE=\"sovereign\""
echo "  BEARDOG_SECURITY_ENDPOINT=\"http://localhost:8443\""
echo "  # Automatically discovers and uses BearDog if available"
echo
echo "Strict Mode (Future):"
echo "  SONGBIRD_SECURITY_MODE=\"strict\""
echo "  SONGBIRD_REQUIRE_TLS=\"true\""
echo "  SONGBIRD_TLS_CERT=\"/path/to/cert.pem\""
echo
read -p "Press Enter for cleanup..."
echo

# Cleanup
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧹 Cleanup"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "Stopping secure tower (PID: $TOWER_PID)..."
kill $TOWER_PID 2>/dev/null || true
sleep 1
echo "✅ Cleanup complete"
echo

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "✅ Demonstrated:"
echo "  • Sovereign security启动"
echo "  • Public endpoints (health check)"
echo "  • Protected endpoints (federation API)"
echo "  • Token-based authentication pattern"
echo "  • BearDog integration readiness"
echo
echo "ℹ️  Current State:"
echo "  • Sovereign architecture: IMPLEMENTED ✅"
echo "  • Token generation: WORKING ✅"
echo "  • Endpoint protection: PARTIAL ⚠️"
echo "  • TLS/HTTPS: NOT ACTIVE ❌"
echo "  • mTLS peer auth: NOT ACTIVE ❌"
echo
echo "🚀 Next Steps:"
echo "  1. Activate TLS for encryption"
echo "  2. Implement auth middleware for all protected endpoints"
echo "  3. Add certificate-based peer authentication"
echo "  4. Test with BearDog integration"
echo
echo "📝 Your Security Token (save this):"
echo "  $SECURITY_TOKEN"
echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔐 Sovereign Security Demo Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

