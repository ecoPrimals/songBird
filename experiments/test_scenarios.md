# Songbird Local Tower Test Scenarios

## 🎯 **Scenario 1: The Awakening - Cold Start Discovery**

### **Narrative**
Four towers come online simultaneously in a dark data center. No central coordinator exists. Each tower must discover the others and form a working mesh network through pure peer-to-peer communication.

### **Test Sequence**
```bash
# T+0: All towers start simultaneously (parallel execution)
./songbird --tower-id tower-a --port-base 8000 --role orchestrator &
./songbird --tower-id tower-b --port-base 8100 --role storage &  
./songbird --tower-id tower-c --port-base 8200 --role compute &
./songbird --tower-id tower-d --port-base 8300 --role gaming &

# T+5: Check initial discovery
curl localhost:8001/api/v1/discovery/peers
# Expected: Empty list or partial discovery

# T+15: Check mesh formation progress  
curl localhost:8001/api/v1/discovery/mesh-status
# Expected: 2-3 peers discovered

# T+30: Verify full mesh formation
curl localhost:8001/api/v1/discovery/peers | jq '.peers | length'
# Expected: 3 peers (all others discovered)
```

### **Success Metrics**
- **Discovery Time**: All peers discovered within 30 seconds
- **Message Efficiency**: <100 discovery messages total
- **Zero Configuration**: No manual peer specification needed
- **Mesh Completeness**: Every tower knows every other tower

### **Failure Modes to Test**
- Network delay simulation (100ms, 500ms, 1000ms)
- Packet loss simulation (1%, 5%, 10%)
- Simultaneous startup vs staggered startup
- Port conflicts and automatic resolution

---

## 🎯 **Scenario 2: The Specialist - Resource Role Assignment**

### **Narrative**
The towers have discovered each other. Now they must negotiate roles based on their capabilities. Tower-B has massive storage, Tower-C has powerful CPUs, Tower-D has low-latency networking. The system should automatically assign optimal roles.

### **Test Sequence**
```bash
# Configure each tower with different resource profiles
export TOWER_A_CPU=4 TOWER_A_MEMORY=8GB TOWER_A_STORAGE=100GB
export TOWER_B_CPU=2 TOWER_B_MEMORY=16GB TOWER_B_STORAGE=2TB  
export TOWER_C_CPU=16 TOWER_C_MEMORY=32GB TOWER_C_STORAGE=250GB
export TOWER_D_CPU=8 TOWER_D_MEMORY=16GB TOWER_D_STORAGE=500GB TOWER_D_NETWORK_PRIORITY=true

# Start capability negotiation
curl -X POST localhost:8001/api/v1/orchestrator/negotiate-roles

# Check role assignments after 60 seconds
curl localhost:8001/api/v1/orchestrator/role-assignments
```

### **Expected Role Assignment**
```json
{
  "tower-a": ["orchestrator", "coordinator"],
  "tower-b": ["storage-primary", "backup-coordinator"], 
  "tower-c": ["compute-primary", "ai-processing"],
  "tower-d": ["gaming-primary", "load-balancer"]
}
```

### **Success Metrics**
- **Role Accuracy**: >95% optimal role assignment
- **Negotiation Time**: <60 seconds to stable assignment
- **Conflict Resolution**: Zero role conflicts
- **Adaptation**: Roles adjust when resources change

---

## 🎯 **Scenario 3: The Workload - Intelligent Task Distribution**

### **Narrative**
Now the real test: Can the system intelligently route different types of work to the appropriate towers? We'll throw CPU-intensive tasks, storage operations, gaming sessions, and mixed workloads at it simultaneously.

### **Test Sequence**
```bash
# Launch diverse workloads simultaneously
curl -X POST localhost:8001/api/v1/tasks -d '{
  "type": "cpu_intensive", 
  "duration": "5m",
  "cpu_cores": 8
}' &

curl -X POST localhost:8001/api/v1/tasks -d '{
  "type": "storage_heavy",
  "data_size": "10GB", 
  "operations": "read_write_mixed"
}' &

curl -X POST localhost:8001/api/v1/tasks -d '{
  "type": "gaming_session",
  "players": 4,
  "latency_requirement": "<20ms"
}' &

curl -X POST localhost:8001/api/v1/tasks -d '{
  "type": "ai_processing", 
  "model": "large_language_model",
  "memory_requirement": "16GB"
}' &

# Monitor task placement every 30 seconds
watch -n 30 'curl -s localhost:8001/api/v1/tasks/placement | jq'
```

### **Expected Behavior**
- **CPU task** → Routes to Tower-C (compute specialist)
- **Storage task** → Routes to Tower-B (storage specialist)  
- **Gaming task** → Routes to Tower-D (gaming specialist)
- **AI task** → Routes to Tower-C (high memory + compute)

### **Success Metrics**
- **Routing Accuracy**: >90% tasks routed to optimal tower
- **Resource Utilization**: 60-80% utilization across all towers
- **Task Completion**: All tasks complete within expected time
- **Load Balance**: No single tower >90% utilized while others <50%

---

## 🎯 **Scenario 4: The Crisis - Chaos Engineering**

### **Narrative**
Murphy's Law strikes. Tower-C (the compute powerhouse) suddenly goes offline during peak load. Tower-B starts running out of storage space. Network latency spikes to 200ms. Can the system adapt and maintain service?

### **Test Sequence**
```bash
# Establish baseline load
./scripts/generate_baseline_load.sh &

# T+60: Kill Tower-C abruptly
pkill -f "tower-c"

# T+120: Fill up Tower-B storage to 95%
dd if=/dev/zero of=/tower-b/storage/bigfile bs=1M count=1900

# T+180: Introduce network latency
tc qdisc add dev lo root netem delay 200ms

# T+240: Bring Tower-C back online
./songbird --tower-id tower-c --port-base 8200 --role compute &

# Monitor system behavior throughout
watch -n 10 'curl -s localhost:8001/api/v1/system/health | jq'
```

### **Expected Resilient Behaviors**
1. **Compute Failure**: Tasks automatically redistribute to Tower-A and Tower-D
2. **Storage Pressure**: New data routes to Tower-A, Tower-C, Tower-D
3. **Network Degradation**: Gaming sessions migrate to minimize latency impact
4. **Recovery**: Tower-C rejoins mesh and gradually takes load back

### **Success Metrics**
- **Service Continuity**: >70% of services remain functional
- **Recovery Time**: Full capability restored within 2 minutes of Tower-C return
- **Data Integrity**: Zero data loss during storage pressure
- **User Experience**: Gaming latency stays <50ms despite network issues

---

## 🎯 **Scenario 5: The Democracy - Consensus Under Pressure**

### **Narrative**
The towers must make critical decisions together: Should they accept a new tower joining? How should they respond to a security threat? Which tower should become the new coordinator if Tower-A fails? This tests the peer-to-peer governance model.

### **Test Sequence**
```bash
# Scenario 5.1: New Tower Joining
./songbird --tower-id tower-e --port-base 8400 --role unknown --join-request &

# Monitor consensus process
curl localhost:8001/api/v1/federation/consensus/status

# Scenario 5.2: Coordinator Election
pkill -f "tower-a"  # Kill current coordinator
# Watch election process
watch -n 5 'curl -s localhost:8101/api/v1/federation/election/status'

# Scenario 5.3: Resource Allocation Conflict
curl -X POST localhost:8001/api/v1/federation/propose -d '{
  "type": "resource_allocation",
  "conflict": {
    "tower-b": "claims 80% storage allocation", 
    "tower-c": "claims 60% storage allocation"
  }
}'
```

### **Expected Consensus Behaviors**
1. **New Tower**: Majority vote (3/4) required for acceptance
2. **Coordinator Election**: Tower-B (backup coordinator) takes over within 30 seconds
3. **Resource Conflict**: Negotiated resolution based on actual capabilities

### **Success Metrics**
- **Consensus Time**: <60 seconds for simple decisions
- **Byzantine Tolerance**: System functions with 1 byzantine node
- **Election Stability**: New coordinator elected within 30 seconds
- **Fairness**: Decisions reflect actual resource capabilities and needs

---

## 🎯 **Scenario 6: The Performance - Gaming Under Load**

### **Narrative**
The ultimate test: Can the system maintain real-time gaming performance while simultaneously handling resource coordination, service discovery, and background tasks? This pushes the zero-copy optimizations and real-time capabilities to their limits.

### **Test Sequence**
```bash
# Start background system load
./scripts/generate_system_background_load.sh &

# Launch multiple gaming sessions
for i in {1..4}; do
  curl -X POST localhost:8301/api/v1/gaming/session -d '{
    "session_id": "game-'$i'",
    "players": 4,
    "game_type": "realtime_strategy",
    "tick_rate": 60
  }' &
done

# Simultaneously stress other systems
curl -X POST localhost:8001/api/v1/stress-test -d '{
  "discovery_queries_per_second": 100,
  "resource_allocations_per_second": 50,
  "health_checks_per_second": 200
}' &

# Monitor gaming performance in real-time
./scripts/monitor_gaming_latency.sh
```

### **Expected Gaming Performance**
```yaml
Latency Targets:
  - Tick processing: <16ms (60 FPS)
  - State synchronization: <20ms
  - Player input response: <10ms
  - Cross-tower coordination: <5ms additional overhead

Quality Metrics:
  - Jitter: <5ms variance
  - Packet loss: <0.1%
  - Desync events: 0 per hour
  - Frame drops: <1% of frames
```

### **Success Metrics**
- **Gaming Latency**: All gaming sessions maintain <20ms sync latency
- **System Overhead**: Background tasks add <5ms to gaming latency
- **Throughput**: System handles 240 game ticks/second (4 games × 60 FPS)
- **Stability**: Zero gaming session crashes during 30-minute test

---

## 📊 **Measurement and Analysis Framework**

### **Real-Time Metrics Collection**
```bash
# System metrics (every second)
./scripts/collect_system_metrics.sh &

# Network metrics (every 100ms for gaming scenarios)  
./scripts/collect_network_metrics.sh &

# Application metrics (every 5 seconds)
./scripts/collect_application_metrics.sh &

# Custom Songbird metrics (every second)
./scripts/collect_songbird_metrics.sh &
```

### **Analysis Pipeline**
```bash
# Real-time analysis
./scripts/analyze_metrics_realtime.py | tee analysis.log

# Post-experiment analysis
./scripts/generate_experiment_report.py --scenario $SCENARIO_ID
```

### **Expected Outputs**
1. **Performance Graphs**: Latency, throughput, resource utilization over time
2. **Behavior Maps**: Service interaction patterns and decision flows
3. **Failure Analysis**: Root cause analysis of any failures or degradations
4. **Optimization Recommendations**: Specific improvements based on observed bottlenecks

---

## 🔬 **Scientific Rigor**

### **Control Variables**
- Hardware specifications (consistent across all tests)
- Network conditions (baseline vs stressed)
- Load patterns (synthetic but realistic)
- Measurement intervals (standardized timing)

### **Independent Variables**
- Tower configurations (CPU, memory, storage, network)
- Failure injection timing and type
- Load intensity and distribution
- Network conditions (latency, packet loss, bandwidth)

### **Dependent Variables**
- Service discovery time
- Resource allocation accuracy
- Task completion time
- Gaming performance metrics
- System recovery time
- Consensus formation time

### **Statistical Validation**
- Multiple test runs (minimum 5 per scenario)
- Statistical significance testing (p < 0.05)
- Confidence intervals for all measurements
- Outlier detection and analysis

This scientific approach will give us concrete, measurable evidence of what the Songbird ecosystem can actually achieve! 🧪🚀 