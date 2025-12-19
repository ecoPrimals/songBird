# 🌐 Cross-Tower Benchmarking Guide

**Purpose**: Test Songbird orchestration across physical towers  
**Status**: Ready to run

---

## 🏗️ Tower Topology

```
Eastgate (192.168.1.144)          Strandgate (192.168.1.134)
├─ AMD Ryzen 9 5950X              ├─ 128 CPU cores
├─ 128GB RAM                      ├─ 229GB RAM
├─ RTX 2070 SUPER (8GB)           ├─ RTX GPU
├─ 1Gb NIC (soon 10Gb)            ├─ 1Gb NIC
│                                 │
├─ Songbird (orchestrator)        ├─ Songbird (orchestrator)
├─ Toadstool (compute)            ├─ Toadstool (compute)
├─ Squirrel (AI)                  └─ Squirrel (AI)
└─ tarpc servers (for testing)
```

**Network**: LAN (192.168.1.0/24)  
**Latency**: ~0.5-2ms (measured)  
**Bandwidth**: ~120 MB/s (1Gb), soon ~1200 MB/s (10Gb)

---

## 🧪 Benchmark Scenarios

### Scenario 1: Protocol Performance Over LAN

**What**: Compare HTTP, JSON-RPC, tarpc across network

**Why**: Network latency changes performance characteristics

**Expected**:
- HTTP: 5-15K req/s (network + parsing overhead)
- JSON-RPC: 4-12K req/s (JSON parsing overhead)
- tarpc: 15-40K req/s (binary, efficient) ✨

**Commands**:
```bash
cd showcase/05-albatross-multiplex/benchmark

# HTTP to Strandgate
cargo run --release --bin bench-http -- \
  -t https://192.168.1.134:8081 -n 10000

# JSON-RPC to Strandgate
cargo run --release --bin bench-jsonrpc -- \
  -t https://192.168.1.134:8081 -n 10000

# tarpc to Strandgate
cargo run --release --bin bench-tarpc-single -- \
  -t 192.168.1.134:8091 -n 10000
```

---

### Scenario 2: Multiplex Scaling Over LAN

**What**: Test if tarpc scales better over network than localhost

**Why**: Network I/O should allow better CPU utilization

**Expected**:
- Localhost: 89K total, 3K per connection (4.8% efficiency)
- Network: 150-300K total, 5-10K per connection (20-30% efficiency) ✨

**Commands**:
```bash
cd showcase/05-albatross-multiplex/benchmark

# Multiplex to Strandgate (30 connections)
cargo run --release --bin bench-tarpc-multiplex -- \
  -t "192.168.1.134:8091,192.168.1.134:8092,192.168.1.134:8093" \
  -c 10 -n 10000
```

---

### Scenario 3: Bidirectional Federation

**What**: Both towers talking to each other simultaneously

**Why**: Real-world scenario, test coordination overhead

**Expected**: 
- Each tower: 20-40K req/s receiving
- Total mesh: 40-80K req/s bidirectional

**Setup**:
```bash
# On Eastgate: Start servers
cd showcase/05-albatross-multiplex
./scripts/start_tarpc_servers.sh

# On Strandgate: Start servers
cd ~/Development/songbird/showcase/05-albatross-multiplex
./scripts/start_tarpc_servers.sh

# On Eastgate: Benchmark to Strandgate
cargo run --release --bin bench-tarpc-single -- \
  -t 192.168.1.134:8091 -n 10000 &

# On Strandgate: Benchmark to Eastgate  
cargo run --release --bin bench-tarpc-single -- \
  -t 192.168.1.144:8091 -n 10000 &

wait
```

---

### Scenario 4: Orchestrated Compute (Real Workload)

**What**: Songbird on Eastgate orchestrating Toadstool on Strandgate

**Why**: Actual primal interaction, not synthetic benchmark

**Expected**:
- Task distribution: ~100-500 tasks/s
- Latency: 10-50ms per task (including compute)

**Commands**:
```bash
# Ensure Toadstool running on Strandgate
ssh strandgate "systemctl status toadstool"

# From Eastgate, orchestrate work
curl -X POST https://localhost:8443/api/compute/schedule \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "matrix_multiply",
    "target": "192.168.1.134:7878",
    "params": {"size": 1024}
  }'

# Benchmark multiple tasks
for i in {1..100}; do
  curl -X POST https://localhost:8443/api/compute/schedule \
    -H "Content-Type: application/json" \
    -d '{"task_type":"echo","params":{"msg":"test"}}'
done
```

---

### Scenario 5: AI Inference Mesh (Squirrel)

**What**: Distribute AI inference across both towers

**Why**: Test high-latency, compute-heavy workloads

**Expected**:
- Single inference: 50-500ms (depending on model)
- Throughput: Limited by GPU, not network
- Orchestration overhead: <5%

**Commands**:
```bash
# From Eastgate, query Squirrel on both towers
curl -X POST https://localhost:8443/api/ai/infer \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Explain distributed computing",
    "distribute": true,
    "targets": ["eastgate", "strandgate"]
  }'
```

---

### Scenario 6: Data Transfer (Future: Nestgate)

**What**: Large file transfers between towers

**Why**: Test bandwidth utilization

**Expected**:
- 1Gb NIC: ~100 MB/s (~800 Mbps)
- 10Gb NIC: ~1000 MB/s (~8000 Mbps) ✨

**Commands** (when Nestgate live):
```bash
# Upload 1GB file via Songbird to Nestgate on Strandgate
time curl -X POST https://192.168.1.134:8081/api/data/upload \
  --data-binary @test_1gb.bin

# Download from Strandgate
time curl https://192.168.1.134:8081/api/data/download/test_1gb.bin \
  -o received.bin
```

---

## 📊 Benchmark Matrix

### Complete Test Suite

| Benchmark | Protocol | Direction | Connections | Expected |
|-----------|----------|-----------|-------------|----------|
| HTTP single | HTTP | E→S | 1 | 5-15K req/s |
| HTTP reverse | HTTP | S→E | 1 | 5-15K req/s |
| JSON-RPC single | JSON-RPC | E→S | 1 | 4-12K req/s |
| tarpc single | tarpc | E→S | 1 | 15-40K req/s ✨ |
| tarpc multiplex | tarpc | E→S | 30 | 150-300K req/s ✨ |
| Bidirectional | tarpc | E↔S | 2 | 30-80K req/s |
| Compute | Orchestration | E→S | - | 100-500 tasks/s |
| AI inference | Orchestration | E↔S | - | GPU-limited |
| Data transfer | Binary | E→S | 1 | 100 MB/s (1Gb) |

---

## 🚀 Quick Start: Run All Cross-Tower Benchmarks

### Automated Script

```bash
cd showcase/05-albatross-multiplex/scripts
./run_all_cross_tower_benchmarks.sh
```

This will:
1. Verify both towers are online
2. Deploy tarpc servers to Strandgate (if needed)
3. Run all benchmark scenarios
4. Generate comparison report
5. Save results

---

## 📈 Expected Results: Localhost vs Network

### Single Connection Performance

| Protocol | Localhost | Network (1Gb) | Network (10Gb) |
|----------|-----------|---------------|----------------|
| HTTP | 28K req/s | 5-15K req/s | 10-20K req/s |
| JSON-RPC | 21K req/s | 4-12K req/s | 8-18K req/s |
| tarpc | **62K req/s** | **15-40K req/s** | **30-60K req/s** ✨ |

**Key insight**: Network adds latency, reduces throughput, but tarpc stays fastest.

---

### Multiplex Scaling

| Config | Localhost | Network (1Gb) | Network (10Gb) |
|--------|-----------|---------------|----------------|
| Total | 89K req/s | 150-300K req/s ✨ | **400-800K req/s** ✨✨ |
| Per-conn | 3K req/s | 5-10K req/s | **13-27K req/s** ✨ |
| Efficiency | 4.8% | 20-30% ✨ | **50-80%** ✨✨ |

**Key insight**: Network I/O allows better CPU utilization during waits!

---

## 🎯 Success Criteria

### Functional Goals
- ✅ All protocols work across network
- ✅ Bidirectional communication works
- ✅ Songbird successfully orchestrates remote work
- ✅ No connection drops or errors

### Performance Goals

**With 1Gb NIC**:
- tarpc single: >10K req/s
- tarpc multiplex: >100K req/s total
- Per-connection: >3K req/s (better than localhost!)

**With 10Gb NIC** (future):
- tarpc single: >30K req/s
- tarpc multiplex: >400K req/s total ✨
- Per-connection: >13K req/s (much better!)
- Data transfer: >800 MB/s

---

## 🔧 Troubleshooting

### Issue: High Latency

**Check network**:
```bash
ping 192.168.1.134
# Should be <1ms

mtr 192.168.1.134
# Check for packet loss or routing issues
```

**Check tower load**:
```bash
ssh strandgate "top -bn1 | head -20"
# CPU should not be 100% idle
```

---

### Issue: Connection Refused

**Check firewall**:
```bash
# On Strandgate
sudo ufw status
sudo ufw allow 8091:8093/tcp  # tarpc ports
```

**Check service running**:
```bash
ssh strandgate "ss -tlnp | grep 8091"
# Should show tarpc-server listening
```

---

### Issue: Low Throughput

**Check NIC utilization**:
```bash
# During benchmark
iftop -i eth0
# Should show ~100 Mbps (1Gb NIC)

# Or
nload eth0
# Visual bandwidth monitor
```

**Check for CPU saturation**:
```bash
# On both towers during benchmark
mpstat 1 10
# %idle should be >10%
```

---

## 📝 Benchmark Checklist

### Pre-flight

- [ ] Both towers online and reachable
- [ ] Songbird running on both towers
- [ ] tarpc servers deployed (if testing tarpc)
- [ ] Firewall ports open
- [ ] No other heavy workloads running

### During Benchmarks

- [ ] Monitor CPU usage (both towers)
- [ ] Monitor network usage (`iftop` / `nload`)
- [ ] Watch for errors in logs
- [ ] Verify no packet loss (`mtr`)

### Post-benchmark

- [ ] Collect results from both towers
- [ ] Generate comparison report
- [ ] Document any anomalies
- [ ] Save for future comparison (especially pre/post 10Gb NIC)

---

## 🎓 Learning Objectives

### What Cross-Tower Benchmarks Teach Us

1. **Network impact on performance**
   - Latency baseline (0.5-2ms)
   - Bandwidth limits (120 MB/s → 1200 MB/s)
   - Protocol efficiency over network

2. **Multiplex scaling characteristics**
   - Better with I/O wait (network)
   - CPU utilization during network waits
   - Optimal connection count

3. **Real-world orchestration overhead**
   - Songbird routing cost
   - Federation coordination
   - Service discovery latency

4. **Protocol trade-offs**
   - HTTP: Universal but slower
   - JSON-RPC: Language-agnostic, moderate
   - tarpc: Fastest but Rust-only

---

*Status: Ready to run*  
*Network: LAN (1Gb, soon 10Gb)*  
*Next: Execute benchmarks and compare with localhost* 🚀

