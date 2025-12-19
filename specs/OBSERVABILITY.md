# Basic Observability Specification

**Status**: 🔴 Not Implemented  
**Priority**: Critical (Week 4)  
**Owner**: Songbird Core

---

## Overview

Songbird must provide basic observability so humans can see what's happening with their tasks in real-time.

---

## Requirements

### Functional Requirements

1. **Task Tracking**
   - "Where is my task right now?"
   - "What tower is executing it?"
   - "What protocol is being used?"
   - Current progress and ETA

2. **Metrics Collection**
   - CPU usage per task
   - Memory usage per task
   - Network bytes sent/received
   - GPU usage (if applicable)

3. **Event Streaming**
   - Real-time task status updates
   - Error events
   - State transition events
   - WebSocket support for live updates

4. **Query Interface**
   - Get task status by ID
   - List tasks by user
   - Filter by status/tower/time

### Non-Functional Requirements

- Query latency < 10ms
- Event delivery latency < 100ms
- Metrics overhead < 2% CPU
- Support 1000+ concurrent observers

---

## API Design

```rust
/// Task observability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskObservability {
    pub task_id: TaskId,
    pub current_tower: Option<TowerId>,
    pub protocol_used: String,
    pub progress: f32,
    pub status: TaskStatus,
    pub metrics: TaskMetrics,
    pub events: Vec<TaskEvent>,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

/// Real-time task metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    pub cpu_usage_percent: f32,
    pub memory_usage_bytes: u64,
    pub network_sent_bytes: u64,
    pub network_received_bytes: u64,
    pub gpu_usage_percent: Option<f32>,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
}

/// Task event (for event stream)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEvent {
    pub task_id: TaskId,
    pub timestamp: DateTime<Utc>,
    pub event_type: TaskEventType,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskEventType {
    Created,
    Queued,
    Started,
    ProgressUpdate { progress: f32 },
    StateChanged { from: TaskStatus, to: TaskStatus },
    TowerChanged { from: TowerId, to: TowerId },
    ProtocolChanged { from: String, to: String },
    CheckpointCreated,
    Error { error: String },
    Completed,
    Failed,
    Cancelled,
}

/// Observability manager
pub trait ObservabilityManager: Send + Sync {
    /// Get task observability info
    async fn get_task_observability(&self, task_id: TaskId) -> Result<TaskObservability>;
    
    /// List observable tasks (with filters)
    async fn list_observable_tasks(&self, filter: TaskFilter) -> Result<Vec<TaskObservability>>;
    
    /// Stream task events
    fn stream_task_events(&self, task_id: TaskId) -> impl Stream<Item = TaskEvent>;
    
    /// Stream all events for user
    fn stream_user_events(&self, user_id: UserId) -> impl Stream<Item = TaskEvent>;
    
    /// Record metrics
    async fn record_metrics(&self, task_id: TaskId, metrics: TaskMetrics) -> Result<()>;
    
    /// Emit event
    async fn emit_event(&self, event: TaskEvent) -> Result<()>;
}
```

---

## Event Stream Architecture

```
┌──────────────┐
│   Task       │
│  Executor    │
└──────┬───────┘
       │
       │ Emit events
       ▼
┌──────────────┐
│   Event      │
│   Channel    │
│  (broadcast) │
└──────┬───────┘
       │
       ├────────────► WebSocket 1 (User A)
       │
       ├────────────► WebSocket 2 (User B)
       │
       └────────────► Event Log (persistent)
```

---

## REST API Endpoints

```rust
// Get task observability
GET /api/tasks/{task_id}/observability

// List user's tasks
GET /api/tasks?user={user_id}&status={status}

// WebSocket for real-time events
WS /api/tasks/{task_id}/events

// Get metrics history
GET /api/tasks/{task_id}/metrics?from={start}&to={end}
```

---

## Storage Schema

```sql
CREATE TABLE task_metrics (
    task_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    cpu_usage_percent REAL,
    memory_usage_bytes INTEGER,
    network_sent_bytes INTEGER,
    network_received_bytes INTEGER,
    gpu_usage_percent REAL,
    PRIMARY KEY (task_id, timestamp)
);

CREATE TABLE task_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    message TEXT,
    data_json TEXT
);

CREATE INDEX idx_events_task ON task_events(task_id, timestamp);
CREATE INDEX idx_events_time ON task_events(timestamp);
```

---

## Implementation Plan

### Phase 1: Event System (Day 1-2)
- [ ] Event types and structures
- [ ] Event channel (broadcast)
- [ ] Event emission API
- [ ] Event persistence

### Phase 2: Metrics Collection (Day 2-3)
- [ ] Metrics structures
- [ ] Metrics collection hooks
- [ ] Metrics aggregation
- [ ] Metrics storage

### Phase 3: Query Interface (Day 3-4)
- [ ] REST API endpoints
- [ ] Task filtering
- [ ] Metrics querying
- [ ] Performance optimization

### Phase 4: WebSocket Streaming (Day 4-5)
- [ ] WebSocket server setup
- [ ] Event streaming to clients
- [ ] Connection management
- [ ] Backpressure handling

---

## Example Usage

### Query Task Status
```bash
curl http://localhost:8080/api/tasks/task-123/observability

# Response:
{
  "task_id": "task-123",
  "current_tower": "eastgate-gpu-1",
  "protocol_used": "tarpc",
  "progress": 0.67,
  "status": "Running",
  "metrics": {
    "cpu_usage_percent": 45.2,
    "memory_usage_bytes": 2147483648,
    "gpu_usage_percent": 87.5
  },
  "events": [
    {
      "timestamp": "2025-12-18T10:00:00Z",
      "event_type": "Started",
      "message": "Task started on eastgate-gpu-1"
    },
    {
      "timestamp": "2025-12-18T10:05:00Z",
      "event_type": "ProgressUpdate",
      "message": "Progress: 67%"
    }
  ]
}
```

### Stream Events (WebSocket)
```javascript
const ws = new WebSocket('ws://localhost:8080/api/tasks/task-123/events');

ws.onmessage = (event) => {
  const taskEvent = JSON.parse(event.data);
  console.log(`Task ${taskEvent.task_id}: ${taskEvent.message}`);
  // Update UI with progress
};
```

---

## Success Criteria

- [ ] Users can query "where is my task?"
- [ ] Real-time progress updates via WebSocket
- [ ] Metrics collected with <2% overhead
- [ ] Query latency < 10ms
- [ ] Event delivery < 100ms

---

## Future Enhancements (NOT Week 4)

- Rich dashboard UI (separate project)
- Grafana integration
- Alert rules
- Anomaly detection (Squirrel integration)

---

## Dependencies

- Task lifecycle (for status integration)
- WebSocket server (axum already supports)

