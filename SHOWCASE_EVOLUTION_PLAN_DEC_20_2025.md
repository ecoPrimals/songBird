# Songbird Showcase Evolution Plan

**Date:** December 20, 2025  
**Context:** Post-Federation Verification & Privacy Boundaries Documentation  
**Key Insight:** "Once other primals understand how to interact with Songbird, they will never set another port themselves"

---

## 🎯 The Universal Port Authority Principle

### Core Architectural Insight

**Songbird is the ecosystem's port authority:**
- Songbird owns port discovery and management
- Other primals **register** with Songbird, never bind ports themselves
- Federation becomes the universal service mesh
- Zero-config discovery for all primals

**Current State:**
- ✅ Songbird federation operational (3 towers verified)
- ✅ Multi-interface coalescence working
- ✅ Identity-based routing complete
- ✅ Privacy boundaries documented
- ✅ Singleton enforcement active

**What We Need:**
Live demos showing:
1. Songbird federation (tower-to-tower)
2. Songbird local task spawning (on-tower compute)
3. Inter-primal communication (Songbird ↔ Toadstool/BearDog/Nestgate/Squirrel)

---

## 📊 Current Showcase Status Across Primals

### Songbird (`/showcase/`)

**Strengths:**
- ✅ 02-federation: Multi-machine setup, mesh formation (LIVE)
- ✅ 03-inter-primal: Ecosystem discovery demos (SIMULATED)
- ✅ 04-multi-protocol: Protocol escalation (LIVE)
- ✅ 05-albatross-multiplex: tarpc benchmarks (LIVE)
- ✅ 06-toadstool-ml-orchestration: ML routing (SIMULATED)
- ✅ 07-student-onboarding: Zero-config student experience (READY)

**Gaps:**
- ⚠️  Inter-primal demos are simulated (not wired to real services)
- ⚠️  No live Toadstool integration
- ⚠️  No live BearDog integration
- ⚠️  Local task spawning examples limited

### Toadstool (`../toadstool/showcase/`)

**Strengths:**
- ✅ GPU demos (CUDA, OpenCL, Metal abstraction)
- ✅ ML inference working (real models)
- ✅ Python/Rust runtime abstraction
- ✅ Biome configuration (declarative compute)

**Gaps:**
- ⚠️  `inter-primal/01-songbird-distributed-compute.sh` - EXISTS but not wired
- ⚠️  Demos expect manual port config (not Songbird-managed)
- ⚠️  No service registration with Songbird
- ⚠️  Hardcoded endpoints

### BearDog (`../beardog/showcase/`)

**Strengths:**
- ✅ Genetic encryption working
- ✅ Human entropy collection
- ✅ Hardware HSM integration
- ✅ Constraint-based crypto

**Gaps:**
- ⚠️  `03-songbird-integration/` - DOCUMENTED but not wired
- ⚠️  No live trust verification demo
- ⚠️  No live Songbird → BearDog → Toadstool flow
- ⚠️  Crypto verification simulated

### Nestgate (`../nestgate/showcase/`)

**Strengths:**
- ✅ ZFS storage working
- ✅ Data service API ready
- ✅ Replication working
- ✅ Real-world scenarios (ML, media, bioinfo)

**Gaps:**
- ⚠️  `02_ecosystem_integration/02_songbird_data_service/` - EXISTS but not wired
- ⚠️  `04_inter_primal_mesh/01_songbird_coordination/` - PLANNED but not built
- ⚠️  No live Songbird registration
- ⚠️  Manual endpoint configuration

### Squirrel (`../squirrel/showcase/`)

**Strengths:**
- ✅ MCP server working
- ✅ Multi-provider AI routing (Ollama, OpenAI, Claude)
- ✅ Cost optimization
- ✅ Hybrid routing (local + cloud)

**Gaps:**
- ⚠️  `demos/04-inter-primal/` - EXISTS but simulated
- ⚠️  `demos/06-cross-tower-mesh/` - PARTIALLY WIRED
- ⚠️  No live Songbird discovery
- ⚠️  Manual endpoint management

---

## 🚀 Showcase Evolution Roadmap

### Phase 1: Foundation Demos (Local Tower) ✅ READY

**Goal:** Show Songbird as local orchestrator

**Demos to Create:**

1. **`showcase/09-local-compute/01-spawn-simple-task.sh`**
   ```bash
   # Show: Songbird spawning local compute task
   # Demonstrates: Local task lifecycle
   # Uses: Songbird's execution agent (no Toadstool yet)
   
   POST /api/v1/compute/task
   {
     "code": "echo 'Hello from Songbird'",
     "runtime": "shell"
   }
   ```

2. **`showcase/09-local-compute/02-python-task.sh`**
   ```bash
   # Show: Songbird running Python locally
   # Demonstrates: Runtime abstraction
   # Uses: Local Python interpreter
   
   POST /api/v1/compute/task
   {
     "code": "print('ML task:', 2+2)",
     "runtime": "python"
   }
   ```

3. **`showcase/09-local-compute/03-resource-limits.sh`**
   ```bash
   # Show: Songbird enforcing resource limits
   # Demonstrates: Resource management
   # Uses: cgroups or similar
   
   POST /api/v1/compute/task
   {
     "code": "stress-ng --cpu 8 --timeout 60",
     "resources": {"max_cpu": "50%"}
   }
   ```

**Status:** Code exists, needs extraction into clean demos

### Phase 2: Federation Demos (Cross-Tower) ✅ MOSTLY READY

**Goal:** Show Songbird federation magic

**Demos to Enhance:**

1. **`showcase/02-federation/demos/01-mesh-formation.sh`** ✅
   - Already working
   - 3-tower federation verified
   - Clean up output, add commentary

2. **`showcase/02-federation/demos/05-cross-tower-task-routing.sh`** (NEW)
   ```bash
   # Show: Task submitted to Eastgate, executed on Westgate
   # Demonstrates: Intelligent routing based on resources
   # Uses: Federation API + graduated disclosure
   
   # Submit to Eastgate:
   POST https://eastgate:8080/api/v1/compute/task
   {
     "code": "nvidia-smi",  # Requires GPU
     "prefer_tower": "any"   # Let Songbird decide
   }
   
   # Songbird sees:
   # - Eastgate: No GPU available
   # - Westgate: H100 GPU available
   # - Routes to Westgate automatically
   # - Returns result to Eastgate
   ```

3. **`showcase/02-federation/demos/06-graduated-disclosure.sh`** (NEW)
   ```bash
   # Show: Trust levels in action
   # Demonstrates: Privacy boundaries
   # Uses: Trust escalation system
   
   # At trust level 0 (anonymous):
   GET https://westgate:8080/api/federation/nodes
   # Returns: capabilities only
   
   # At trust level 3 (identity-verified):
   GET https://westgate:8080/api/federation/nodes
   # Returns: capabilities + identities + hostnames
   ```

**Status:** Federation working, needs showcase polish

### Phase 3: Inter-Primal Wiring (CRITICAL) ⚠️ NEEDS WORK

**Goal:** Wire Songbird to real primal services

**Priority 1: Songbird ↔ Toadstool**

**What Exists:**
- Toadstool has working GPU compute
- Toadstool has REST API
- Songbird has task routing
- `showcase/06-toadstool-ml-orchestration/` (simulated)

**What's Missing:**
1. Toadstool doesn't register with Songbird (manual ports)
2. No service discovery protocol
3. No capability advertisement

**Solution:**

```rust
// toadstool/crates/server/src/main.rs

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Start Toadstool server (NO PORT BINDING)
    let toadstool = ToadstoolServer::new().await?;
    
    // 2. Register with Songbird
    let songbird_client = SongbirdClient::discover_local().await?;
    
    let registration = ServiceRegistration {
        service_name: "Toadstool",
        service_type: ServiceType::Compute,
        capabilities: vec![
            "python".to_string(),
            "rust".to_string(),
            "gpu".to_string(),
            "cuda".to_string(),
        ],
        // NO PORT - Songbird assigns
    };
    
    let assigned_endpoint = songbird_client
        .register_service(registration)
        .await?;
    
    // 3. Bind to Songbird-assigned endpoint
    toadstool.bind(assigned_endpoint).await?;
    
    // 4. Run
    toadstool.serve().await?;
}
```

**Demo to Create:**

```bash
# showcase/10-inter-primal/01-toadstool-local.sh

# 1. Start Songbird
./target/release/songbird-orchestrator &

# 2. Start Toadstool (registers automatically)
cd ../toadstool
./target/release/toadstool-server &
# Logs: "Registered with Songbird at port 8091"

# 3. Songbird now knows about Toadstool
curl https://localhost:8080/api/federation/services | jq
# Shows:
# {
#   "services": [
#     {
#       "name": "Toadstool",
#       "type": "compute",
#       "capabilities": ["python", "gpu"],
#       "endpoint": "https://localhost:8091"
#     }
#   ]
# }

# 4. Submit GPU task to Songbird
curl -X POST https://localhost:8080/api/v1/compute/task \
  -d '{
    "code": "import torch; print(torch.cuda.is_available())",
    "runtime": "python",
    "requires": ["gpu"]
  }'

# 5. Songbird routes to Toadstool
# Logs: "Routing task to Toadstool (GPU capability)"

# 6. Task executes on Toadstool, returns to Songbird
# Result: "True" (GPU available)
```

**Priority 2: Songbird ↔ BearDog**

**Demo:**

```bash
# showcase/10-inter-primal/02-beardog-trust.sh

# 1. Start Songbird
./target/release/songbird-orchestrator &

# 2. Start BearDog (registers as security primal)
cd ../beardog
./target/release/beardog &
# Logs: "Registered with Songbird as trust provider"

# 3. Songbird discovers BearDog
curl https://localhost:8080/api/federation/services | jq
# Shows: BearDog with "trust_verification" capability

# 4. Task requires trust verification
curl -X POST https://localhost:8080/api/v1/compute/task \
  -d '{
    "code": "sensitive_operation",
    "requires_trust_level": "IdentityVerified"
  }'

# 5. Songbird asks BearDog: "Can I trust this?"
# BearDog: "Trust level: IdentityVerified ✓"

# 6. Songbird executes task
```

**Priority 3: Songbird ↔ Nestgate**

**Demo:**

```bash
# showcase/10-inter-primal/03-nestgate-storage.sh

# 1. Start Songbird + Nestgate
./target/release/songbird-orchestrator &
cd ../nestgate && ./target/release/nestgate &

# 2. Task generates large dataset
curl -X POST https://localhost:8080/api/v1/compute/task \
  -d '{
    "code": "generate_large_dataset()",
    "store_result": true
  }'

# 3. Songbird routes storage to Nestgate
# Logs: "Storing result via Nestgate (140GB dataset)"

# 4. Nestgate deduplicates and compresses
# Result: "Stored at nestgate://dataset-abc123 (actual: 42GB)"
```

**Priority 4: Songbird ↔ Squirrel**

**Demo:**

```bash
# showcase/10-inter-primal/04-squirrel-intent.sh

# 1. Start Songbird + Squirrel
./target/release/songbird-orchestrator &
cd ../squirrel && ./target/release/squirrel &

# 2. User submits natural language task
curl -X POST https://localhost:8080/api/v1/compute/task \
  -d '{
    "intent": "Train a model to classify cats and dogs",
    "data": "pets_dataset.zip"
  }'

# 3. Songbird asks Squirrel: "What does this mean?"
# Squirrel: "Needs: Python, GPU, 8GB RAM, ML framework"

# 4. Songbird routes based on Squirrel's analysis
# Logs: "Routing to GPU tower (Squirrel suggests: Westgate)"
```

### Phase 4: Full Ecosystem Demo 🎯 ULTIMATE GOAL

**Demo: `showcase/11-full-ecosystem/distributed-ml-training.sh`**

```bash
#!/bin/bash
# Full ecoPrimals Ecosystem Demonstration
# Shows all 5 primals working together

echo "🎵 Starting ecoPrimals Ecosystem..."

# 1. Start all primals
echo "Starting Songbird (orchestrator)..."
./target/release/songbird-orchestrator &

echo "Starting Toadstool (compute)..."
cd ../toadstool && ./target/release/toadstool-server &

echo "Starting BearDog (security)..."
cd ../beardog && ./target/release/beardog &

echo "Starting Nestgate (storage)..."
cd ../nestgate && ./target/release/nestgate &

echo "Starting Squirrel (AI routing)..."
cd ../squirrel && ./target/release/squirrel &

sleep 10

# 2. Check ecosystem health
echo "🌐 Checking ecosystem..."
curl https://localhost:8080/api/ecosystem/status | jq

# Should show:
# {
#   "primals": [
#     {"name": "Songbird", "status": "operational", "role": "orchestrator"},
#     {"name": "Toadstool", "status": "operational", "role": "compute"},
#     {"name": "BearDog", "status": "operational", "role": "security"},
#     {"name": "Nestgate", "status": "operational", "role": "storage"},
#     {"name": "Squirrel", "status": "operational", "role": "ai_routing"}
#   ],
#   "network_effects": ["distributed_compute", "trust_verification", "intelligent_routing"]
# }

# 3. Submit complex distributed ML task
echo "📊 Submitting distributed ML training..."
curl -X POST https://localhost:8080/api/v1/ml/train \
  -d '{
    "intent": "Train ResNet-50 on ImageNet",
    "dataset": "imagenet_100gb.tar",
    "distribute": true,
    "encrypt": true
  }'

# Behind the scenes:
# 1. Squirrel: "This needs 3 GPU towers, 100GB storage, and encryption"
# 2. Songbird: "I have Westgate (H100), Strandgate (RTX 4090), Eastgate (RTX 3090)"
# 3. BearDog: "All towers verified at trust level 3"
# 4. Nestgate: "Dataset already cached, deduplicated to 42GB"
# 5. Toadstool (all towers): "Executing partition 1/2/3"
# 6. Songbird: "Aggregating results..."
# 7. User: "Training complete! Model: 94.2% accuracy"

# Network effects achieved:
# ✅ 3x faster (distributed across towers)
# ✅ Encrypted (BearDog)
# ✅ 58% storage saved (Nestgate dedup)
# ✅ Intelligent routing (Squirrel + Songbird)
# ✅ Zero manual configuration
```

---

## 🎯 Immediate Next Steps

### Step 1: Create Songbird Demos (Showcase 09 & 10)

**Priority: HIGH (Foundation for inter-primal)**

**Create:**
1. `showcase/09-local-compute/`
   - 01-spawn-simple-task.sh
   - 02-python-task.sh
   - 03-resource-limits.sh
   - 04-task-lifecycle.sh (pause/resume/cancel)
   - README.md

2. `showcase/10-inter-primal-foundation/`
   - 01-service-discovery.sh (conceptual)
   - 02-capability-advertisement.sh (conceptual)
   - 03-port-authority-principle.sh (architectural demo)
   - README.md

**Time:** 2-3 hours  
**Dependencies:** None (uses existing code)  
**Outcome:** Foundation for inter-primal wiring

### Step 2: Wire Toadstool to Songbird

**Priority: HIGH (Proves the principle)**

**Tasks:**
1. Add Songbird client to Toadstool
2. Implement service registration on Toadstool startup
3. Make Toadstool listen on Songbird-assigned port
4. Test registration → discovery → routing flow
5. Create live demo: `showcase/10-inter-primal/01-toadstool-live.sh`

**Code Changes:**
- `toadstool/crates/server/src/main.rs` (add registration)
- `toadstool/Cargo.toml` (add songbird-client)
- Create `toadstool/crates/songbird-integration/` (new crate)

**Time:** 4-6 hours  
**Dependencies:** Songbird client library (may need to create)  
**Outcome:** First live inter-primal demo!

### Step 3: Document the Pattern

**Priority: MEDIUM (Others will follow)**

**Create:**
1. `specs/UNIVERSAL_PORT_AUTHORITY_SPEC.md`
   - The principle
   - Registration protocol
   - Discovery protocol
   - Reference implementation (Toadstool)

2. `docs/guides/INTER_PRIMAL_INTEGRATION_GUIDE.md`
   - How any primal registers with Songbird
   - Code examples for each primal
   - Testing checklist

**Time:** 2-3 hours  
**Dependencies:** Step 2 complete (reference impl)  
**Outcome:** Blueprint for BearDog, Nestgate, Squirrel

### Step 4: Wire Remaining Primals (Parallel)

**Priority: MEDIUM (Can be done by primal teams)**

**BearDog Team:**
- Follow integration guide
- Add Songbird registration to BearDog
- Create showcase demo

**Nestgate Team:**
- Follow integration guide
- Add Songbird registration to Nestgate
- Create showcase demo

**Squirrel Team:**
- Follow integration guide
- Add Songbird registration to Squirrel
- Create showcase demo

**Time:** 3-4 hours per primal  
**Dependencies:** Step 3 complete (guide)  
**Outcome:** Full ecosystem wired

### Step 5: Full Ecosystem Demo

**Priority: LOW (Grand finale)**

**Create:**
1. `showcase/11-full-ecosystem/`
   - Start script (all 5 primals)
   - Distributed ML training demo
   - Network effects showcase
   - Visual output (real-time status)

**Time:** 4-6 hours  
**Dependencies:** All primals wired  
**Outcome:** Ultimate demo for presentations

---

## 📊 Current Capabilities Matrix

| Primal | Has Showcase | Has API | Has Registration | Songbird Integration | Status |
|--------|--------------|---------|------------------|---------------------|--------|
| **Songbird** | ✅ Yes | ✅ Yes | N/A (orchestrator) | ✅ Self | ✅ Ready |
| **Toadstool** | ✅ Yes (GPU) | ✅ Yes (REST) | ❌ No | ❌ Simulated | ⚠️  Needs wiring |
| **BearDog** | ✅ Yes (crypto) | ✅ Yes (gRPC) | ❌ No | ❌ Documented only | ⚠️  Needs wiring |
| **Nestgate** | ✅ Yes (storage) | ✅ Yes (REST) | ❌ No | ❌ Planned | ⚠️  Needs wiring |
| **Squirrel** | ✅ Yes (AI) | ✅ Yes (MCP) | ❌ No | ❌ Simulated | ⚠️  Needs wiring |

---

## 🎯 Success Criteria

### Phase 1 (Foundation) ✅
- [x] Songbird federation operational
- [x] Local task spawning demos created
- [x] Privacy boundaries documented

### Phase 2 (Single Primal) 🎯 CURRENT TARGET
- [ ] Toadstool registers with Songbird
- [ ] Songbird routes task to Toadstool
- [ ] Live demo working end-to-end
- [ ] Integration guide published

### Phase 3 (Multi-Primal)
- [ ] BearDog integrated
- [ ] Nestgate integrated
- [ ] Squirrel integrated
- [ ] Each has working demo

### Phase 4 (Ecosystem)
- [ ] All 5 primals running together
- [ ] Distributed ML training demo
- [ ] Network effects demonstrated
- [ ] Zero manual configuration

---

## 💡 Key Architectural Insights

### 1. Port Authority Principle

**Old Way:**
```
Toadstool: Binds to port 8091 (hardcoded)
BearDog: Binds to port 8092 (hardcoded)
Nestgate: Binds to port 8093 (hardcoded)
Squirrel: Binds to port 8094 (hardcoded)
```

**Problems:**
- Port conflicts on multi-primal towers
- No service discovery
- Manual configuration required
- Doesn't scale

**New Way (Universal Port Authority):**
```
Songbird: Orchestrator (port 8080)
  ↓
  Discovers available ports
  ↓
Toadstool: Registers with Songbird
  ← Songbird assigns port 8091
  ↓
BearDog: Registers with Songbird
  ← Songbird assigns port 8092
  ↓
All communication routed through Songbird
Zero manual configuration
```

**Benefits:**
- ✅ No port conflicts (Songbird manages)
- ✅ Service discovery automatic
- ✅ Zero configuration
- ✅ Scales to any number of primals
- ✅ Federation becomes service mesh

### 2. Graduated Service Discovery

**Level 0 (Anonymous):**
```
Toadstool registers: "I can compute"
Songbird sees: "A compute service is available"
```

**Level 3 (Identity-Verified):**
```
Toadstool registers: "I am Toadstool on Eastgate"
Songbird sees: "Toadstool (Eastgate) has GPU, Python, Rust"
```

**Level 4 (Hardware-Verified):**
```
Toadstool registers: Full hardware specs
Songbird sees: "RTX 3090, 24GB VRAM, 128GB RAM, NVMe"
```

### 3. Network Effects Without Hardcoding

**Each primal knows ONLY itself:**
- Toadstool: "I execute code"
- BearDog: "I verify trust"
- Nestgate: "I store data"
- Squirrel: "I route intelligently"

**Songbird knows the ecosystem:**
- "I have a compute service (Toadstool)"
- "I have a trust service (BearDog)"
- "I have a storage service (Nestgate)"
- "I have an AI service (Squirrel)"

**Magic happens:**
User submits task → Songbird orchestrates across all primals → Network effects emerge

---

## 🚀 Next Session Plan

**Session Goal:** Create Showcase 09 & 10, Wire Toadstool

**Tasks:**
1. Create `showcase/09-local-compute/` (4 demos)
2. Create `showcase/10-inter-primal-foundation/` (3 demos)
3. Design Songbird registration protocol
4. Add `songbird-client` crate to Toadstool
5. Implement registration in Toadstool
6. Create first live inter-primal demo
7. Document the pattern

**Time Estimate:** 6-8 hours  
**Expected Outcome:** Toadstool fully wired to Songbird

---

**Status:** Planning Complete - Ready to Execute  
**Next:** Build Showcase 09 & 10 (foundation demos)  
**Vision:** Universal Port Authority + Zero-Config Ecosystem

