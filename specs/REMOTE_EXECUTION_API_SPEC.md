# Songbird Remote Execution API Specification

**Version:** 1.0  
**Status:** 🔴 Required for Distributed ML Demo  
**Priority:** HIGH  
**Assignee:** TBD  
**Estimated Effort:** 2-3 days  

---

## Problem Statement

**Current Gap:**
Songbird orchestrator can deploy binaries to federated towers, but cannot execute arbitrary commands or scripts remotely. This blocks automated distributed ML training where a coordinator needs to launch worker processes on remote towers.

**Current Workaround:**
Manual SSH/terminal access to each tower to run commands.

**Required for:**
- Automated distributed ML training across 3+ towers
- Remote task orchestration
- Dynamic workload distribution
- HPC job scheduling
- collaborator demo automation

---

## Requirements

### Functional Requirements

**FR-1: Remote Command Execution**
- Execute shell commands on any federated tower
- Support for both foreground and background execution
- Return stdout, stderr, and exit code
- Timeout support (configurable per command)

**FR-2: Remote Script Execution**
- Execute Python scripts remotely
- Execute shell scripts remotely
- Support for script upload + execution
- Environment variable passing

**FR-3: Process Management**
- Launch long-running processes in background
- Query process status (running/stopped/failed)
- Stop/kill remote processes by PID or job ID
- Get process logs (stdout/stderr)

**FR-4: Security**
- Authentication via Songbird federation tokens
- Authorization - only allowed towers can execute
- Command whitelist/blacklist (optional)
- Audit logging of all executions

**FR-5: Distributed Coordination**
- Execute command on multiple towers simultaneously
- Wait for all to complete before returning
- Aggregate results from all towers
- Partial failure handling (continue if some fail)

### Non-Functional Requirements

**NFR-1: Performance**
- Command execution latency < 100ms (local network)
- Support for 100+ concurrent executions
- Streaming output for long-running commands

**NFR-2: Reliability**
- Automatic retry on transient failures
- Timeout and cancellation support
- Connection pooling for efficiency

**NFR-3: Observability**
- Metrics: execution count, success rate, latency
- Logs: full audit trail of executions
- Distributed tracing across towers

---

## API Design

### HTTP REST API

#### Execute Command (Single Tower)

```http
POST /api/v1/execution/command
Content-Type: application/json
Authorization: Bearer <federation-token>

{
  "tower_id": "southgate",
  "command": "python3 train.py --rank 1",
  "working_dir": "/home/user/project",
  "env": {
    "MASTER_ADDR": "192.0.2.10",
    "CUDA_VISIBLE_DEVICES": "0"
  },
  "background": true,
  "timeout_seconds": 3600,
  "capture_output": true
}

Response 202 Accepted:
{
  "job_id": "exec-abc123",
  "tower_id": "southgate",
  "status": "running",
  "pid": 12345,
  "started_at": "2025-11-09T10:30:00Z"
}
```

#### Execute Command (Multi-Tower)

```http
POST /api/v1/execution/broadcast
Content-Type: application/json

{
  "tower_ids": ["strandgate", "southgate"],
  "command": "nvidia-smi",
  "wait_for_completion": true,
  "timeout_seconds": 30
}

Response 200 OK:
{
  "broadcast_id": "bcast-xyz789",
  "results": [
    {
      "tower_id": "strandgate",
      "status": "success",
      "exit_code": 0,
      "stdout": "GPU 0: RTX 3070...",
      "stderr": "",
      "duration_ms": 234
    },
    {
      "tower_id": "southgate",
      "status": "success",
      "exit_code": 0,
      "stdout": "GPU 0: RTX 3090...",
      "stderr": "",
      "duration_ms": 198
    }
  ]
}
```

#### Get Job Status

```http
GET /api/v1/execution/jobs/{job_id}

Response 200 OK:
{
  "job_id": "exec-abc123",
  "tower_id": "southgate",
  "status": "completed",
  "exit_code": 0,
  "pid": 12345,
  "started_at": "2025-11-09T10:30:00Z",
  "completed_at": "2025-11-09T10:35:23Z",
  "stdout": "Training complete...",
  "stderr": "",
  "duration_ms": 323000
}
```

#### Stop Job

```http
POST /api/v1/execution/jobs/{job_id}/stop

Response 200 OK:
{
  "job_id": "exec-abc123",
  "status": "stopped",
  "signal": "SIGTERM"
}
```

#### Execute Python Script

```http
POST /api/v1/execution/python
Content-Type: application/json

{
  "tower_id": "southgate",
  "script": "import torch\nprint(torch.cuda.is_available())",
  "background": false,
  "timeout_seconds": 10
}

Response 200 OK:
{
  "status": "completed",
  "exit_code": 0,
  "stdout": "True\n",
  "stderr": "",
  "duration_ms": 1234
}
```

---

## Architecture

### Components

```
┌─────────────────────────────────────────────────────────────┐
│                   Songbird Orchestrator                     │
│  ┌────────────────────────────────────────────────────┐    │
│  │          Remote Execution Manager                   │    │
│  │  - Job queue & scheduler                           │    │
│  │  - Federation client pool                          │    │
│  │  - Result aggregator                               │    │
│  └────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                           │
                           │ gRPC/HTTP
                           ▼
        ┌──────────────────┬──────────────────┬──────────────────┐
        │                  │                  │                  │
   ┌────▼─────┐      ┌────▼─────┐      ┌────▼─────┐           
   │ Tower A  │      │ Tower B  │      │ Tower C  │           
   │          │      │          │      │          │           
   │ Execution│      │ Execution│      │ Execution│           
   │  Agent   │      │  Agent   │      │  Agent   │           
   └──────────┘      └──────────┘      └──────────┘           
```

### Execution Agent (Runs on Each Tower)

**Responsibilities:**
1. Listen for execution requests from orchestrator
2. Validate and execute commands in isolated environment
3. Capture stdout/stderr streams
4. Report status and results back to orchestrator
5. Manage background processes (track PIDs, allow stopping)

**Implementation:**
- New crate: `songbird-execution-agent`
- HTTP server on port 9020 (or configurable)
- Process management using `tokio::process::Command`
- Output capture and streaming
- Job state persistence (in-memory + optional disk)

### Orchestrator Extensions

**New Module:** `crates/songbird-orchestrator/src/execution/`

**Files:**
```
execution/
├── mod.rs              # Public API
├── manager.rs          # ExecutionManager - coordinates jobs
├── job.rs              # Job state & lifecycle
├── client.rs           # HTTP client to execution agents
├── broadcast.rs        # Multi-tower execution logic
└── security.rs         # Auth, validation, sandboxing
```

**Key Types:**
```rust
pub struct ExecutionRequest {
    pub tower_id: String,
    pub command: String,
    pub working_dir: Option<PathBuf>,
    pub env: HashMap<String, String>,
    pub background: bool,
    pub timeout: Duration,
    pub capture_output: bool,
}

pub struct ExecutionResult {
    pub job_id: String,
    pub status: ExecutionStatus,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration: Duration,
}

pub enum ExecutionStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Timeout,
    Stopped,
}
```

---

## Security Considerations

### Risks

1. **Arbitrary Command Execution** - Remote code execution vulnerability
2. **Resource Exhaustion** - Fork bombs, infinite loops
3. **Privilege Escalation** - Running commands as wrong user
4. **Data Exfiltration** - Commands could read sensitive files

### Mitigations

1. **Authentication & Authorization**
   - Require federation token for all requests
   - Token should include allowed tower IDs and capabilities
   - Verify caller is authorized orchestrator

2. **Command Sandboxing**
   - Run commands as unprivileged user
   - Use namespaces/cgroups to limit resources
   - Optional: command whitelist (only allow specific patterns)
   - Reject commands with shell injection patterns

3. **Resource Limits**
   - CPU time limits (cgroup)
   - Memory limits (cgroup)
   - Max concurrent processes per tower
   - Rate limiting on execution requests

4. **Audit Logging**
   - Log every command execution attempt
   - Include: timestamp, caller, tower, command, result
   - Store in append-only audit log
   - Alert on suspicious patterns

5. **Network Security**
   - TLS for all agent-orchestrator communication
   - Mutual TLS optional
   - Network-level firewall rules

---

## Implementation Plan

### Phase 1: Basic Execution (1 day)

**Goal:** Execute simple commands on single tower

Tasks:
1. Create `songbird-execution-agent` crate
2. Implement HTTP server with `/execute` endpoint
3. Basic command execution with `tokio::process`
4. Return stdout/stderr/exit_code
5. Add to orchestrator as new API endpoint
6. Test with simple commands (echo, ls, etc.)

**Deliverables:**
- ✅ Execute command on one tower via API
- ✅ Get results back synchronously
- ✅ Basic error handling

### Phase 2: Background Jobs (1 day)

**Goal:** Support long-running processes

Tasks:
1. Job state management (in-memory HashMap)
2. Background execution (tokio::spawn)
3. Job status query API
4. Job stop/kill API
5. Process tracking (store PID, monitor)
6. Log capture for background jobs

**Deliverables:**
- ✅ Launch background job, get job_id
- ✅ Query job status while running
- ✅ Stop job remotely
- ✅ Retrieve logs after completion

### Phase 3: Multi-Tower (0.5 days)

**Goal:** Execute on multiple towers simultaneously

Tasks:
1. Broadcast execution logic
2. Parallel requests with `join_all`
3. Result aggregation
4. Partial failure handling

**Deliverables:**
- ✅ Execute same command on N towers
- ✅ Wait for all or fail-fast
- ✅ Aggregate results

### Phase 4: Security & Hardening (0.5 days)

**Goal:** Production-ready security

Tasks:
1. Add authentication checks
2. Command validation & sanitization
3. Resource limits (timeout, memory)
4. Audit logging
5. Error handling improvements
6. Integration tests

**Deliverables:**
- ✅ Auth enforced on all endpoints
- ✅ Commands run with resource limits
- ✅ All executions logged
- ✅ Test suite covering edge cases

### Phase 5: Documentation & Examples (0.5 days)

**Goal:** Make it easy to use

Tasks:
1. API documentation (OpenAPI spec)
2. Usage examples (Python, Rust, curl)
3. Distributed ML training example
4. Troubleshooting guide
5. Update main Songbird docs

**Deliverables:**
- ✅ Complete API docs
- ✅ 3+ usage examples
- ✅ Distributed training guide

---

## Testing Strategy

### Unit Tests
- Command parsing and validation
- Job state transitions
- Security checks (auth, command sanitization)
- Error handling

### Integration Tests
- Execute simple command, verify output
- Background job lifecycle (start, query, stop)
- Multi-tower execution
- Timeout handling
- Network failure scenarios

### E2E Tests
- Deploy agents to 3 test towers
- Execute distributed training launch
- Verify all workers start
- Verify training completes
- Test failure recovery

### Load Tests
- 100 concurrent executions
- 1000 jobs queued
- Long-running jobs (hours)
- Network latency simulation

---

## Success Criteria

### MVP (Minimum Viable Product)

1. ✅ Execute shell command on remote tower via API
2. ✅ Get stdout/stderr/exit_code back
3. ✅ Support background execution with job tracking
4. ✅ Query job status while running
5. ✅ Works across all 3 towers in the federation

### Full Feature Set

6. ✅ Multi-tower broadcast execution
7. ✅ Authentication & authorization
8. ✅ Resource limits & sandboxing
9. ✅ Audit logging
10. ✅ Complete API documentation
11. ✅ Distributed ML training example working end-to-end

### Performance Targets

- Command execution latency: < 100ms (P50), < 500ms (P99)
- Support 100+ concurrent executions
- Zero crashes under normal load
- Graceful degradation under overload

---

## Use Cases

### Use Case 1: Distributed ML Training

**Scenario:** Launch PyTorch DDP training across 3 GPUs

```python
# Master launches on orchestrator
master_job = orchestrator.execute_command(
    tower_id="eastgate",
    command="python3 train_distributed.py --rank 0 --world-size 3",
    background=True
)

# Launch workers
worker_jobs = orchestrator.broadcast_command(
    tower_ids=["strandgate", "southgate"],
    commands=[
        "python3 train_distributed.py --rank 1 --world-size 3",
        "python3 train_distributed.py --rank 2 --world-size 3"
    ],
    background=True
)

# Monitor until all complete
while not all_complete(master_job, worker_jobs):
    statuses = get_all_statuses()
    print(f"Master: {statuses['master']}, Workers: {statuses['workers']}")
    sleep(10)

print("Distributed training complete!")
```

### Use Case 2: Health Check Across Fleet

**Scenario:** Check GPU status on all towers

```bash
curl -X POST http://localhost:8080/api/v1/execution/broadcast \
  -H "Content-Type: application/json" \
  -d '{
    "tower_ids": ["eastgate", "strandgate", "southgate"],
    "command": "nvidia-smi --query-gpu=name,utilization.gpu --format=csv",
    "wait_for_completion": true,
    "timeout_seconds": 10
  }'
```

### Use Case 3: Remote Debugging

**Scenario:** Get logs from failed service

```bash
curl -X POST http://localhost:8080/api/v1/execution/command \
  -H "Content-Type: application/json" \
  -d '{
    "tower_id": "southgate",
    "command": "journalctl -u squirrel -n 100",
    "background": false
  }'
```

---

## Dependencies

### Rust Crates
- `tokio` - Async runtime, process spawning
- `warp` or `axum` - HTTP server for agent
- `reqwest` - HTTP client for orchestrator
- `serde` - JSON serialization
- `tracing` - Logging
- `anyhow` / `thiserror` - Error handling

### System Requirements
- Linux (tested on Ubuntu 22.04+)
- Network connectivity between orchestrator and towers
- Ports: 9020 (execution agent) open on all towers

### Optional
- `nix` crate - For sandbox namespaces (Linux)
- `libc` - For cgroup resource limits
- `openssl` - For TLS/mTLS

---

## Future Enhancements

**v1.1:**
- File upload before execution (transfer scripts/data)
- Real-time output streaming (WebSocket)
- Job dependencies (job B starts after job A completes)

**v1.2:**
- Container execution (Docker/Podman)
- GPU affinity selection (run on specific GPU)
- Distributed job scheduling (queue, priorities)

**v2.0:**
- DAG-based workflow execution
- Checkpoint & resume for long jobs
- Job templates & pipelines
- Web UI for job management

---

## Related Specifications

- `FEDERATION_SPEC.md` - Songbird federation architecture
- `DEPLOYMENT_API_SPEC.md` - Binary deployment (already implemented)
- `SECURITY_MODEL.md` - Authentication & authorization
- `DISTRIBUTED_ML_DEMO_SPEC.md` - End-to-end ML training demo

---

## Contact & Questions

**Spec Owner:** AI Assistant  
**Target Team:** Songbird Core Team  
**Slack Channel:** #songbird-execution  
**Design Doc:** [Link TBD]  

**Questions?**
- How should we handle authentication tokens?
- Should we support Docker container execution in v1?
- What resource limits are appropriate for different workloads?

---

**Last Updated:** 2025-11-09  
**Status:** 🔴 Specification Complete, Implementation Needed  
**Target Completion:** 2-3 days after assignment

