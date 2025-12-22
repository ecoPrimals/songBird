# Primal Responsibility Separation - Security Architecture

## The Problem: Songbird Taking Too Much

We've been loading Songbird with security concerns that belong to **BearDog**!

### Current Confusion:
```
Songbird doing:
- Orchestration ✅ (correct)
- Federation ✅ (correct)
- Discovery ✅ (correct)
- TLS management ⚠️ (should be BearDog)
- Certificate verification ⚠️ (should be BearDog)
- Mutual authentication ⚠️ (should be BearDog)
- Trust verification ⚠️ (should be BearDog)
- Encryption keys ⚠️ (should be BearDog)
```

## Correct Separation of Concerns

### 🎵 Songbird - Orchestration & Federation
**Focus:** "Who does what and where"

**Responsibilities:**
- Service discovery (finding nodes)
- Task routing (sending work where it needs to go)
- Port allocation (Universal Port Authority)
- Federation coordination (keeping nodes connected)
- Capability matching (finding who can do what)
- Load distribution

**NOT Responsible For:**
- How connections are secured (ask BearDog)
- Whether to trust a node (ask BearDog)
- How to encrypt (ask BearDog)
- Key management (ask BearDog)

**Songbird's Security Role:**
```rust
// Songbird just asks: "BearDog, can I trust this connection?"
let can_connect = beardog.verify_trust(
    node_id,
    endpoint,
    certificate,
).await?;

if can_connect {
    // Songbird does the orchestration
    route_task_to_node(node_id, task).await?;
}
```

### 🐻 BearDog - Security & Trust
**Focus:** "Can I trust this? Is this secure?"

**Responsibilities:**
- Certificate management (issue, verify, rotate)
- Mutual TLS (mTLS) implementation
- Key generation and storage
- Trust verification (TOFU, pinning, chains)
- Encryption/decryption
- Authentication (who is this node?)
- Authorization (what can they do?)
- Audit logging (who did what?)

**NOT Responsible For:**
- Where to route tasks (ask Songbird)
- Who has what capabilities (ask Songbird)
- Federation topology (ask Songbird)

**BearDog's Role:**
```rust
// BearDog provides security primitives
impl BearDog {
    // Verify if we should trust a peer
    async fn verify_trust(&self, peer: &PeerInfo) -> TrustDecision;
    
    // Establish secure connection
    async fn secure_connect(&self, endpoint: &str) -> SecureConnection;
    
    // Issue certificates for federation
    async fn issue_federation_cert(&self, node_id: &str) -> Certificate;
    
    // Verify certificate chain
    async fn verify_certificate(&self, cert: &Certificate) -> bool;
    
    // Generate keys
    async fn generate_keypair(&self) -> KeyPair;
}
```

### 🍄 Toadstool - Compute & ML
**Focus:** "Running workloads and training models"

**Responsibilities:**
- Task execution
- GPU management
- ML training
- Model inference
- Container orchestration

**NOT Responsible For:**
- How to find work (ask Songbird)
- Security (ask BearDog)

### 🏠 Nestgate - Data & Storage
**Focus:** "Persistent data and state"

**Responsibilities:**
- Data storage
- State persistence
- Backup/replication
- Data retrieval

**NOT Responsible For:**
- Task routing (ask Songbird)
- Encryption (ask BearDog for keys)

### 🐿️ Squirrel - AI/MCP Interface
**Focus:** "LLM and AI coordination"

**Responsibilities:**
- LLM interfacing
- MCP protocol
- AI task coordination
- Model management

**NOT Responsible For:**
- Security (ask BearDog)
- Where to run (ask Songbird)

## Correct Architecture: Primal Collaboration

### Example: Secure Federation

**Old Way (Songbird doing everything):**
```rust
// ❌ Songbird has too much responsibility
impl Songbird {
    async fn connect_to_peer(&self, peer: Endpoint) {
        // Generate TLS config
        let tls = self.generate_tls()?;
        
        // Verify certificate
        let cert_ok = self.verify_cert(peer.cert)?;
        
        // Establish connection
        let conn = self.tls_connect(peer, tls)?;
        
        // Now route tasks
        self.route_tasks(conn)?;
    }
}
```

**New Way (Primals collaborate):**
```rust
// ✅ Each primal does its job
impl Songbird {
    async fn connect_to_peer(&self, peer: Endpoint) {
        // 1. Ask BearDog: Is this secure?
        let security_session = self.beardog
            .establish_secure_connection(peer)
            .await?;
        
        // 2. Songbird does orchestration
        self.route_tasks_over(security_session).await?;
    }
}

impl BearDog {
    async fn establish_secure_connection(&self, peer: Endpoint) -> SecureSession {
        // BearDog handles ALL security
        let cert = self.verify_certificate(peer.cert)?;
        let trust = self.check_trust_level(peer.node_id)?;
        let tls = self.configure_mtls(peer)?;
        
        SecureSession::new(tls, trust)
    }
}
```

### Example: Internet Deployment

**Security Concerns → BearDog:**
```rust
// BearDog implements rendezvous security
impl BearDog {
    // Secure rendezvous registration
    async fn register_with_rendezvous(&self, 
        rendezvous: &str,
        node_id: &NodeId,
    ) -> Result<RendezvousToken> {
        // 1. Generate proof of identity
        let proof = self.sign_registration(node_id)?;
        
        // 2. Establish encrypted channel
        let channel = self.secure_channel(rendezvous)?;
        
        // 3. Register with authentication
        channel.register(node_id, proof).await
    }
    
    // Verify peer through rendezvous
    async fn verify_via_rendezvous(&self,
        peer_id: &NodeId,
        rendezvous_token: &Token,
    ) -> TrustLevel {
        // Cryptographic verification
        self.verify_signature(peer_id, rendezvous_token)
    }
}
```

**Orchestration Concerns → Songbird:**
```rust
// Songbird uses BearDog's security, handles routing
impl Songbird {
    async fn connect_to_roaming_peer(&self, peer_id: &NodeId) {
        // 1. Ask BearDog for secure connection
        let secure_conn = self.beardog
            .connect_via_rendezvous(peer_id)
            .await?;
        
        // 2. Songbird routes tasks
        self.federation.add_peer(peer_id, secure_conn)?;
        self.route_tasks_to(peer_id).await?;
    }
}
```

## Interaction Pattern: Runtime Capabilities

### Discovery Flow:
```rust
// Songbird discovers orchestrator with "security" capability
let orchestrators = discover_orchestrators().await?;

for orchestrator in orchestrators {
    // Check if this orchestrator provides security services
    if orchestrator.has_capability("security") {
        // This is BearDog!
        let beardog = BearDog::connect(orchestrator).await?;
        
        // Use it for all security needs
        self.security_provider = Some(beardog);
    }
}
```

### No Compile-Time Dependencies:
```rust
// ✅ Songbird doesn't import BearDog
// They interact via HTTP at runtime

// Songbird asks for security via API
let response = reqwest::post("http://localhost:9000/api/v1/security/verify")
    .json(&VerifyRequest {
        peer_id: "abc123",
        certificate: cert_data,
    })
    .await?;

// BearDog responds with trust decision
let trust: TrustDecision = response.json().await?;
```

## What Songbird Should Do for Internet

### Minimal Security (Delegate to BearDog):

**Songbird's Internet Responsibilities:**
```rust
impl Songbird {
    // 1. Discover rendezvous (orchestration)
    async fn find_rendezvous() -> Vec<RendezvousServer>;
    
    // 2. Register presence (orchestration)
    async fn register_with_rendezvous(&self, 
        server: &RendezvousServer,
        security: &BearDog, // Use BearDog for security
    );
    
    // 3. Coordinate connections (orchestration)
    async fn coordinate_peer_connection(&self,
        peer_id: &NodeId,
        security: &BearDog, // BearDog secures it
    );
    
    // 4. Route tasks (orchestration)
    async fn route_task_via_connection(&self, conn: SecureConnection);
}
```

**BearDog's Internet Responsibilities:**
```rust
impl BearDog {
    // 1. Certificate management
    async fn issue_node_certificate(&self) -> Certificate;
    
    // 2. Mutual TLS
    async fn establish_mtls(&self, peer: &Peer) -> SecureConnection;
    
    // 3. Trust decisions
    async fn should_trust(&self, peer: &Peer) -> TrustDecision;
    
    // 4. Encryption
    async fn encrypt_message(&self, data: &[u8]) -> EncryptedData;
    
    // 5. NAT traversal security
    async fn secure_nat_traversal(&self, stun: &StunServer) -> SecureNATSession;
}
```

## Implementation Strategy

### Phase 1: Define BearDog API
```rust
// crates/beardog-api/
pub trait SecurityProvider {
    async fn verify_peer(&self, peer: &PeerInfo) -> TrustDecision;
    async fn secure_connection(&self, endpoint: &str) -> SecureConnection;
    async fn issue_certificate(&self, identity: &Identity) -> Certificate;
}
```

### Phase 2: Songbird Uses BearDog
```rust
// Songbird discovers BearDog via capability
let security = discover_capability("security").await?;

// Songbird delegates security decisions
let trust = security.verify_peer(peer_info).await?;
if trust.is_trusted() {
    // Songbird does orchestration
    self.add_to_federation(peer).await?;
}
```

### Phase 3: BearDog Implements Security
```rust
// BearDog registers with Songbird
let registration = songbird.register_service(ServiceInfo {
    name: "BearDog",
    capabilities: vec![
        Capability { name: "security", ... },
        Capability { name: "authentication", ... },
        Capability { name: "encryption", ... },
    ],
}).await?;

// BearDog handles all security requests
server.route("/api/v1/security/verify", verify_handler);
server.route("/api/v1/security/encrypt", encrypt_handler);
server.route("/api/v1/security/certificates", cert_handler);
```

## Current State Assessment

### What Songbird Currently Does (That Should Move to BearDog):

1. **TLS Configuration** → BearDog
   - File: `crates/songbird-orchestrator/src/app/http_server.rs`
   - Should: Query BearDog for TLS config

2. **Certificate Generation** → BearDog
   - File: `crates/songbird-network-federation/src/tls.rs`
   - Should: BearDog generates and manages

3. **Trust Decisions** → BearDog
   - Currently: Implicit (accept all on LAN)
   - Should: BearDog explicit trust verification

### What Songbird Should Keep:

1. **Federation Discovery** ✅
   - UDP broadcast (LAN)
   - Rendezvous coordination (Internet)
   - Node registration

2. **Task Routing** ✅
   - Capability matching
   - Load balancing
   - Priority routing

3. **Port Management** ✅
   - Universal Port Authority
   - Dynamic allocation

## Summary

### Songbird's Role:
**"I orchestrate - I don't secure"**
- Find nodes
- Route tasks
- Manage ports
- Coordinate federation

### BearDog's Role:
**"I secure - I don't orchestrate"**
- Verify trust
- Encrypt connections
- Manage certificates
- Authenticate peers

### Collaboration:
```
Songbird: "I need to connect to node X"
    ↓
BearDog: "Here's a secure connection"
    ↓
Songbird: "Thanks, now I'll route tasks"
```

### For Internet Deployment:

**Songbird adds:**
- Rendezvous discovery
- NAT traversal coordination
- Connection migration

**BearDog adds:**
- mTLS implementation
- Certificate verification
- Encrypted channels
- Trust-on-first-use

**Each primal stays focused on its domain!**

---

*ecoPrimals - Each Primal Knows Its Domain*

