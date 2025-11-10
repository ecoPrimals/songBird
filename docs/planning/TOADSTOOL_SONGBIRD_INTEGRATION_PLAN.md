# 🍄🐦 Toadstool ↔ Songbird Integration Plan

**Last Updated**: November 10, 2025  
**Status**: Ready to Implement

---

## 🎯 **Current State**

### **What Exists:**

**✅ Toadstool Side (Complete):**
- `crates/distributed/src/songbird_integration/` - Full integration module
- **Discovery**: Toadstool can discover Songbird nodes
- **Connection**: Toadstool can connect to Songbird via HTTP/gRPC/WebSocket
- **Job Submission**: Toadstool can submit jobs TO Songbird for distribution
- **Job Reception**: Toadstool can receive jobs FROM Songbird

**✅ Songbird Side (Complete):**
- **Intelligent Routing**: 150/150 tests passing
- **HTTP Compute API**: `/api/v1/compute/task` endpoint
- **Capability Router**: Routes based on task complexity
- **Federation API**: Service registration and discovery
- **Execution Agent**: Lightweight command execution

---

## 🔌 **The Integration**

### **The Missing Link:**

**Toadstool needs to REGISTER itself as a capability provider WITH Songbird**

**Flow:**
```
1. Toadstool starts up
2. Toadstool registers with Songbird's Federation API
3. Toadstool advertises capabilities: ["compute_gpu", "compute_heavy", "ml_training"]
4. User submits task to Songbird
5. Songbird analyzes complexity (Heavy + GPU required)
6. Songbird routes to Toadstool via Capability Registry
7. Toadstool executes the workload
8. Toadstool returns results to Songbird
9. Songbird returns results to user
```

---

## 📋 **Implementation Steps**

### **Step 1: Toadstool Registration with Songbird** (Toadstool side)

**File**: `toadstool/crates/distributed/src/songbird_integration/registration.rs` (new file)

```rust
//! Register Toadstool as a capability provider with Songbird

use toadstool::error::ToadStoolResult;
use super::types::SongbirdConnection;

pub struct ToadstoolCapabilityProvider {
    instance_id: String,
    connection: Arc<SongbirdConnection>,
}

impl ToadstoolCapabilityProvider {
    pub async fn register_with_songbird(&self) -> ToadStoolResult<()> {
        let registration = CapabilityRegistration {
            service_name: "toadstool".to_string(),
            instance_id: self.instance_id.clone(),
            capabilities: vec![
                Capability {
                    name: "compute_gpu".to_string(),
                    version: "1.0.0".to_string(),
                    metadata: json!({
                        "gpu_types": ["CUDA", "OpenCL"],
                        "max_gpus": 8,
                    }),
                },
                Capability {
                    name: "compute_heavy".to_string(),
                    version: "1.0.0".to_string(),
                    metadata: json!({
                        "max_cpu_cores": 64,
                        "max_memory_gb": 256,
                    }),
                },
                Capability {
                    name: "ml_training".to_string(),
                    version: "1.0.0".to_string(),
                    metadata: json!({
                        "frameworks": ["pytorch", "tensorflow"],
                        "distributed": true,
                    }),
                },
            ],
            endpoints: vec![
                format!("http://{}:9000/api/v1/workload/execute", self.get_local_ip()),
            ],
            health_endpoint: format!("http://{}:9000/health", self.get_local_ip()),
        };

        // POST to Songbird's Federation API
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/v1/federation/register", self.connection.active_endpoint))
            .json(&registration)
            .send()
            .await?;

        if response.status().is_success() {
            tracing::info!("✅ Registered Toadstool with Songbird");
            Ok(())
        } else {
            Err(ToadStoolError::runtime(format!(
                "Registration failed: {}",
                response.status()
            )))
        }
    }

    async fn send_heartbeat(&self) -> ToadStoolResult<()> {
        let client = reqwest::Client::new();
        client
            .post(format!(
                "{}/api/v1/federation/heartbeat/{}",
                self.connection.active_endpoint, self.instance_id
            ))
            .send()
            .await?;
        Ok(())
    }
}
```

### **Step 2: Songbird Capability Registry** (Songbird side)

**File**: `songbird/crates/songbird-orchestrator/src/server/capability_registry.rs` (enhance existing)

```rust
//! Enhanced capability registry for routing

pub struct CapabilityRegistry {
    capabilities: Arc<RwLock<HashMap<String, Vec<CapabilityProvider>>>>,
}

impl CapabilityRegistry {
    pub async fn register_capability(&self, provider: CapabilityProvider) -> Result<()> {
        let mut caps = self.capabilities.write().await;
        
        for capability in &provider.capabilities {
            caps.entry(capability.name.clone())
                .or_insert_with(Vec::new)
                .push(provider.clone());
        }
        
        tracing::info!("✅ Registered {} with capabilities: {:?}", 
            provider.service_name, 
            provider.capabilities.iter().map(|c| &c.name).collect::<Vec<_>>()
        );
        
        Ok(())
    }

    pub async fn find_provider(&self, capability: &str) -> Option<CapabilityProvider> {
        let caps = self.capabilities.read().await;
        caps.get(capability)
            .and_then(|providers| providers.first().cloned())
    }
}
```

### **Step 3: Update Songbird's Capability Router** (Songbird side)

**File**: `songbird/crates/songbird-orchestrator/src/core/routing/router.rs` (enhance existing)

```rust
// In route_task method, after complexity analysis:

match self.analyzer.analyze(&task) {
    TaskComplexity::Heavy if task.requires_gpu() => {
        // Route to Toadstool via capability registry
        if let Some(provider) = self.registry.find_provider("compute_gpu").await {
            tracing::info!("Routing GPU task to Toadstool at {}", provider.endpoints[0]);
            self.execute_on_capability(task, provider).await?
        } else {
            return Err(Error::NoCapabilityProvider("compute_gpu".to_string()));
        }
    },
    TaskComplexity::Heavy => {
        // Route to Toadstool for heavy compute
        if let Some(provider) = self.registry.find_provider("compute_heavy").await {
            tracing::info!("Routing heavy task to Toadstool at {}", provider.endpoints[0]);
            self.execute_on_capability(task, provider).await?
        } else {
            return Err(Error::NoCapabilityProvider("compute_heavy".to_string()));
        }
    },
    TaskComplexity::Lightweight => {
        // Execute locally via Execution Agent
        self.execute_on_execution_agent(task).await?
    },
    TaskComplexity::Moderate => {
        // Try local first, fall back to Toadstool
        if self.local_capacity_available() {
            self.execute_locally(task).await?
        } else if let Some(provider) = self.registry.find_provider("compute_heavy").await {
            self.execute_on_capability(task, provider).await?
        } else {
            return Err(Error::InsufficientCapacity);
        }
    },
}

async fn execute_on_capability(&self, task: Task, provider: CapabilityProvider) -> Result<TaskResult> {
    let client = reqwest::Client::new();
    
    let workload_request = json!({
        "task_type": task.task_type,
        "payload": task.payload,
        "resource_requirements": task.resource_requirements,
    });
    
    let response = client
        .post(&provider.endpoints[0])
        .json(&workload_request)
        .send()
        .await?;
    
    let result: TaskResult = response.json().await?;
    Ok(result)
}
```

### **Step 4: Toadstool Workload Execution API** (Toadstool side)

**File**: `toadstool/crates/api/src/handlers.rs` (enhance existing)

```rust
//! Workload execution endpoint for Songbird integration

pub async fn execute_workload(
    Json(request): Json<WorkloadRequest>,
) -> Result<Json<WorkloadResponse>, ApiError> {
    tracing::info!("Received workload from Songbird: {}", request.task_type);
    
    // Convert Songbird task to Toadstool UniversalJob
    let job = UniversalJob {
        job_id: Uuid::new_v4(),
        job_type: match request.task_type.as_str() {
            "ml_training" => JobType::MlTraining,
            "data_processing" => JobType::DataProcessing,
            _ => JobType::Generic,
        },
        execution_request: ExecutionRequest {
            substrate: Substrate::Native,
            payload: request.payload,
            environment: request.environment.unwrap_or_default(),
        },
        resource_requirements: request.resource_requirements,
        priority: request.priority.unwrap_or(5),
    };
    
    // Execute via ToadStool's universal scheduler
    let result = SCHEDULER.execute_job(job).await?;
    
    Ok(Json(WorkloadResponse {
        job_id: result.job_id,
        status: "completed".to_string(),
        output: result.output,
        metrics: result.metrics,
    }))
}
```

---

## 🚀 **Deployment Flow**

### **Step 1: Start Songbird (Already Running)**
```bash
# On Tower A, B, C
systemctl start songbird-orchestrator
# OR
cargo run --bin songbird-orchestrator
```

### **Step 2: Start Toadstool with Songbird Registration**
```bash
# On Tower A, B, C
SONGBIRD_ENDPOINT="http://192.168.1.144:8080" \
TOADSTOOL_INSTANCE_ID="toadstool-tower-a" \
cargo run --bin toadstool-server
```

**Toadstool Startup Sequence:**
1. Read `SONGBIRD_ENDPOINT` from env
2. Connect to Songbird
3. Register capabilities (`compute_gpu`, `compute_heavy`, `ml_training`)
4. Start workload execution API on port 9000
5. Send periodic heartbeats to Songbird

### **Step 3: Submit Task to Songbird**
```bash
curl -X POST http://192.168.1.144:8080/api/v1/compute/task \
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
        "memory_mb": 24576
      }
    }
  }'
```

**Execution Flow:**
```
User → Songbird Compute API
     → Complexity Analyzer (Heavy + GPU)
     → Capability Router
     → Capability Registry (find "compute_gpu")
     → Toadstool Workload API
     → ToadStool Universal Scheduler
     → GPU Execution
     → Results back through chain
     → User receives response
```

---

## 📊 **Testing Plan**

### **Test 1: Registration**
```bash
# Start Toadstool, verify it registers
tail -f /tmp/toadstool.log | grep "Registered"
tail -f /tmp/songbird.log | grep "Capability registered"
```

### **Test 2: Lightweight Task (Songbird Execution Agent)**
```bash
curl -X POST http://192.168.1.144:8080/api/v1/compute/task \
  -d '{"task":{"task_type":"shell_command","payload":{"command":"echo hello"}}}'
# Should execute via Execution Agent
```

### **Test 3: Heavy Task (Toadstool)**
```bash
curl -X POST http://192.168.1.144:8080/api/v1/compute/task \
  -d '{"task":{"task_type":"ml_training","resource_requirements":{"gpu_required":true}}}'
# Should route to Toadstool
```

### **Test 4: Distributed ML Training (Full Flow)**
```bash
# Submit ImageNet-100 training
curl -X POST http://192.168.1.144:8080/api/v1/compute/task \
  -d '{
    "task": {
      "task_type": "ml_training",
      "payload": {
        "model": "resnet50",
        "dataset": "imagenet100",
        "epochs": 90,
        "distributed": true,
        "world_size": 3
      },
      "resource_requirements": {
        "gpu_required": true,
        "gpu_count": 3,
        "memory_mb": 49152
      }
    }
  }'
# Toadstool should coordinate distributed training across all 3 GPUs
```

---

## ✅ **Success Criteria**

1. ✅ Toadstool successfully registers with Songbird on startup
2. ✅ Songbird's Capability Registry contains Toadstool's capabilities
3. ✅ Lightweight tasks execute via Songbird's Execution Agent
4. ✅ Heavy tasks route to Toadstool
5. ✅ GPU tasks route to Toadstool
6. ✅ Distributed ML training works end-to-end
7. ✅ All via HTTP (no SSH)
8. ✅ Pure capability-based (no hardcoded paths)

---

## 🎯 **Implementation Priority**

### **Phase 1: Basic Integration** (1-2 hours)
1. ✅ Document responsibilities (DONE)
2. → Add registration endpoint to Songbird (`/api/v1/federation/register`)
3. → Add Toadstool registration call on startup
4. → Update Capability Router to call Toadstool

### **Phase 2: Workload Execution** (2-3 hours)
1. → Add workload execution endpoint to Toadstool (`/api/v1/workload/execute`)
2. → Test lightweight vs heavy task routing
3. → Verify GPU task routing

### **Phase 3: Distributed ML** (3-4 hours)
1. → Test full distributed training flow
2. → Run 90-epoch ImageNet-100 benchmark
3. → Document performance metrics

---

## 📝 **Files to Create/Modify**

### **Toadstool:**
- ✅ `crates/distributed/src/songbird_integration/` (EXISTS)
- → `crates/distributed/src/songbird_integration/registration.rs` (NEW)
- → `crates/api/src/handlers.rs` (MODIFY - add workload execution endpoint)
- → `crates/server/src/main.rs` (MODIFY - register on startup)

### **Songbird:**
- ✅ `crates/songbird-orchestrator/src/core/routing/` (EXISTS)
- → `crates/songbird-orchestrator/src/server/capability_registry.rs` (ENHANCE)
- → `crates/songbird-orchestrator/src/server/federation_api.rs` (ENHANCE - add registration)
- → `crates/songbird-orchestrator/src/core/routing/router.rs` (ENHANCE - call Toadstool)

---

## 🏆 **Expected Outcome**

**A complete capability-based distributed compute system:**

```
User submits task
    ↓
Songbird analyzes complexity
    ↓
Songbird routes intelligently:
    • Lightweight → Execution Agent
    • Heavy/GPU → Toadstool
    ↓
Toadstool executes workload
    ↓
Results flow back to user

All via HTTP, all capability-based, zero hardcoded paths!
```

**This is proper ecoPrimals Sovereign Science!** 🐦🍄🔐

---

## 📚 **References**

- **Primal Responsibilities**: `PRIMAL_RESPONSIBILITY_MATRIX.md`
- **Intelligent Routing**: `specs/INTELLIGENT_ROUTING_SYSTEM.md`
- **Compute API**: `specs/COMPUTE_API_INTEGRATION.md`
- **Toadstool Integration**: `../toadstool/crates/distributed/src/songbird_integration/`
- **Federation API**: `crates/songbird-orchestrator/src/server/federation_api.rs`

