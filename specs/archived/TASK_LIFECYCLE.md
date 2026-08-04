# Task Lifecycle Management Specification

**Status**: 🔴 Not Implemented  
**Priority**: Critical (Week 1)  
**Owner**: Songbird Core

---

## Overview

Songbird must manage the complete lifecycle of tasks from creation through completion, including progress tracking, pause/resume, and checkpointing.

---

## Requirements

### Functional Requirements

1. **Task Creation**
   - Accept task specifications
   - Assign unique task IDs
   - Validate task parameters
   - Estimate resource requirements

2. **Task Status Tracking**
   - Track current state (Queued, Running, Paused, Completed, Failed, Cancelled)
   - Record timestamps (created, started, completed)
   - Calculate progress (0.0 - 1.0)
   - Estimate time remaining

3. **Task Control**
   - Pause running tasks
   - Resume paused tasks
   - Cancel tasks (with cleanup)
   - Retry failed tasks

4. **Checkpointing**
   - Save task state periodically
   - Resume from checkpoint after failure
   - Configurable checkpoint frequency

### Non-Functional Requirements

- Task state persisted to disk
- State updates atomic
- Checkpoint overhead < 5% of task time
- Status query latency < 10ms

---

## API Design

```rust
/// Task lifecycle state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is queued, waiting for resources
    Queued,
    /// Task is currently running
    Running { started_at: DateTime<Utc> },
    /// Task is paused (can be resumed)
    Paused { paused_at: DateTime<Utc> },
    /// Task completed successfully
    Completed { completed_at: DateTime<Utc> },
    /// Task failed
    Failed { 
        failed_at: DateTime<Utc>,
        error: String,
        retry_count: u32,
    },
    /// Task was cancelled by user
    Cancelled { cancelled_at: DateTime<Utc> },
}

/// Task metadata and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskLifecycle {
    /// Unique task identifier
    pub id: TaskId,
    
    /// Current status
    pub status: TaskStatus,
    
    /// Progress (0.0 = not started, 1.0 = complete)
    pub progress: f32,
    
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    
    /// Estimated completion time (seconds)
    pub eta_seconds: Option<u64>,
    
    /// Tower currently executing the task
    pub current_tower: Option<TowerId>,
    
    /// User who owns this task
    pub owner: UserId,
    
    /// Checkpoints (for resume)
    pub checkpoints: Vec<Checkpoint>,
    
    /// Task can be paused
    pub pausable: bool,
    
    /// Task can be cancelled
    pub cancellable: bool,
    
    /// Task can be resumed from checkpoint
    pub resumable: bool,
}

/// Checkpoint for resuming tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Checkpoint ID
    pub id: String,
    
    /// Timestamp
    pub created_at: DateTime<Utc>,
    
    /// Progress at checkpoint
    pub progress: f32,
    
    /// Serialized task state
    pub state: Vec<u8>,
    
    /// Checkpoint size in bytes
    pub size_bytes: u64,
}

/// Task lifecycle manager
pub trait TaskLifecycleManager: Send + Sync {
    /// Create a new task
    async fn create_task(&self, spec: TaskSpec) -> Result<TaskId>;
    
    /// Get task status
    async fn get_task(&self, id: TaskId) -> Result<TaskLifecycle>;
    
    /// List tasks (with filters)
    async fn list_tasks(&self, filter: TaskFilter) -> Result<Vec<TaskLifecycle>>;
    
    /// Update task progress
    async fn update_progress(&self, id: TaskId, progress: f32) -> Result<()>;
    
    /// Pause a running task
    async fn pause_task(&self, id: TaskId) -> Result<()>;
    
    /// Resume a paused task
    async fn resume_task(&self, id: TaskId) -> Result<()>;
    
    /// Cancel a task
    async fn cancel_task(&self, id: TaskId, reason: String) -> Result<()>;
    
    /// Create checkpoint
    async fn checkpoint_task(&self, id: TaskId) -> Result<Checkpoint>;
    
    /// Resume from checkpoint
    async fn resume_from_checkpoint(&self, checkpoint: Checkpoint) -> Result<TaskId>;
    
    /// Stream task events
    fn stream_events(&self, id: TaskId) -> impl Stream<Item = TaskEvent>;
}
```

---

## Storage Schema

### Task State (SQLite)

```sql
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    owner TEXT NOT NULL,
    status TEXT NOT NULL,
    progress REAL NOT NULL DEFAULT 0.0,
    created_at INTEGER NOT NULL,
    started_at INTEGER,
    completed_at INTEGER,
    current_tower TEXT,
    pausable INTEGER NOT NULL DEFAULT 1,
    cancellable INTEGER NOT NULL DEFAULT 1,
    resumable INTEGER NOT NULL DEFAULT 1,
    spec_json TEXT NOT NULL
);

CREATE TABLE checkpoints (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    progress REAL NOT NULL,
    state_blob BLOB NOT NULL,
    size_bytes INTEGER NOT NULL,
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);

CREATE INDEX idx_tasks_owner ON tasks(owner);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_checkpoints_task ON checkpoints(task_id);
```

---

## Implementation Plan

### Phase 1: Core Infrastructure (Day 1-2)
- [ ] Define types and traits
- [ ] SQLite storage implementation
- [ ] Task ID generation
- [ ] Basic CRUD operations

### Phase 2: State Management (Day 2-3)
- [ ] Status transitions
- [ ] Progress tracking
- [ ] ETA calculation
- [ ] State persistence

### Phase 3: Control Operations (Day 3-4)
- [ ] Pause/resume implementation
- [ ] Cancel with cleanup
- [ ] Atomic state updates

### Phase 4: Checkpointing (Day 4-5)
- [ ] Checkpoint creation
- [ ] State serialization
- [ ] Resume from checkpoint
- [ ] Checkpoint cleanup

### Phase 5: Event Streaming (Day 5)
- [ ] Event channel setup
- [ ] WebSocket integration
- [ ] Real-time updates

---

## Testing Requirements

### Unit Tests
- State transitions
- Progress calculations
- Checkpoint serialization
- Resume logic

### Integration Tests
- Full task lifecycle (create → run → complete)
- Pause/resume cycle
- Checkpoint and resume after failure
- Concurrent task management

### Performance Tests
- 1000 concurrent tasks
- Checkpoint overhead measurement
- Status query latency

---

## Success Criteria

- [ ] Tasks can be created and tracked
- [ ] Progress updates in real-time
- [ ] Pause/resume works reliably
- [ ] Checkpoints enable recovery after crashes
- [ ] Status queries < 10ms
- [ ] Checkpoint overhead < 5%

---

## Dependencies

- **Internal**: None (core functionality)
- **External**: 
  - `sqlx` for database
  - `tokio` for async
  - `serde` for serialization

---

## Notes

- Keep checkpoints configurable (frequency, retention)
- Consider checkpoint compression for large state
- Implement checkpoint cleanup (remove old checkpoints)
- Progress tracking should be optional (not all tasks can report progress)

