# 🗄️ Nestgate Integration: Data Primal Benchmarking

**Date**: December 18, 2025  
**Status**: Planning (Nestgate coming online soon)  
**Purpose**: Benchmark data-intensive workloads with Songbird orchestration

---

## 🎯 Overview

**Nestgate**: Data primal (NAS, database, blob storage)  
**Role**: Persistent storage, data streaming, slow tower integration  
**Integration**: Via Songbird orchestration layer

---

## 📊 Data Workload Characteristics

### Different from Compute Workloads

| Aspect | Compute (Toadstool) | Data (Nestgate) |
|--------|---------------------|-----------------|
| Request size | Small (KB) | Large (MB-GB) |
| Latency | Low (ms) | Higher (ms-s) |
| Throughput metric | Req/s | MB/s or GB/s |
| Bottleneck | CPU | Bandwidth |
| Scaling | CPU cores | Network/disk |

**Key difference**: Data workloads are **bandwidth-bound**, not CPU-bound

---

## 🔄 Integration Patterns

### Pattern 1: Object Storage (S3-style)

**Use case**: Store ML models, datasets, results

```rust
// Via Songbird orchestration
let data = vec![0u8; 10_000_000]; // 10MB
songbird.route_to("nestgate")
    .store_object("models/llama3-70b.safetensors", data)
    .await?;

// Later retrieve
let model_data = songbird.route_to("nestgate")
    .get_object("models/llama3-70b.safetensors")
    .await?;
```

**Benchmark**:
- Upload: MB/s throughput
- Download: MB/s throughput
- Concurrent operations: Scaling

---

### Pattern 2: Streaming Data

**Use case**: Real-time data ingestion, log streaming

```rust
// Stream data to Nestgate
let stream = songbird.route_to("nestgate")
    .create_stream("telemetry/sensors")
    .await?;

for reading in sensor_readings {
    stream.append(reading).await?;
}
```

**Benchmark**:
- Ingest rate: Records/s
- Latency: ms to persist
- Concurrent streams: How many?

---

### Pattern 3: Distributed Query

**Use case**: Query data across multiple towers

```rust
// Query data distributed across towers
let results = songbird.broadcast_query()
    .to_capability("data-storage")
    .query("SELECT * FROM logs WHERE timestamp > NOW() - INTERVAL '1 hour'")
    .await?;

// Songbird aggregates results
```

**Benchmark**:
- Query latency: ms
- Result aggregation: Records/s
- Network overhead: vs single-tower

---

### Pattern 4: Slow Tower as NAS

**Use case**: Old hardware as network-attached storage

```rust
// Nest tower (slow hardware) provides storage
// Songbird routes data workloads appropriately

// Heavy compute → Eastgate (RTX 2070)
let result = songbird.route_to_capability("gpu-inference")
    .infer(prompt)
    .await?;

// Persistent storage → Nestgate (slow tower, big disks)
songbird.route_to_capability("bulk-storage")
    .store(result)
    .await?;
```

**Benchmark**:
- Storage write: MB/s
- Storage read: MB/s
- Overhead of routing: ms

---

## 🧪 Benchmark Suite for Nestgate

### Benchmark 1: Upload Throughput

**Scenario**: Upload files of varying sizes

```rust
// Test: Upload 100 files of 10MB each
for i in 0..100 {
    let data = vec![0u8; 10_000_000];
    let start = Instant::now();
    
    songbird.route_to("nestgate")
        .upload(format!("test/file_{}.bin", i), data)
        .await?;
    
    let duration = start.elapsed();
    record_metric("upload_latency", duration);
}

// Metrics:
// - Throughput: 100MB / total_time
// - Latency: per-file upload time
// - Concurrent: Run multiple uploads in parallel
```

**Expected**: 50-500 MB/s depending on network

---

### Benchmark 2: Download Throughput

**Scenario**: Download files concurrently

```rust
// Test: Download 50 files concurrently
let handles = (0..50).map(|i| {
    tokio::spawn(async move {
        let start = Instant::now();
        let data = songbird.route_to("nestgate")
            .download(format!("test/file_{}.bin", i))
            .await?;
        (data.len(), start.elapsed())
    })
}).collect::<Vec<_>>();

// Metrics:
// - Total throughput: sum(file_sizes) / total_time
// - Per-connection: MB/s per download
// - Scaling: Does throughput increase with concurrency?
```

**Expected**: Better scaling than compute (network-bound)

---

### Benchmark 3: Mixed Workload

**Scenario**: Simulate real application

```rust
// Simulate: Squirrel AI inference + result storage
// 1. Download model from Nestgate
let model = songbird.route_to("nestgate")
    .download("models/gpt2.safetensors")
    .await?;

// 2. Run inference on Eastgate (GPU)
let result = songbird.route_to_capability("gpu-inference")
    .infer_with_model(model, prompt)
    .await?;

// 3. Store result in Nestgate
songbird.route_to("nestgate")
    .upload("results/inference_123.json", result)
    .await?;

// Metrics:
// - End-to-end latency
// - Bandwidth utilization
// - Songbird routing overhead
```

**Expected**: Dominated by network and inference time

---

### Benchmark 4: Distributed Data Processing

**Scenario**: Process data across multiple towers

```rust
// Nestgate has 1TB dataset
// Split processing across Eastgate + Strandgate

// Songbird orchestrates:
// 1. Chunk data
let chunks = songbird.route_to("nestgate")
    .chunk_dataset("large_dataset.parquet", chunk_size: 100_000)
    .await?;

// 2. Distribute processing
let results = songbird.broadcast()
    .to_capability("data-processing")
    .map(|chunk| process_chunk(chunk))
    .collect()
    .await?;

// 3. Aggregate results in Nestgate
songbird.route_to("nestgate")
    .aggregate_results(results)
    .await?;

// Metrics:
// - Total processing time
// - Network transfer time
// - Coordination overhead
// - Speedup vs single-tower
```

**Expected**: Linear scaling up to network saturation

---

## 📈 Expected Performance Profiles

### Localhost Testing (Development)

**Nestgate on same machine**:
- Upload/Download: 1-5 GB/s (memory speed)
- Not representative of real deployment
- Good for functional testing

### LAN Testing (Production-like)

**Nestgate on Gigabit LAN**:
- Upload/Download: 100-120 MB/s (gigabit limit)
- Latency: 0.5-2ms
- Realistic performance profile

**Nestgate on 10Gb LAN** (if available):
- Upload/Download: 800-1200 MB/s
- Latency: 0.2-1ms
- High-performance scenario

### Slow Tower as NAS

**Old hardware** (spinning disks):
- Upload: 50-100 MB/s (disk write speed)
- Download: 80-150 MB/s (disk read speed)
- Latency: 2-10ms (disk seek)
- Good for bulk storage, not hot data

---

## 🎯 Integration Checklist

### Phase 1: Basic Connectivity ✅ (Ready when Nestgate live)

- [ ] Nestgate registers with Songbird federation
- [ ] Songbird discovers Nestgate capabilities
- [ ] Basic health check: `/health` endpoint
- [ ] Capability advertisement: `["data-storage", "blob-store"]`

### Phase 2: Upload/Download (Week 1)

- [ ] Implement upload API via Songbird
- [ ] Implement download API via Songbird
- [ ] Benchmark single file upload/download
- [ ] Benchmark concurrent operations

### Phase 3: Streaming (Week 2)

- [ ] Implement streaming upload
- [ ] Implement streaming download
- [ ] Benchmark stream throughput
- [ ] Test with large files (>1GB)

### Phase 4: Distributed Operations (Week 3)

- [ ] Implement distributed query
- [ ] Implement result aggregation
- [ ] Benchmark distributed processing
- [ ] Test multi-tower data workflow

---

## 🚀 Quick Start (When Nestgate Live)

### 1. Verify Nestgate Registration

```bash
# Check if Nestgate is discovered
curl -s https://localhost:8443/api/federation/services | jq '.[] | select(.capabilities[] | contains("data"))'
```

### 2. Run Basic Upload Test

```bash
# Upload test file via Songbird
curl -X POST https://localhost:8443/api/route/nestgate/upload \
  -F "file=@test.bin" \
  -F "path=test/benchmark.bin"
```

### 3. Run Upload Benchmark

```bash
cd showcase/05-albatross-multiplex/benchmark
cargo run --release --bin bench-nestgate-upload -- -n 100 --size 10MB
```

### 4. Run Mixed Workload

```bash
# Simulate Squirrel + Nestgate workflow
cargo run --release --bin bench-data-ml-pipeline
```

---

## 📊 Success Metrics

### Functional Goals

- ✅ Nestgate discovered by Songbird
- ✅ Upload/download working
- ✅ Concurrent operations work
- ✅ Multi-tower data processing works

### Performance Goals

**Upload**:
- Target: 80% of network bandwidth
- Good: 50-100 MB/s on Gigabit
- Excellent: 800-1000 MB/s on 10Gb

**Download**:
- Target: 90% of network bandwidth
- Good: 80-110 MB/s on Gigabit
- Excellent: 900-1100 MB/s on 10Gb

**Scaling**:
- Target: Linear up to network saturation
- Good: 80% efficiency with 10 concurrent ops
- Excellent: 90% efficiency with 100 concurrent ops

---

## 🔮 Future: Multi-Tower Data Mesh

### Vision

```
    Eastgate (Compute)
         |
    Songbird (Orchestrator)
       /   |   \
      /    |    \
Nestgate  Slow   Slow
 (Fast)  Tower1 Tower2
 (SSD)   (HDD)  (HDD)
```

**Capabilities**:
- Hot data → Nestgate (fast SSD)
- Cold data → Slow towers (cheap HDD)
- Songbird routes based on data temperature
- Automatic tiering and migration

**Benchmark**: Full mesh data processing with tiering

---

## 📝 Notes for Implementation

### Songbird Routing

```rust
// Songbird needs to learn about data-specific routing:
match request.workload_type {
    Workload::Compute => route_to_gpu(),
    Workload::Data => route_to_storage(),
    Workload::Mixed => orchestrate_pipeline(),
}
```

### Capability Advertisement

```json
{
  "node_id": "nestgate-1",
  "capabilities": [
    "data-storage",
    "blob-store",
    "object-storage",
    "bulk-storage"
  ],
  "resources": {
    "storage_gb": 10000,  // 10TB
    "bandwidth_mbps": 1000  // Gigabit
  }
}
```

---

*Status: Ready to integrate when Nestgate is live*  
*Next: Run distributed benchmarks, then add Nestgate*  
*Goal: Full multi-primal data + compute orchestration* 🚀

