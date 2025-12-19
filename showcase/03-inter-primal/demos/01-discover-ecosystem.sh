#!/bin/bash
# Songbird Showcase: Ecosystem Discovery
# Demonstrates: How Songbird discovers and coordinates primals

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'

SONGBIRD_URL=${SONGBIRD_URL:-"https://localhost:8080"}
CURL_OPTS="-k -s"  # -k for self-signed certs, -s for silent

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║       🎭 Songbird Orchestration: Ecosystem Discovery 🎭         ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo -e "${CYAN}Demonstrating: How Songbird discovers and coordinates primals${NC}"
echo -e "${CYAN}Songbird's Role: Service discovery + Capability matching + Routing${NC}"
echo ""

# Check if Songbird is running (try HTTPS first, then HTTP)
echo -e "${BLUE}[0/6]${NC} Checking Songbird availability..."
if curl $CURL_OPTS "${SONGBIRD_URL}/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Songbird is running at ${SONGBIRD_URL}${NC}"
elif curl -s "http://localhost:8080/health" > /dev/null 2>&1; then
    SONGBIRD_URL="http://localhost:8080"
    CURL_OPTS="-s"
    echo -e "${GREEN}✅ Songbird is running at ${SONGBIRD_URL}${NC}"
else
    echo -e "${YELLOW}⚠️  Songbird not running${NC}"
    echo "Start Songbird first:"
    echo "  cargo run --release --bin songbird-orchestrator"
    exit 1
fi
echo ""

# 1. SHOW INITIAL STATE
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${BLUE}[1/6]${NC} Initial State: Songbird starts with zero knowledge"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Querying Songbird's federation status..."
INITIAL_STATE=$(curl $CURL_OPTS "${SONGBIRD_URL}/api/federation/status" 2>/dev/null || echo '{}')
INITIAL_NODES=$(echo "$INITIAL_STATE" | jq -r '.total_nodes // 0')
echo "Current federation:"
echo "$INITIAL_STATE" | jq '{federation_id, total_nodes, active_nodes}' 2>/dev/null || echo "  No federation yet"
echo ""

echo "Querying available services..."
INITIAL_SERVICES=$(curl $CURL_OPTS "${SONGBIRD_URL}/api/discovery/services" 2>/dev/null || echo '[]')
SERVICE_COUNT=$(echo "$INITIAL_SERVICES" | jq 'length' 2>/dev/null || echo "0")
echo "Registered services: ${SERVICE_COUNT}"
if [ "$SERVICE_COUNT" -gt 0 ]; then
    echo "$INITIAL_SERVICES" | jq -r '.[] | "  • \(.name) (\(.type))"' 2>/dev/null
else
    echo "  (none yet - Songbird will discover them)"
fi
echo ""
sleep 2

# 2. DEMONSTRATE DISCOVERY
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${BLUE}[2/6]${NC} Discovery: Songbird actively searches for primals"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Discovery methods Songbird uses:"
echo "  1. mDNS/Bonjour - Local network broadcast"
echo "  2. Subnet scanning - Common ports (3000, 8080, 9090)"
echo "  3. Capability registry - Service announcements"
echo "  4. Federation mesh - Other Songbird nodes"
echo ""

echo "Simulating discovery (checking common ports)..."
PRIMALS_FOUND=()

# Check for Toadstool (typically port 3000)
echo -n "  Looking for Toadstool (compute primal) on port 3000... "
if curl -s --connect-timeout 1 http://localhost:3000/health > /dev/null 2>&1; then
    echo -e "${GREEN}FOUND!${NC}"
    PRIMALS_FOUND+=("Toadstool (Compute)")
else
    echo "not found"
fi

# Check for Squirrel (typically port 8080 or 9090)
echo -n "  Looking for Squirrel (AI primal) on port 9090... "
if curl -s --connect-timeout 1 http://localhost:9090/health > /dev/null 2>&1; then
    echo -e "${GREEN}FOUND!${NC}"
    PRIMALS_FOUND+=("Squirrel (AI)")
else
    echo "not found"
fi

# Check for BearDog (future - typically port 7777)
echo -n "  Looking for BearDog (crypto primal) on port 7777... "
if curl -s --connect-timeout 1 http://localhost:7777/health > /dev/null 2>&1; then
    echo -e "${GREEN}FOUND!${NC}"
    PRIMALS_FOUND+=("BearDog (Crypto)")
else
    echo "not found (coming soon)"
fi

# Check for other Songbird nodes
echo -n "  Looking for other Songbird towers (federation)... "
REMOTE_SONGBIRD_COUNT=0
for port in 8081 8082 8083; do
    if curl -s --connect-timeout 1 http://localhost:${port}/health > /dev/null 2>&1; then
        REMOTE_SONGBIRD_COUNT=$((REMOTE_SONGBIRD_COUNT + 1))
    fi
done
if [ $REMOTE_SONGBIRD_COUNT -gt 0 ]; then
    echo -e "${GREEN}FOUND ${REMOTE_SONGBIRD_COUNT} tower(s)!${NC}"
    PRIMALS_FOUND+=("Songbird x${REMOTE_SONGBIRD_COUNT} (Federation)")
else
    echo "none (single tower mode)"
fi

echo ""
echo "Discovery complete. Found:"
if [ ${#PRIMALS_FOUND[@]} -eq 0 ]; then
    echo -e "${YELLOW}  No other primals detected (yet)${NC}"
    echo "  This is normal! Primals announce themselves when they start."
else
    for primal in "${PRIMALS_FOUND[@]}"; do
        echo -e "  ${GREEN}✓${NC} $primal"
    done
fi
echo ""
sleep 2

# 3. SHOW CAPABILITY MAPPING
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${BLUE}[3/6]${NC} Capability Mapping: What each primal can do"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Songbird builds a capability map from discovered primals:"
echo ""
cat << 'EOF'
┌─────────────────────────────────────────────────────────────┐
│                   CAPABILITY MAP                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  🐿️  Squirrel (AI):                                        │
│     ✓ Text generation (local LLM, cloud APIs)              │
│     ✓ Image generation (DALL-E, Stable Diffusion)          │
│     ✓ Code completion (GPT, CodeLlama)                     │
│     ✓ Provider routing (cost optimization)                 │
│                                                             │
│  🍄 Toadstool (Compute):                                    │
│     ✓ GPU workloads (CUDA, ROCm)                           │
│     ✓ Biome execution (native, container, Python)          │
│     ✓ Distributed training (PyTorch, TensorFlow)           │
│     ✓ Resource management (CPU, GPU, memory)               │
│                                                             │
│  🐻 BearDog (Crypto - Future):                              │
│     ✓ BTSP encryption (genetic cryptography)               │
│     ✓ Key lineage tracking                                 │
│     ✓ Multi-party key renewal                              │
│     ✓ VPN-free secure channels                             │
│                                                             │
│  🐦 Songbird (Orchestration):                               │
│     ✓ Service discovery (mDNS, registry, mesh)             │
│     ✓ Protocol selection (HTTP, JSON-RPC, tarpc)           │
│     ✓ Load balancing (capability-based routing)            │
│     ✓ Federation (multi-tower coordination)                │
│                                                             │
└─────────────────────────────────────────────────────────────┘
EOF
echo ""
sleep 3

# 4. DEMONSTRATE ROUTING LOGIC
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${BLUE}[4/6]${NC} Routing Intelligence: How Songbird matches tasks to primals"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Example scenarios:"
echo ""

cat << 'EOF'
Scenario 1: "Generate an image"
  Songbird thinks:
    • Need: Image generation capability
    • Privacy: Low (generic image)
    • Performance: Important
  
  Options discovered:
    ✓ Squirrel → Cloud API (OpenAI DALL-E)
      Pros: High quality, easy
      Cons: $0.02 cost, 10s latency, external
    
    ✓ Toadstool → Local GPU (Stable Diffusion)
      Pros: FREE, 3s latency, private
      Cons: Requires GPU
  
  Songbird routes to: Toadstool ✅
  Reason: Faster, free, private


Scenario 2: "Train ML model on sensitive medical data"
  Songbird thinks:
    • Need: Compute + GPU
    • Privacy: CRITICAL (medical data)
    • Must stay local
  
  Options:
    ✗ Cloud GPU (AWS/GCP) - REJECTED (privacy)
    ✓ Toadstool (Local GPU) - SELECTED
  
  Songbird routes to: Local Toadstool only ✅
  Reason: Data sovereignty requirement


Scenario 3: "Analyze code across 3 repositories"
  Songbird thinks:
    • Need: AI + Distributed processing
    • Privacy: Medium (private code)
    • Scale: Parallel processing would help
  
  Options:
    ✓ Single Squirrel (Serial processing)
    ✓ Multiple towers with Squirrel (Parallel)
  
  Songbird routes to: 3 towers (parallel) ✅
  Reason: 3x faster with parallel execution
  Distribution:
    - Tower A: Repo 1
    - Tower B: Repo 2  
    - Tower C: Repo 3
  Aggregates results automatically


Scenario 4: "Secure file transfer between towers"
  Songbird thinks:
    • Need: Encryption + Transfer
    • Security: High
    • Network: Internet (untrusted)
  
  Options:
    ✓ HTTPS (TLS 1.3) - Available now
    ✓ BTSP (Genetic crypto) - When BearDog ready
  
  Songbird uses: HTTPS now, upgrades to BTSP later ✅
  Reason: Fail-secure by default, automatic upgrade
EOF
echo ""
sleep 3

# 5. SHOW LIVE CAPABILITIES
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${BLUE}[5/6]${NC} Live Capabilities: What Songbird can orchestrate RIGHT NOW"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Checking Songbird's current capabilities..."
CAPABILITIES=$(curl $CURL_OPTS "${SONGBIRD_URL}/api/protocol/capabilities" 2>/dev/null || echo '{}')
echo "$CAPABILITIES" | jq '.' 2>/dev/null || echo "{}"
echo ""

echo "Protocol support:"
PROTOCOLS=$(echo "$CAPABILITIES" | jq -r '.protocols | keys[]' 2>/dev/null || echo "")
if [ -n "$PROTOCOLS" ]; then
    echo "$PROTOCOLS" | while read protocol; do
        echo "  ✓ $protocol"
    done
else
    echo "  ✓ HTTP (REST)"
    echo "  ✓ HTTPS (TLS 1.3)"
    echo "  ✓ JSON-RPC 2.0"
    echo "  ✓ tarpc (native Rust)"
fi
echo ""
sleep 2

# 6. SUMMARY
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${BLUE}[6/6]${NC} Summary: Songbird's Orchestration Value"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

cat << 'EOF'
WITHOUT SONGBIRD:
  ❌ Manual configuration of each primal
  ❌ Hardcoded endpoints
  ❌ No automatic discovery
  ❌ Manual protocol selection
  ❌ No intelligent routing
  ❌ No failover
  ❌ Single point of failure

WITH SONGBIRD:
  ✅ Automatic primal discovery (mDNS, registry, mesh)
  ✅ Zero-configuration networking
  ✅ Intelligent capability matching
  ✅ Optimal protocol selection (100x speedup available)
  ✅ Automatic load balancing
  ✅ Failover and resilience
  ✅ Sovereignty by design

SONGBIRD'S ROLE:
  🎭 Conductor of the ecosystem
  🔍 Discovers services automatically
  🧠 Matches tasks to optimal primals
  🚀 Routes for performance and cost
  🔒 Ensures security (TLS by default)
  🌐 Coordinates across towers
  📊 Monitors and adapts

EMERGENT PROPERTY:
  Individual primals → Simple services
  Songbird coordination → Intelligent ecosystem
  
  The whole is GREATER than the sum of its parts!
EOF
echo ""

echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                    ✨ DEMO COMPLETE ✨                           ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Next Steps:"
echo "  1. Start other primals (Squirrel, Toadstool)"
echo "  2. Run: ./02-route-to-primal.sh"
echo "  3. See real orchestration in action!"
echo ""
echo "Learn more:"
echo "  • showcase/SONGBIRD_SHOWCASE_EVOLUTION.md"
echo "  • showcase/03-inter-primal/README.md"
echo ""

