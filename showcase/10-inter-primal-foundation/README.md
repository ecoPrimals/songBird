# Showcase 10: Inter-Primal Foundation

**Status:** ✅ Phase 1-2 Complete (Registration + Discovery)  
**Principle:** Songbird as Universal Port Authority  
**Architecture:** Zero compile-time dependencies, pure runtime interaction

## Overview

This showcase demonstrates the **foundational pattern for inter-primal communication** in the ecoPrimals ecosystem:

- **Universal Port Authority**: Songbird assigns ALL ports (no primal binds its own)
- **Capability-Based Discovery**: Primals discover each other by capability, not name
- **Each Primal Knows Only Itself**: Zero compile-time dependencies between primals
- **Runtime Integration**: All interaction happens at runtime via HTTP/JSON

## Key Insight

> "Once other primals understand how to interact with Songbird, they will never set another port themselves"

This provides:
- ✅ Zero port conflicts
- ✅ Automatic service discovery
- ✅ Zero configuration
- ✅ Infinite scalability

## Demos

### 01. Toadstool Staging Concept ✅
**File:** `01-toadstool-staging-concept.sh`  
**Type:** Staging visualization  
**Status:** Complete

**What it shows:**
- How Toadstool *would* discover Songbird
- How registration *would* work
- What the end-state looks like

**Run:**
```bash
./01-toadstool-staging-concept.sh
```

### 02. Toadstool Live Integration ✅
**File:** `02-toadstool-live-integration.sh`  
**Type:** Live runtime demo  
**Status:** Complete

**What it demonstrates:**
1. Toadstool discovers Songbird (`GET /api/v1/info`)
2. Toadstool registers capabilities (`POST /api/v1/services/register`)
3. Songbird assigns port dynamically (8091-8200)
4. Toadstool sends heartbeat (`POST /api/v1/services/{id}/heartbeat`)
5. Songbird tracks service health

**Architecture compliance:**
- ✅ No compile-time deps (pure HTTP/JSON)
- ✅ Capability-based (discovers by "service_registry" capability)
- ✅ Dynamic port (assigned by Songbird)
- ✅ Zero hardcoding (no "Songbird" string in Toadstool)

**Prerequisites:**
- Songbird running on localhost:8080
- `curl` and `jq` installed

**Run:**
```bash
./02-toadstool-live-integration.sh
```

**Output:** Service ID, assigned port, heartbeat interval

### 03. Task Routing Demo ✅
**File:** `03-task-routing-demo.sh`  
**Type:** End-to-end flow demo  
**Status:** Complete (conceptual routing)

**What it demonstrates:**
1. User submits task to Songbird
2. Songbird queries for "compute" capability
3. Songbird finds Toadstool
4. (Future) Songbird routes task to Toadstool
5. (Future) Toadstool executes, returns results

**Current status:**
- ✅ Service discovery working
- ✅ Capability query working
- ⏳ Task execution (Phase 4)

**Prerequisites:**
- Songbird running
- Toadstool registered (run `02-` first)

**Run:**
```bash
./03-task-routing-demo.sh
```

## Architecture Pattern

```
USER
  ↓
SONGBIRD (localhost:8080) ← Port Authority
  ↓ (capability: "compute")
TOADSTOOL (assigned port, e.g., 8091)
```

**Key Insight:** User only knows Songbird. Songbird handles all routing.

## Protocol Flow

### 1. Discovery
```http
GET https://localhost:8080/api/v1/info
```
Response:
```json
{
  "name": "Songbird",
  "capabilities": ["service_registry", "federation", ...],
  "protocols": ["https", "tarpc", "jsonrpc"]
}
```

### 2. Registration
```http
POST https://localhost:8080/api/v1/services/register
Content-Type: application/json

{
  "primal_name": "Toadstool",
  "capabilities": [{"name": "compute", "type": "execution", ...}],
  "protocols": ["https"]
}
```
Response:
```json
{
  "service_id": "abc-123",
  "assigned_endpoint": {"port": 8091, "protocol": "https"},
  "token": "xyz-789",
  "heartbeat_interval_sec": 30
}
```

### 3. Heartbeat
```http
POST https://localhost:8080/api/v1/services/abc-123/heartbeat
Content-Type: application/json

{
  "service_id": "abc-123",
  "token": "xyz-789",
  "status": "operational"
}
```

### 4. Query by Capability
```http
GET https://localhost:8080/api/v1/services/query/compute
```
Response:
```json
{
  "capability": "compute",
  "services": [
    {"service_name": "Toadstool", "assigned_endpoint": {"port": 8091}, ...}
  ],
  "count": 1
}
```

## Success Criteria

✅ **Achieved (Phase 1-2):**
- Toadstool discovers Songbird without hardcoding
- Registration works (port assignment)
- Heartbeat maintains connection
- Capability query finds services
- **Zero compile-time dependencies**

⏳ **Next (Phase 4):**
- Task routing implementation
- Load balancing
- Result caching
- Multi-tower federation

## Universal Pattern

This pattern applies to **ALL** inter-primal communication:

- **Toadstool** (compute) → registers "compute" capability
- **BearDog** (security) → registers "encryption", "validation"
- **Nestgate** (storage) → registers "storage", "persistence"
- **Squirrel** (AI-MCP) → registers "ai", "mcp_server"

**All use the same protocol. Zero special cases.**

## Architecture Compliance

✅ **Each Primal Knows Only Itself**
- Toadstool doesn't import Songbird code
- Songbird doesn't import Toadstool code
- Pure runtime interaction via HTTP

✅ **Capability-Based Discovery**
- No hardcoded service names
- Discover by capability, not identity
- Infinite extensibility

✅ **Universal Port Authority**
- Songbird assigns ALL ports
- Zero port conflicts
- Dynamic allocation

## Implementation Status

| Component | Status |
|-----------|--------|
| **songbird-primal-sdk** | ✅ Complete (registration protocol) |
| **Service Registry** | ✅ Complete (port allocation, tracking) |
| **HTTP API Endpoints** | ✅ Complete (7 endpoints) |
| **Live Demos** | ✅ Complete (3 demos) |
| **Task Routing** | ⏳ Phase 4 |
| **Multi-Tower Federation** | ⏳ Phase 5 |

## Next Steps

1. **Phase 4**: Implement task routing in Songbird compute API
2. **BearDog Integration**: Same pattern for security services
3. **Nestgate Integration**: Same pattern for storage
4. **Squirrel Integration**: Same pattern for AI/MCP
5. **Federation**: Deploy across multiple towers

---

*Universal Port Authority - December 20, 2025*  
*Each Primal Knows Only Itself. Network Effects Emerge.*  
*Zero Compile-Time Dependencies. Pure Runtime Interaction.*
