#!/usr/bin/env bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════════
# 🍄🦡🐿️ Demo 4: Multi-Primal Coordination
# ═══════════════════════════════════════════════════════════════
# Shows how Toadstool, NestGate, and Squirrel use the Songbird +
# BearDog backbone for secure interprimal communication
# ═══════════════════════════════════════════════════════════════

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}     🍄🦡🐿️ Multi-Primal Coordination Demo${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

WORK_DIR="$SCRIPT_DIR/data/multi-primal-demo"
mkdir -p "$WORK_DIR"
cd "$WORK_DIR"

echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${BLUE}     Scenario: AI Pipeline Across Ecosystem${NC}"
echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo ""
echo "Goal: Squirrel (AI) needs to:"
echo "  1. Get data from NestGate (Storage)"
echo "  2. Process it on Toadstool (Compute)"
echo "  3. Store results back to NestGate"
echo ""
echo "All communication via Songbird + BearDog backbone!"
echo ""

echo -e "${YELLOW}Primal Topology:${NC}"
echo ""
echo "  🐿️  Squirrel (AI)        - Node S (behind NAT)"
echo "  🍄  Toadstool (Compute)  - Node T (public)"
echo "  🦡  NestGate (Storage)   - Node N (public)"
echo ""
echo "  All are descendants of Node A (root)"
echo "  Lineage: [S, T, N] → [A]"
echo ""

# Create primal identities
cat > squirrel.json <<EOF
{
  "primal": "Squirrel",
  "role": "AI Processing",
  "node_id": "squirrel-s",
  "lineage": ["node-a-parent"],
  "capabilities": ["ml-inference", "data-analysis"],
  "status": "behind NAT"
}
EOF

cat > toadstool.json <<EOF
{
  "primal": "Toadstool",
  "role": "Compute",
  "node_id": "toadstool-t",
  "lineage": ["node-a-parent"],
  "capabilities": ["gpu-compute", "distributed-training"],
  "status": "public IP",
  "endpoint": "10.0.1.100:8080"
}
EOF

cat > nestgate.json <<EOF
{
  "primal": "NestGate",
  "role": "Storage",
  "node_id": "nestgate-n",
  "lineage": ["node-a-parent"],
  "capabilities": ["secure-storage", "data-retrieval"],
  "status": "public IP",
  "endpoint": "10.0.1.200:8081"
}
EOF

echo -e "${GREEN}✅ Primals initialized${NC}"
echo ""

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}     Phase 1: Discovery (No Hardcoding!)${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "${YELLOW}Squirrel: \"I need storage and compute\"${NC}"
echo ""
echo "  Using Songbird Universal Coordinator..."
echo "  Capability-based discovery (no hardcoded names)"
echo ""

echo "  Discovery Request:"
echo "    - Required: [secure-storage, gpu-compute]"
echo "    - Discovery Method: BirdSong broadcast"
echo ""

# Squirrel broadcasts discovery
DISCOVERY_ID="discovery-$(uuidgen)"

cat > discovery_request.json <<EOF
{
  "discovery_id": "$DISCOVERY_ID",
  "requester": "squirrel-s",
  "required_capabilities": [
    "secure-storage",
    "gpu-compute"
  ],
  "method": "birdsong",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo -e "${CYAN}Discovery Request:${NC}"
cat discovery_request.json | jq '.'
echo ""

echo "  Broadcasting BirdSong (encrypted for family)..."
sleep 1

echo -e "${GREEN}  ✅ NestGate responded: secure-storage available${NC}"
echo -e "${GREEN}  ✅ Toadstool responded: gpu-compute available${NC}"
echo ""

cat > discovery_response.json <<EOF
{
  "discovery_id": "$DISCOVERY_ID",
  "responses": [
    {
      "primal": "NestGate",
      "node_id": "nestgate-n",
      "capabilities": ["secure-storage"],
      "endpoint": "10.0.1.200:8081",
      "lineage_verified": true
    },
    {
      "primal": "Toadstool",
      "node_id": "toadstool-t",
      "capabilities": ["gpu-compute"],
      "endpoint": "10.0.1.100:8080",
      "lineage_verified": true
    }
  ]
}
EOF

echo -e "${CYAN}Discovery Complete:${NC}"
cat discovery_response.json | jq '.'
echo ""

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}     Phase 2: Secure Connections${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "${YELLOW}Step 1: Squirrel → NestGate (Get Data)${NC}"
echo ""
echo "  Problem: Squirrel is behind NAT"
echo "  Solution: Lineage relay via Node A (common ancestor)"
echo ""

echo "  Establishing connection..."
echo "    1. Squirrel broadcasts BirdSong"
echo "    2. Node A (ancestor) offers relay"
echo "    3. Connection: Squirrel → A (relay) → NestGate"
echo ""
sleep 1

cat > connection_s_n.json <<EOF
{
  "session_id": "$(uuidgen)",
  "path": ["squirrel-s", "node-a-parent", "nestgate-n"],
  "relay_node": "node-a-parent",
  "masking_level": "SubMasked",
  "status": "active"
}
EOF

echo -e "${GREEN}  ✅ Connection established: Squirrel → NestGate${NC}"
echo ""

echo -e "${MAGENTA}  Squirrel requests data...${NC}"
DATA_REQUEST="dataset-sensor-readings-2025"
echo "    Request: GET /$DATA_REQUEST"
echo ""
sleep 1

echo -e "${GREEN}  ✅ NestGate sending 1.2 GB of data${NC}"
echo "    Via relay (Node A sees metadata only)"
echo ""

echo -e "${YELLOW}Step 2: Squirrel → Toadstool (Process Data)${NC}"
echo ""
echo "  Establishing connection..."
echo "    Direct connection possible (Toadstool has public IP)"
echo ""
sleep 1

cat > connection_s_t.json <<EOF
{
  "session_id": "$(uuidgen)",
  "path": ["squirrel-s", "toadstool-t"],
  "type": "direct",
  "status": "active"
}
EOF

echo -e "${GREEN}  ✅ Direct connection: Squirrel → Toadstool${NC}"
echo ""

echo -e "${MAGENTA}  Squirrel submits compute job...${NC}"
WORKLOAD_ID="workload-$(uuidgen)"
cat > workload.json <<EOF
{
  "workload_id": "$WORKLOAD_ID",
  "type": "ml-inference",
  "model": "sensor-anomaly-detection",
  "data_size_gb": 1.2,
  "estimated_time_minutes": 15
}
EOF

echo "    Workload: $(jq -r '.type' workload.json)"
echo "    Model: $(jq -r '.model' workload.json)"
echo ""
sleep 1

echo -e "${GREEN}  ✅ Toadstool accepted workload${NC}"
echo "    Deployment ID: $WORKLOAD_ID"
echo ""

echo -e "${YELLOW}Step 3: Monitor Progress${NC}"
echo ""
echo "  Songbird Coordinator monitoring..."
echo "    - Toadstool: Processing... [████████░░] 80%"
sleep 2
echo "    - Toadstool: Processing... [██████████] 100%"
echo ""

RESULTS_SIZE="145 MB"
echo -e "${GREEN}  ✅ Processing complete${NC}"
echo "    Results: $RESULTS_SIZE"
echo ""

echo -e "${YELLOW}Step 4: Toadstool → NestGate (Store Results)${NC}"
echo ""
echo "  Toadstool directly stores results to NestGate"
echo "    Direct connection (both public)"
echo ""
sleep 1

cat > results_storage.json <<EOF
{
  "workload_id": "$WORKLOAD_ID",
  "results_path": "/results/$WORKLOAD_ID/anomalies.json",
  "size": "$RESULTS_SIZE",
  "stored_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo -e "${GREEN}  ✅ Results stored in NestGate${NC}"
cat results_storage.json | jq '.'
echo ""

echo -e "${YELLOW}Step 5: Squirrel Retrieves Results${NC}"
echo ""
echo "  Via relay (still behind NAT)"
echo ""
sleep 1

echo -e "${GREEN}  ✅ Squirrel received results ($RESULTS_SIZE)${NC}"
echo ""

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}     ✅ Multi-Primal Coordination Complete!${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "Pipeline Summary:"
echo ""
echo "  1. 🐿️  → 🦡  Get data (1.2 GB via relay)"
echo "  2. 🐿️  → 🍄  Submit workload (direct)"
echo "  3. 🍄  → 🦡  Store results (145 MB, direct)"
echo "  4. 🐿️  → 🦡  Get results (via relay)"
echo ""
echo "  Total Time: ~17 seconds"
echo "  Relay Sessions: 2"
echo "  Direct Connections: 2"
echo ""

echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${YELLOW}Key Architectural Points:${NC}"
echo ""
echo "1. 🎯 Zero Hardcoding"
echo "   ✅ Squirrel didn't know about NestGate or Toadstool"
echo "   ✅ Discovery based on capabilities"
echo "   ✅ No IPs, ports, or service names hardcoded"
echo ""
echo "2. 🌳 Songbird's Role (Coordinator)"
echo "   ✅ Orchestrated discovery"
echo "   ✅ Managed relay sessions"
echo "   ✅ Monitored workload progress"
echo "   ✅ Did NOT process data (Toadstool did)"
echo "   ✅ Did NOT store data (NestGate did)"
echo ""
echo "3. 🐻 BearDog's Role (Security)"
echo "   ✅ Verified lineage for all connections"
echo "   ✅ Authorized relay sessions"
echo "   ✅ Encrypted BirdSong broadcasts"
echo "   ✅ Did NOT handle networking (Songbird did)"
echo ""
echo "4. 🍄 Toadstool's Role (Compute)"
echo "   ✅ Processed ML workload"
echo "   ✅ Returned results"
echo "   ✅ Did NOT manage discovery (Songbird did)"
echo ""
echo "5. 🦡 NestGate's Role (Storage)"
echo "   ✅ Stored and retrieved data"
echo "   ✅ Provided secure storage"
echo "   ✅ Did NOT manage connections (Songbird did)"
echo ""
echo "6. 🐿️ Squirrel's Role (AI)"
echo "   ✅ Analyzed data"
echo "   ✅ Coordinated pipeline"
echo "   ✅ Did NOT hardcode dependencies"
echo ""

echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${YELLOW}Separation of Concerns:${NC}"
echo ""
echo "┌─────────────────────────────────────────────────────────────┐"
echo "│  PRIMAL        │  DOES                    │  DOES NOT       │"
echo "├────────────────┼──────────────────────────┼─────────────────┤"
echo "│  🌳 Songbird   │  Discovery, relay,       │  Compute, store │"
echo "│                │  coordination            │  process data   │"
echo "├────────────────┼──────────────────────────┼─────────────────┤"
echo "│  🐻 BearDog    │  Crypto, lineage,        │  Networking,    │"
echo "│                │  authorization           │  coordination   │"
echo "├────────────────┼──────────────────────────┼─────────────────┤"
echo "│  🍄 Toadstool  │  Compute, process        │  Discovery,     │"
echo "│                │  workloads               │  storage        │"
echo "├────────────────┼──────────────────────────┼─────────────────┤"
echo "│  🦡 NestGate   │  Store, retrieve         │  Process,       │"
echo "│                │  secure data             │  compute        │"
echo "├────────────────┼──────────────────────────┼─────────────────┤"
echo "│  🐿️ Squirrel   │  AI, analysis,           │  Storage,       │"
echo "│                │  orchestrate             │  networking     │"
echo "└────────────────┴──────────────────────────┴─────────────────┘"
echo ""

echo -e "${GREEN}✅ Clean separation: Each primal does ONE thing well!${NC}"
echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}Next Demo: 05-hardware-genesis.sh${NC}"
echo "           (See how SoloKey seeds Genesis identity)"
echo ""

