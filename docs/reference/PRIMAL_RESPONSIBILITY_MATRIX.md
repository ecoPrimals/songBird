# 🐦 ecoPrimals Responsibility Matrix

**Last Updated**: November 10, 2025  
**Context**: Post-Intelligent Routing Implementation

---

## 🎯 **Executive Summary**

The ecoPrimals ecosystem is a **capability-based distributed system** where each primal has **clear, focused responsibilities**. This document defines the **canonical role** of each primal and **when to use which**.

**Key Principle**: **Capability-based routing**, not hardcoded dependencies.

---

## 🧬 **The Six Active Primals**

### **1. 🐦 Songbird - Federation & Orchestration**

**Role**: **Distributed service coordination and intelligent task routing**

**Handles**:
- ✅ Service discovery and federation across towers/nodes
- ✅ Intelligent task routing based on complexity and capabilities
- ✅ Lightweight command execution (via Execution Agent)
- ✅ Health monitoring and metrics aggregation
- ✅ HTTP API for task submission
- ✅ Peer-to-peer coordination

**Does NOT Handle**:
- ❌ Heavy compute workloads (→ Toadstool)
- ❌ Security/authentication (→ BearDog)
- ❌ Data storage (→ NestGate)
- ❌ AI model management (→ Squirrel)

**Example Use Cases**:
- "Find all available compute nodes"
- "Route this task to the best available service"
- "Execute this lightweight shell command remotely"
- "Health check all federated services"

**API Endpoints**:
```
POST /api/v1/compute/task         # Submit task for intelligent routing
GET  /api/v1/compute/task/:id     # Get task status
GET  /health                       # Health check
POST /api/v1/federation/register  # Register service
```

**Integration Pattern**:
```rust
// Songbird routes TO other capabilities
let task = Task {
    task_type: "ml_training",
    resource_requirements: ResourceRequirements {
        gpu_required: true,
        memory_mb: 8192,
        ...
    },
    ...
};

// Songbird's router analyzes complexity and routes to Toadstool
let result = songbird.compute_api.submit_task(task).await?;
```

---

### **2. 🍄 Toadstool - Universal Compute Platform**

**Role**: **Heavy workload execution across diverse substrates**

**Handles**:
- ✅ GPU-accelerated compute (CUDA, OpenCL)
- ✅ Distributed ML training (PyTorch DDP, TensorFlow)
- ✅ Container execution (Docker, containerd)
- ✅ WASM runtime execution
- ✅ Native code execution (sandboxed)
- ✅ Python runtime execution
- ✅ Live workload migration
- ✅ Multi-substrate orchestration

**Does NOT Handle**:
- ❌ Service discovery (→ Songbird)
- ❌ Lightweight shell commands (→ Songbird Execution Agent)
- ❌ Data storage orchestration (→ NestGate)
- ❌ Security operations (→ BearDog)

**Example Use Cases**:
- "Train this ResNet-50 model on ImageNet-100"
- "Run this Python script with GPU acceleration"
- "Execute this containerized workload"
- "Migrate this running workload to another node"

**API Pattern**:
```rust
// Toadstool receives tasks FROM Songbird's intelligent router
let workload = ToadstoolWorkload {
    workload_type: WorkloadType::GpuCompute,
    script_path: "/path/to/train.py",
    resources: ResourceRequirements {
        gpu_count: 1,
        memory_gb: 16,
    },
    ...
};

toadstool.execute_workload(workload).await?;
```

**Songbird Integration**:
```rust
// When Songbird detects a HEAVY task with GPU requirements:
if task.complexity == TaskComplexity::Heavy && task.requires_gpu() {
    // Route to Toadstool capability
    let toadstool = registry.find_capability("compute_gpu").await?;
    toadstool.execute(task).await?;
}
```

---

### **3. 🐻 BearDog - Security & Privacy Infrastructure**

**Role**: **Sovereign security, authentication, and HSM operations**

**Handles**:
- ✅ Authentication and authorization
- ✅ HSM operations (FIDO2, PKCS#11, StrongBox, Secure Enclave)
- ✅ Cryptographic operations (quantum-resistant)
- ✅ Human-owned entropy generation
- ✅ Token management (JWT, OAuth)
- ✅ Audit and compliance
- ✅ Identity management

**Does NOT Handle**:
- ❌ Compute orchestration (→ Songbird/Toadstool)
- ❌ Data storage (→ NestGate)
- ❌ AI coordination (→ Squirrel)

**Example Use Cases**:
- "Authenticate this user via FIDO2"
- "Sign this payload with HSM"
- "Generate secure entropy"
- "Validate this JWT token"

**Integration Pattern**:
```rust
// All primals consult BearDog for security
let security = registry.find_capability("security").await?;
let auth_result = security.authenticate(credentials).await?;

if auth_result.authorized {
    // Proceed with operation
}
```

---

### **4. 🐿️ Squirrel - Universal AI Coordination**

**Role**: **AI model management and coordination**

**Handles**:
- ✅ AI model registry
- ✅ Model versioning and lifecycle
- ✅ AI capability discovery
- ✅ Model serving coordination
- ✅ AI inference routing
- ✅ Model metadata management

**Does NOT Handle**:
- ❌ Actual model training (→ Toadstool)
- ❌ GPU execution (→ Toadstool)
- ❌ General compute (→ Toadstool)
- ❌ Data storage (→ NestGate)

**Example Use Cases**:
- "Register this trained model"
- "Find the best model for image classification"
- "Route this inference request"
- "Update model metadata"

**Integration Pattern**:
```rust
// Squirrel coordinates AI, Toadstool executes
let model = squirrel.find_model("resnet50", "latest").await?;
let training_task = Task {
    model: model.metadata,
    dataset: "imagenet100",
    ...
};

// Squirrel routes to Toadstool for actual execution
let toadstool = registry.find_capability("gpu_compute").await?;
toadstool.train_model(training_task).await?;
```

---

### **5. 🏗️ NestGate - Storage & Network Gateway**

**Role**: **Data persistence and network operations**

**Handles**:
- ✅ ZFS operations
- ✅ Filesystem management
- ✅ Object storage (S3-compatible)
- ✅ Network gateway operations
- ✅ Data replication
- ✅ Backup and restore
- ✅ Storage tiering

**Does NOT Handle**:
- ❌ Compute orchestration (→ Songbird/Toadstool)
- ❌ Security operations (→ BearDog)
- ❌ AI coordination (→ Squirrel)

**Example Use Cases**:
- "Store this dataset in ZFS"
- "Retrieve this file from object storage"
- "Replicate this data to backup node"
- "Create network gateway for this subnet"

**Integration Pattern**:
```rust
// Other primals use NestGate for storage
let storage = registry.find_capability("storage").await?;
storage.store_file("/path/to/model.pth", model_data).await?;
```

---

### **6. 🧬 biomeOS - Ecosystem Orchestrator**

**Role**: **Overall ecosystem lifecycle and primal coordination**

**Handles**:
- ✅ Primal lifecycle management (start, stop, restart)
- ✅ Cross-primal dependency resolution
- ✅ Ecosystem-wide health monitoring
- ✅ Resource allocation across primals
- ✅ Primal discovery and registration
- ✅ Configuration management
- ✅ Biome (environment) orchestration

**Does NOT Handle**:
- ❌ Specific workload execution (→ Toadstool)
- ❌ Task routing (→ Songbird)
- ❌ Security operations (→ BearDog)

**Example Use Cases**:
- "Start all primals for the 'ai-research' biome"
- "Ensure BearDog is running before starting other primals"
- "Monitor overall ecosystem health"
- "Coordinate primal updates"

**Integration Pattern**:
```rust
// biomeOS manages the lifecycle
biome_os.start_primal("toadstool", config).await?;
biome_os.start_primal("songbird", config).await?;

// biomeOS ensures dependencies are met
biome_os.ensure_dependency_tree().await?;
```

---

## 🎯 **Task Routing Decision Tree**

### **"I want to..."**

| Task | Route To | Why |
|------|----------|-----|
| Execute a shell command remotely | **Songbird** (Execution Agent) | Lightweight, no heavy runtime needed |
| Train an ML model on GPU | **Toadstool** | Heavy compute, GPU required |
| Find available compute nodes | **Songbird** | Federation/discovery |
| Authenticate a user | **BearDog** | Security operation |
| Store a large dataset | **NestGate** | Data storage |
| Register an AI model | **Squirrel** | AI coordination |
| Start all primals | **biomeOS** | Ecosystem orchestration |
| Route a task intelligently | **Songbird** | Task complexity analysis |
| Run distributed PyTorch DDP | **Toadstool** | Heavy compute, multi-GPU |
| Check health of all services | **Songbird** | Federation/monitoring |

---

## 🚀 **Distributed ML Training: The Complete Flow**

**Scenario**: Train ResNet-50 on ImageNet-100 across 3 GPUs

### **Step 1: User Submits Task to Songbird**

```bash
curl -X POST http://songbird:8080/api/v1/compute/task \
  -H "Content-Type: application/json" \
  -d '{
    "task": {
      "task_type": "ml_training",
      "payload": {
        "model": "resnet50",
        "dataset": "imagenet100",
        "epochs": 90
      },
      "resource_requirements": {
        "gpu_required": true,
        "memory_mb": 24576,
        "cpu_cores": 8
      }
    }
  }'
```

### **Step 2: Songbird Analyzes Complexity**

```rust
let complexity = TaskComplexityAnalyzer::analyze(&task);
// Result: TaskComplexity::Heavy (GPU required, large memory, long duration)
```

### **Step 3: Songbird Routes to Toadstool**

```rust
// Songbird's CapabilityRouter finds Toadstool
let toadstool = registry.find_capability("compute_gpu").await?;

// Forward the task
let response = toadstool.execute_ml_training(task).await?;
```

### **Step 4: Toadstool Executes Distributed Training**

```rust
// Toadstool sets up PyTorch DDP
toadstool.spawn_workers(vec![
    Worker { rank: 0, gpu: "tower-a:0", ip: "192.168.1.144" },
    Worker { rank: 1, gpu: "tower-b:0", ip: "192.168.1.134" },
    Worker { rank: 2, gpu: "tower-c:0", ip: "192.168.1.207" },
]).await?;

// Toadstool manages MASTER_ADDR, MASTER_PORT, CUDA_VISIBLE_DEVICES
toadstool.coordinate_ddp_training().await?;
```

### **Step 5: Progress Monitoring**

```bash
# User checks status via Songbird
curl http://songbird:8080/api/v1/compute/task/:job_id

# Songbird proxies to Toadstool
# Returns: { status: "running", progress: "Epoch 45/90", gpu_utilization: "95%" }
```

### **Step 6: Completion**

```rust
// Toadstool completes training, stores model to NestGate
let storage = registry.find_capability("storage").await?;
storage.store_model("/models/resnet50_imagenet100.pth", model).await?;

// Toadstool registers model with Squirrel
let ai_coord = registry.find_capability("ai_coordination").await?;
ai_coord.register_model(model_metadata).await?;

// Songbird returns success to user
```

---

## 📋 **Implementation Status (Nov 10, 2025)**

| Component | Status | Notes |
|-----------|--------|-------|
| **Songbird Intelligent Routing** | ✅ Complete | 150/150 tests passing |
| **Songbird HTTP Compute API** | ✅ Complete | Working, needs Toadstool integration |
| **Songbird Execution Agent** | ✅ Complete | Deployed on all 3 towers |
| **Songbird Federation** | ✅ Complete | Active discovery |
| **Toadstool GPU Compute** | ✅ Complete | Verified with direct training |
| **Toadstool ↔ Songbird Integration** | 🔄 Next Step | Need capability registration |
| **BearDog Security Capability** | ✅ Complete | HSM integration working |
| **Squirrel AI Coordination** | ✅ Complete | Capability-based |
| **NestGate Storage** | ✅ Complete | ZFS + object storage |
| **biomeOS Orchestration** | ✅ Complete | Primal lifecycle management |

---

## 🎯 **Next Steps**

### **Immediate (This Session)**:
1. ✅ Document primal responsibilities (this doc)
2. 🔄 Complete Toadstool ↔ Songbird integration
3. 🔄 Test distributed ML via Compute API → Toadstool
4. 🔄 Verify full capability-based routing

### **Short-Term (Next Session)**:
1. Complete 90-epoch training run
2. Prove heterogeneous distributed == HPC performance
3. Document success metrics
4. Create showcase demos

### **Long-Term**:
1. Expand capability registry
2. Add more primal integrations
3. Implement workload migration
4. Add advanced scheduling algorithms

---

## 🏆 **Success Criteria**

**The system is working correctly when**:

1. ✅ User submits task to Songbird
2. ✅ Songbird analyzes complexity
3. ✅ Songbird routes to appropriate capability (Toadstool for GPU)
4. ✅ Toadstool executes workload
5. ✅ Results flow back through Songbird to user
6. ✅ All via HTTP (no SSH)
7. ✅ Pure capability-based (no hardcoded paths)

**This is proper ecoPrimals Sovereign Science!** 🐦🍄🔐

---

## 📚 **References**

- **Songbird Architecture**: `ARCHITECTURE_OVERVIEW.md`
- **Toadstool Docs**: `../toadstool/00_START_HERE.md`
- **BearDog Docs**: `../beardog/START_HERE.md`
- **Squirrel Docs**: `../squirrel/START_HERE.md`
- **NestGate Docs**: `../nestgate/START_HERE.md`
- **biomeOS Docs**: `../biomeOS/README.md`
- **Intelligent Routing Spec**: `specs/INTELLIGENT_ROUTING_SYSTEM.md`
- **Compute API Spec**: `specs/COMPUTE_API_INTEGRATION.md`

