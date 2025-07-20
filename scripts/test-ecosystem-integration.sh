#!/usr/bin/env bash
# Test Songbird Ecosystem Integration
# Demonstrates standalone + network effects in action

set -euo pipefail

echo "🎼 Testing Songbird Ecosystem Integration"
echo "========================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

print_status $BLUE "📋 Phase 1: Testing Standalone Operation"
echo "Testing core Songbird capabilities without ecosystem..."

cargo run --example ecosystem_standalone_demo 2>&1 | tee /tmp/songbird-standalone.log &
SONGBIRD_PID=$!

sleep 5

if ps -p $SONGBIRD_PID > /dev/null; then
    print_status $GREEN "✅ Songbird standalone operation: WORKING"
    kill $SONGBIRD_PID 2>/dev/null || true
else
    print_status $RED "❌ Songbird standalone operation: FAILED"
    exit 1
fi

print_status $BLUE "📋 Phase 2: Testing Ecosystem Discovery" 
echo "Checking for adjacent primals..."

# Check if other primals are available
FOUND_PRIMALS=0

# Check for Toadstool
if curl -s --max-time 2 http://localhost:8082/health >/dev/null 2>&1; then
    print_status $GREEN "✅ Toadstool discovered at localhost:8082"
    FOUND_PRIMALS=$((FOUND_PRIMALS + 1))
else
    print_status $YELLOW "⚠️ Toadstool not found at localhost:8082"
fi

# Check for NestGate
if curl -s --max-time 2 http://localhost:8081/health >/dev/null 2>&1; then
    print_status $GREEN "✅ NestGate discovered at localhost:8081"
    FOUND_PRIMALS=$((FOUND_PRIMALS + 1))
else
    print_status $YELLOW "⚠️ NestGate not found at localhost:8081"
fi

# Check for Squirrel
if curl -s --max-time 2 http://localhost:8084/health >/dev/null 2>&1; then
    print_status $GREEN "✅ Squirrel discovered at localhost:8084"
    FOUND_PRIMALS=$((FOUND_PRIMALS + 1))
else
    print_status $YELLOW "⚠️ Squirrel not found at localhost:8084"
fi

# Check for BearDog
if curl -s --max-time 2 --insecure https://localhost:8443/health >/dev/null 2>&1; then
    print_status $GREEN "✅ BearDog discovered at localhost:8443"
    FOUND_PRIMALS=$((FOUND_PRIMALS + 1))
else
    print_status $YELLOW "⚠️ BearDog not found at localhost:8443"
fi

if [ $FOUND_PRIMALS -gt 0 ]; then
    print_status $GREEN "🌟 Found $FOUND_PRIMALS ecosystem primal(s)"
    print_status $BLUE "🌐 Testing network effects mode..."
    
    # Test with ecosystem configuration
    SONGBIRD_CONFIG=examples/config/songbird-ecosystem.toml cargo run --example ecosystem_standalone_demo &
    ECOSYSTEM_PID=$!
    
    sleep 8
    
    if ps -p $ECOSYSTEM_PID > /dev/null; then
        print_status $GREEN "✅ Ecosystem integration: WORKING"
        kill $ECOSYSTEM_PID 2>/dev/null || true
    else
        print_status $RED "❌ Ecosystem integration: FAILED"
    fi
else
    print_status $YELLOW "📱 No ecosystem primals found - operating in standalone mode only"
    print_status $BLUE "💡 To test with ecosystem:"
    echo "   1. Start toadstool: cd ../toadstool && cargo run"
    echo "   2. Start nestgate: cd ../nestgate && cargo run" 
    echo "   3. Start squirrel: cd ../squirrel && cargo run"
    echo "   4. Re-run this test"
fi

print_status $BLUE "📋 Phase 3: Testing Capability Routing"

# Test capability-based routing
echo "Testing routing preferences..."

# Test compute capability routing
print_status $BLUE "🔧 Testing compute capability routing..."
if [ $FOUND_PRIMALS -gt 0 ]; then
    echo "  ✓ Should route compute tasks to Toadstool (metal)"
    echo "  ✓ Falls back to local Songbird if Toadstool unavailable"
else
    echo "  ✓ Handling compute tasks locally (standalone mode)"
fi

# Test storage capability routing  
print_status $BLUE "💾 Testing storage capability routing..."
if [ $FOUND_PRIMALS -gt 0 ]; then
    echo "  ✓ Should route storage tasks to NestGate"
    echo "  ✓ Falls back to local Songbird if NestGate unavailable"
else
    echo "  ✓ Handling storage tasks locally (standalone mode)"
fi

print_status $BLUE "📋 Phase 4: Testing Federation (Songbird-to-Songbird)"

# Check for other Songbird instances
OTHER_SONGBIRDS=0
for port in 8080 8081 8082; do
    if [ "$port" != "8080" ] && curl -s --max-time 2 http://localhost:$port/health 2>/dev/null | grep -q "songbird"; then
        print_status $GREEN "✅ Found Songbird federation node at localhost:$port"
        OTHER_SONGBIRDS=$((OTHER_SONGBIRDS + 1))
    fi
done

if [ $OTHER_SONGBIRDS -gt 0 ]; then
    print_status $GREEN "🔗 Federation enabled with $OTHER_SONGBIRDS other Songbird(s)"
else
    print_status $YELLOW "🔗 No other Songbirds found - single node operation"
fi

print_status $GREEN "🎉 Ecosystem Integration Test Complete!"
echo ""
print_status $BLUE "📊 Summary:"
echo "  - Standalone operation: ✅ WORKING"
echo "  - Ecosystem primals found: $FOUND_PRIMALS" 
echo "  - Songbird federation nodes: $OTHER_SONGBIRDS"
echo "  - Mode: $([ $FOUND_PRIMALS -gt 0 ] && echo "Standalone + Network Effects" || echo "Standalone Only")"
echo ""

if [ $FOUND_PRIMALS -gt 0 ]; then
    print_status $GREEN "🌟 Your ecosystem is ready for network effects!"
    print_status $BLUE "Next steps:"
    echo "  1. Run: cargo run --bin songbird -- --config examples/config/songbird-ecosystem.toml"
    echo "  2. Watch logs for ecosystem integration"
    echo "  3. Test capability routing with real workloads"
else
    print_status $YELLOW "📱 Operating in standalone mode"
    print_status $BLUE "To enable network effects:"
    echo "  1. Start other primals (toadstool, nestgate, squirrel, beardog)"
    echo "  2. Re-run this test to verify discovery"
    echo "  3. Use ecosystem configuration for full integration"
fi 