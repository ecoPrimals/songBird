# Intelligent Capability Routing Specification

**Version**: 1.0  
**Date**: November 9, 2025  
**Status**: Approved for Implementation

## Overview

Songbird's intelligent routing system enables dynamic task distribution across the federation:
- **Small/Simple tasks** → Route to peer Songbird instances (lightweight federation)
- **Large/Complex tasks** → Analyze requirements and relay to specialized capabilities (Toadstool, BearDog, Squirrel, NestGate)

## Motivation

Current state: Songbird has execution APIs and capability adapters, but lacks intelligent routing logic to decide **when** to use federation vs **when** to use specialized capabilities.

Goal: Enable Songbird to automatically route tasks based on complexity, resource requirements, and capability needs.

## Architecture

### Components

#### 1. Task Complexity Analyzer
**Location**: `crates/songbird-orchestrator/src/core/routing/analyzer.rs`

Analyzes incoming tasks to determine complexity level:
- **Lightweight**: < 1 CPU core, < 512MB RAM, < 10 seconds
- **Moderate**: < 4 CPU cores, < 4GB RAM, < 5 minutes  
- **Heavy**: Requires GPU, > 4GB RAM, or > 5 minutes

```rust
pub enum TaskComplexity {
    Lightweight,  // Handle via Songbird federation
    Moderate,     // Prefer Songbird, fallback to capabilities
    Heavy,        // Always route to specialized capability
}

pub struct TaskComplexityAnalyzer;

impl TaskComplexityAnalyzer {
    pub fn analyze(task: &Task) -> TaskComplexity;
}
```

#### 2. Capability Router
**Location**: `crates/songbird-orchestrator/src/core/routing/router.rs`

Makes intelligent routing decisions based on task analysis:

```rust
pub enum RoutingDecision {
    ExecuteLocally,
    RouteToSongbird { node_id: String, endpoint: String },
    RouteToCapability { capability_type: CapabilityType, provider_endpoint: String },
}

pub struct CapabilityRouter {
    federation_state: Arc<FederationState>,
    service_registry: Arc<FederatedServiceRegistry>,
    capability_resolver: CapabilityEndpointResolver,
}

impl CapabilityRouter {
    pub async fn route_task(&self, task: &Task) -> SongbirdResult<RoutingDecision>;
}
```

#### 3. Compute API
**Location**: `crates/songbird-orchestrator/src/server/compute_api.rs`

Provides unified HTTP endpoint for task submission:

```rust
POST /api/v1/compute/task
{
  "task": {
    "task_type": "ml_training",
    "payload": {...},
    "resource_requirements": {
      "cpu_cores": 4,
      "memory_mb": 8192,
      "gpu_required": true
    }
  }
}
```

Response:
```json
{
  "job_id": "uuid",
  "routed_to": "capability:compute:http://tower-a:9000",
  "status": "submitted"
}
```

## Task Types

### Type 1: Lightweight Federation Tasks
**Examples**: Health checks, status queries, simple API calls  
**Requirements**: < 1 CPU, < 512MB, < 10s  
**Routing**: `ExecuteLocally` or `RouteToSongbird`  
**Handled By**: Any healthy Songbird instance

### Type 2: Moderate Processing Tasks
**Examples**: Data transformation, CSV processing, batch jobs  
**Requirements**: 2-4 CPU, 1-4GB, 30s-5min  
**Routing**: Prefer `RouteToSongbird`, fallback to `RouteToCapability`  
**Handled By**: Available Songbird with capacity, or Toadstool if needed

### Type 3: Heavy Compute Tasks
**Examples**: ML training, GPU workloads, video processing  
**Requirements**: GPU, > 4GB, > 5min  
**Routing**: Always `RouteToCapability(Compute)`  
**Handled By**: Toadstool compute platform

## Routing Algorithm

```
1. Receive task via POST /api/v1/compute/task
2. TaskComplexityAnalyzer.analyze(task) → complexity
3. CapabilityRouter.route_task(task) → decision
   
   IF complexity == Lightweight:
     IF local_capacity_available:
       RETURN ExecuteLocally
     ELSE:
       RETURN RouteToSongbird(find_healthy_peer())
   
   ELSE IF complexity == Moderate:
     TRY RouteToSongbird(find_available_peer())
     CATCH: FALLBACK to RouteToCapability(determine_capability())
   
   ELSE IF complexity == Heavy:
     capability_type = determine_capability_from_requirements(task)
     endpoint = CapabilityEndpointResolver.get_endpoint(capability_type)
     RETURN RouteToCapability(capability_type, endpoint)

4. Execute based on routing decision
5. Return job_id and status
```

## Capability Determination

Task requirements map to capability types:

| Requirement | Capability | Provider |
|------------|------------|----------|
| `gpu_required: true` | Compute | Toadstool |
| `task_type: "ml_training"` | Compute | Toadstool |
| `task_type: "encrypt"` | Security | BearDog |
| `task_type: "sign"` | Security | BearDog |
| `task_type: "inference"` | AI | Squirrel |
| `task_type: "store"` | Storage | NestGate |

## Integration Points

### With Existing Systems

1. **Federation API** (`server/federation_api.rs`)
   - Router queries `FederationState` for available nodes
   - Uses `FederatedServiceRegistry` for capability discovery

2. **Execution API** (`server/execution_api.rs`)
   - Router uses `ExecutionManager` for peer Songbird execution
   - Leverages `BroadcastExecutor` for multi-node tasks

3. **Capability Adapters** (`universal/adapters/`)
   - Router uses `ComputeAdapter`, `SecurityAdapter`, etc.
   - Adapters handle capability-specific communication

### With Toadstool

Router forwards heavy compute tasks to Toadstool via HTTP:

```rust
POST http://toadstool-endpoint:9000/api/v1/jobs/submit
{
  "job_id": "uuid",
  "job_payload": {...},
  "resource_requirements": {...}
}
```

Toadstool responds with job status and execution results.

## Implementation Plan

### Phase 1: Core Routing (Week 1)
- [ ] Create `crates/songbird-orchestrator/src/core/routing/` module
- [ ] Implement `analyzer.rs` with `TaskComplexityAnalyzer`
- [ ] Implement `router.rs` with `CapabilityRouter`
- [ ] Add `mod.rs` to export routing components
- [ ] Write unit tests for analyzer and router

### Phase 2: Compute API (Week 1-2)
- [ ] Create `server/compute_api.rs`
- [ ] Implement `submit_compute_task` handler
- [ ] Implement `get_task_status` handler
- [ ] Add routes to server router
- [ ] Write integration tests

### Phase 3: Toadstool Integration (Week 2)
- [ ] Add job receiver endpoint to Toadstool: `/api/v1/jobs/submit`
- [ ] Implement job execution via Toadstool runtime engines
- [ ] Add job status tracking
- [ ] Test end-to-end: Songbird → Toadstool → GPU

### Phase 4: Testing & Validation (Week 3)
- [ ] Test lightweight task routing (Songbird-to-Songbird)
- [ ] Test heavy task routing (Songbird-to-Toadstool)
- [ ] Test load balancing across multiple Songbirds
- [ ] Test failover when capabilities unavailable
- [ ] Performance benchmarks

## Success Criteria

1. **Routing Accuracy**: 95%+ tasks routed to correct destination
2. **Load Distribution**: Even task distribution across healthy Songbirds
3. **Capability Utilization**: GPU tasks always reach Toadstool
4. **Response Time**: < 100ms routing decision latency
5. **Fault Tolerance**: Graceful degradation when providers unavailable
6. **Scalability**: Support 100+ concurrent routing decisions

## API Reference

### POST /api/v1/compute/task

Submit a compute task for intelligent routing.

**Request:**
```json
{
  "task": {
    "task_type": "string",
    "payload": {},
    "resource_requirements": {
      "cpu_cores": 4.0,
      "memory_mb": 8192,
      "gpu_required": true,
      "storage_mb": 1024
    },
    "estimated_duration_secs": 600
  },
  "priority": 5,
  "timeout_secs": 1800
}
```

**Response:**
```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "routed_to": "capability:compute:http://192.168.1.144:9000",
  "status": "submitted",
  "estimated_completion": "2025-11-09T18:30:00Z"
}
```

### GET /api/v1/compute/task/:job_id

Query task execution status.

**Response:**
```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "running",
  "progress": 0.45,
  "started_at": "2025-11-09T18:20:00Z",
  "estimated_completion": "2025-11-09T18:30:00Z"
}
```

## Security Considerations

1. **Authentication**: All routing decisions respect existing Songbird auth
2. **Authorization**: Tasks routed based on requester's permissions
3. **Resource Limits**: Enforce quotas to prevent resource exhaustion
4. **Audit Logging**: Log all routing decisions for compliance

## Monitoring & Observability

Metrics to track:
- `routing_decisions_total{destination="local|songbird|capability"}`
- `routing_latency_seconds{percentile="p50|p95|p99"}`
- `task_complexity_distribution{complexity="lightweight|moderate|heavy"}`
- `capability_invocations_total{capability_type="compute|security|ai|storage"}`
- `routing_errors_total{error_type="no_capacity|timeout|unavailable"}`

## Future Enhancements

1. **ML-Based Routing**: Train model to predict optimal routing
2. **Cost-Based Routing**: Consider compute costs in routing decisions
3. **Geographic Routing**: Route based on data locality and latency
4. **Priority Queues**: Support task prioritization and preemption
5. **Auto-Scaling**: Automatically spin up capabilities when needed

## References

- Existing Implementation: `crates/songbird-orchestrator/src/server/execution_api.rs`
- Capability System: `crates/songbird-universal/src/capabilities/`
- Federation: `crates/songbird-network-federation/`
- Routing Tests: `crates/songbird-universal/tests/routing_tests.rs`

---

**Approved By**: Architecture Review  
**Implementation Start**: November 9, 2025  
**Target Completion**: November 30, 2025

