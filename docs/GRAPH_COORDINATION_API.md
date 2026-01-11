# 📊 Graph Coordination Validation API

**Version**: v3.21.0 (Collaborative Intelligence - Week 3)  
**API**: `coordination.validate_pattern`  
**Status**: ✅ Production Ready  
**Last Updated**: January 13, 2026

---

## 🎯 Overview

The **Coordination Validation API** analyzes computational graphs to detect coordination patterns and validate resource availability. It automatically identifies execution patterns (sequential, parallel, pipeline, MapReduce, hybrid) and verifies that sufficient primals are available to execute the graph successfully.

### Key Features

- 🔍 **Automatic Pattern Detection**: Identifies coordination patterns from graph topology
- 📊 **Resource Validation**: Checks primal availability via service registry
- 🎯 **5 Pattern Types**: Sequential, Parallel, Pipeline, MapReduce, Hybrid
- ⚡ **DFS-Based Analysis**: Fast cycle detection and topology analysis
- 🔐 **Zero Hardcoding**: All primal discovery via capability matching
- 🧠 **Smart Diagnostics**: Detailed issues with severity levels

---

## 📋 API Specification

### Method: `coordination.validate_pattern`

**Purpose**: Validate coordination patterns and resource availability for a computational graph.

**Protocol**: JSON-RPC 2.0 over Unix Socket

**Socket Path**: `/run/user/{uid}/songbird-{family_id}.sock`

---

## 📥 Request Format

### Request Structure

```json
{
  "jsonrpc": "2.0",
  "method": "coordination.validate_pattern",
  "params": {
    "graph": {
      "id": "string",
      "name": "string",
      "nodes": [
        {
          "id": "string",
          "primal_name": "string (optional)",
          "capability": "string",
          "inputs": ["string"],
          "outputs": ["string"],
          "config": {},
          "preferred_protocol": "string (optional)",
          "timeout_secs": 0
        }
      ],
      "edges": [
        {
          "from": "string",
          "to": "string",
          "data_mapping": {
            "input_key": "string",
            "output_key": "string"
          }
        }
      ],
      "metadata": {
        "created_by": "string",
        "version": "string",
        "description": "string",
        "tags": ["string"]
      }
    }
  },
  "id": 1
}
```

### Request Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `graph.id` | string | ✅ Yes | Unique graph identifier |
| `graph.name` | string | ✅ Yes | Human-readable graph name |
| `graph.nodes` | array | ✅ Yes | List of computation nodes |
| `graph.edges` | array | ✅ Yes | Dependencies between nodes |
| `graph.metadata` | object | ❌ No | Optional metadata |

#### Node Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | ✅ Yes | Unique node identifier within graph |
| `capability` | string | ✅ Yes | Required capability (e.g., "encryption") |
| `primal_name` | string | ❌ No | Preferred primal (optional hint) |
| `inputs` | array | ✅ Yes | Expected input keys |
| `outputs` | array | ✅ Yes | Produced output keys |
| `config` | object | ❌ No | Node-specific configuration |
| `preferred_protocol` | string | ❌ No | Preferred protocol (json-rpc, grpc) |
| `timeout_secs` | number | ❌ No | Execution timeout |

#### Edge Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `from` | string | ✅ Yes | Source node ID |
| `to` | string | ✅ Yes | Target node ID |
| `data_mapping` | object | ❌ No | Input/output key mapping |

---

## 📤 Response Format

### Success Response

```json
{
  "jsonrpc": "2.0",
  "result": {
    "valid": true,
    "pattern": "sequential",
    "description": "Linear sequential execution pattern (3 stages)",
    "issues": []
  },
  "id": 1
}
```

### Response with Warnings

```json
{
  "jsonrpc": "2.0",
  "result": {
    "valid": true,
    "pattern": "mapreduce",
    "description": "Map-reduce pattern with limited parallelism",
    "issues": [
      {
        "severity": "warning",
        "message": "Parallel execution requires 3 'compute' primals, but only 1 available. Execution will be sequential.",
        "node_id": "compute-phase",
        "suggestion": "Register more compute workers for optimal performance"
      }
    ]
  },
  "id": 1
}
```

### Response with Errors

```json
{
  "jsonrpc": "2.0",
  "result": {
    "valid": false,
    "pattern": "sequential",
    "description": "Sequential pattern with missing capabilities",
    "issues": [
      {
        "severity": "error",
        "message": "No primal found with capability 'encryption'",
        "node_id": "encrypt",
        "suggestion": "Register BearDog or another encryption provider"
      },
      {
        "severity": "error",
        "message": "No primal found with capability 'storage'",
        "node_id": "store",
        "suggestion": "Register NestGate or another storage provider"
      }
    ]
  },
  "id": 1
}
```

### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `valid` | boolean | Whether the graph is valid for execution |
| `pattern` | string | Detected coordination pattern |
| `description` | string | Human-readable pattern description |
| `issues` | array | List of validation issues |

#### Pattern Types

| Pattern | Description | Topology |
|---------|-------------|----------|
| `sequential` | Linear chain of nodes | A → B → C → D |
| `parallel` | Concurrent execution branches | A → (B1, B2, B3) → C |
| `pipeline` | Streaming stages with overlap | Stage1 → Stage2 → Stage3 |
| `mapreduce` | Map phase + reduce phase | A → (M1, M2, M3) → R |
| `hybrid` | Complex multi-pattern graph | Mixed patterns |

#### Issue Fields

| Field | Type | Description |
|-------|------|-------------|
| `severity` | string | `"error"` or `"warning"` |
| `message` | string | Human-readable issue description |
| `node_id` | string | Affected node (if applicable) |
| `suggestion` | string | Recommended action |

---

## 💡 Usage Examples

### Example 1: Sequential Data Pipeline

**Use Case**: Validate a simple data ingestion → processing → storage pipeline.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "coordination.validate_pattern",
  "params": {
    "graph": {
      "id": "data-pipeline-001",
      "name": "Daily Data Pipeline",
      "nodes": [
        {
          "id": "ingest",
          "capability": "data-ingestion",
          "inputs": [],
          "outputs": ["raw_data"]
        },
        {
          "id": "process",
          "capability": "data-processing",
          "inputs": ["raw_data"],
          "outputs": ["processed_data"]
        },
        {
          "id": "store",
          "capability": "storage",
          "inputs": ["processed_data"],
          "outputs": []
        }
      ],
      "edges": [
        {"from": "ingest", "to": "process"},
        {"from": "process", "to": "store"}
      ],
      "metadata": {
        "created_by": "biomeOS",
        "version": "1.0"
      }
    }
  },
  "id": 1
}
```

**Response** (All Primals Available):
```json
{
  "jsonrpc": "2.0",
  "result": {
    "valid": true,
    "pattern": "sequential",
    "description": "Linear sequential execution pattern (3 stages)",
    "issues": []
  },
  "id": 1
}
```

**Interpretation**:
- ✅ Pattern detected: Sequential (linear chain)
- ✅ All required capabilities available
- ✅ Graph ready for execution

---

### Example 2: Parallel Computation (MapReduce)

**Use Case**: Validate a parallel computation with map and reduce phases.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "coordination.validate_pattern",
  "params": {
    "graph": {
      "id": "parallel-compute-001",
      "name": "Parallel Data Analysis",
      "nodes": [
        {
          "id": "input",
          "capability": "input",
          "inputs": [],
          "outputs": ["dataset"]
        },
        {
          "id": "map1",
          "capability": "compute",
          "inputs": ["dataset"],
          "outputs": ["result1"]
        },
        {
          "id": "map2",
          "capability": "compute",
          "inputs": ["dataset"],
          "outputs": ["result2"]
        },
        {
          "id": "map3",
          "capability": "compute",
          "inputs": ["dataset"],
          "outputs": ["result3"]
        },
        {
          "id": "reduce",
          "capability": "aggregation",
          "inputs": ["result1", "result2", "result3"],
          "outputs": ["final_result"]
        }
      ],
      "edges": [
        {"from": "input", "to": "map1"},
        {"from": "input", "to": "map2"},
        {"from": "input", "to": "map3"},
        {"from": "map1", "to": "reduce"},
        {"from": "map2", "to": "reduce"},
        {"from": "map3", "to": "reduce"}
      ]
    }
  },
  "id": 2
}
```

**Response** (Sufficient Parallel Resources):
```json
{
  "jsonrpc": "2.0",
  "result": {
    "valid": true,
    "pattern": "mapreduce",
    "description": "Map-reduce pattern with 3 parallel map tasks and 1 reduce task",
    "issues": []
  },
  "id": 2
}
```

**Response** (Insufficient Parallel Resources):
```json
{
  "jsonrpc": "2.0",
  "result": {
    "valid": true,
    "pattern": "mapreduce",
    "description": "Map-reduce pattern with limited parallelism",
    "issues": [
      {
        "severity": "warning",
        "message": "Parallel execution requires 3 'compute' primals, but only 1 available. Execution will be sequential.",
        "node_id": null,
        "suggestion": "Register more compute workers (ToadStool) for optimal parallel performance"
      }
    ]
  },
  "id": 2
}
```

**Interpretation**:
- ✅ Pattern detected: MapReduce (fan-out → fan-in)
- ⚠️ Warning: Limited parallelism (will execute sequentially)
- ✅ Graph is still valid (can execute, just not optimally parallel)

---

### Example 3: Missing Capabilities

**Use Case**: Validate a graph with missing primals.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "coordination.validate_pattern",
  "params": {
    "graph": {
      "id": "incomplete-pipeline",
      "name": "Incomplete Pipeline",
      "nodes": [
        {
          "id": "input",
          "capability": "input",
          "inputs": [],
          "outputs": ["data"]
        },
        {
          "id": "encrypt",
          "capability": "encryption",
          "inputs": ["data"],
          "outputs": ["encrypted_data"]
        },
        {
          "id": "store",
          "capability": "storage",
          "inputs": ["encrypted_data"],
          "outputs": []
        }
      ],
      "edges": [
        {"from": "input", "to": "encrypt"},
        {"from": "encrypt", "to": "store"}
      ]
    }
  },
  "id": 3
}
```

**Response** (Missing Encryption and Storage Primals):
```json
{
  "jsonrpc": "2.0",
  "result": {
    "valid": false,
    "pattern": "sequential",
    "description": "Sequential pattern with missing capabilities",
    "issues": [
      {
        "severity": "error",
        "message": "No primal found with capability 'encryption'",
        "node_id": "encrypt",
        "suggestion": "Register BearDog or another encryption provider"
      },
      {
        "severity": "error",
        "message": "No primal found with capability 'storage'",
        "node_id": "store",
        "suggestion": "Register NestGate or another storage provider"
      }
    ]
  },
  "id": 3
}
```

**Interpretation**:
- ✅ Pattern detected: Sequential
- ❌ Errors: Missing encryption and storage capabilities
- ❌ Graph is NOT ready for execution

---

## 🔧 Integration Guide

### Python Client Example

```python
import json
import socket
import os

class SongbirdClient:
    def __init__(self, family_id: str):
        uid = os.getuid()
        self.socket_path = f"/run/user/{uid}/songbird-{family_id}.sock"
        
    def validate_coordination(self, graph: dict) -> dict:
        """Validate coordination pattern for a graph."""
        request = {
            "jsonrpc": "2.0",
            "method": "coordination.validate_pattern",
            "params": {"graph": graph},
            "id": 1
        }
        
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.connect(self.socket_path)
        
        sock.sendall(json.dumps(request).encode() + b'\n')
        response = json.loads(sock.recv(4096).decode())
        
        sock.close()
        return response["result"]

# Usage
client = SongbirdClient("nat0")

graph = {
    "id": "test-pipeline",
    "name": "Test Pipeline",
    "nodes": [
        {
            "id": "input",
            "capability": "input",
            "inputs": [],
            "outputs": ["data"]
        },
        {
            "id": "process",
            "capability": "compute",
            "inputs": ["data"],
            "outputs": ["result"]
        }
    ],
    "edges": [
        {"from": "input", "to": "process"}
    ]
}

result = client.validate_coordination(graph)

if result["valid"]:
    print(f"✅ Graph is valid! Pattern: {result['pattern']}")
else:
    print(f"❌ Graph has issues:")
    for issue in result["issues"]:
        print(f"  - [{issue['severity']}] {issue['message']}")
```

---

### Rust Client Example

```rust
use serde_json::json;
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn validate_coordination(
    socket_path: &str,
    graph: serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect(socket_path).await?;
    
    let request = json!({
        "jsonrpc": "2.0",
        "method": "coordination.validate_pattern",
        "params": {"graph": graph},
        "id": 1
    });
    
    let request_str = serde_json::to_string(&request)?;
    stream.write_all(request_str.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    
    let mut buffer = vec![0u8; 4096];
    let n = stream.read(&mut buffer).await?;
    
    let response: serde_json::Value = serde_json::from_slice(&buffer[..n])?;
    Ok(response["result"].clone())
}

// Usage
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uid = std::process::id();
    let socket_path = format!("/run/user/{}/songbird-nat0.sock", uid);
    
    let graph = json!({
        "id": "test-pipeline",
        "name": "Test Pipeline",
        "nodes": [
            {
                "id": "input",
                "capability": "input",
                "inputs": [],
                "outputs": ["data"]
            },
            {
                "id": "process",
                "capability": "compute",
                "inputs": ["data"],
                "outputs": ["result"]
            }
        ],
        "edges": [
            {"from": "input", "to": "process"}
        ]
    });
    
    let result = validate_coordination(&socket_path, graph).await?;
    
    if result["valid"].as_bool().unwrap_or(false) {
        println!("✅ Graph is valid! Pattern: {}", result["pattern"]);
    } else {
        println!("❌ Graph has issues:");
        if let Some(issues) = result["issues"].as_array() {
            for issue in issues {
                println!("  - [{}] {}", 
                    issue["severity"], 
                    issue["message"]
                );
            }
        }
    }
    
    Ok(())
}
```

---

## 🎯 Pattern Detection Logic

### Sequential Pattern

**Characteristics**:
- Linear chain (each node has at most 1 incoming and 1 outgoing edge)
- No branches or merges
- Single execution path

**Example**:
```
A → B → C → D
```

**Use Cases**:
- Data pipelines
- ETL workflows
- Sequential processing stages

---

### Parallel Pattern

**Characteristics**:
- One or more fan-out points (1 node → many nodes)
- May or may not have fan-in (many nodes → 1 node)
- Multiple execution paths

**Example**:
```
    ┌→ B1 →┐
A →→ B2 →→ C
    └→ B3 →┘
```

**Use Cases**:
- Parallel data processing
- Concurrent API calls
- Multi-branch workflows

---

### Pipeline Pattern

**Characteristics**:
- Multiple stages with clear layering
- Each stage processes data before passing to next stage
- Stages may overlap in execution (streaming)

**Example**:
```
Stage 1 → Stage 2 → Stage 3 → Stage 4
```

**Use Cases**:
- Streaming data processing
- Multi-stage transformations
- Image/video processing pipelines

---

### MapReduce Pattern

**Characteristics**:
- Single entry point (data source)
- Fan-out to multiple map tasks
- Fan-in to single or few reduce tasks
- Clear map and reduce phases

**Example**:
```
        ┌→ Map1 →┐
Input →→ Map2 →→ Reduce → Output
        └→ Map3 →┘
```

**Use Cases**:
- Distributed data analysis
- Parallel aggregation
- Batch processing

---

### Hybrid Pattern

**Characteristics**:
- Complex graphs with multiple patterns
- May contain sequential, parallel, and pipeline segments
- Multiple entry or exit points

**Example**:
```
A → B → (C1, C2) → D → (E1, E2, E3) → F
```

**Use Cases**:
- Complex workflows
- Multi-stage data processing with branches
- Advanced orchestration scenarios

---

## 📊 Validation Rules

### Resource Checking

The coordination validator checks primal availability via the service registry:

1. **Sequential Pattern**: Checks each node's capability sequentially
2. **Parallel Pattern**: Checks if enough primals exist for concurrent execution
3. **Pipeline Pattern**: Checks each stage's primals
4. **MapReduce Pattern**: Checks map phase parallelism and reduce phase availability

### Issue Severity

| Severity | Meaning | Graph Valid? |
|----------|---------|--------------|
| `error` | Blocking issue (missing capability) | ❌ No |
| `warning` | Non-blocking issue (suboptimal performance) | ✅ Yes |

### Warnings vs Errors

**Errors** (blocking):
- Missing required capability
- No primals registered for a node
- Unhealthy primals only available

**Warnings** (non-blocking):
- Insufficient primals for optimal parallelism
- Degraded primal health
- Protocol mismatches

---

## 🔍 Troubleshooting

### Issue: "No primal found with capability X"

**Cause**: No primal is registered with the required capability.

**Solution**:
1. Check registered primals: Use `discover_by_capability("*")` to see all registered primals
2. Register the missing primal: Ensure the required primal (e.g., BearDog for encryption) is running and registered
3. Check capability names: Ensure graph uses the same capability names as registered primals

---

### Issue: "Parallel execution requires N primals, but only M available"

**Cause**: Graph requires parallel execution but insufficient primals are registered.

**Solution**:
1. **Accept sequential execution**: Graph is still valid, will execute nodes sequentially
2. **Register more primals**: Launch additional instances of the required primal
3. **Modify graph**: Reduce parallelism requirements if sequential is acceptable

---

### Issue: "Pattern: hybrid"

**Cause**: Graph has complex topology that doesn't fit simple patterns.

**Solution**:
- This is informational, not an error
- Hybrid patterns are fully supported
- Review the issues array for any actual validation problems

---

## 🚀 Performance

### Benchmarks

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Small graph (< 10 nodes) | < 10ms | 1000+ req/sec |
| Medium graph (10-50 nodes) | < 50ms | 200+ req/sec |
| Large graph (50-200 nodes) | < 200ms | 50+ req/sec |
| Concurrent validation | < 100ms | Thread-safe |

### Optimization Tips

1. **Cache results**: Validation results can be cached if graph and registry haven't changed
2. **Batch validate**: Validate multiple graphs in parallel
3. **Early termination**: Validation stops on first error (for invalid graphs)

---

## 🔐 Security

### Access Control

- Coordination validation is read-only (no side effects)
- No authentication required (local Unix socket only)
- Graph structure is not persisted

### Data Privacy

- Graph definitions are not stored
- Validation is stateless
- No logging of sensitive graph data

---

## 📚 Related APIs

### Service Registry APIs

- `register_service`: Register primals for discovery
- `discover_by_capability`: Find primals by capability
- `get_service_health`: Check primal health

### Graph Validation APIs (v3.21.0)

- `graph.validate`: Validate graph structure and schema
- `graph.check_availability`: Check primal availability
- `graph.suggest_alternatives`: Get alternative primals

---

## 📝 Changelog

### v3.21.0 (January 13, 2026)
- ✅ Initial release of coordination validation
- ✅ 5 pattern types supported
- ✅ DFS-based pattern detection
- ✅ Resource availability checking
- ✅ Issue severity (errors vs warnings)

---

## 🙏 Support

For questions or issues:
- **Documentation**: See specs/COLLABORATIVE_INTELLIGENCE_GRAPH_VALIDATION.md
- **Examples**: See tests/e2e_coordination_validation.rs
- **Issues**: GitHub issues with tag `coordination-validation`

---

**Status**: ✅ **PRODUCTION READY**  
**Version**: v3.21.0  
**Last Updated**: January 13, 2026

🎵 **Songbird - Collaborative Intelligence Coordination Validation** 🎵

