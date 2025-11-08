# 🏗️ Multi-Tower LAN Test Plan
**Songbird Distributed Orchestration Testing**  
**Version**: 1.0  
**Date**: November 8, 2025

---

## 🎯 OBJECTIVES

Validate Songbird's distributed orchestration capabilities on physical hardware:

1. **Service Discovery** - Towers find each other automatically
2. **Task Distribution** - Work routes to capable towers
3. **Performance** - Sub-millisecond orchestration overhead
4. **Resilience** - Circuit breakers and failover work
5. **Real-World Readiness** - Production deployment confidence

---

## 🏢 TEST TOPOLOGY

### **Minimum Configuration** (2 Towers)
```
Tower A (192.168.1.100)          Tower B (192.168.1.101)
├─ Songbird Orchestrator         ├─ Toadstool Compute
├─ Ports: 8080, 8090             ├─ Ports: 8081, 8091
└─ Role: Coordinator             └─ Role: Worker

Connected via: Gigabit Switch (or WiFi for testing)
```

### **Recommended Configuration** (3 Towers)
```
Tower A (192.168.1.100)          Tower B (192.168.1.101)          Tower C (192.168.1.102)
├─ Songbird Orchestrator         ├─ Toadstool Compute             ├─ NestGate Storage
├─ BearDog Security              ├─ BearDog Security              ├─ BearDog Security
├─ Ports: 8080, 8443             ├─ Ports: 8081, 8443             ├─ Ports: 8082, 8443
└─ Role: Coordinator             └─ Role: Compute Worker          └─ Role: Storage Worker
```

---

## 📋 PREREQUISITES

### **Hardware Requirements (Per Tower)**
- **CPU**: 2+ cores (4+ recommended)
- **RAM**: 2GB+ (4GB+ recommended)
- **Storage**: 10GB+ free space
- **Network**: Gigabit Ethernet (or 100Mbps minimum)
- **OS**: Linux (Ubuntu 20.04+) or macOS

### **Software Requirements**
- **Rust**: 1.70+ installed
- **Git**: For cloning repositories
- **Network Tools**: `ping`, `netstat`, `curl`
- **Optional**: `iperf3` for network testing

### **Network Setup**
- All towers on same subnet (e.g., 192.168.1.x)
- Firewall rules allow ports 8080-8099
- Static IPs assigned (or DHCP reservations)
- Router allows inter-device communication

---

## 🛠️ SETUP PROCEDURE

### **Step 1: Prepare All Towers** (15 min per tower)

```bash
# On each tower:

# 1. Clone repository
git clone https://github.com/ecoPrimals/songbird
cd songbird

# 2. Build
cargo build --workspace --release

# 3. Verify build
cargo test --workspace

# Expected: Build success, all tests pass
```

### **Step 2: Configure Tower A** (Orchestrator)

```bash
# Tower A Configuration
export SERVICE_ID=tower-a-orchestrator
export SERVICE_PORT=8080
export SONGBIRD_HOST=192.168.1.100  # Tower A's IP
export SONGBIRD_ENV=development

# Discovery configuration
export DISCOVERY_METHODS=environment,network
export DISCOVERY_TIMEOUT=30

# Federation configuration
export SONGBIRD_FEDERATION_ENABLED=true
export SONGBIRD_FEDERATION_PEERS="http://192.168.1.101:8081,http://192.168.1.102:8082"

# Health monitoring
export HEALTH_CHECK_ENABLED=true
export HEALTH_CHECK_INTERVAL=5

# Start orchestrator
cd target/release
./songbird-orchestrator
```

### **Step 3: Configure Tower B** (Compute Worker)

```bash
# Tower B Configuration
export SERVICE_ID=tower-b-compute
export SERVICE_PORT=8081
export SONGBIRD_HOST=192.168.1.101  # Tower B's IP
export SONGBIRD_ENV=development

# Discovery configuration
export DISCOVERY_METHODS=environment,network
export DISCOVERY_TIMEOUT=30

# Capability declaration
export SERVICE_CAPABILITIES=compute,cpu-intensive

# Federation configuration
export SONGBIRD_FEDERATION_ENABLED=true
export SONGBIRD_FEDERATION_PEERS="http://192.168.1.100:8080,http://192.168.1.102:8082"

# Start compute service
cd target/release
# If you have toadstool:
./toadstool-server
# Or run orchestrator in worker mode:
./songbird-orchestrator --mode=worker
```

### **Step 4: Configure Tower C** (Storage Worker) [Optional]

```bash
# Tower C Configuration
export SERVICE_ID=tower-c-storage
export SERVICE_PORT=8082
export SONGBIRD_HOST=192.168.1.102  # Tower C's IP
export SONGBIRD_ENV=development

# Capability declaration
export SERVICE_CAPABILITIES=storage,data-persistence

# Federation configuration
export SONGBIRD_FEDERATION_ENABLED=true
export SONGBIRD_FEDERATION_PEERS="http://192.168.1.100:8080,http://192.168.1.101:8081"

# Start storage service
cd target/release
# If you have nestgate:
./nestgate-server
# Or run orchestrator in worker mode:
./songbird-orchestrator --mode=worker
```

---

## 🧪 TEST SCENARIOS

### **Test 1: Basic Connectivity** (5 minutes)

**Objective**: Verify towers can communicate

```bash
# From Tower A:
ping 192.168.1.101  # Tower B
ping 192.168.1.102  # Tower C

# Check services are listening:
curl http://192.168.1.100:8080/health  # Tower A
curl http://192.168.1.101:8081/health  # Tower B
curl http://192.168.1.102:8082/health  # Tower C

# Expected: All respond with {"status": "healthy"}
```

**Success Criteria**:
- [ ] All towers pingable (<2ms latency)
- [ ] All health endpoints respond
- [ ] No network errors

### **Test 2: Service Discovery** (10 minutes)

**Objective**: Towers discover each other automatically

```bash
# From Tower A:
curl http://192.168.1.100:8080/discovery/peers

# Expected output:
{
  "peers": [
    {"id": "tower-b-compute", "host": "192.168.1.101", "port": 8081},
    {"id": "tower-c-storage", "host": "192.168.1.102", "port": 8082}
  ],
  "discovery_time_ms": 3200
}
```

**Success Criteria**:
- [ ] All towers discovered within 5 seconds
- [ ] Peer list complete and accurate
- [ ] No discovery errors in logs

### **Test 3: Task Distribution** (15 minutes)

**Objective**: Work routes to appropriate towers

```bash
# From Tower A, submit compute task
curl -X POST http://192.168.1.100:8080/orchestrate/task \
  -H "Content-Type: application/json" \
  -d '{
    "task_id": "test-compute-1",
    "capability": "compute",
    "payload": {"operation": "cpu_test", "duration_secs": 5}
  }'

# Expected: Task routes to Tower B, executes, returns result
{
  "task_id": "test-compute-1",
  "status": "completed",
  "executed_on": "tower-b-compute",
  "duration_ms": 5023,
  "result": "success"
}
```

**Test Variations**:
```bash
# Test storage routing (should go to Tower C)
curl -X POST http://192.168.1.100:8080/orchestrate/task \
  -d '{"capability": "storage", "payload": {"op": "write", "data": "test"}}'

# Test load balancing (submit 10 tasks rapidly)
for i in {1..10}; do
  curl -X POST http://192.168.1.100:8080/orchestrate/task \
    -d "{\"task_id\": \"test-$i\", \"capability\": \"compute\"}" &
done
wait

# Check distribution
curl http://192.168.1.100:8080/metrics/task_distribution
```

**Success Criteria**:
- [ ] Tasks route to correct tower based on capability
- [ ] No routing errors
- [ ] Load distributed evenly (±20%)
- [ ] All tasks complete successfully

### **Test 4: Performance Measurement** (20 minutes)

**Objective**: Validate sub-millisecond orchestration

```bash
# Benchmark tool (create simple script)
cat > benchmark.sh << 'EOF'
#!/bin/bash
for i in {1..100}; do
  start=$(date +%s%N)
  curl -s http://192.168.1.100:8080/orchestrate/task \
    -d '{"task_id":"bench-'$i'","capability":"compute","payload":{"op":"noop"}}' \
    > /dev/null
  end=$(date +%s%N)
  echo "Request $i: $(( ($end - $start) / 1000000 ))ms"
done
EOF

chmod +x benchmark.sh
./benchmark.sh | awk '{sum+=$3; count++} END {print "Average:", sum/count, "ms"}'
```

**Expected Results**:
- Average orchestration overhead: <5ms
- P95 latency: <10ms
- P99 latency: <20ms
- No timeouts or errors

**Success Criteria**:
- [ ] Average latency meets target
- [ ] No requests fail
- [ ] Performance consistent across runs

### **Test 5: Resilience Testing** (20 minutes)

**Objective**: Circuit breakers and failover work

```bash
# Test 5a: Tower failure simulation
# On Tower B, stop the service (Ctrl+C or kill)

# From Tower A, submit tasks
curl -X POST http://192.168.1.100:8080/orchestrate/task \
  -d '{"capability": "compute", "payload": {}}'

# Expected: Circuit breaker opens, tasks queue or failover
# Check circuit breaker status:
curl http://192.168.1.100:8080/circuit_breaker/status

# Restart Tower B and verify recovery
# Tasks should resume routing to Tower B

# Test 5b: Network partition
# Temporarily block traffic between Tower A and B
sudo iptables -A INPUT -s 192.168.1.101 -j DROP

# Submit tasks, verify graceful degradation
# Restore network
sudo iptables -D INPUT -s 192.168.1.101 -j DROP

# Verify automatic recovery
```

**Success Criteria**:
- [ ] Circuit breaker opens after failures
- [ ] No cascading failures
- [ ] Automatic recovery after tower returns
- [ ] No data loss

### **Test 6: Multi-Client Load** (15 minutes)

**Objective**: Handle concurrent requests from multiple clients

```bash
# Run from multiple terminals or machines:

# Client 1:
for i in {1..50}; do
  curl -X POST http://192.168.1.100:8080/orchestrate/task \
    -d '{"task_id":"client1-'$i'"}' &
done

# Client 2:
for i in {1..50}; do
  curl -X POST http://192.168.1.100:8080/orchestrate/task \
    -d '{"task_id":"client2-'$i'"}' &
done

# Wait for all to complete
wait

# Check metrics
curl http://192.168.1.100:8080/metrics
```

**Success Criteria**:
- [ ] All 100 requests complete
- [ ] No timeouts or errors
- [ ] Consistent performance
- [ ] Even load distribution

---

## 📊 SUCCESS METRICS

### **Connectivity**
- [ ] LAN latency: <2ms between towers
- [ ] Packet loss: <0.1%
- [ ] All services reachable

### **Discovery**
- [ ] Discovery time: <5 seconds
- [ ] Peer list accuracy: 100%
- [ ] No discovery failures

### **Orchestration**
- [ ] Task routing accuracy: 100%
- [ ] Orchestration overhead: <5ms
- [ ] Throughput: >100 tasks/second

### **Resilience**
- [ ] Circuit breaker activation: <3 seconds after failure
- [ ] Recovery time: <30 seconds after tower returns
- [ ] No data loss during failures

### **Performance**
- [ ] P50 latency: <2ms
- [ ] P95 latency: <10ms
- [ ] P99 latency: <20ms
- [ ] No outliers >100ms

---

## 📝 TEST EXECUTION CHECKLIST

### **Pre-Test**
- [ ] All towers have Songbird built
- [ ] Network connectivity verified
- [ ] Static IPs assigned
- [ ] Firewall rules configured
- [ ] Test scripts prepared

### **During Test**
- [ ] Record start time
- [ ] Monitor logs on all towers
- [ ] Capture metrics
- [ ] Note any issues
- [ ] Take screenshots/recordings

### **Post-Test**
- [ ] Collect logs from all towers
- [ ] Analyze performance data
- [ ] Document any failures
- [ ] Calculate success metrics
- [ ] Write test report

---

## 🐛 TROUBLESHOOTING

### **Issue: Towers can't discover each other**
**Solutions**:
- Verify same subnet: `ip addr show`
- Check firewall: `sudo ufw status`
- Test connectivity: `nc -zv <ip> <port>`
- Check discovery config: `echo $DISCOVERY_METHODS`

### **Issue: Tasks not routing correctly**
**Solutions**:
- Verify capability declaration: `echo $SERVICE_CAPABILITIES`
- Check orchestrator logs
- Verify peer registration: `curl .../discovery/peers`

### **Issue: High latency**
**Solutions**:
- Test network: `iperf3 -c <other_tower>`
- Check CPU usage: `top`
- Review system logs: `journalctl -f`
- Disable debug logging if enabled

### **Issue: Circuit breaker not triggering**
**Solutions**:
- Verify resilience config: Check `CircuitBreakerConfig`
- Review failure threshold settings
- Check circuit breaker status endpoint
- Verify error detection is working

---

## 📈 EXPECTED RESULTS

### **Baseline Performance (Good Network)**
```
Service Discovery: 2-5 seconds first time
Task Routing: 0.5-2ms overhead
Request Latency: 1-5ms (LAN)
Throughput: 200-1000 tasks/second
Failure Detection: 1-3 seconds
Recovery Time: 10-30 seconds
```

### **Comparison with K8s (Same Hardware)**
| Metric | Songbird | K8s + Consul | Winner |
|--------|----------|--------------|--------|
| Discovery | 2-5s | 30-60s | Songbird (10x) |
| Routing | 0.5-2ms | 50-200ms | Songbird (100x) |
| Memory | 200MB | 2-4GB | Songbird (10x) |
| Setup | 15min | 2-4hours | Songbird (10x) |

---

## 🎯 TEST REPORT TEMPLATE

```markdown
# Multi-Tower LAN Test Report

**Date**: [Date]
**Tester**: [Name]
**Duration**: [Hours]

## Configuration
- Towers: [Number]
- Network: [Type, speed]
- Hardware: [Specs per tower]

## Results

### Discovery
- Time: [seconds]
- Success Rate: [%]
- Issues: [None / List]

### Task Distribution
- Accuracy: [%]
- Latency: [ms]
- Throughput: [tasks/s]

### Resilience
- Failover Time: [seconds]
- Recovery Time: [seconds]
- Data Loss: [Yes/No]

### Performance
- P50: [ms]
- P95: [ms]
- P99: [ms]

## Issues Encountered
[List any problems]

## Conclusions
[Overall assessment]

## Recommendations
[Suggestions for improvement]
```

---

## ✅ FINAL CHECKLIST

- [ ] All prerequisites met
- [ ] All towers configured
- [ ] All tests executed
- [ ] Metrics collected
- [ ] Test report written
- [ ] Issues documented
- [ ] Ready for production assessment

---

**Status**: Ready for execution!  
**Next Steps**: Schedule test session, execute plan, document results!

---

*This test plan validates Songbird's production readiness on real hardware!* 🚀

