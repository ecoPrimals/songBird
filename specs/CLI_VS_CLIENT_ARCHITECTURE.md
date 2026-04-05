# songbird-cli vs songbird-client: Architectural Distinction

**Date:** December 20, 2025  
**Context:** Clarifying the purpose of each crate for inter-primal integration

---

## 🎯 Quick Answer

**They are DIFFERENT and serve DIFFERENT purposes:**

| Aspect | `songbird-cli` | `songbird-client` |
|--------|----------------|-------------------|
| **Who uses it?** | **Humans** (operators, admins) | **Other primals** (Compute provider, Security Provider, etc.) |
| **What is it?** | Command-line tool | Library/SDK |
| **Interface** | Terminal commands (`songbird status`) | Rust API (`client.register_service()`) |
| **Purpose** | Manage Songbird via CLI | Integrate with Songbird programmatically |
| **Already exists?** | ✅ Yes (gaming-focused) | ❌ No (we need to create it) |

---

## 📋 songbird-cli (Existing)

### Purpose
**Human-facing command-line interface** for managing Songbird.

### Location
`crates/songbird-cli/`

### What It Does
```bash
# Gaming session management
songbird gaming host --name "My Game"
songbird gaming join GAME-CODE-123

# Network diagnostics
songbird network test
songbird network optimize

# Federation management
songbird federation status
songbird federation join

# Configuration
songbird config show
songbird config set key=value

# Status checks
songbird status
songbird version
```

### Architecture
```
┌──────────┐
│  HUMAN   │
└────┬─────┘
     │ (types commands)
     ↓
┌─────────────────┐
│ songbird-cli    │  (Terminal UI)
│ (Binary)        │  - Clap for arg parsing
└────┬────────────┘  - Colored output
     │ (HTTP/IPC)    - Interactive prompts
     ↓
┌─────────────────┐
│ Songbird        │  (Running server)
│ Orchestrator    │
└─────────────────┘
```

### Use Cases
- **Operators:** Start/stop Songbird, check status
- **Admins:** Configure federation, manage gaming sessions
- **Developers:** Debug, test, inspect state
- **Students:** Join gaming sessions, check status

### Key Dependencies
- `clap` - CLI argument parsing
- `colored` - Terminal output
- `dialoguer` - Interactive prompts
- `reqwest` - HTTP client to talk to orchestrator

### Examples
```bash
# Check if Songbird is running
songbird status

# Show federation status
songbird federation status

# Join a gaming session
songbird gaming join GAME-CODE-123

# Configure network settings
songbird config set network.protocol=udp
```

---

## 📋 songbird-client (New - To Be Created)

### Purpose
**Primal-facing library** for programmatic integration with Songbird.

### Location (Planned)
`crates/songbird-client/`

### What It Will Do
```rust
// Discovery
let client = SongbirdClient::discover_local().await?;

// Registration
let registration = client.register_service(
    "Compute provider",
    vec![Capability { name: "compute", ... }]
).await?;

// Heartbeat
client.heartbeat(service_id).await?;

// Deregistration
client.deregister(service_id).await?;

// Query services
let services = client.query_services("compute").await?;
```

### Architecture
```
┌──────────────┐
│ COMPUTE PROV. │  (Or Security Provider, Storage Provider, AI provider)
│  (Binary)    │
└──────┬───────┘
       │ (imports as library)
       ↓
┌─────────────────┐
│ songbird-client │  (Rust crate/library)
│ (Library)       │  - Discovery methods
└────┬────────────┘  - Registration API
     │ (HTTP/UDP)    - Heartbeat
     ↓
┌─────────────────┐
│ Songbird        │  (Running orchestrator)
│ Orchestrator    │
└─────────────────┘
```

### Use Cases
- **Compute provider:** Register compute capabilities, receive tasks
- **Security Provider:** Register security capabilities, verify trust
- **Storage provider:** Register storage capabilities, handle data
- **AI provider:** Register AI capabilities, route requests

### Key Dependencies (Planned)
- `tokio` - Async runtime
- `reqwest` - HTTP client
- `serde` - Serialization
- `uuid` - Service IDs
- NO CLI dependencies (no clap, colored, dialoguer)

### Examples
```rust
// In Compute provider's main.rs
use songbird_client::{SongbirdClient, Capability};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Discover Songbird
    let songbird = SongbirdClient::discover_local().await?;
    
    // 2. Register our capabilities
    let registration = songbird.register_service(
        "Compute provider",
        vec![
            Capability {
                name: "compute".to_string(),
                type_: CapabilityType::Execution,
                details: json!({
                    "runtimes": ["python", "rust"],
                    "gpu": true,
                }),
            }
        ]
    ).await?;
    
    // 3. Use assigned endpoint
    let port = registration.assigned_endpoint.port;
    server.bind(port).await?;
    
    // 4. Start heartbeat
    tokio::spawn(async move {
        loop {
            songbird.heartbeat(&service_id).await?;
            sleep(Duration::from_secs(30)).await;
        }
    });
    
    // 5. Serve
    server.serve().await?;
}
```

---

## 🔄 How They Interact

### Scenario: Developer Checks Compute provider Status

```
┌──────────┐
│  HUMAN   │
└────┬─────┘
     │
     │ "Is Compute provider registered?"
     ↓
┌─────────────────┐
│ songbird-cli    │  $ songbird services list
└────┬────────────┘
     │ (HTTP GET /api/v1/services)
     ↓
┌─────────────────┐
│ Songbird        │  Service Registry
│ Orchestrator    │  {
└────┬────────────┘    "Compute provider": { ... }
     ↑               }
     │ (Registered via songbird-client)
     │
┌─────────────────┐
│ Compute provider       │  (Using songbird-client library)
└─────────────────┘
```

**Flow:**
1. Compute provider uses `songbird-client` to register
2. Songbird stores registration in service registry
3. Human uses `songbird-cli` to query status
4. CLI fetches data from orchestrator
5. CLI displays it nicely to human

---

## 🎯 Why Two Separate Crates?

### 1. Separation of Concerns
- **CLI:** Human interaction (prompts, colors, formatting)
- **Client:** Programmatic integration (async, types, networking)

### 2. Dependency Management
- **CLI needs:** `clap`, `colored`, `dialoguer`, `indicatif` (UI)
- **Client needs:** `tokio`, `reqwest`, `serde` (networking)
- **No overlap!** Keep dependencies minimal

### 3. Different Consumers
- **CLI consumers:** Operators, admins, developers (humans)
- **Client consumers:** Compute provider, Security Provider, Storage provider, AI provider (primals)

### 4. Different Interfaces
- **CLI interface:** Subcommands, flags, help text
- **Client interface:** Rust structs, methods, types

### 5. Versioning
- **CLI:** Can break between versions (humans adapt)
- **Client:** Must maintain API stability (primals depend on it)

---

## 📊 Comparison Matrix

| Feature | songbird-cli | songbird-client |
|---------|--------------|-----------------|
| **Type** | Binary (executable) | Library (crate) |
| **Interface** | Terminal commands | Rust API |
| **Output** | Formatted text, colors | Rust types |
| **Input** | Flags, arguments | Function calls |
| **Async** | Optional | Required |
| **Error Handling** | Exit codes, messages | Result types |
| **Documentation** | `--help` flags | Rustdoc |
| **Testing** | Integration tests | Unit + integration |
| **Distribution** | Installed binary | Cargo dependency |
| **Updates** | User upgrades | Cargo version |

---

## 🚀 Implementation Plan

### Phase 1: Create songbird-client Crate ✅ NEXT

```bash
cd crates/
cargo new --lib songbird-client
```

**Structure:**
```
crates/songbird-client/
├── Cargo.toml
├── src/
│   ├── lib.rs               # Public API
│   ├── client.rs            # SongbirdClient struct
│   ├── discovery.rs         # Discovery methods
│   ├── registration.rs      # Registration protocol
│   ├── types.rs             # Request/Response types
│   └── error.rs             # Error types
└── tests/
    └── integration_tests.rs
```

**Public API:**
```rust
pub struct SongbirdClient { ... }

impl SongbirdClient {
    // Discovery
    pub async fn discover_local() -> Result<Self>;
    pub async fn discover_udp() -> Result<Self>;
    pub async fn discover_env() -> Result<Self>;
    
    // Registration
    pub async fn register_service(
        &self,
        name: &str,
        version: &str,
        capabilities: Vec<Capability>,
    ) -> Result<Registration>;
    
    // Lifecycle
    pub async fn heartbeat(&self, service_id: &str) -> Result<()>;
    pub async fn deregister(&self, service_id: &str) -> Result<()>;
    
    // Query
    pub async fn query_services(&self, capability: &str) -> Result<Vec<Service>>;
}
```

### Phase 2: Use songbird-client in Compute provider

```toml
# compute_provider/Cargo.toml
[dependencies]
songbird-client = { path = "../songbird/crates/songbird-client" }
```

```rust
// compute_provider/src/main.rs
use songbird_client::SongbirdClient;

#[tokio::main]
async fn main() -> Result<()> {
    let songbird = SongbirdClient::discover_local().await?;
    let registration = songbird.register_service(...).await?;
    // ... bind to assigned port, serve
}
```

### Phase 3: Update songbird-cli to Show Registered Services

```rust
// crates/songbird-cli/src/cli/services.rs

pub async fn handle_services_list() -> Result<()> {
    // Use songbird-client internally for consistency
    let client = SongbirdClient::discover_local().await?;
    let services = client.query_services("*").await?;
    
    // Pretty print for humans
    println!("📋 Registered Services:");
    for service in services {
        println!("  • {} ({})", service.name, service.capabilities.join(", "));
    }
}
```

---

## 🤔 Could CLI Use Client Internally?

**Yes! And it should!**

```rust
// crates/songbird-cli/Cargo.toml
[dependencies]
songbird-client = { path = "../songbird-client" }  # Share the client!
```

**Benefits:**
1. **DRY:** CLI reuses client's discovery logic
2. **Consistency:** Same API, same behavior
3. **Maintenance:** Fix once, works everywhere

**CLI becomes a thin wrapper:**
```rust
// CLI command
pub async fn handle_status() -> Result<()> {
    // Use client internally
    let client = SongbirdClient::discover_local().await?;
    let status = client.get_status().await?;
    
    // Pretty print for humans
    println!("✅ Songbird is running");
    println!("   Version: {}", status.version);
    println!("   Uptime: {}s", status.uptime);
}
```

---

## 🎯 Summary

### songbird-cli (Exists)
- **Purpose:** Human-facing command-line tool
- **Users:** Operators, admins, developers
- **Interface:** Terminal commands
- **Examples:** `songbird status`, `songbird gaming join`

### songbird-client (New)
- **Purpose:** Primal-facing integration library
- **Users:** Compute provider, Security Provider, Storage provider, AI provider
- **Interface:** Rust API
- **Examples:** `client.register_service()`, `client.heartbeat()`

### Relationship
- **Separate concerns:** Humans vs primals
- **Can share code:** CLI can use client internally
- **Different interfaces:** Commands vs function calls
- **Same endpoint:** Both talk to Songbird orchestrator

### Next Steps
1. ✅ Understand the distinction (done!)
2. 🎯 Create `songbird-client` crate
3. 🎯 Add registration endpoints to orchestrator
4. 🎯 Wire Compute provider to use client
5. 🎯 Optionally: Refactor CLI to use client internally

---

**Answer:** They are **different crates** for **different users**. CLI is for humans, client is for primals. We need to create the client! 🎵

