# 🍄 Toadstool Integration & HPC Mesh Plan

**Goal:** Distributed GPU/CPU task orchestration across LAN towers  
**Status:** Planning Phase  
**Target:** Production-ready HPC mesh for local development

---

## 🎯 Architecture Overview

### Primal Roles

#### Songbird (Orchestrator)
- **Role:** Federation coordinator & task router
- **Responsibilities:**
  - Node discovery & health monitoring
  - Service registration & discovery
  - Capability-based task routing
  - Load balancing across nodes
  - Resource aggregation & tracking

#### Toadstool (Compute Engine)
- **Role:** HPC workload executor
- **Responsibilities:**
  - GPU/CPU compute task execution
  - Container orchestration
  - Resource isolation
  - Performance monitoring
  - Result aggregation

### Federation Architecture
```
┌─────────────────────────────────────────────────────────────┐
│  Tower A (192.168.1.144)                                    │
│  ┌─────────────┐              ┌──────────────┐            │
│  │  Songbird   │◄────────────►│  Toadstool   │            │
│  │ Orchestrator│              │ Compute Node │            │
│  └──────┬──────┘              └──────────────┘            │
│         │                                                   │
└─────────┼───────────────────────────────────────────────────┘
          │ LAN Federation
          │
┌─────────▼───────────────────────────────────────────────────┐
│  Tower B (192.168.1.134 - Strandgate)                      │
│  ┌─────────────┐              ┌──────────────┐            │
│  │  Songbird   │◄────────────►│  Toadstool   │            │
│  │   Worker    │              │ Compute Node │            │
│  └─────────────┘              └──────┬───────┘            │
│                                       │                     │
│                               128 cores, 251GB RAM          │
│                               GPU(s) available?             │
└─────────────────────────────────────────────────────────────┘
```

---

## 📋 Phase 1: Toadstool Preparation (Today)

### 1.1 Verify Toadstool Build
```bash
cd ../toadstool
cargo build --workspace --release
cargo test --workspace
```

### 1.2 Configure Toadstool for Federation
**Environment variables needed:**
```bash
export TOADSTOOL_NODE_ID=toadstool-tower-a
export TOADSTOOL_HOST=192.168.1.144
export TOADSTOOL_PORT=9000
export TOADSTOOL_SONGBIRD_ENDPOINT=http://192.168.1.144:8080
export TOADSTOOL_CAPABILITIES="compute,gpu,cpu,batch-processing,ml-inference"
```

### 1.3 GPU Detection
**Check GPU availability on both towers:**
```bash
# On Tower A & B:
nvidia-smi  # For NVIDIA GPUs
rocm-smi    # For AMD GPUs
lspci | grep -i vga  # General GPU detection

# Export GPU info:
export GPU_COUNT=$(nvidia-smi --query-gpu=count --format=csv,noheader | head -1)
export GPU_MODEL=$(nvidia-smi --query-gpu=name --format=csv,noheader | head -1)
export GPU_MEMORY=$(nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits | head -1)
```

---

## 📋 Phase 2: Service Registration (Today)

### 2.1 Register Toadstool Services with Songbird

**Tower A - Register Toadstool Compute Service:**
```bash
curl -X POST http://192.168.1.144:8080/api/federation/services \
  -H "Content-Type: application/json" \
  -d '{
    "service_id": "toadstool-compute-tower-a",
    "service_name": "Toadstool Compute Service (Tower A)",
    "service_type": "compute",
    "tower_id": "tower-a-orchestrator",
    "tower_name": "Tower A",
    "endpoint": "http://192.168.1.144:9000",
    "capabilities": [
      "compute",
      "gpu",
      "cpu",
      "batch-processing",
      "ml-inference",
      "container-orchestration"
    ],
    "metadata": {
      "gpu_count": "1",
      "gpu_model": "NVIDIA RTX 4090",
      "gpu_memory_gb": "24",
      "cpu_cores": "16",
      "memory_gb": "64",
      "platform": "linux-x86_64"
    },
    "health_status": "healthy",
    "registered_at": "2025-11-08T20:00:00Z",
    "last_seen": "2025-11-08T20:00:00Z"
  }'
```

**Tower B - Register Toadstool Compute Service:**
```bash
curl -X POST http://192.168.1.144:8080/api/federation/services \
  -H "Content-Type: application/json" \
  -d '{
    "service_id": "toadstool-compute-tower-b",
    "service_name": "Toadstool Compute Service (Tower B - Strandgate)",
    "service_type": "compute",
    "tower_id": "tower-b-strandgate",
    "tower_name": "Strandgate",
    "endpoint": "http://192.168.1.134:9000",
    "capabilities": [
      "compute",
      "gpu",
      "cpu",
      "batch-processing",
      "ml-inference",
      "parallel-computing"
    ],
    "metadata": {
      "gpu_count": "0",
      "cpu_cores": "128",
      "memory_gb": "251",
      "storage_gb": "1000",
      "platform": "linux-x86_64",
      "specialty": "high-core-count-cpu"
    },
    "health_status": "healthy",
    "registered_at": "2025-11-08T20:00:00Z",
    "last_seen": "2025-11-08T20:00:00Z"
  }'
```

### 2.2 Verify Service Registration
```bash
# List all registered services
curl http://192.168.1.144:8080/api/federation/services | jq '.'

# Find compute services
curl http://192.168.1.144:8080/api/federation/services/type/compute | jq '.'

# Service stats
curl http://192.168.1.144:8080/api/federation/services/stats | jq '.'
```

---

## 📋 Phase 3: Load Testing Framework (This Weekend)

### 3.1 Create Load Test Scenarios

**Scenario 1: CPU-Only Tasks**
- Matrix multiplication (1000x1000)
- FFT computation
- Prime number generation
- Text processing

**Scenario 2: GPU Tasks**
- Neural network inference
- Image processing
- Ray tracing
- Video encoding

**Scenario 3: Mixed CPU+GPU Tasks**
- ML training (GPU) + data preprocessing (CPU)
- Video processing pipeline
- Scientific simulations

### 3.2 Load Test Script
```bash
#!/bin/bash
# load_test_distributed_compute.sh

TOWER_A_SONGBIRD="192.168.1.144:8080"
TOWER_A_TOADSTOOL="192.168.1.144:9000"
TOWER_B_TOADSTOOL="192.168.1.134:9000"

echo "🧪 Distributed Compute Load Test"
echo "=================================="

# Test 1: Submit 100 CPU tasks
for i in {1..100}; do
  curl -X POST http://$TOWER_A_SONGBIRD/api/tasks/submit \
    -H "Content-Type: application/json" \
    -d "{
      \"task_id\": \"cpu-task-$i\",
      \"task_type\": \"compute\",
      \"required_capabilities\": [\"cpu\"],
      \"resources\": {
        \"cpu_cores\": 4,
        \"memory_mb\": 2048
      },
      \"payload\": {
        \"operation\": \"matrix_multiply\",
        \"size\": 1000
      }
    }" &
done

wait

echo "✅ Submitted 100 CPU tasks"

# Test 2: Submit 10 GPU tasks
for i in {1..10}; do
  curl -X POST http://$TOWER_A_SONGBIRD/api/tasks/submit \
    -H "Content-Type: application/json" \
    -d "{
      \"task_id\": \"gpu-task-$i\",
      \"task_type\": \"compute\",
      \"required_capabilities\": [\"gpu\"],
      \"resources\": {
        \"gpu_count\": 1,
        \"gpu_memory_mb\": 4096
      },
      \"payload\": {
        \"operation\": \"ml_inference\",
        \"model\": \"resnet50\"
      }
    }" &
done

wait

echo "✅ Submitted 10 GPU tasks"
```

### 3.3 Performance Metrics to Track
- **Task Submission Latency:** Time from submit to acknowledgment
- **Task Routing Latency:** Time from acknowledgment to worker assignment
- **Task Execution Time:** Worker processing time
- **Total Task Latency:** End-to-end time
- **Throughput:** Tasks per second
- **Resource Utilization:** CPU/GPU usage across towers
- **Network Latency:** Inter-tower communication overhead

---

## 📋 Phase 4: Distributed Task Execution (This Week)

### 4.1 Task Types to Implement

#### CPU Tasks
```rust
pub enum CpuTask {
    MatrixMultiply { size: usize },
    FFT { samples: usize },
    PrimeGeneration { range: u64 },
    Sorting { elements: usize },
    Compression { data_size_mb: usize },
}
```

#### GPU Tasks
```rust
pub enum GpuTask {
    MLInference { model: String, batch_size: usize },
    ImageProcessing { images: usize, operation: String },
    RayTracing { resolution: (u32, u32), samples: u32 },
    VideoEncode { frames: usize, codec: String },
}
```

#### Hybrid Tasks
```rust
pub struct HybridTask {
    pub cpu_stage: CpuTask,
    pub gpu_stage: GpuTask,
    pub pipeline: bool,  // Execute stages in parallel if true
}
```

### 4.2 Capability-Based Routing Logic
```rust
// Songbird routing logic

fn route_task(task: &Task, nodes: &[Node]) -> Result<&Node> {
    // 1. Filter nodes by required capabilities
    let capable_nodes: Vec<&Node> = nodes
        .iter()
        .filter(|n| n.has_capabilities(&task.required_capabilities))
        .collect();
    
    // 2. Score nodes by resource availability
    let scored_nodes: Vec<(f64, &Node)> = capable_nodes
        .iter()
        .map(|n| (score_node(n, task), *n))
        .collect();
    
    // 3. Select best node
    scored_nodes
        .into_iter()
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .map(|(_, node)| node)
        .ok_or(RoutingError::NoCapableNode)
}

fn score_node(node: &Node, task: &Task) -> f64 {
    let mut score = 0.0;
    
    // CPU availability
    let cpu_available = node.cpu_cores_free() as f64 / node.cpu_cores_total() as f64;
    score += cpu_available * 0.3;
    
    // Memory availability
    let mem_available = node.memory_free() as f64 / node.memory_total() as f64;
    score += mem_available * 0.2;
    
    // GPU availability (if needed)
    if task.requires_gpu() {
        let gpu_score = if node.has_gpu_free() { 1.0 } else { 0.0 };
        score += gpu_score * 0.4;
    } else {
        score += 0.4;  // No GPU needed, full score
    }
    
    // Network latency (lower is better)
    let latency_score = 1.0 / (1.0 + node.network_latency_ms());
    score += latency_score * 0.1;
    
    score
}
```

### 4.3 Expected Performance Targets

| Metric | Target | K8s Baseline | Improvement |
|--------|--------|--------------|-------------|
| Task Routing | < 1ms | ~50ms | 50x faster |
| Node Discovery | < 100ms | ~5s | 50x faster |
| Resource Allocation | < 5ms | ~500ms | 100x faster |
| Task Distribution | < 10ms | ~1s | 100x faster |
| Mesh Formation | < 1s | Minutes | >100x faster |

---

## 📋 Phase 5: HPC Mesh Formation (Next Week)

### 5.1 Mesh Topology
```
┌──────────────────────────────────────────────────────┐
│              GPU/CPU Compute Mesh                     │
│                                                       │
│   Tower A (GPU)  ◄─────────────►  Tower B (CPU)     │
│   ┌───────────┐                    ┌──────────────┐ │
│   │  GPU      │                    │  128 Cores   │ │
│   │  Tasks    │                    │  CPU Tasks   │ │
│   └─────┬─────┘                    └──────┬───────┘ │
│         │                                  │         │
│         └──────────► Songbird ◄───────────┘         │
│                   Task Router                        │
└──────────────────────────────────────────────────────┘
```

### 5.2 Task Pipeline Example
```rust
// Hybrid task: Data preprocessing (CPU) + ML Inference (GPU)

let pipeline = TaskPipeline::new()
    .add_stage(Stage {
        name: "preprocess",
        capability: "cpu",
        task: CpuTask::DataPreprocessing {
            dataset_size_gb: 10,
            operations: vec!["normalize", "augment"],
        },
    })
    .add_stage(Stage {
        name: "inference",
        capability: "gpu",
        task: GpuTask::MLInference {
            model: "resnet50",
            batch_size: 256,
        },
    })
    .add_stage(Stage {
        name: "postprocess",
        capability: "cpu",
        task: CpuTask::ResultAggregation {
            format: "json",
        },
    });

// Songbird automatically routes:
// - Stage 1 → Tower B (128 CPU cores)
// - Stage 2 → Tower A (GPU available)
// - Stage 3 → Tower B (CPU again)
```

---

## 📋 Phase 6: Production Readiness (2 Weeks)

### 6.1 Reliability Features
- [x] Health monitoring
- [x] Node discovery
- [x] Service registration
- [ ] Task retry logic
- [ ] Failover handling
- [ ] Resource quota management
- [ ] Priority scheduling
- [ ] Task cancellation

### 6.2 Observability
- [ ] Grafana dashboards
- [ ] Prometheus metrics
- [ ] Distributed tracing
- [ ] Task execution logs
- [ ] Resource utilization heatmaps

### 6.3 Security (with BearDog)
- [ ] TLS 1.3 for inter-tower communication
- [ ] mTLS for service authentication
- [ ] Task payload encryption
- [ ] Resource access control
- [ ] Audit logging

---

## 🚀 Quick Start Commands

### Start Toadstool on Tower A
```bash
cd /home/eastgate/Development/ecoPrimals/toadstool

export TOADSTOOL_NODE_ID=toadstool-tower-a
export TOADSTOOL_HOST=192.168.1.144
export TOADSTOOL_PORT=9000
export TOADSTOOL_SONGBIRD_ENDPOINT=http://192.168.1.144:8080

./target/release/toadstool-server
```

### Start Toadstool on Tower B
```bash
cd /home/strandgate/Development/toadstool

export TOADSTOOL_NODE_ID=toadstool-tower-b
export TOADSTOOL_HOST=192.168.1.134
export TOADSTOOL_PORT=9000
export TOADSTOOL_SONGBIRD_ENDPOINT=http://192.168.1.144:8080

./target/release/toadstool-server
```

### Verify Mesh
```bash
# Check federation
curl http://192.168.1.144:8080/api/federation/status | jq '.'

# Check compute services
curl http://192.168.1.144:8080/api/federation/services/type/compute | jq '.'

# Test GPU detection
curl http://192.168.1.144:9000/api/resources/gpu | jq '.'
curl http://192.168.1.134:9000/api/resources/gpu | jq '.'
```

---

## 📊 Success Metrics

### MVP Success Criteria
- ✅ Songbird + Toadstool running on both towers
- ✅ Services registered in federation
- ✅ GPU detection working
- ✅ CPU detection working
- ✅ Task submission API operational
- ✅ Capability-based routing functional
- ✅ Sub-10ms task routing latency
- ✅ 100+ tasks/second throughput

### Production Success Criteria
- 1000+ tasks/second throughput
- < 1ms average task routing
- 99.9% task success rate
- < 0.1% network overhead
- Automatic failover in < 1s
- Zero-downtime node additions
- Horizontal scaling to 10+ towers

---

## 🎯 Next Steps (Immediate)

1. **Verify Toadstool Structure** ✅
2. **Check GPU Availability** on both towers
3. **Build Toadstool** on both towers
4. **Start Toadstool Services** with Songbird endpoints
5. **Register Services** via Federation API
6. **Create Load Test Script**
7. **Submit First Distributed Task**
8. **Measure Performance Metrics**

---

**Status:** Ready to begin Toadstool integration  
**Timeline:** MVP by end of weekend  
**Target:** Production-ready HPC mesh for local development

🍄🎵 **Songbird + Toadstool = Distributed HPC Mesh** 🚀

