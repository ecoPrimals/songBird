# Resource Management & Fairness Specification

**Status**: 🔴 Not Implemented  
**Priority**: Critical (Week 2)  
**Owner**: Songbird Core

---

## Overview

Songbird must manage resources fairly across multiple users, preventing starvation and ensuring equitable access to compute, network, and storage resources.

---

## Requirements

### Functional Requirements

1. **Resource Quotas**
   - Per-user limits (CPU hours, GPU hours, network GB, storage GB)
   - Per-task resource allocation
   - Quota enforcement
   - Quota tracking and reporting

2. **Fair Scheduling**
   - Queue tasks when resources unavailable
   - Prioritize based on user tier and urgency
   - Prevent starvation (oldest task eventually runs)
   - Load balancing across towers

3. **Admission Control**
   - Reject tasks that exceed quotas
   - Reject tasks that would overload system
   - Provide alternatives (smaller resources, later time)

4. **Resource Tracking**
   - Track resource usage per user
   - Track resource usage per task
   - Historical usage data
   - Cost calculation

### Non-Functional Requirements

- Scheduling decision latency < 50ms
- Fair scheduling (no user can monopolize)
- Graceful degradation under load
- Accurate resource tracking (±5%)

---

## API Design

```rust
/// Resource quota for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    pub user_id: UserId,
    pub max_concurrent_tasks: u32,
    pub max_cpu_hours_monthly: f32,
    pub max_gpu_hours_monthly: f32,
    pub max_network_gb_monthly: u64,
    pub max_storage_gb: u64,
    pub priority: Priority,
}

/// Priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Standard = 1,
    High = 2,
    Critical = 3,
}

/// Resource allocation for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub task_id: TaskId,
    pub allocated_at: DateTime<Utc>,
    pub cpu_cores: u32,
    pub memory_gb: u64,
    pub gpu: Option<GpuAllocation>,
    pub network_bandwidth_mbps: u32,
    pub estimated_cost: f32,
}

/// Resource usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub user_id: UserId,
    pub period: Period,
    pub cpu_hours: f32,
    pub gpu_hours: f32,
    pub network_gb: u64,
    pub storage_gb: u64,
    pub cost: f32,
}

/// Resource manager trait
pub trait ResourceManager: Send + Sync {
    /// Check user's quota
    async fn get_quota(&self, user_id: UserId) -> Result<ResourceQuota>;
    
    /// Update user's quota
    async fn update_quota(&self, quota: ResourceQuota) -> Result<()>;
    
    /// Check if task can be admitted
    async fn check_admission(&self, user_id: UserId, spec: &TaskSpec) -> Result<AdmissionDecision>;
    
    /// Allocate resources for task
    async fn allocate(&self, task_id: TaskId, spec: &TaskSpec) -> Result<ResourceAllocation>;
    
    /// Release resources
    async fn release(&self, task_id: TaskId) -> Result<()>;
    
    /// Get resource usage
    async fn get_usage(&self, user_id: UserId, period: Period) -> Result<ResourceUsage>;
    
    /// Get scheduling queue
    async fn get_queue(&self) -> Result<Vec<QueuedTask>>;
}

/// Admission decision
#[derive(Debug, Clone)]
pub enum AdmissionDecision {
    Admitted,
    Rejected { reason: String },
    Delayed { estimated_wait: Duration, reason: String },
}
```

---

## Scheduling Algorithm

### Fair Queuing (Weighted Fair Queuing)

```
For each priority level:
  1. Calculate fair share = total_resources / active_users
  2. For each user in priority level:
     - If usage < fair_share: allocate resources
     - If usage >= fair_share: queue task
  3. If resources remain, move to next priority level
  
Anti-starvation:
  - Boost priority of tasks waiting > threshold (e.g., 10 minutes)
  - Ensure minimum resource allocation per user
```

---

## Storage Schema

```sql
CREATE TABLE resource_quotas (
    user_id TEXT PRIMARY KEY,
    max_concurrent_tasks INTEGER NOT NULL DEFAULT 5,
    max_cpu_hours_monthly REAL NOT NULL DEFAULT 100.0,
    max_gpu_hours_monthly REAL NOT NULL DEFAULT 10.0,
    max_network_gb_monthly INTEGER NOT NULL DEFAULT 100,
    max_storage_gb INTEGER NOT NULL DEFAULT 50,
    priority INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE resource_allocations (
    task_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    allocated_at INTEGER NOT NULL,
    released_at INTEGER,
    cpu_cores INTEGER NOT NULL,
    memory_gb INTEGER NOT NULL,
    gpu_id TEXT,
    estimated_cost REAL NOT NULL,
    actual_cost REAL,
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);

CREATE TABLE resource_usage (
    user_id TEXT NOT NULL,
    period TEXT NOT NULL,
    cpu_hours REAL NOT NULL DEFAULT 0.0,
    gpu_hours REAL NOT NULL DEFAULT 0.0,
    network_gb INTEGER NOT NULL DEFAULT 0,
    storage_gb INTEGER NOT NULL DEFAULT 0,
    cost REAL NOT NULL DEFAULT 0.0,
    PRIMARY KEY (user_id, period)
);
```

---

## Implementation Plan

### Phase 1: Quota System (Day 1-2)
- [ ] Define quota structures
- [ ] Quota storage and retrieval
- [ ] Quota checking logic
- [ ] Quota enforcement

### Phase 2: Resource Tracking (Day 2-3)
- [ ] Track resource usage per task
- [ ] Aggregate usage per user
- [ ] Monthly rollover logic
- [ ] Cost calculation

### Phase 3: Fair Scheduling (Day 3-4)
- [ ] Implement weighted fair queuing
- [ ] Priority-based scheduling
- [ ] Anti-starvation mechanism
- [ ] Load balancing

### Phase 4: Admission Control (Day 4-5)
- [ ] Admission decision logic
- [ ] Resource availability checking
- [ ] Queue management
- [ ] Alternative suggestions

---

## Success Criteria

- [ ] Users have enforced quotas
- [ ] No single user can monopolize resources
- [ ] Tasks queued fairly (oldest + priority)
- [ ] Resource usage tracked accurately
- [ ] Admission control prevents overload

---

## Testing Requirements

- Multi-user fairness test (3+ users competing)
- Quota enforcement test (reject over-quota)
- Anti-starvation test (low priority eventually runs)
- Load test (100+ queued tasks)

---

## Dependencies

- Task lifecycle management (for queue integration)

