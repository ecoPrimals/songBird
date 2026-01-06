# 🗄️ NestGate Integration Guide - Quick Start

**Date**: January 7, 2026  
**Status**: 🚀 **READY FOR INTERACTION TESTING**  
**For**: NestGate Team

---

## 🎯 Goal

Integrate NestGate (database primal) with Songbird v3.13.0 for **capability-based, protocol-agnostic, trust-secured database operations**.

**Timeline**: 2 weeks for full integration  
**Current Status**: Songbird ready, waiting for NestGate

---

## ✅ **What Songbird v3.13.0 Provides (Already Working)**

### **1. Universal Primal Adapter**
```rust
// Songbird discovers ANY primal by capability
let adapter = UniversalAdapter::new(capability_registry);

// No hardcoded "NestGate" - just capability!
let result = adapter.call("storage/database", "insert", json!({
    "table": "peers",
    "data": {"peer_id": "tower2", "trust_level": 1}
})).await?;
```

**Features**:
- ✅ Capability-based discovery (finds database providers)
- ✅ Protocol auto-negotiation (tarpc → JSON-RPC → HTTP)
- ✅ QoS-based selection (picks best provider)
- ✅ Fault tolerance (tries next provider on failure)

### **2. Capability Registry**
```bash
# Query available database providers
echo '{
  "jsonrpc":"2.0",
  "method":"capabilities.list",
  "params":{"capability":"storage/database"},
  "id":1
}' | nc -U /tmp/songbird-nat0-tower1.sock | jq
```

### **3. Trust-Based Access**
- Songbird can query BearDog for trust levels
- NestGate can enforce access control based on trust
- Progressive trust: Limited → Elevated → Highest

---

## 🚀 **Quick Start: 3 Steps**

### **Step 1: Implement JSON-RPC Server** ⏰ **2 days**

**Minimal NestGate Server**:

```rust
// nestgate-server/src/main.rs

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::net::UnixListener;

#[derive(Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: Value,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    id: Value,
}

#[derive(Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

struct NestGateDatabase {
    // Simple in-memory database for testing
    tables: HashMap<String, Vec<Value>>,
}

impl NestGateDatabase {
    fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }
    
    fn insert(&mut self, table: &str, data: Value) -> Result<Value, String> {
        let table_data = self.tables.entry(table.to_string()).or_insert_with(Vec::new);
        table_data.push(data.clone());
        Ok(json!({"inserted": true, "id": table_data.len()}))
    }
    
    fn query(&self, table: &str, filter: Option<Value>) -> Result<Value, String> {
        let table_data = self.tables.get(table).ok_or("Table not found")?;
        
        // Simple filter (match all fields)
        let results: Vec<_> = if let Some(filter) = filter {
            table_data.iter()
                .filter(|row| matches_filter(row, &filter))
                .cloned()
                .collect()
        } else {
            table_data.clone()
        };
        
        Ok(json!(results))
    }
}

fn matches_filter(row: &Value, filter: &Value) -> bool {
    if let (Some(row_obj), Some(filter_obj)) = (row.as_object(), filter.as_object()) {
        filter_obj.iter().all(|(key, value)| {
            row_obj.get(key) == Some(value)
        })
    } else {
        false
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let family_id = std::env::var("FAMILY_ID").unwrap_or_else(|_| "nat0".to_string());
    let node_id = std::env::var("NODE_ID").unwrap_or_else(|_| "tower1".to_string());
    let socket_path = format!("/tmp/nestgate-{}-{}.sock", family_id, node_id);
    
    // Remove old socket if exists
    let _ = std::fs::remove_file(&socket_path);
    
    let listener = UnixListener::bind(&socket_path)?;
    println!("🗄️  NestGate listening on: {}", socket_path);
    
    let mut db = NestGateDatabase::new();
    
    loop {
        let (mut stream, _) = listener.accept().await?;
        
        // Read request
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        
        // Parse JSON-RPC request
        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("Parse error: {}", e);
                continue;
            }
        };
        
        println!("📥 Request: {} {:?}", request.method, request.params);
        
        // Handle request
        let response = match request.method.as_str() {
            "insert" => {
                let table = request.params["table"].as_str().unwrap_or("default");
                let data = request.params["data"].clone();
                match db.insert(table, data) {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(result),
                        error: None,
                        id: request.id,
                    },
                    Err(e) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32000,
                            message: e,
                        }),
                        id: request.id,
                    },
                }
            }
            "query" => {
                let table = request.params["table"].as_str().unwrap_or("default");
                let filter = request.params.get("filter").cloned();
                match db.query(table, filter) {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(result),
                        error: None,
                        id: request.id,
                    },
                    Err(e) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32000,
                            message: e,
                        }),
                        id: request.id,
                    },
                }
            }
            "health" => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(json!({"status": "healthy", "tables": db.tables.len()})),
                error: None,
                id: request.id,
            },
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: "Method not found".to_string(),
                }),
                id: request.id,
            },
        };
        
        // Send response
        let response_json = serde_json::to_string(&response)? + "\n";
        stream.write_all(response_json.as_bytes()).await?;
        
        println!("📤 Response sent");
    }
}
```

**Build & Run**:
```bash
# Build
cargo build --release

# Run
FAMILY_ID=nat0 NODE_ID=tower1 ./target/release/nestgate-server
```

---

### **Step 2: Register with Songbird** ⏰ **1 hour**

**Manual Registration** (for testing):

```bash
# Register NestGate with Songbird
echo '{
  "jsonrpc":"2.0",
  "method":"capabilities.register",
  "params":{
    "provider_id": "nestgate-tower1",
    "capabilities": [
      "storage/database",
      "storage/query"
    ],
    "endpoints": [
      {
        "protocol": "json-rpc",
        "address": "unix:///tmp/nestgate-nat0-tower1.sock"
      }
    ],
    "qos": {
      "latency_ms": 5,
      "success_rate": 0.999,
      "availability": 0.99
    }
  },
  "id":1
}' | nc -U /tmp/songbird-nat0-tower1.sock | jq
```

**Verify Registration**:
```bash
# List database providers
echo '{
  "jsonrpc":"2.0",
  "method":"capabilities.list",
  "params":{"capability":"storage/database"},
  "id":1
}' | nc -U /tmp/songbird-nat0-tower1.sock | jq

# Expected output:
# {
#   "result": {
#     "providers": [
#       {
#         "provider_id": "nestgate-tower1",
#         "capabilities": ["storage/database", "storage/query"],
#         ...
#       }
#     ]
#   }
# }
```

---

### **Step 3: Test Basic Operations** ⏰ **1 hour**

**Test Insert**:
```bash
# Insert data via NestGate
echo '{
  "jsonrpc":"2.0",
  "method":"insert",
  "params":{
    "table": "discovered_peers",
    "data": {
      "peer_id": "tower2",
      "node_name": "Tower 2",
      "family_id": "nat0",
      "trust_level": 1,
      "discovered_at": "2026-01-07T12:00:00Z"
    }
  },
  "id":1
}' | nc -U /tmp/nestgate-nat0-tower1.sock | jq
```

**Test Query**:
```bash
# Query data
echo '{
  "jsonrpc":"2.0",
  "method":"query",
  "params":{
    "table": "discovered_peers",
    "filter": {"family_id": "nat0"}
  },
  "id":1
}' | nc -U /tmp/nestgate-nat0-tower1.sock | jq
```

**Test Health**:
```bash
# Health check
echo '{"jsonrpc":"2.0","method":"health","id":1}' | \
  nc -U /tmp/nestgate-nat0-tower1.sock | jq
```

---

## 🧪 **Integration Testing Scenarios**

### **Scenario 1: Peer Discovery Persistence**

**Goal**: Songbird discovers peer → stores in NestGate → persists across restarts

**Test**:
1. Start Songbird + NestGate on tower1
2. Start Songbird on tower2
3. Verify tower1 discovers tower2 (check Songbird logs)
4. Query NestGate: Should have tower2 in `discovered_peers` table
5. Restart Songbird on tower1
6. Query NestGate: tower2 should still be there!

---

### **Scenario 2: Trust-Based Access Control**

**Goal**: Only trusted callers can write to NestGate

**Test**:
1. Songbird (trusted) tries to insert → ✅ Success
2. Unknown caller tries to insert → ❌ Rejected
3. Verify access control via trust levels

**Implementation** (NestGate side):
```rust
async fn check_caller_trust(&self, caller_id: &str) -> TrustLevel {
    // Query Songbird for trust level
    // (via UniversalAdapter calling security/trust capability)
    
    let trust_response = reqwest::Client::new()
        .post("unix:///tmp/songbird-nat0-tower1.sock")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "trust.query",
            "params": {"peer_id": caller_id},
            "id": 1
        }))
        .send()
        .await?;
    
    // Parse trust level
    // ...
}
```

---

### **Scenario 3: Federation State Recovery**

**Goal**: Songbird crashes → restarts → recovers state from NestGate

**Test**:
1. Songbird discovers 3 peers
2. All 3 peers stored in NestGate
3. Kill Songbird process
4. Restart Songbird
5. Songbird queries NestGate on startup
6. Songbird restores federation state from database
7. Verify all 3 peers are in federation without rediscovery

---

## 📋 **API Contract**

### **Required Methods**

#### **1. `insert` - Insert data**
```json
{
  "jsonrpc": "2.0",
  "method": "insert",
  "params": {
    "table": "string",
    "data": { ... }
  },
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "inserted": true,
    "id": 123
  },
  "id": 1
}
```

---

#### **2. `query` - Query data**
```json
{
  "jsonrpc": "2.0",
  "method": "query",
  "params": {
    "table": "string",
    "filter": { "key": "value" },  // Optional
    "order_by": "field ASC",        // Optional
    "limit": 100                    // Optional
  },
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": [
    { ... },
    { ... }
  ],
  "id": 1
}
```

---

#### **3. `health` - Health check**
```json
{
  "jsonrpc": "2.0",
  "method": "health",
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "status": "healthy",
    "tables": 5,
    "uptime_seconds": 3600
  },
  "id": 1
}
```

---

## 🎯 **Success Criteria**

### **Phase 1: Discovery** ✅ **DONE when**:
- [ ] NestGate JSON-RPC server running
- [ ] Registered in Songbird capability registry
- [ ] `capabilities.list` shows NestGate

### **Phase 2: Basic Operations** ✅ **DONE when**:
- [ ] Can insert data via JSON-RPC
- [ ] Can query data with filters
- [ ] Health check returns success

### **Phase 3: Songbird Integration** ✅ **DONE when**:
- [ ] Songbird stores discovered peers in NestGate
- [ ] Peers persist across Songbird restarts
- [ ] Federation state recoverable from database

### **Phase 4: Trust Integration** ✅ **DONE when**:
- [ ] NestGate queries trust levels
- [ ] Access control enforced based on trust
- [ ] Audit logs for sensitive operations

---

## 📞 **Support & Communication**

### **Questions for NestGate Team**:
1. What database backend? (SQLite, PostgreSQL, custom?)
2. Do you already have JSON-RPC server? (If yes, just need capability registration)
3. Preferred timeline? (We can help accelerate!)
4. Want tarpc for performance? (Optional, JSON-RPC works great)

### **Songbird Team Contact**:
- **Code**: `crates/songbird-orchestrator/src/universal_adapter.rs`
- **Tests**: `crates/songbird-orchestrator/tests/capability_integration_tests.rs`
- **Docs**: `MULTI_PRIMAL_INTERACTION_ARCHITECTURE.md`

---

## 🎊 **Summary**

### **Ready to Start**:
✅ **Songbird v3.13.0** has all infrastructure  
✅ **3-step quick start** (server, register, test)  
✅ **Example code** provided (copy-paste ready!)  
✅ **Testing scenarios** defined  
✅ **Success criteria** clear  

### **Timeline**:
- **Phase 1**: 2 days (JSON-RPC server)
- **Phase 2**: 1 day (basic operations)
- **Phase 3**: 2 days (Songbird integration)
- **Phase 4**: 2 days (trust integration)

**Total**: ~2 weeks for full integration

---

🚀 **LET'S BUILD THE NETWORK EFFECT!** 🌐

**NestGate team**: Copy the example code above and you'll have basic integration working in 2 days! Songbird is ready and waiting! ✨

