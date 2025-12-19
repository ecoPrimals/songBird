# Testing Checklist - Student Onboarding Showcase

**Version:** 1.0  
**Date:** December 19, 2025  
**Status:** Ready for Testing

---

## Pre-Testing Setup

### Infrastructure Verification

- [ ] **Eastgate** (Registry + Compute)
  - [ ] Registry running on port 8000
  - [ ] Compute node registered
  - [ ] GPU available (RTX 3090 24GB)
  - [ ] Health check passing

- [ ] **Strandgate** (Compute)
  - [ ] Compute node registered
  - [ ] GPU available (RTX 3070 8GB)
  - [ ] Health check passing
  - [ ] Can reach Eastgate registry

- [ ] **Network Connectivity**
  - [ ] All nodes on same network
  - [ ] No firewall blocking
  - [ ] VPN connected (if using Strandgate remotely)

### Songbird Build

- [ ] **Compilation**
  ```bash
  cd /home/eastgate/Development/ecoPrimals/songbird
  cargo build --release --bin songbird-orchestrator
  ```

- [ ] **Integration Tests Pass**
  ```bash
  cargo test --package songbird-orchestrator --test orchestrator_integration_tests
  ```
  - [ ] All 10 tests passing
  - [ ] No warnings or errors

- [ ] **Access Control Tests Pass**
  ```bash
  cargo test --package songbird-orchestrator access_control
  ```

---

## Phase 1: Local Development Testing

### Unit Tests

- [ ] **Access Control Tests**
  ```bash
  cargo test --package songbird-orchestrator access_control
  ```
  - [ ] Student permissions
  - [ ] TA permissions
  - [ ] Professor permissions
  - [ ] Admin permissions
  - [ ] Token encoding/decoding
  - [ ] Token expiry
  - [ ] Information layers

- [ ] **Task Lifecycle Tests**
  ```bash
  cargo test --package songbird-orchestrator task_lifecycle
  ```

- [ ] **Registry Tests**
  ```bash
  cargo test --package songbird-registry
  ```

### Integration Tests

- [ ] **Orchestrator Integration**
  ```bash
  cargo test --package songbird-orchestrator --test orchestrator_integration_tests
  ```
  - Expected: `test result: ok. 10 passed`

---

## Phase 2: Local Deployment Testing

### Deploy Orchestrator (Linux)

- [ ] **Start Orchestrator**
  ```bash
  cd /home/eastgate/Development/ecoPrimals/songbird
  cargo run --bin songbird-orchestrator -- \
    --config showcase/07-student-onboarding/config/local-network.toml
  ```

- [ ] **Verify Startup**
  - [ ] Connects to registry: `http://192.168.1.144:8000`
  - [ ] Discovers compute nodes: eastgate, strandgate
  - [ ] WebSocket server starts on port 8080
  - [ ] No errors in logs

- [ ] **Health Check**
  ```bash
  curl http://localhost:8080/health
  ```
  - Expected: `{"status": "healthy"}`

- [ ] **Federation Status**
  ```bash
  curl http://localhost:8080/api/federation/status
  ```
  - Expected: Lists eastgate, strandgate nodes

### Test Authentication

- [ ] **Anonymous Access**
  ```bash
  curl http://localhost:8080/api/tasks
  ```
  - Expected: Public info only

- [ ] **Student Login**
  ```bash
  curl -X POST http://localhost:8080/api/auth/login \
    -H "Content-Type: application/json" \
    -d '{"user_id": "student-test", "role": "student", "course_id": "CSE-847"}'
  ```
  - Expected: JWT token returned

- [ ] **Token Validation**
  ```bash
  STUDENT_TOKEN="<token-from-login>"
  curl http://localhost:8080/api/tasks \
    -H "Authorization: Bearer $STUDENT_TOKEN"
  ```
  - Expected: Educational info included

### Test Information Disclosure

- [ ] **Student View**
  ```bash
  # Use student token
  curl http://localhost:8080/api/tasks/task-123 \
    -H "Authorization: Bearer $STUDENT_TOKEN"
  ```
  - [ ] Public info visible
  - [ ] Educational info visible
  - [ ] Operational info HIDDEN
  - [ ] Infrastructure info HIDDEN
  - [ ] IPs ANONYMIZED

- [ ] **TA View**
  ```bash
  # Get TA token
  TA_TOKEN=$(curl -X POST http://localhost:8080/api/auth/login \
    -H "Content-Type: application/json" \
    -d '{"user_id": "ta-test", "role": "ta", "course_id": "CSE-847"}' | jq -r '.token')
  
  curl http://localhost:8080/api/tasks/task-123 \
    -H "Authorization: Bearer $TA_TOKEN"
  ```
  - [ ] Public info visible
  - [ ] Educational info visible
  - [ ] Operational info visible
  - [ ] Infrastructure info HIDDEN

- [ ] **Admin View**
  ```bash
  # Get admin token
  ADMIN_TOKEN=$(curl -X POST http://localhost:8080/api/auth/login \
    -H "Content-Type: application/json" \
    -d '{"user_id": "admin", "role": "admin"}' | jq -r '.token')
  
  curl http://localhost:8080/api/admin/nodes \
    -H "Authorization: Bearer $ADMIN_TOKEN"
  ```
  - [ ] Public info visible
  - [ ] Educational info visible
  - [ ] Operational info visible
  - [ ] Administrative info visible
  - [ ] Infrastructure info visible (including IPs)

---

## Phase 3: Client Testing

### Install Client

- [ ] **Install from Source**
  ```bash
  cd showcase/07-student-onboarding/client
  pip install -e .
  ```

- [ ] **Verify Installation**
  ```bash
  python -c "import ecoprimals_client; print('OK')"
  ```

### Test Connection

- [ ] **Connect to Orchestrator**
  ```bash
  export SONGBIRD_URL="ws://localhost:8080"
  python -m ecoprimals_client.connect
  ```
  - Expected: Connection successful, nodes listed

- [ ] **Get Status**
  ```python
  from ecoprimals_client import SongbirdClient
  
  client = SongbirdClient("ws://localhost:8080")
  client.connect()
  status = client.get_federation_status()
  print(status)
  ```
  - Expected: Node count, GPU count, available VRAM

### Test Task Submission

- [ ] **Submit MNIST Task**
  ```bash
  cd showcase/07-student-onboarding/projects/01-mnist-digits
  python submit.py
  ```
  - [ ] Task accepted
  - [ ] Task ID returned
  - [ ] Progress updates received
  - [ ] Results returned
  - [ ] Receipt generated

- [ ] **Check Task Status**
  ```python
  task_id = "task-abc123"
  status = client.get_task_status(task_id)
  print(status)
  ```
  - Expected: Task status, progress, ETA

- [ ] **Get Task Result**
  ```python
  result = client.get_task_result(task_id)
  print(result)
  ```
  - Expected: Final accuracy, training time, receipt

### Test Resource Quotas

- [ ] **Check Quota**
  ```python
  quota = client.get_my_quota()
  print(quota)
  ```
  - Expected: Max concurrent tasks (3), GPU hours (10/week)

- [ ] **Exceed Quota**
  ```python
  # Submit 4 tasks simultaneously
  tasks = []
  for i in range(4):
      task_id = client.submit_task(...)
      tasks.append(task_id)
  ```
  - Expected: 4th task rejected with quota error

---

## Phase 4: Windows Deployment Testing

### Deploy on Windows Laptop

- [ ] **Copy Binary**
  - [ ] Build on Linux: `cargo build --release`
  - [ ] Copy to Windows: `target/release/songbird-orchestrator.exe`

- [ ] **Create Config**
  - [ ] Create `C:\songbird\config\local-network.toml`
  - [ ] Set registry URL: `http://192.168.1.144:8000`
  - [ ] Set server host: `0.0.0.0`
  - [ ] Set server port: `8080`

- [ ] **Start Orchestrator**
  ```powershell
  cd C:\songbird
  .\songbird-orchestrator.exe --config config\local-network.toml
  ```
  - [ ] Starts successfully
  - [ ] Discovers nodes
  - [ ] No errors

- [ ] **Firewall Rules**
  ```powershell
  New-NetFirewallRule -DisplayName "Songbird" -Direction Inbound -LocalPort 8080 -Protocol TCP -Action Allow
  ```

### Test from Student Device

- [ ] **Find Laptop IP**
  ```powershell
  ipconfig | Select-String -Pattern "IPv4"
  ```
  - Note IP (e.g., 192.168.1.50)

- [ ] **Test from Another Device**
  ```bash
  export SONGBIRD_URL="ws://192.168.1.50:8080"
  python -m ecoprimals_client.connect
  ```
  - Expected: Connection successful

- [ ] **Submit Task from Student Device**
  ```bash
  cd showcase/07-student-onboarding/projects/01-mnist-digits
  python submit.py
  ```
  - [ ] Task accepted
  - [ ] Runs on federation (Eastgate or Strandgate)
  - [ ] Results returned to student
  - [ ] Receipt generated

---

## Phase 5: Multi-Device Testing

### Test from Laptop

- [ ] **Linux Laptop**
  - [ ] Install client
  - [ ] Connect to federation
  - [ ] Submit task
  - [ ] Receive results

- [ ] **Mac Laptop**
  - [ ] Install client
  - [ ] Connect to federation
  - [ ] Submit task
  - [ ] Receive results

- [ ] **Windows Laptop**
  - [ ] Install client (pip)
  - [ ] Connect to federation
  - [ ] Submit task
  - [ ] Receive results

### Test from Mobile (Optional)

- [ ] **Android Tablet**
  - [ ] Install Termux + Python
  - [ ] Install client
  - [ ] Connect to federation
  - [ ] Submit task

- [ ] **iPad**
  - [ ] Install Pythonista or similar
  - [ ] Install client
  - [ ] Connect to federation
  - [ ] Submit task

---

## Phase 6: Performance Testing

### Load Testing

- [ ] **Single Task**
  - [ ] Submit 1 task
  - [ ] Measure latency
  - Expected: < 1 second queue time

- [ ] **Multiple Tasks (Sequential)**
  - [ ] Submit 10 tasks sequentially
  - [ ] Measure throughput
  - Expected: ~1-2 tasks/minute completion

- [ ] **Multiple Tasks (Parallel)**
  - [ ] Submit 5 tasks simultaneously
  - [ ] Verify distribution across nodes
  - Expected: 2-3 run concurrently (depending on availability)

- [ ] **Quota Enforcement**
  - [ ] Student submits 4 tasks
  - [ ] 3 queued, 4th rejected
  - Expected: Quota error message

### Network Testing

- [ ] **High Latency**
  - [ ] Simulate network delay
  - [ ] Submit task
  - Expected: Task still completes, progress updates arrive

- [ ] **Network Interruption**
  - [ ] Start task
  - [ ] Disconnect WiFi briefly
  - [ ] Reconnect
  - Expected: Task continues, client reconnects

- [ ] **Node Failure**
  - [ ] Start task on Strandgate
  - [ ] Kill Strandgate process
  - Expected: Task fails gracefully, error to student

---

## Phase 7: Security Testing

### Authentication

- [ ] **No Token**
  ```bash
  curl http://localhost:8080/api/tasks
  ```
  - Expected: Public info only

- [ ] **Invalid Token**
  ```bash
  curl http://localhost:8080/api/tasks \
    -H "Authorization: Bearer invalid-token"
  ```
  - Expected: 401 Unauthorized

- [ ] **Expired Token**
  - [ ] Create token with past expiry
  - [ ] Attempt to use
  - Expected: 401 Unauthorized, "Token expired"

### Authorization

- [ ] **Student Access Other Student Tasks**
  ```bash
  # Student A tries to view Student B's task
  curl http://localhost:8080/api/tasks/student-b-task-id \
    -H "Authorization: Bearer $STUDENT_A_TOKEN"
  ```
  - Expected: 403 Forbidden

- [ ] **Student Access Admin Endpoints**
  ```bash
  curl http://localhost:8080/api/admin/nodes \
    -H "Authorization: Bearer $STUDENT_TOKEN"
  ```
  - Expected: 403 Forbidden

- [ ] **TA Access Professor Endpoints**
  ```bash
  curl http://localhost:8080/api/professor/statistics \
    -H "Authorization: Bearer $TA_TOKEN"
  ```
  - Expected: 403 Forbidden

### Information Disclosure

- [ ] **IP Leakage Check**
  - [ ] Submit task as student
  - [ ] Check all returned data
  - Expected: NO internal IPs in student view

- [ ] **Topology Leakage Check**
  - [ ] Get federation status as student
  - Expected: Anonymized node names only

---

## Success Criteria

### Technical

- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Client connects successfully
- [ ] Tasks execute successfully
- [ ] Results returned correctly
- [ ] Receipts generated
- [ ] No IP leakage
- [ ] Authentication works
- [ ] Authorization enforced
- [ ] Quotas enforced

### Performance

- [ ] < 1 second task queue time
- [ ] < 5 second federation discovery
- [ ] Task completion time reasonable
- [ ] Progress updates real-time
- [ ] No memory leaks
- [ ] CPU usage acceptable

### User Experience

- [ ] Student can connect easily
- [ ] Clear error messages
- [ ] Progress updates visible
- [ ] Results easy to retrieve
- [ ] Documentation clear

---

## Known Issues

(Document any issues found during testing)

### Issue 1: [Description]
- **Severity:** [High/Medium/Low]
- **Impact:** [User/System]
- **Workaround:** [If any]
- **Fix:** [Planned/In Progress]

---

## Sign-off

- [ ] All tests completed
- [ ] Issues documented
- [ ] Ready for Prof. Murillo demo
- [ ] Ready for student onboarding

**Tester:** _______________  
**Date:** _______________  
**Signature:** _______________

---

**Testing complete! Ready for deployment.** 🎓✅🎵

