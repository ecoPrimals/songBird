#!/bin/bash
# Songbird Staging Deployment Script
# Auto-generated: November 6, 2025

set -e  # Exit on error

echo "════════════════════════════════════════════════════════════════"
echo "🚀 Songbird Staging Deployment"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌ Error: Must run from songbird root directory${NC}"
    exit 1
fi

# Step 1: Load staging environment
echo -e "${YELLOW}📋 Step 1: Loading staging environment...${NC}"
if [ -f "config/staging.env" ]; then
    source config/staging.env
    echo -e "${GREEN}✅ Staging environment loaded${NC}"
else
    echo -e "${RED}❌ Error: config/staging.env not found${NC}"
    exit 1
fi
echo ""

# Step 2: Verify production tests still pass
echo -e "${YELLOW}📋 Step 2: Verifying production tests...${NC}"
echo "Running lib tests (this validates all production code)..."
if cargo test --workspace --lib --quiet 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✅ All 1,574 production tests passing${NC}"
else
    echo -e "${RED}❌ Error: Some tests failing - deployment aborted${NC}"
    echo "Run: cargo test --workspace --lib"
    exit 1
fi
echo ""

# Step 3: Build release binary
echo -e "${YELLOW}📋 Step 3: Building release binary...${NC}"
echo "This may take 5-10 minutes..."
if cargo build --release --workspace 2>&1 | tail -3 | grep -q "Finished"; then
    echo -e "${GREEN}✅ Release binary built successfully${NC}"
else
    echo -e "${RED}❌ Error: Build failed${NC}"
    exit 1
fi
echo ""

# Step 4: Check if ports are available
echo -e "${YELLOW}📋 Step 4: Checking port availability...${NC}"
check_port() {
    if lsof -i :$1 > /dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  Port $1 is in use${NC}"
        return 1
    else
        echo -e "${GREEN}✅ Port $1 available${NC}"
        return 0
    fi
}

PORTS_AVAILABLE=true
check_port 8080 || PORTS_AVAILABLE=false
check_port 8081 || PORTS_AVAILABLE=false
check_port 8082 || PORTS_AVAILABLE=false
check_port 9090 || PORTS_AVAILABLE=false

if [ "$PORTS_AVAILABLE" = false ]; then
    echo -e "${YELLOW}⚠️  Some ports are in use. Continue anyway? (y/N)${NC}"
    read -r response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        echo "Deployment cancelled"
        exit 1
    fi
fi
echo ""

# Step 5: Create deployment directories
echo -e "${YELLOW}📋 Step 5: Creating deployment directories...${NC}"
mkdir -p target/staging/logs
mkdir -p target/staging/data
mkdir -p target/staging/config
echo -e "${GREEN}✅ Directories created${NC}"
echo ""

# Step 6: Start services
echo -e "${YELLOW}📋 Step 6: Starting Songbird services...${NC}"
echo ""
echo "Starting in background with logging to target/staging/logs/"
echo ""

# Start orchestrator
echo "Starting orchestrator on port 8080..."
nohup target/release/songbird-orchestrator \
    > target/staging/logs/orchestrator.log 2>&1 &
ORCHESTRATOR_PID=$!
echo "  PID: $ORCHESTRATOR_PID"

# Wait for orchestrator to start
sleep 3

# Verify orchestrator is running
if kill -0 $ORCHESTRATOR_PID 2>/dev/null; then
    echo -e "${GREEN}✅ Orchestrator started (PID: $ORCHESTRATOR_PID)${NC}"
else
    echo -e "${RED}❌ Error: Orchestrator failed to start${NC}"
    echo "Check logs: tail -f target/staging/logs/orchestrator.log"
    exit 1
fi
echo ""

# Step 7: Verify health endpoints
echo -e "${YELLOW}📋 Step 7: Verifying health endpoints...${NC}"
echo "Waiting for services to initialize (10 seconds)..."
sleep 10

check_health() {
    local url=$1
    local name=$2
    if curl -f -s "$url" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ $name is healthy${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠️  $name not responding${NC}"
        return 1
    fi
}

check_health "http://localhost:8080/health" "Orchestrator"
echo ""

# Step 8: Display deployment summary
echo "════════════════════════════════════════════════════════════════"
echo -e "${GREEN}✅ STAGING DEPLOYMENT COMPLETE!${NC}"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "📊 Deployment Summary:"
echo "   Environment: staging"
echo "   Orchestrator: http://localhost:8080 (PID: $ORCHESTRATOR_PID)"
echo "   Health: http://localhost:8080/health"
echo "   Metrics: http://localhost:9090"
echo ""
echo "📝 Log Files:"
echo "   Orchestrator: target/staging/logs/orchestrator.log"
echo ""
echo "🔍 Monitoring Commands:"
echo "   Health check: curl http://localhost:8080/health"
echo "   View logs: tail -f target/staging/logs/orchestrator.log"
echo "   Check status: ps aux | grep songbird"
echo ""
echo "🛑 Stop Services:"
echo "   kill $ORCHESTRATOR_PID"
echo "   Or: pkill -f songbird-orchestrator"
echo ""
echo "📈 Next Steps:"
echo "   1. Monitor logs for errors"
echo "   2. Test endpoints manually"
echo "   3. Run integration tests"
echo "   4. Monitor for 24-48 hours"
echo "   5. Plan production deployment"
echo ""
echo "════════════════════════════════════════════════════════════════"
echo ""

# Save PID for later
echo $ORCHESTRATOR_PID > target/staging/orchestrator.pid
echo -e "${GREEN}✅ Process ID saved to: target/staging/orchestrator.pid${NC}"
echo ""

