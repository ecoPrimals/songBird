# Graph Availability API Documentation

**Version**: v3.21.0 (Collaborative Intelligence - Week 2)  
**Date**: January 13, 2026  
**APIs**: `graph.check_availability` + `graph.suggest_alternatives`

---

## Overview

The Graph Availability APIs enable biomeOS and other clients to:
1. **Check if required primals are available** before executing a graph
2. **Get ranked alternatives** when preferred primals are unavailable
3. **Make intelligent decisions** based on primal health and compatibility

These APIs integrate with Songbird's **Service Registry (v3.20.0)** for runtime primal discovery with zero hardcoding.

---

## API 1: `graph.check_availability`

### Purpose

Check if all nodes in a graph have available primals registered and healthy.

### Request Format

```json
{
  "jsonrpc": "2.0",
  "method": "graph.check_availability",
  "params": {
    "graph": {
      "id": "workflow-123",
      "name": "Data Processing Pipeline",
      "nodes": [
        {
          "id": "encrypt-input",
          "capability": "encryption",
          "preferred_protocol": "json-rpc",
          "inputs": [],
          "outputs": ["encrypted_data"],
          "config": {},
          "timeout_secs": 30
        },
        {
          "id": "store-data",
          "capability": "storage",
          "preferred_protocol": "json-rpc",
          "inputs": ["encrypted_data"],
          "outputs": ["storage_id"],
          "config": {"path": "/data/secure"},
          "timeout_secs": 60
        }
      ],
      "edges": [
        {
          "from": "encrypt-input",
          "to": "store-data",
          "data_key": "encrypted_data"
        }
      ],
      "metadata": {
        "created_at": "2026-01-13T10:00:00Z",
        "created_by": "biomeOS",
        "version": "1.0"
      }
    }
  },
  "id": 1
}
```

### Response Format (Success)

```json
{
  "jsonrpc": "2.0",
  "result": {
    "available": ["encrypt-input", "store-data"],
    "unavailable": [],
    "unhealthy": [],
    "degraded": [],
    "details": {
      "encrypt-input": {
        "status": "available",
        "primal": "BearDog",
        "service_id": "beardog-abc123",
        "endpoint": "/run/user/1000/beardog-nat0.sock",
        "protocol": "json-rpc",
        "health_status": "healthy",
        "last_seen": "2026-01-13T10:05:00Z"
      },
      "store-data": {
        "status": "available",
        "primal": "NestGate",
        "service_id": "nestgate-def456",
        "endpoint": "/run/user/1000/nestgate-nat0.sock",
        "protocol": "json-rpc",
        "health_status": "healthy",
        "last_seen": "2026-01-13T10:04:55Z"
      }
    },
    "summary": {
      "total_nodes": 2,
      "available_nodes": 2,
      "availability_percent": 100.0
    }
  },
  "id": 1
}
```

### Response Format (Partial Availability)

```json
{
  "jsonrpc": "2.0",
  "result": {
    "available": ["encrypt-input"],
    "unavailable": ["ai-inference"],
    "unhealthy": [],
    "degraded": ["store-data"],
    "details": {
      "encrypt-input": {
        "status": "available",
        "primal": "BearDog",
        "service_id": "beardog-abc123",
        "endpoint": "/run/user/1000/beardog-nat0.sock",
        "protocol": "json-rpc",
        "health_status": "healthy",
        "last_seen": "2026-01-13T10:05:00Z"
      },
      "store-data": {
        "status": "degraded",
        "primal": "NestGate",
        "service_id": "nestgate-def456",
        "endpoint": "/run/user/1000/nestgate-nat0.sock",
        "protocol": "json-rpc",
        "health_status": "degraded",
        "last_seen": "2026-01-13T10:04:55Z"
      },
      "ai-inference": {
        "status": "unavailable",
        "required_capability": "ai_inference",
        "reason": "No primal registered with capability 'ai_inference'",
        "suggested_action": "Register a primal with capability 'ai_inference' or use an alternative capability"
      }
    },
    "summary": {
      "total_nodes": 3,
      "available_nodes": 1,
      "availability_percent": 33.3
    }
  },
  "id": 1
}
```

### Status Definitions

| Status | Meaning | Can Execute? |
|--------|---------|--------------|
| **available** | Primal registered, healthy, ready | ✅ Yes |
| **degraded** | Primal registered, performance issues | ⚠️ Caution |
| **unhealthy** | Primal registered, health check failed | ❌ No |
| **unavailable** | No primal registered for capability | ❌ No |

### Error Responses

**Invalid Graph Structure**:
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": "missing field `nodes`"
  },
  "id": 1
}
```

**Internal Error**:
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32603,
    "message": "Availability check failed",
    "data": "Service registry unavailable"
  },
  "id": 1
}
```

---

## API 2: `graph.suggest_alternatives`

### Purpose

Get ranked alternatives for a node when preferred primal is unavailable or unhealthy.

### Request Format

```json
{
  "jsonrpc": "2.0",
  "method": "graph.suggest_alternatives",
  "params": {
    "node": {
      "id": "encrypt-input",
      "capability": "encryption",
      "preferred_protocol": "json-rpc",
      "inputs": [],
      "outputs": ["encrypted_data"],
      "config": {},
      "timeout_secs": 30
    }
  },
  "id": 1
}
```

### Response Format (Alternatives Available)

```json
{
  "jsonrpc": "2.0",
  "result": {
    "alternatives": [
      {
        "rank": 1,
        "service_id": "beardog-abc123",
        "primal_name": "BearDog",
        "endpoint": "/run/user/1000/beardog-nat0.sock",
        "protocol": "json-rpc",
        "health_status": "healthy",
        "last_seen": "2026-01-13T10:05:00Z",
        "reason": "healthy, protocol match, capability 'encryption'",
        "compatibility_score": 100
      },
      {
        "rank": 2,
        "service_id": "fastcrypto-xyz789",
        "primal_name": "FastCrypto",
        "endpoint": "tcp://localhost:5000",
        "protocol": "tarpc",
        "health_status": "healthy",
        "last_seen": "2026-01-13T10:04:58Z",
        "reason": "healthy, universal protocol (json-rpc), capability 'encryption'",
        "compatibility_score": 80
      },
      {
        "rank": 3,
        "service_id": "slowcrypto-mno456",
        "primal_name": "SlowCrypto",
        "endpoint": "/run/user/1000/slowcrypto-nat0.sock",
        "protocol": "json-rpc",
        "health_status": "degraded",
        "last_seen": "2026-01-13T10:04:50Z",
        "reason": "degraded but functional, protocol match, capability 'encryption'",
        "compatibility_score": 70
      }
    ],
    "recommendation": {
      "service_id": "beardog-abc123",
      "reason": "Best match: BearDog (compatibility score: 100)"
    }
  },
  "id": 1
}
```

### Response Format (No Alternatives)

```json
{
  "jsonrpc": "2.0",
  "result": {
    "alternatives": [],
    "unavailable_reason": "No primal registered with capability 'quantum_computing'"
  },
  "id": 1
}
```

### Compatibility Scoring Algorithm

**Total Score**: 0-100 points

| Category | Points | Details |
|----------|--------|---------|
| **Health Status** | 0-50 | healthy=50, unknown=45, degraded=30, down=0 |
| **Protocol Match** | 0-40 | exact match=40, json-rpc fallback=20, other=0 |
| **Recency** | 0-10 | has timestamp=10, no timestamp=0 |

**Example Calculations**:
- **Perfect Match** (100): healthy + json-rpc match + recent = 50 + 40 + 10 = 100
- **Good Match** (80): healthy + tarpc (fallback) + recent = 50 + 20 + 10 = 80
- **Degraded Match** (70): degraded + json-rpc match + recent = 30 + 40 + 10 = 80
- **Unhealthy Match** (50): down + json-rpc match + recent = 0 + 40 + 10 = 50

### Error Responses

**Invalid Node**:
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": "missing field `capability`"
  },
  "id": 1
}
```

**Internal Error**:
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32603,
    "message": "Alternative suggestion failed",
    "data": "Service registry query failed"
  },
  "id": 1
}
```

---

## Usage Examples

### Example 1: Pre-Execution Validation (biomeOS)

```rust
use songbird_client::SongbirdClient;
use anyhow::Result;

async fn validate_graph_before_execution(graph: Graph) -> Result<bool> {
    let client = SongbirdClient::connect("/run/user/1000/songbird-nat0.sock").await?;
    
    // Check availability
    let report = client.check_availability(graph.clone()).await?;
    
    if report.summary.availability_percent == 100.0 {
        println!("✅ All nodes available, proceeding with execution");
        return Ok(true);
    }
    
    if !report.unavailable.is_empty() {
        println!("❌ Missing capabilities: {:?}", report.unavailable);
        return Ok(false);
    }
    
    if !report.unhealthy.is_empty() {
        println!("⚠️  Unhealthy nodes: {:?}", report.unhealthy);
        // Try to find alternatives
        for node_id in report.unhealthy {
            let node = graph.nodes.iter().find(|n| n.id == node_id).unwrap();
            let alternatives = client.suggest_alternatives(node.clone()).await?;
            if let Some(alt) = alternatives.recommendation {
                println!("   → Suggested alternative: {}", alt.reason);
            }
        }
        return Ok(false);
    }
    
    Ok(true)
}
```

### Example 2: Automatic Fallback (petalTongue)

```rust
async fn execute_with_fallback(node: GraphNode) -> Result<()> {
    let client = SongbirdClient::connect("/run/user/1000/songbird-nat0.sock").await?;
    
    // Get alternatives ranked by compatibility
    let alternatives = client.suggest_alternatives(node.clone()).await?;
    
    if alternatives.alternatives.is_empty() {
        return Err(anyhow!("No primal available for capability '{}'", node.capability));
    }
    
    // Try each alternative in order
    for alt in alternatives.alternatives {
        println!("Trying {} (score: {})...", alt.primal_name, alt.compatibility_score);
        
        match execute_on_primal(&alt.endpoint, &node).await {
            Ok(result) => {
                println!("✅ Success using {}", alt.primal_name);
                return Ok(result);
            }
            Err(e) => {
                println!("⚠️  Failed: {}, trying next...", e);
                continue;
            }
        }
    }
    
    Err(anyhow!("All alternatives failed"))
}
```

### Example 3: Health Monitoring Dashboard

```rust
async fn monitor_graph_health(graph: Graph) -> Result<()> {
    let client = SongbirdClient::connect("/run/user/1000/songbird-nat0.sock").await?;
    
    loop {
        let report = client.check_availability(graph.clone()).await?;
        
        println!("\n📊 Graph Health Dashboard");
        println!("   Total Nodes: {}", report.summary.total_nodes);
        println!("   Available: {} ({:.1}%)", 
            report.summary.available_nodes,
            report.summary.availability_percent
        );
        
        if !report.degraded.is_empty() {
            println!("   ⚠️  Degraded: {:?}", report.degraded);
        }
        
        if !report.unhealthy.is_empty() {
            println!("   ❌ Unhealthy: {:?}", report.unhealthy);
        }
        
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }
}
```

---

## Integration Guide

### Step 1: Connect to Songbird

```rust
use songbird_client::SongbirdClient;

let socket_path = format!("/run/user/{}/songbird-{}.sock", 
    std::process::id(), 
    "nat0" // family_id
);
let client = SongbirdClient::connect(&socket_path).await?;
```

### Step 2: Check Graph Availability

```rust
let report = client.check_availability(graph).await?;

match report.summary.availability_percent {
    100.0 => println!("✅ Ready to execute"),
    0.0 => println!("❌ Cannot execute (no primals available)"),
    p => println!("⚠️  Partial availability ({:.1}%)", p),
}
```

### Step 3: Handle Unavailable Nodes

```rust
for node_id in report.unavailable {
    let node = graph.nodes.iter().find(|n| n.id == node_id)?;
    let alternatives = client.suggest_alternatives(node.clone()).await?;
    
    if let Some(alt) = alternatives.alternatives.first() {
        println!("Alternative for {}: {} (score: {})", 
            node_id, alt.primal_name, alt.compatibility_score);
    } else {
        println!("No alternatives for {}", node_id);
    }
}
```

---

## Best Practices

### 1. Always Check Before Execution

```rust
// ✅ GOOD
let report = client.check_availability(&graph).await?;
if report.summary.availability_percent == 100.0 {
    execute_graph(&graph).await?;
}

// ❌ BAD - Execute without checking
execute_graph(&graph).await?; // May fail mid-execution
```

### 2. Handle Degraded Nodes Appropriately

```rust
// ✅ GOOD - Allow degraded with warning
if report.degraded.is_empty() || user_accepts_degraded_performance() {
    execute_graph(&graph).await?;
}

// ❌ BAD - Treat degraded as unavailable
if report.summary.availability_percent < 100.0 {
    return Err(anyhow!("Not ready")); // Too strict
}
```

### 3. Use Alternatives for Resilience

```rust
// ✅ GOOD - Try alternatives
let alternatives = client.suggest_alternatives(&node).await?;
for alt in alternatives.alternatives {
    if let Ok(result) = try_primal(&alt).await {
        return Ok(result);
    }
}

// ❌ BAD - Give up on first failure
execute_on_preferred_primal(&node).await?; // No fallback
```

### 4. Monitor Health Continuously

```rust
// ✅ GOOD - Periodic health checks
tokio::spawn(async move {
    loop {
        let report = client.check_availability(&graph).await?;
        if report.summary.availability_percent < 75.0 {
            alert_ops_team(&report).await?;
        }
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
});

// ❌ BAD - Check once at startup only
```

---

## Performance Characteristics

| Operation | Latency | Notes |
|-----------|---------|-------|
| **check_availability** | < 10ms | O(n) where n = number of nodes |
| **suggest_alternatives** | < 5ms | O(m) where m = number of registered primals |
| **Service registry query** | < 1ms | In-memory HashMap lookup |
| **Health status check** | Instant | Cached, updated by primals |

---

## Troubleshooting

### Issue: "No primal registered with capability"

**Cause**: Required primal not running or not registered  
**Solution**: Start the required primal or register it manually:
```bash
# Check running primals
ls /run/user/$(id -u)/songbird-*.sock

# Start missing primal
./beardog # Or whichever is missing
```

### Issue: "All nodes unhealthy"

**Cause**: Primals are running but failing health checks  
**Solution**: Check primal logs for errors:
```bash
journalctl -u beardog -n 50
```

### Issue: "Compatibility score unexpectedly low"

**Cause**: Protocol mismatch or degraded health  
**Solution**: Check protocol preference and health:
```rust
for alt in alternatives.alternatives {
    println!("{}: health={}, protocol={}, score={}", 
        alt.primal_name, alt.health_status, alt.protocol, alt.compatibility_score);
}
```

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| **v3.21.0** | Jan 13, 2026 | Initial release - graph.check_availability + suggest_alternatives |

---

## See Also

- [Service Registry API (v3.20.0)](../BIOMEOS_HANDOFF_V3_20_0.md) - Primal registration and discovery
- [Graph Validation API](./GRAPH_VALIDATION_API.md) - Structure validation
- [Collaborative Intelligence Spec](../specs/COLLABORATIVE_INTELLIGENCE_GRAPH_VALIDATION.md) - Full specification

---

**Status**: ✅ **PRODUCTION READY** ✅

🎵 **Songbird v3.21.0 - Collaborative Intelligence Week 2 Complete** 🎵

