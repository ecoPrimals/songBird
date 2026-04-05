# 🤝 Collaborative Intelligence - Graph Validation Specification

**Version**: v3.21.0  
**Date**: January 11, 2026  
**Status**: 📋 **SPECIFICATION** - Ready for Implementation  
**Owner**: Songbird Team  
**Timeline**: 3 weeks (Jan 13 - Feb 3, 2026)

---

## 🎯 Overview

Songbird will provide graph validation capabilities for the Collaborative Intelligence system, enabling users and AI to collaboratively design and deploy execution graphs.

**Core Mission**: Validate graph structure, check primal availability, suggest alternatives, and validate coordination patterns.

---

## 📋 Requirements (from biomeOS)

### Functional Requirements

1. **Graph Structure Validation**
   - Validate graph structure before execution
   - Detect cycles, orphan nodes, broken dependencies
   - Validate schema compliance

2. **Primal Availability Checking**
   - Check if required primals are registered and available
   - Verify health status of primals
   - Real-time availability monitoring

3. **Alternative Suggestions**
   - Suggest alternative primals if primary unavailable
   - Rank alternatives by health, protocol compatibility
   - Provide reasoning for suggestions

4. **Coordination Pattern Validation**
   - Validate coordination patterns (sequential, parallel, pipeline, MapReduce)
   - Detect bottlenecks and estimate latency
   - Suggest optimizations

### Non-Functional Requirements

- **Performance**: Validation < 100ms for graphs with < 50 nodes
- **Scalability**: Handle up to 1000 concurrent validation requests
- **Reliability**: 99.9% uptime for validation service
- **Thread Safety**: All operations must be thread-safe
- **Zero Hardcoding**: Dynamic discovery only

---

## 🏗️ Architecture

### Module Structure

```
crates/songbird-orchestrator/src/graph/
├── mod.rs                    # Module declaration & exports
├── types.rs                  # Graph data types (Graph, Node, Edge)
├── validator.rs              # Structure validation logic
├── availability.rs           # Availability checking (uses service registry)
├── coordination.rs           # Coordination pattern validation
└── tests.rs                  # Unit tests
```

### Integration Points

```
┌─────────────────────────────────────────────────────┐
│ Songbird Orchestrator (v3.21.0)                    │
│                                                     │
│  ┌──────────────────┐      ┌───────────────────┐  │
│  │  Service Registry│◄─────┤ Graph Validator   │  │
│  │  (v3.20.0)      │      │                   │  │
│  └──────────────────┘      └───────────────────┘  │
│           ▲                         │              │
│           │                         │              │
│           │  1. Check capabilities  │              │
│           │  2. Check health        │              │
│           │  3. Suggest alts        │              │
│           │                         ▼              │
│           │                ┌───────────────────┐  │
│           └────────────────┤  JSON-RPC Server  │  │
│                            └───────────────────┘  │
└─────────────────────────────────────────────────────┘
                                     │
                                     │ Unix Socket
                                     │
                         ┌───────────▼───────────┐
                         │ petalTongue / biomeOS │
                         └───────────────────────┘
```

---

## 📊 Data Types

### Graph

```rust
/// Represents a complete execution graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    /// Unique graph identifier
    pub id: String,
    
    /// Human-readable name
    pub name: String,
    
    /// List of nodes in the graph
    pub nodes: Vec<GraphNode>,
    
    /// List of edges (dependencies) between nodes
    pub edges: Vec<GraphEdge>,
    
    /// Graph metadata
    pub metadata: GraphMetadata,
}

/// Graph metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub created_by: String,
    pub created_at: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub version: String,
}
```

### GraphNode

```rust
/// Represents a single node in the execution graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique node identifier within the graph
    pub id: String,
    
    /// Primal name (optional - for informational purposes only)
    pub primal_name: Option<String>,
    
    /// Required capability (e.g., "encryption", "storage")
    pub capability: String,
    
    /// Input data keys this node expects
    pub inputs: Vec<String>,
    
    /// Output data keys this node produces
    pub outputs: Vec<String>,
    
    /// Node-specific configuration
    pub config: serde_json::Value,
    
    /// Preferred protocol (e.g., "json-rpc", "tarpc")
    pub preferred_protocol: Option<String>,
    
    /// Timeout in seconds
    pub timeout_secs: Option<u64>,
}
```

### GraphEdge

```rust
/// Represents a dependency between two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node ID
    pub from: String,
    
    /// Target node ID
    pub to: String,
    
    /// Data key mapping (optional)
    /// Maps source output to target input
    pub data_mapping: Option<HashMap<String, String>>,
}
```

---

## 🔧 API Specifications

### 1. graph.validate

**Purpose**: Validate graph structure and schema

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "graph.validate",
  "params": {
    "graph": {
      "id": "graph-123",
      "name": "Data Processing Pipeline",
      "nodes": [
        {
          "id": "node-1",
          "capability": "encryption",
          "inputs": ["raw_data"],
          "outputs": ["encrypted_data"],
          "config": {}
        },
        {
          "id": "node-2",
          "capability": "storage",
          "inputs": ["encrypted_data"],
          "outputs": ["storage_id"],
          "config": {}
        }
      ],
      "edges": [
        {
          "from": "node-1",
          "to": "node-2",
          "data_mapping": {"encrypted_data": "encrypted_data"}
        }
      ],
      "metadata": {
        "created_by": "user@example.com",
        "created_at": "2026-01-11T10:00:00Z",
        "version": "1.0"
      }
    }
  },
  "id": 1
}
```

**Response (Success)**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "valid": true,
    "issues": [],
    "warnings": [
      "Node 'node-1' has input 'raw_data' with no source (graph entry point)"
    ],
    "info": {
      "node_count": 2,
      "edge_count": 1,
      "entry_points": ["node-1"],
      "exit_points": ["node-2"],
      "has_cycles": false
    }
  },
  "id": 1
}
```

**Response (Error)**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "valid": false,
    "issues": [
      {
        "severity": "error",
        "code": "CYCLE_DETECTED",
        "message": "Cycle detected: node-1 → node-2 → node-1",
        "nodes": ["node-1", "node-2"]
      },
      {
        "severity": "error",
        "code": "ORPHAN_NODE",
        "message": "Node 'node-3' has no inputs or outputs",
        "nodes": ["node-3"]
      }
    ],
    "warnings": [],
    "info": null
  },
  "id": 1
}
```

**Validation Rules**:
1. ✅ No cycles in dependency graph
2. ✅ All nodes reachable from at least one entry point
3. ✅ All node inputs satisfied by edge sources or graph inputs
4. ✅ No duplicate node IDs
5. ✅ All edges reference valid node IDs
6. ✅ Required fields present (id, capability, inputs, outputs)

---

### 2. graph.check_availability

**Purpose**: Check if required primals are available

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "graph.check_availability",
  "params": {
    "graph": {
      "id": "graph-123",
      "nodes": [...]
    }
  },
  "id": 2
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "available": ["node-1", "node-2"],
    "unavailable": ["node-3"],
    "unhealthy": [],
    "degraded": [],
    "details": {
      "node-1": {
        "status": "available",
        "primal": "Security Provider",
        "service_id": "security_provider-nat0-abc123",
        "endpoint": "/run/user/1000/security_provider-nat0.sock",
        "protocol": "json-rpc",
        "health_status": "healthy",
        "last_seen": "2026-01-11T10:00:00Z"
      },
      "node-2": {
        "status": "available",
        "primal": "Storage Provider",
        "service_id": "storage_provider-nat0-def456",
        "endpoint": "/run/user/1000/storage_provider-nat0.sock",
        "protocol": "json-rpc",
        "health_status": "healthy",
        "last_seen": "2026-01-11T10:00:05Z"
      },
      "node-3": {
        "status": "unavailable",
        "required_capability": "compute",
        "reason": "No primal registered with capability 'compute'",
        "suggested_action": "Deploy Compute Provider or register alternative compute provider"
      }
    },
    "summary": {
      "total_nodes": 3,
      "available_nodes": 2,
      "availability_percent": 66.67
    }
  },
  "id": 2
}
```

**Availability Status**:
- `available`: Primal registered, healthy, ready to use
- `unavailable`: No primal with required capability registered
- `unhealthy`: Primal registered but health status is "down" or "unknown"
- `degraded`: Primal registered but health status is "degraded"

---

### 3. graph.suggest_alternatives

**Purpose**: Suggest alternative primals for a node

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "graph.suggest_alternatives",
  "params": {
    "node": {
      "id": "node-1",
      "capability": "encryption",
      "preferred_primal": "Security Provider",
      "preferred_protocol": "json-rpc"
    }
  },
  "id": 3
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "alternatives": [
      {
        "rank": 1,
        "service_id": "security_provider-nat0-abc123",
        "primal_name": "Security Provider",
        "endpoint": "/run/user/1000/security_provider-nat0.sock",
        "protocol": "json-rpc",
        "health_status": "healthy",
        "last_seen": "2026-01-11T10:00:00Z",
        "reason": "Primary primal with capability 'encryption', protocol match, healthy",
        "compatibility_score": 100
      }
    ],
    "recommendation": {
      "service_id": "security_provider-nat0-abc123",
      "reason": "Best match: healthy, protocol compatible, primary provider"
    },
    "unavailable_reason": null
  },
  "id": 3
}
```

**Ranking Criteria**:
1. Health status (healthy > degraded > down)
2. Protocol compatibility (exact match > compatible > incompatible)
3. Last seen timestamp (more recent = better)
4. Preferred primal match (if specified)

**Compatibility Score**:
- 100: Perfect match (healthy, protocol match, preferred primal)
- 80-99: Good match (healthy, protocol compatible)
- 60-79: Acceptable match (degraded but usable)
- 40-59: Poor match (protocol incompatible but capability present)
- < 40: Not recommended

---

### 4. coordination.validate_pattern

**Purpose**: Validate coordination pattern

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "coordination.validate_pattern",
  "params": {
    "pattern": {
      "type": "pipeline",
      "nodes": ["node-1", "node-2", "node-3"],
      "dependencies": [
        {"from": "node-1", "to": "node-2"},
        {"from": "node-2", "to": "node-3"}
      ]
    }
  },
  "id": 4
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "valid": true,
    "pattern_type": "pipeline",
    "characteristics": {
      "is_linear": true,
      "parallelizable": false,
      "has_fan_out": false,
      "has_fan_in": false
    },
    "performance": {
      "estimated_latency_ms": 500,
      "bottlenecks": ["node-2"],
      "parallelization_opportunities": []
    },
    "suggestions": [
      {
        "type": "optimization",
        "message": "Node 'node-2' appears compute-heavy, consider splitting if possible",
        "priority": "low"
      }
    ]
  },
  "id": 4
}
```

**Supported Patterns**:
1. **Sequential**: A → B → C (linear execution)
2. **Parallel**: A → [B, C, D] → E (fan-out, fan-in)
3. **Pipeline**: A → B → C → D (streaming pipeline)
4. **MapReduce**: A → [B1, B2, B3] → C (map phase, reduce phase)

---

## 🧪 Testing Strategy

### Unit Tests (24 total)

**Graph Validation (10 tests)**:
- `test_valid_simple_graph` - Valid 2-node graph
- `test_detect_cycle` - Cycle detection
- `test_detect_orphan_node` - Orphan node detection
- `test_validate_dependencies` - Input/output matching
- `test_duplicate_node_ids` - Duplicate ID detection
- `test_invalid_edge_reference` - Edge to non-existent node
- `test_missing_required_fields` - Schema validation
- `test_complex_graph_validation` - Large graph (20+ nodes)
- `test_multiple_entry_points` - Multiple start nodes
- `test_multiple_exit_points` - Multiple end nodes

**Availability Checking (8 tests)**:
- `test_all_available` - All primals available
- `test_some_unavailable` - Mixed availability
- `test_unhealthy_primal` - Primal down
- `test_degraded_primal` - Primal degraded
- `test_no_primals_registered` - Empty registry
- `test_protocol_filtering` - Protocol compatibility
- `test_health_status_changes` - Real-time status updates
- `test_suggest_alternatives_ranking` - Alternative ranking

**Coordination Patterns (6 tests)**:
- `test_validate_sequential_pattern` - Sequential validation
- `test_validate_parallel_pattern` - Parallel validation
- `test_validate_pipeline_pattern` - Pipeline validation
- `test_validate_mapreduce_pattern` - MapReduce validation
- `test_detect_bottleneck` - Bottleneck detection
- `test_suggest_parallelization` - Optimization suggestions

### E2E Tests (5 total)

**End-to-End Workflows**:
1. `test_full_validation_workflow` - Complete validation flow
2. `test_availability_with_real_registry` - Real service registry
3. `test_live_graph_modification` - Modify during execution
4. `test_multi_primal_coordination` - Complex coordination
5. `test_alternative_suggestion_workflow` - Full alternative flow

### Integration Tests (Week 4)

**With Other Primals**:
- Test with real Security Provider registration
- Test with real Storage Provider registration
- Test with multiple primals registered
- Test with primal health status changes

**With petalTongue**:
- Test graph editor integration
- Test real-time validation
- Test alternative suggestions in UI

**With biomeOS**:
- Test graph deployment workflow
- Test live modification during execution
- Test coordination pattern validation

---

## 📋 Implementation Checklist

### Week 1: Graph Validation

**Days 1-2: Type Definitions**
- [ ] Define `Graph`, `GraphNode`, `GraphEdge` structs
- [ ] Define `ValidationResult`, `ValidationIssue` structs
- [ ] Implement serialization/deserialization
- [ ] Add schema validation helpers

**Days 3-4: Structure Validation**
- [ ] Implement cycle detection algorithm
- [ ] Implement orphan node detection
- [ ] Implement dependency validation
- [ ] Implement duplicate ID checking

**Day 5: Testing & Integration**
- [ ] Write 10 unit tests
- [ ] Integrate with IPC server
- [ ] Add API documentation
- [ ] Manual testing

### Week 2: Availability Checking

**Days 1-2: Availability Checker**
- [ ] Implement availability checker (queries service registry)
- [ ] Implement health status checking
- [ ] Implement protocol compatibility matching
- [ ] Add real-time status monitoring

**Days 3-4: Alternative Suggestions**
- [ ] Implement alternative discovery
- [ ] Implement ranking algorithm
- [ ] Implement compatibility scoring
- [ ] Generate suggestions with reasoning

**Day 5: Testing & Integration**
- [ ] Write 8 unit tests
- [ ] Write 3 E2E tests
- [ ] Add API documentation
- [ ] Manual testing

### Week 3: Coordination Validation

**Days 1-2: Pattern Validation**
- [ ] Implement sequential pattern validator
- [ ] Implement parallel pattern validator
- [ ] Implement pipeline pattern validator
- [ ] Implement MapReduce pattern validator

**Days 3-4: Live Modification**
- [ ] Implement add node handler
- [ ] Implement remove node handler
- [ ] Implement modify node handler
- [ ] Implement rebalancing logic

**Day 5: Testing & Documentation**
- [ ] Write 6 unit tests
- [ ] Write 2 E2E tests
- [ ] Complete API documentation
- [ ] Write integration guide

---

## 🎯 Success Criteria

### Functional Success
- [ ] All 4 APIs implemented and working
- [ ] All validation rules enforced
- [ ] Service registry integration working
- [ ] Alternative suggestions accurate

### Quality Success
- [ ] 24 unit tests passing (100%)
- [ ] 5 E2E tests passing (100%)
- [ ] Zero unsafe code
- [ ] Zero hardcoded primal names
- [ ] < 100ms validation latency

### Integration Success
- [ ] Integrated with petalTongue graph editor
- [ ] Integrated with biomeOS orchestration
- [ ] Tested with real primal registrations
- [ ] Documentation complete

---

## 📊 Performance Targets

| Metric | Target | Method |
|--------|--------|--------|
| **Validation Latency** | < 100ms | Small graphs (< 50 nodes) |
| **Validation Latency** | < 500ms | Large graphs (< 500 nodes) |
| **Availability Check** | < 50ms | Per graph |
| **Alternative Suggestions** | < 100ms | Per node |
| **Concurrent Requests** | 1000/sec | Load testing |
| **Memory Usage** | < 50MB | During validation |

---

## 🔒 Security Considerations

### Input Validation
- [ ] Validate all JSON inputs against schema
- [ ] Limit graph size (max 1000 nodes)
- [ ] Limit edge count (max 5000 edges)
- [ ] Sanitize all string inputs
- [ ] Prevent JSON injection attacks

### Resource Limits
- [ ] Timeout validation after 5 seconds
- [ ] Limit recursion depth (cycle detection)
- [ ] Rate limit validation requests
- [ ] Monitor memory usage

### Data Privacy
- [ ] Don't log sensitive graph data
- [ ] Don't expose internal service details
- [ ] Audit all validation requests
- [ ] Respect user permissions (via Security Provider)

---

## 📚 Documentation Deliverables

### Technical Documentation
1. **API Reference** - Complete JSON-RPC API docs
2. **Data Type Reference** - All structs and their fields
3. **Validation Rules** - Complete list of validation checks
4. **Performance Guide** - Optimization tips

### Integration Guides
1. **biomeOS Integration** - How to call from biomeOS
2. **petalTongue Integration** - How to use in graph editor
3. **Testing Guide** - How to test graph validation
4. **Troubleshooting** - Common issues and solutions

### Examples
1. **Python Client** - Example validation client
2. **Rust Client** - Example validation client
3. **Graph Examples** - Common graph patterns
4. **Error Handling** - How to handle validation errors

---

## 🤝 Dependencies

### Internal Dependencies
- Service Registry (v3.20.0) ✅ Ready
- JSON-RPC Server (v3.19.3+) ✅ Ready
- Unix Socket IPC (v3.19.3+) ✅ Ready

### External Dependencies
- petalTongue graph editor (in development)
- biomeOS orchestration (ready)
- Real primal registrations (for testing)

### Blocked By
- Graph schema finalization (from biomeOS)
- Coordination pattern examples (from biomeOS)

### Blocks
- petalTongue graph editor validation
- biomeOS graph deployment
- AI provider graph learning

---

## 📞 Contact & Coordination

**Owner**: Songbird Team  
**Slack**: #collaborative-intelligence  
**Weekly Sync**: Wednesdays, 2pm UTC  
**Status Updates**: Daily in Slack channel

**Key Contacts**:
- Architecture questions: @songbird-team
- Integration questions: @biomeos-team
- Testing questions: @qa-team

---

**Version**: v3.21.0  
**Status**: 📋 Specification Ready  
**Timeline**: 3 weeks (Jan 13 - Feb 3, 2026)  
**Confidence**: 💯 100%

🎵 **Songbird: From Service Registry → Graph Intelligence!** 🎵

