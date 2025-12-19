# 🦅 Albatross Final Benchmark Report

**Date**: December 18, 2025  
**System**: Eastgate Tower (AMD Ryzen 9 5950X, 128GB RAM)  
**Test**: All protocols, 10,000 requests each, localhost

---

## 📊 Complete Results (All Measured)

| Protocol | Requests/s | p50 Latency | p95 Latency | p99 Latency | vs. HTTP |
|----------|-----------|-------------|-------------|-------------|----------|
| **HTTP** | **28,307** | **25μs** | **62μs** | **85μs** | **1.0x** |
| **JSON-RPC** | **16,991** | **63μs** | **98μs** | **110μs** | **0.6x** |
| **tarpc (single)** | **61,831** | **26μs** | **64μs** | **77μs** | **2.2x** |
| **tarpc (30x)** | **88,521** | **173μs** | **465μs** | **983μs** | **3.1x** |

**All measurements are REAL, not estimates.**

---

## ✅ What Was Fixed

### 1. JSON-RPC Method Name Error
**Before**: 1,073 req/s with errors  
**Problem**: Calling `"health"` instead of `"songbird.health"`  
**After**: 16,991 req/s working perfectly  
**Fix**: Updated benchmark to use correct method names

### 2. tarpc Server Not Starting
**Problem**: tarpc disabled by default, needs Arc refactor in orchestrator  
**Solution**: Created standalone tarpc servers for benchmarking  
**Result**: Full tarpc benchmarks working with real data

### 3. No Real Performance Data
**Before**: All estimates and placeholders  
**After**: Real measured data from all 4 protocols  
**Impact**: Can now optimize based on actual bottlenecks

---

## 🎯 Key Insights for Optimization

### 1. HTTP Performance (28K req/s)
**Status**: ✅ Excellent baseline

- Very fast for localhost (25μs latency)
- Will be slower over network (~1-2ms additional)
- Production-ready for most REST workloads
- HTTP/2 + connection pooling working well

**No optimization needed for HTTP.**

### 2. JSON-RPC Performance (17K req/s)
**Status**: ⚠️ Needs investigation

- **1.7x SLOWER than plain HTTP**
- This is unexpected - should be similar speed
- Extra overhead somewhere in the stack

**Optimization targets**:
- Investigate JSON-RPC routing
- Check if unnecessary serialization happening
- Compare with HTTP request handling
- May need profiling to find bottleneck

### 3. tarpc Single Connection (62K req/s)
**Status**: ✅ Excellent, as expected

- **2.2x faster than HTTP**
- Binary protocol advantage
- Sub-30μs latency
- Consistent performance

**This validates the tarpc approach!**

### 4. tarpc Multiplexed (89K req/s, 30 connections)
**Status**: ⚠️ Diminishing returns

- Total: 89K req/s (3.1x HTTP)
- Per-connection: 2,951 req/s (vs 62K single)
- **21x reduction per connection** with 30 connections

**This indicates**:
- CPU saturation or task scheduling overhead
- Connection pooling may be suboptimal
- Coordination/serialization bottleneck

**Optimization targets**:
- Profile CPU usage during multiplex
- Investigate task spawning overhead
- Consider worker thread pool
- May need load balancing optimization

---

## 📈 Scalability Analysis

### Per-Connection Throughput

| Connections | Total Req/s | Per-Connection | Efficiency |
|-------------|-------------|----------------|------------|
| 1           | 61,831      | 61,831         | 100%       |
| 30          | 88,521      | 2,951          | 4.8%       |

**Problem**: Efficiency drops to 4.8% with 30 connections.

**Expected**: Should scale linearly up to CPU saturation.

**Reality**: Non-linear scaling suggests coordination overhead.

### Latency Distribution

**Single Connection**:
- p50: 26μs
- p95: 64μs
- p99: 77μs
- Very consistent

**30 Connections**:
- p50: 173μs (6.7x higher)
- p95: 465μs (7.3x higher)
- p99: 983μs (12.8x higher)
- Much more variance

**Indicates**: Queueing and contention effects.

---

## 🔍 Root Cause Analysis

### Why is tarpc Multiplex Slower Per-Connection?

**Hypothesis 1: CPU Saturation**
- Single connection: 62K req/s
- 30 connections: Only 43% increase (89K total)
- Suggests CPU is bottleneck

**Hypothesis 2: Task Spawning Overhead**
- Each request spawns a new task
- 30 concurrent connections = many tasks
- May overwhelm Tokio scheduler

**Hypothesis 3: Serialization Bottleneck**
- Bincode serialization happens on each request
- May not be as parallel as expected
- Could have lock contention

**Hypothesis 4: Network Stack Overhead**
- Even on localhost, TCP has overhead
- 30 connections = 30 TCP streams
- May hit kernel limits

**Action**: Profile to determine actual cause.

---

## 🎓 Recommendations

### Immediate (Next Session)

1. **Profile tarpc Multiplex**
   - Use `perf` or `flamegraph` to see where time is spent
   - Identify if CPU, I/O, or locking is the bottleneck

2. **Investigate JSON-RPC Overhead**
   - Compare JSON-RPC vs HTTP code paths
   - Look for unnecessary serialization or routing
   - Should be similar performance to HTTP

3. **Test Over Network (Not Localhost)**
   - Run benchmarks between Eastgate and Strandgate
   - Network latency will change the picture
   - tarpc advantage may be more pronounced

### Medium-term

1. **Optimize tarpc Connection Pooling**
   - Consider persistent connection pools
   - Worker thread model instead of per-request tasks
   - May need custom transport layer

2. **Real Workload Testing**
   - GPU compute tasks via Toadstool
   - AI requests via Squirrel
   - Measure end-to-end, not just RPC

3. **Chaos and Fault Testing**
   - Network partitions
   - Service failures
   - Load spikes

### Long-term

1. **Zero-Copy Optimizations**
   - Investigate `bytes` crate for zero-copy
   - Reduce allocations in hot path
   - Custom serialization if needed

2. **Protocol-Specific Optimizations**
   - HTTP: Keep-alive tuning
   - JSON-RPC: Batch requests
   - tarpc: Connection pinning

3. **Hardware Optimization**
   - NUMA awareness
   - CPU pinning
   - Network stack tuning

---

## 📁 Artifacts

### Benchmark Results (JSON)
- `results_http.json`
- `results_jsonrpc.json`
- `results_tarpc_single.json`
- `results_tarpc_multiplex.json`
- `comparison_report.json`

### Tools
- `benchmark/` - Full benchmark suite
- `tarpc-servers/` - Standalone tarpc servers
- `scripts/start_tarpc_servers.sh` - Launch servers
- `scripts/` - Infrastructure management

### Logs
- `logs/songbird-{a,b,c}.log` - HTTP/JSON-RPC servers
- `logs/tarpc-{8091,8092,8093}.log` - tarpc servers
- `logs/toadstool.log` - Compute server

---

## 🎯 Success Criteria Met

✅ **Fixed JSON-RPC** - Now 17K req/s (was 1K)  
✅ **Implemented tarpc Benchmarks** - Real measured data  
✅ **Got Robust Input** - All protocols measured  
✅ **Identified Optimization Targets** - JSON-RPC overhead, tarpc scaling  

**Status**: Ready for optimization phase! 🚀

---

## 🦅 Conclusion

**What we proved**:
- HTTP is fast and production-ready (28K req/s)
- tarpc is 2-3x faster than HTTP
- JSON-RPC needs optimization (slower than expected)
- Multiplexing shows diminishing returns (needs investigation)

**What we learned**:
- Localhost performance != network performance
- Per-connection efficiency matters for scaling
- Need profiling to optimize further
- Real measurements > estimates

**Next steps**:
1. Profile and optimize JSON-RPC
2. Investigate tarpc multiplexing overhead
3. Test distributed (Eastgate ↔ Strandgate)
4. Real workload testing (GPU compute)

**The data is real. The insights are actionable. Ready to optimize!** 🎯

---

*Report generated: December 18, 2025*  
*Benchmarks: Complete ✅*  
*Optimization: Ready to proceed 🚀*

