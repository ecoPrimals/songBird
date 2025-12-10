#!/usr/bin/env bash
#
# Demo 07: Docker Container Integration
# Show Songbird routing to Docker containers
#

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🐳 Songbird Demo: Docker Container Integration${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo -e "${CYAN}Scenario: Route to services running in Docker containers${NC}"
echo -e "${CYAN}Goal: Show Songbird as Docker-aware intelligent router${NC}"
echo
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

# Configuration
SONGBIRD_BIN="../../../target/release/songbird-orchestrator"
SONGBIRD_PORT=8000

# Check Docker
echo -e "${CYAN}🔍 Checking Docker availability...${NC}"
if ! command -v docker &> /dev/null; then
    echo -e "${YELLOW}⚠️  Docker not found in PATH${NC}"
    echo -e "${CYAN}   This demo will show what WOULD happen with Docker${NC}"
    DOCKER_AVAILABLE=false
else
    if docker ps >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Docker is available and accessible${NC}"
        DOCKER_AVAILABLE=true
    else
        echo -e "${YELLOW}⚠️  Docker found but not accessible${NC}"
        echo -e "${CYAN}   (May need: sudo usermod -aG docker $USER)${NC}"
        DOCKER_AVAILABLE=false
    fi
fi
echo

# Check Songbird
if [[ ! -f "$SONGBIRD_BIN" ]]; then
    echo -e "${RED}❌ Songbird not found${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Songbird ready${NC}"
echo

# Scan for existing containers
echo -e "${BLUE}━━━ Step 1: Scan Docker Containers ━━━${NC}"
echo

if [[ "$DOCKER_AVAILABLE" == "true" ]]; then
    CONTAINER_COUNT=$(docker ps --format "{{.Names}}" 2>/dev/null | wc -l)
    
    if [[ $CONTAINER_COUNT -gt 0 ]]; then
        echo -e "${GREEN}✅ Found $CONTAINER_COUNT running container(s):${NC}"
        echo
        docker ps --format "table {{.Names}}\t{{.Image}}\t{{.Ports}}" | head -10
        echo
    else
        echo -e "${YELLOW}ℹ️  No containers currently running${NC}"
        echo -e "${CYAN}   Let's show how Songbird WOULD integrate${NC}"
        echo
    fi
else
    echo -e "${CYAN}📋 Example Docker containers Songbird could route to:${NC}"
    echo
    echo -e "   ${GREEN}redis-cache${NC}        redis:latest          6379→6379"
    echo -e "   ${GREEN}postgres-db${NC}       postgres:15           5432→5432"
    echo -e "   ${GREEN}mongo-data${NC}        mongo:latest          27017→27017"
    echo -e "   ${GREEN}nginx-web${NC}         nginx:alpine          80→8080"
    echo -e "   ${GREEN}api-service${NC}       node:18-alpine        3000→3000"
    echo
fi

# Start Songbird
echo -e "${BLUE}━━━ Step 2: Start Songbird${NC}"
echo

echo -e "${CYAN}🚀 Starting Songbird orchestrator...${NC}"
export RUST_LOG=info
$SONGBIRD_BIN > /tmp/songbird-docker-demo.log 2>&1 &
SONGBIRD_PID=$!
sleep 3

if ! kill -0 $SONGBIRD_PID 2>/dev/null; then
    echo -e "${RED}❌ Songbird failed to start${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Songbird running (PID: $SONGBIRD_PID)${NC}"
echo

# Show integration concept
echo -e "${BLUE}━━━ Step 3: Container Discovery & Registration ━━━${NC}"
echo
echo -e "${CYAN}📝 How Songbird discovers Docker containers:${NC}"
echo
echo -e "   ${MAGENTA}1. Docker Socket Integration${NC}"
echo -e "      → Songbird connects to /var/run/docker.sock"
echo -e "      → Monitors container lifecycle (start/stop/restart)"
echo
echo -e "   ${MAGENTA}2. Automatic Capability Detection${NC}"
echo -e "      → Reads container labels and exposed ports"
echo -e "      → Infers capabilities from image name"
echo -e "      → Example: 'redis:latest' → cache, key-value capabilities"
echo
echo -e "   ${MAGENTA}3. Dynamic Registration${NC}"
echo -e "      → New container starts → auto-registered with Songbird"
echo -e "      → Container stops → auto-removed from routing"
echo -e "      → Zero manual configuration"
echo

# Show routing examples
echo -e "${BLUE}━━━ Step 4: Intelligent Routing to Containers ━━━${NC}"
echo
echo -e "${CYAN}🎯 Routing examples:${NC}"
echo

echo -e "${YELLOW}Example 1: Redis Container${NC}"
echo -e "   Container: ${GREEN}redis-cache${NC} (port 6379)"
echo -e "   Capabilities: cache, key-value-store"
echo -e "   "
echo -e "   Request: ${BLUE}GET /cache/session:abc123${NC}"
echo -e "   ${GREEN}→ Songbird routes to redis-cache container${NC}"
echo

echo -e "${YELLOW}Example 2: PostgreSQL Container${NC}"
echo -e "   Container: ${GREEN}postgres-db${NC} (port 5432)"
echo -e "   Capabilities: database, sql, relational"
echo -e "   "
echo -e "   Request: ${BLUE}SELECT * FROM users WHERE id = 1${NC}"
echo -e "   ${GREEN}→ Songbird routes to postgres-db container${NC}"
echo

echo -e "${YELLOW}Example 3: Web API Container${NC}"
echo -e "   Container: ${GREEN}api-service${NC} (port 3000)"
echo -e "   Capabilities: http, api, rest"
echo -e "   "
echo -e "   Request: ${BLUE}GET /api/v1/users${NC}"
echo -e "   ${GREEN}→ Songbird routes to api-service container${NC}"
echo

# Show multi-container scenarios
echo -e "${BLUE}━━━ Step 5: Multi-Container Scenarios ━━━${NC}"
echo
echo -e "${CYAN}💡 Advanced routing with multiple containers:${NC}"
echo

echo -e "${MAGENTA}Scenario A: Multiple Redis Instances${NC}"
echo -e "   Containers: ${GREEN}redis-1, redis-2, redis-3${NC}"
echo -e "   "
echo -e "   ${GREEN}→ Songbird automatically load balances${NC}"
echo -e "   ${GREEN}→ Round-robin or least-loaded${NC}"
echo -e "   ${GREEN}→ Health checks ensure only healthy containers get traffic${NC}"
echo

echo -e "${MAGENTA}Scenario B: Blue-Green Deployment${NC}"
echo -e "   Containers: ${GREEN}api-blue${NC} (current), ${CYAN}api-green${NC} (new)"
echo -e "   "
echo -e "   ${GREEN}→ Both registered with 'api' capability${NC}"
echo -e "   ${GREEN}→ Gradually shift traffic: 90% blue, 10% green${NC}"
echo -e "   ${GREEN}→ Monitor errors, rollback if needed${NC}"
echo -e "   ${GREEN}→ Eventually: 100% green, remove blue${NC}"
echo

echo -e "${MAGENTA}Scenario C: Service Mesh${NC}"
echo -e "   Containers: ${GREEN}frontend${NC}, ${GREEN}backend${NC}, ${GREEN}cache${NC}, ${GREEN}db${NC}"
echo -e "   "
echo -e "   ${GREEN}→ Frontend asks for 'api' capability${NC}"
echo -e "   ${GREEN}→ Backend asks for 'cache' capability${NC}"
echo -e "   ${GREEN}→ Backend asks for 'database' capability${NC}"
echo -e "   ${GREEN}→ Songbird routes all connections automatically${NC}"
echo

# Show benefits
echo -e "${BLUE}━━━ Step 6: Benefits Over Traditional Docker Networking ━━━${NC}"
echo
echo -e "${CYAN}🆚 Traditional Docker Networking:${NC}"
echo -e "   ${RED}✗${NC} Hardcoded container names: ${YELLOW}redis-cache:6379${NC}"
echo -e "   ${RED}✗${NC} Manual load balancing configuration"
echo -e "   ${RED}✗${NC} No automatic failover"
echo -e "   ${RED}✗${NC} Complex docker-compose files"
echo -e "   ${RED}✗${NC} Restart required for network changes"
echo

echo -e "${CYAN}✅ Songbird Docker Integration:${NC}"
echo -e "   ${GREEN}✓${NC} Capability-based: ask for ${YELLOW}'cache'${NC}, get best container"
echo -e "   ${GREEN}✓${NC} Automatic load balancing across containers"
echo -e "   ${GREEN}✓${NC} Instant failover if container dies"
echo -e "   ${GREEN}✓${NC} Simple service discovery"
echo -e "   ${GREEN}✓${NC} Zero-downtime deployments"
echo -e "   ${GREEN}✓${NC} Dynamic routing updates"
echo

# Show docker-compose integration
echo -e "${BLUE}━━━ Step 7: Docker Compose Integration ━━━${NC}"
echo
echo -e "${CYAN}📝 Example docker-compose.yml with Songbird:${NC}"
echo
cat <<'EOF' | sed 's/^/   /'
version: '3.8'
services:
  songbird:
    image: songbird:latest
    ports:
      - "8000:8000"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    environment:
      - SONGBIRD_DOCKER_DISCOVERY=true
  
  redis:
    image: redis:latest
    labels:
      - "songbird.capabilities=cache,key-value"
  
  postgres:
    image: postgres:15
    labels:
      - "songbird.capabilities=database,sql,relational"
  
  api:
    image: myapi:latest
    labels:
      - "songbird.capabilities=http,api,rest"
    depends_on:
      - songbird
    environment:
      - DATABASE_URL=songbird://database
      - CACHE_URL=songbird://cache
EOF
echo

echo -e "${GREEN}✨ All services route through Songbird!${NC}"
echo

# Summary
echo -e "${MAGENTA}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${MAGENTA}🐳 SONGBIRD + DOCKER = SMART CONTAINER ROUTING 🐳${NC}"
echo -e "${MAGENTA}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo -e "${CYAN}What you learned:${NC}"
echo -e "   ${GREEN}✓${NC} Songbird integrates with Docker socket"
echo -e "   ${GREEN}✓${NC} Automatic container discovery"
echo -e "   ${GREEN}✓${NC} Capability-based routing to containers"
echo -e "   ${GREEN}✓${NC} Load balancing across container replicas"
echo -e "   ${GREEN}✓${NC} Zero-downtime deployments"
echo -e "   ${GREEN}✓${NC} Works with docker-compose"
echo
echo -e "${CYAN}Real-world use cases:${NC}"
echo -e "   ${YELLOW}→${NC} Microservices mesh without Kubernetes"
echo -e "   ${YELLOW}→${NC} Development environment routing"
echo -e "   ${YELLOW}→${NC} Testing with multiple service versions"
echo -e "   ${YELLOW}→${NC} Production container orchestration"
echo
echo -e "${MAGENTA}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

# Cleanup
echo -e "${YELLOW}🧹 Stopping Songbird...${NC}"
kill $SONGBIRD_PID 2>/dev/null || true
sleep 1
echo -e "${GREEN}✅ Demo complete${NC}"
echo
echo -e "${BLUE}💡 Next steps:${NC}"
echo -e "   1. Start some Docker containers"
echo -e "   2. Run Songbird with Docker integration"
echo -e "   3. See automatic container discovery"
echo -e "   4. Route through Songbird instead of direct ports"
echo
echo -e "${MAGENTA}🎵 Songbird: Container-aware routing made easy! 🎵${NC}"
echo

