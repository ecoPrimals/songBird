# Primal Registration Protocol

**Version:** 1.0  
**Date:** December 20, 2025  
**Status:** Specification  
**Purpose:** Define how primals register with Songbird for zero-config service discovery

---

## 🎯 Core Principle: Universal Port Authority

**Songbird manages all ports. Primals never bind ports themselves.**

### Old Way (Hardcoded)
```rust
// ❌ Each primal binds its own port
ToadstoolServer::bind("0.0.0.0:8091").await?;
BeardogServer::bind("0.0.0.0:8092").await?;
NestgateServer::bind("0.0.0.0:8093").await?;
```

**Problems:**
- Port conflicts on multi-primal towers
- Manual configuration required
- No service discovery
- Doesn't scale

### New Way (Universal Port Authority)
```rust
// ✅ Primals register with Songbird
let songbird = SongbirdClient::discover_local().await?;
let endpoint = songbird.register_service(my_capabilities).await?;
ToadstoolServer::bind(endpoint).await?; // Songbird-assigned port
```

**Benefits:**
- Zero port conflicts (Songbird manages)
- Automatic service discovery
- Zero configuration
- Scales infinitely

---

## 📋 Registration Protocol Specification

### 1. Discovery Phase

**Primal Startup:**
```rust
// Step 1: Find Songbird on the local tower
let songbird = SongbirdClient::discover_local().await?;
```

**Discovery Methods (in priority order):**

1. **Environment Variable:**
   ```bash
   SONGBIRD_URL=https://localhost:8080
   ```

2. **UDP Broadcast (Anonymous Discovery):**
   ```rust
   // Listen for Songbird broadcasts on UDP 2300
   // Message: "I am Songbird at https://192.0.2.10:8080"
   ```

3. **Well-Known Port:**
   ```rust
   // Try standard Songbird port
   https://localhost:8080
   ```

4. **mDNS/Avahi:**
   ```rust
   // Look for "_songbird._tcp.local"
   ```

**Fallback:** If no Songbird found, primal can:
- Run in standalone mode (limited functionality)
- Bind to well-known port (8091, 8092, etc.) with warning
- Exit with clear error message

### 2. Registration Phase

**Primal → Songbird: Register Request**

```json
POST https://songbird:8080/api/v1/services/register

{
  "primal_name": "Toadstool",
  "primal_version": "0.1.0",
  "capabilities": [
    {
      "name": "compute",
      "type": "execution",
      "details": {
        "runtimes": ["python", "rust", "native"],
        "resources": {
          "gpu": {
            "available": true,
            "model": "RTX 3090",
            "memory_gb": 24,
            "cuda_version": "12.3"
          },
          "cpu_cores": 16,
          "memory_gb": 128
        }
      }
    },
    {
      "name": "ml_training",
      "type": "ai",
      "details": {
        "frameworks": ["pytorch", "tensorflow"],
        "models": ["cnn", "transformer", "diffusion"]
      }
    }
  ],
  "protocols": ["https", "tarpc"],
  "preferred_protocol": "tarpc",
  "health_check_path": "/health",
  "metadata": {
    "node_id": "toadstool-eastgate-abc123",
    "hostname": "eastgate",
    "platform": "linux-x86_64"
  }
}
```

**Songbird → Primal: Register Response**

```json
{
  "status": "registered",
  "service_id": "svc-toadstool-abc123",
  "assigned_endpoint": {
    "protocol": "tarpc",
    "host": "0.0.0.0",
    "port": 8091,
    "full_url": "tarpc://0.0.0.0:8091"
  },
  "fallback_endpoint": {
    "protocol": "https",
    "host": "0.0.0.0",
    "port": 8092,
    "full_url": "https://0.0.0.0:8092"
  },
  "registration_token": "sb-reg-xyz789-abc123",
  "heartbeat_interval_sec": 30,
  "trust_level": "anonymous"
}
```

### 3. Binding Phase

**Primal binds to Songbird-assigned endpoint:**

```rust
// Use the assigned endpoint
let endpoint = registration_response.assigned_endpoint;

// Bind server
ToadstoolServer::bind(endpoint.host, endpoint.port).await?;

// Start serving
info!("✅ Toadstool registered with Songbird at {}", endpoint.full_url);
```

### 4. Heartbeat Phase

**Primal → Songbird: Periodic Heartbeat**

```json
POST https://songbird:8080/api/v1/services/{service_id}/heartbeat

{
  "service_id": "svc-toadstool-abc123",
  "token": "sb-reg-xyz789-abc123",
  "status": "operational",
  "current_load": {
    "cpu_usage_percent": 23.5,
    "memory_usage_percent": 45.2,
    "gpu_usage_percent": 78.3,
    "active_tasks": 3,
    "queued_tasks": 1
  },
  "capabilities_changed": false
}
```

**Songbird → Primal: Heartbeat Response**

```json
{
  "status": "acknowledged",
  "next_heartbeat_sec": 30,
  "commands": []  // Songbird can send commands (e.g., "shutdown", "update")
}
```

**Missed Heartbeats:**
- 1 missed: Warning logged
- 3 missed: Service marked "degraded"
- 5 missed: Service de-registered, port released

### 5. Deregistration Phase

**Graceful Shutdown:**

```json
DELETE https://songbird:8080/api/v1/services/{service_id}

{
  "service_id": "svc-toadstool-abc123",
  "token": "sb-reg-xyz789-abc123",
  "reason": "graceful_shutdown"
}
```

**Songbird Response:**
```json
{
  "status": "deregistered",
  "message": "Service deregistered. Port 8091 released."
}
```

---

## 🔄 Service Lifecycle

```
┌─────────────────────────────────────────────────────────────┐
│                     Primal Startup                          │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ↓
              ┌──────────────┐
              │  Discovery   │ ← Find Songbird (UDP/mDNS/env)
              └──────┬───────┘
                     │
                     ↓
              ┌──────────────┐
              │ Registration │ ← Send capabilities
              └──────┬───────┘
                     │
                     ↓
              ┌──────────────┐
              │   Binding    │ ← Bind to assigned port
              └──────┬───────┘
                     │
                     ↓
              ┌──────────────┐
              │   Serving    │ ← Accept tasks from Songbird
              │  (+ Heartbeat)│ ← Send periodic health updates
              └──────┬───────┘
                     │
                     ↓
              ┌──────────────┐
              │Deregistration│ ← Graceful shutdown
              └──────────────┘
```

---

## 🎯 Implementation Checklist

### For Primal Developers (Toadstool, BearDog, Nestgate, Squirrel)

- [ ] **Add Songbird Client Dependency**
  ```toml
  [dependencies]
  songbird-client = { path = "../songbird/crates/songbird-client" }
  ```

- [ ] **Implement Discovery**
  ```rust
  let songbird = SongbirdClient::discover_local().await?;
  ```

- [ ] **Define Capabilities**
  ```rust
  let capabilities = vec![
      Capability {
          name: "compute".to_string(),
          type_: CapabilityType::Execution,
          details: json!({...}),
      }
  ];
  ```

- [ ] **Register with Songbird**
  ```rust
  let registration = songbird.register_service(
      "Toadstool",
      capabilities,
  ).await?;
  ```

- [ ] **Bind to Assigned Endpoint**
  ```rust
  server.bind(registration.assigned_endpoint).await?;
  ```

- [ ] **Start Heartbeat Task**
  ```rust
  tokio::spawn(async move {
      loop {
          songbird.heartbeat(service_id).await?;
          sleep(Duration::from_secs(30)).await;
      }
  });
  ```

- [ ] **Handle Graceful Shutdown**
  ```rust
  tokio::signal::ctrl_c().await?;
  songbird.deregister(service_id).await?;
  ```

### For Songbird (Orchestrator)

- [ ] **Create `songbird-client` Crate**
  - Discovery methods (UDP, mDNS, env)
  - Registration client
  - Heartbeat client

- [ ] **Implement Service Registry**
  - Store registered services
  - Manage port allocation
  - Track heartbeats
  - Handle deregistration

- [ ] **Add Registration Endpoints**
  - `POST /api/v1/services/register`
  - `POST /api/v1/services/{id}/heartbeat`
  - `DELETE /api/v1/services/{id}`
  - `GET /api/v1/services` (list all)

- [ ] **Port Management**
  - Track allocated ports
  - Prevent conflicts
  - Release on deregistration

- [ ] **Service Discovery API**
  - Query by capability
  - Query by primal type
  - Query by resource requirements

---

## 📊 Example: Toadstool Registration

### Toadstool Code

```rust
// toadstool/crates/server/src/main.rs

use songbird_client::{SongbirdClient, Capability, CapabilityType};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("🍄 Starting Toadstool Compute Server...");
    
    // 1. Discover Songbird
    info!("🔍 Discovering Songbird orchestrator...");
    let songbird = SongbirdClient::discover_local().await?;
    info!("✅ Found Songbird at {}", songbird.url());
    
    // 2. Define our capabilities
    let capabilities = vec![
        Capability {
            name: "compute".to_string(),
            type_: CapabilityType::Execution,
            details: json!({
                "runtimes": ["python", "rust"],
                "gpu": true,
            }),
        },
        Capability {
            name: "ml_training".to_string(),
            type_: CapabilityType::AI,
            details: json!({
                "frameworks": ["pytorch"],
            }),
        },
    ];
    
    // 3. Register with Songbird
    info!("📝 Registering with Songbird...");
    let registration = songbird.register_service(
        "Toadstool",
        env!("CARGO_PKG_VERSION"),
        capabilities,
    ).await?;
    
    info!("✅ Registered! Assigned endpoint: {}", 
          registration.assigned_endpoint.full_url);
    
    // 4. Create server
    let server = ToadstoolServer::new().await?;
    
    // 5. Bind to assigned endpoint
    server.bind(
        registration.assigned_endpoint.host,
        registration.assigned_endpoint.port,
    ).await?;
    
    // 6. Start heartbeat task
    let service_id = registration.service_id.clone();
    let songbird_heartbeat = songbird.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) = songbird_heartbeat.heartbeat(&service_id).await {
                error!("Heartbeat failed: {}", e);
            }
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });
    
    // 7. Start serving
    info!("🚀 Toadstool is operational!");
    info!("   Endpoint: {}", registration.assigned_endpoint.full_url);
    info!("   Capabilities: compute, ml_training");
    info!("   Managed by: Songbird");
    
    server.serve().await?;
    
    // 8. Graceful shutdown
    info!("🛑 Shutting down...");
    songbird.deregister(&service_id).await?;
    info!("✅ Deregistered from Songbird");
    
    Ok(())
}
```

---

## 🌐 Federation Extension

**For multi-tower deployments:**

### Tower-Local Registration
```
Eastgate:
  Songbird (port 8080)
  ├─ Toadstool (port 8091)
  ├─ BearDog (port 8092)
  └─ Nestgate (port 8093)
```

### Cross-Tower Discovery

**Songbird broadcasts service capabilities to other towers:**

```json
// Westgate discovers Eastgate's Toadstool
{
  "service": "Toadstool",
  "tower": "eastgate",
  "endpoint": "tarpc://192.0.2.10:8091",
  "capabilities": ["compute", "ml_training"],
  "resources": {
    "gpu": "RTX 3090",
    "available": true
  }
}
```

**Intelligent Routing:**
```rust
// User on Westgate submits GPU task
// Westgate's Songbird:
//   - Checks local Toadstool (no GPU)
//   - Discovers Eastgate's Toadstool (RTX 3090)
//   - Routes task to Eastgate
//   - Returns result to user
```

---

## 🔐 Security & Trust

### Trust Levels (from PRIVACY_BOUNDARIES doc)

**Level 0 (Anonymous):**
- Primal registers with capabilities only
- No identity required

**Level 3 (Identity-Verified):**
- Primal provides node_id, hostname
- Songbird verifies via BearDog
- Full service mesh access

### Authentication

**Registration Token:**
- Generated by Songbird on registration
- Required for heartbeat and deregistration
- Rotates every 24 hours

**API Authentication:**
- Songbird routes tasks with bearer token
- Primal validates before execution

---

## 📝 Next Steps

### Immediate (Phase 1)
1. Create `songbird-client` crate
2. Implement basic discovery (env var, well-known port)
3. Add registration endpoint to Songbird
4. Wire Toadstool as proof-of-concept

### Short-Term (Phase 2)
1. Add UDP/mDNS discovery
2. Implement heartbeat mechanism
3. Add service query API
4. Test cross-tower routing

### Medium-Term (Phase 3)
1. Wire BearDog (security)
2. Wire Nestgate (storage)
3. Wire Squirrel (AI-MCP)
4. Full ecosystem demo

---

## 🎯 Success Criteria

**Phase 1 Complete When:**
- [ ] Toadstool registers with Songbird
- [ ] Songbird assigns port to Toadstool
- [ ] Task submitted to Songbird routes to Toadstool
- [ ] Zero manual configuration
- [ ] Live demo working

**Ecosystem Complete When:**
- [ ] All 5 primals register automatically
- [ ] Zero port conflicts
- [ ] Cross-tower routing working
- [ ] Distributed ML demo operational
- [ ] Pattern documented for future primals

---

**Status:** Specification Ready for Implementation  
**Next:** Build `songbird-client` crate and wire Toadstool  
**Vision:** Universal Port Authority + Zero-Config Ecosystem 🎵

