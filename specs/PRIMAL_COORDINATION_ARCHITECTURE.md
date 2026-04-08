# 🎵 Primal Coordination Architecture

**Version**: 1.0  
**Status**: Specification  
**Date**: December 24, 2025

---

## 🎯 Vision

**Songbird as Universal Signal and Coordinator**

Songbird is the **nervous system** of the ecoPrimals ecosystem, coordinating specialized **primals** (organs) without attempting to be them. Each primal maintains sovereignty over its domain while Songbird provides universal communication and orchestration.

---

## 🏗️ Architectural Principles

### 1. Primal Sovereignty
- Each primal owns its domain completely
- Songbird coordinates, doesn't execute
- No functionality overlap or violation

### 2. Evolution Pattern: Specific → Generic → Agnostic
```
Phase 1: Specific Implementation
  ├─ Security Provider integration (security operations)
  ├─ Compute provider integration (compute operations)
  └─ Concrete types, direct calls

Phase 2: Generic Abstraction
  ├─ PrimalBridge trait
  ├─ Capability negotiation
  └─ Request/response patterns

Phase 3: Agnostic Coordination
  ├─ Dynamic primal discovery
  ├─ Capability-based routing
  └─ Plugin architecture for new primals
```

### 3. Universal Signal
- Songbird is the communication substrate
- All inter-primal coordination flows through Songbird
- Pure messaging, no domain logic

---

## 📋 Domain Boundaries

### What Songbird OWNS (Communication & Coordination)

```
✅ P2P Networking
  - BTSP secure tunnels
  - BirdSong discovery
  - Pure Rust BLE channels
  - Transport abstraction (USB, UART, TCP, etc.)

✅ Federation & Discovery
  - Multi-federation coordinator
  - Capability discovery
  - Service registration
  - Trust escalation

✅ Orchestration
  - Multi-primal coordination
  - Workflow management
  - Event distribution
  - State synchronization

✅ Bridges (Interface Only)
  - Primal connection management
  - Request/response routing
  - Capability negotiation
  - Health monitoring
```

### What Songbird DOES NOT Own

```
❌ Security Operations (Security Provider's Domain)
  - Entropy collection
  - Key generation
  - Signing/verification
  - Certificate management

❌ Compute Operations (Compute provider's Domain)
  - ML workload execution
  - Resource allocation
  - Result computation
  - Hardware management

❌ Primal-Internal Hardware
  - Each primal manages its own USB/peripherals
  - Songbird only uses hardware for communication
```

---

## 🔧 Architecture Design

### Layer 1: Primal Bridge (Generic Interface)

```rust
/// Generic interface to any primal
#[async_trait]
pub trait PrimalBridge: Send + Sync {
    /// Get primal metadata
    fn metadata(&self) -> &PrimalMetadata;
    
    /// Connect to primal
    async fn connect(&mut self) -> Result<()>;
    
    /// Discover capabilities
    async fn discover_capabilities(&self) -> Result<Capabilities>;
    
    /// Send request, get response
    async fn request(&self, req: Request) -> Result<Response>;
    
    /// Subscribe to events
    async fn subscribe(&self, events: Vec<EventType>) -> Result<EventStream>;
    
    /// Health check
    async fn health_check(&self) -> Result<HealthStatus>;
}
```

### Layer 2: Specific Primal Implementations

```rust
/// Security Provider Bridge (Security Operations)
pub struct SecurityProviderBridge {
    connection: P2PConnection,
    capabilities: SecurityProviderCapabilities,
}

#[async_trait]
impl PrimalBridge for SecurityProviderBridge {
    fn metadata(&self) -> &PrimalMetadata {
        &PrimalMetadata {
            name: "Security Provider",
            version: "1.0.0",
            domain: PrimalDomain::Security,
            capabilities: vec![
                "signature_generation",
                "signature_verification",
                "key_derivation",
                "entropy_provision",
                "genesis_witness",
            ],
        }
    }
    
    async fn connect(&mut self) -> Result<()> {
        // Establish secure P2P connection
        self.connection.connect().await
    }
    
    // ... implement trait methods
}

impl SecurityProviderBridge {
    /// Request signature (Security Provider-specific operation)
    pub async fn request_signature(&self, 
        data: &[u8]
    ) -> Result<Signature> {
        let request = Request::new("sign", json!({ "data": data }));
        let response = self.request(request).await?;
        response.parse_signature()
    }
    
    /// Verify signature (delegate to Security Provider)
    pub async fn verify_signature(&self,
        data: &[u8],
        signature: &Signature
    ) -> Result<bool> {
        let request = Request::new("verify", json!({
            "data": data,
            "signature": signature
        }));
        let response = self.request(request).await?;
        response.parse_bool()
    }
}

/// Compute provider Bridge (Compute Operations)
pub struct ComputeProviderBridge {
    connection: P2PConnection,
    capabilities: ComputeProviderCapabilities,
}

#[async_trait]
impl PrimalBridge for ComputeProviderBridge {
    fn metadata(&self) -> &PrimalMetadata {
        &PrimalMetadata {
            name: "Compute provider",
            version: "1.0.0",
            domain: PrimalDomain::Compute,
            capabilities: vec![
                "ml_inference",
                "model_training",
                "distributed_compute",
                "gpu_acceleration",
            ],
        }
    }
    
    // ... implement trait methods
}

impl ComputeProviderBridge {
    /// Deploy compute workload (Compute provider-specific)
    pub async fn deploy_workload(&self,
        workload: Workload
    ) -> Result<DeploymentHandle> {
        let request = Request::new("deploy", json!({
            "workload": workload
        }));
        let response = self.request(request).await?;
        response.parse_deployment_handle()
    }
    
    /// Monitor workload progress
    pub async fn monitor_workload(&self,
        handle: &DeploymentHandle
    ) -> Result<WorkloadStatus> {
        let request = Request::new("status", json!({
            "handle": handle
        }));
        let response = self.request(request).await?;
        response.parse_status()
    }
}
```

### Layer 3: Primal Coordinator (Universal Orchestration)

```rust
/// Universal coordinator for all primals
pub struct PrimalCoordinator {
    // Registry of available primals
    primals: HashMap<PrimalId, Box<dyn PrimalBridge>>,
    
    // Discovery service
    discovery: PrimalDiscovery,
    
    // Event bus for coordination
    event_bus: EventBus,
}

impl PrimalCoordinator {
    /// Register a primal bridge
    pub fn register(&mut self, 
        id: PrimalId,
        bridge: Box<dyn PrimalBridge>
    ) -> Result<()> {
        self.primals.insert(id, bridge);
        Ok(())
    }
    
    /// Discover primals by capability
    pub async fn discover_by_capability(&self,
        capability: &str
    ) -> Result<Vec<PrimalId>> {
        let mut capable = Vec::new();
        
        for (id, bridge) in &self.primals {
            let caps = bridge.discover_capabilities().await?;
            if caps.has(capability) {
                capable.push(id.clone());
            }
        }
        
        Ok(capable)
    }
    
    /// Route request to capable primal
    pub async fn route_request(&self,
        capability: &str,
        request: Request
    ) -> Result<Response> {
        // Find capable primal
        let primals = self.discover_by_capability(capability).await?;
        
        if primals.is_empty() {
            return Err(Error::NoCapablePrimal { capability: capability.into() });
        }
        
        // Use first capable primal (could add load balancing)
        let bridge = self.primals.get(&primals[0])
            .ok_or(Error::PrimalNotFound)?;
        
        bridge.request(request).await
    }
    
    /// Coordinate multi-primal workflow
    pub async fn coordinate_workflow(&self,
        workflow: Workflow
    ) -> Result<WorkflowResult> {
        let mut results = HashMap::new();
        
        for step in workflow.steps {
            // Route to appropriate primal based on capability
            let response = self.route_request(
                &step.capability,
                step.request
            ).await?;
            
            results.insert(step.id, response);
        }
        
        Ok(WorkflowResult { results })
    }
}
```

### Layer 4: High-Level Coordination Patterns

```rust
/// Genesis Ceremony Coordinator
/// Orchestrates Security Provider + network without doing crypto
pub struct GenesisCoordinator {
    coordinator: PrimalCoordinator,
}

impl GenesisCoordinator {
    pub async fn conduct_genesis(&self,
        new_node: NodeId
    ) -> Result<Identity> {
        // 1. Discover Security Provider primal
        let security_provider = self.coordinator
            .discover_by_capability("genesis_witness")
            .await?;
        
        // 2. Request key generation (Security Provider does the crypto)
        let keys_request = Request::new("generate_keys", json!({
            "node_id": new_node
        }));
        let keys_response = self.coordinator
            .route_request("key_generation", keys_request)
            .await?;
        
        // 3. Establish witness connections (Songbird's job)
        let witnesses = self.discover_witnesses().await?;
        
        // 4. Request lineage signature (Security Provider does signing)
        let lineage_request = Request::new("sign_lineage", json!({
            "node_id": new_node,
            "witnesses": witnesses
        }));
        let lineage = self.coordinator
            .route_request("signature_generation", lineage_request)
            .await?;
        
        // 5. Assemble and return identity (coordination only)
        Ok(Identity::new(new_node, keys_response, lineage))
    }
}

/// Compute Deployment Coordinator
/// Routes workloads to Compute provider without executing them
pub struct ComputeCoordinator {
    coordinator: PrimalCoordinator,
}

impl ComputeCoordinator {
    pub async fn deploy_compute(&self,
        workload: Workload
    ) -> Result<DeploymentHandle> {
        // 1. Discover capable Compute provider nodes
        let compute_nodes = self.coordinator
            .discover_by_capability("ml_inference")
            .await?;
        
        if compute_nodes.is_empty() {
            return Err(Error::NoComputeAvailable);
        }
        
        // 2. Select best node (could add load balancing)
        let selected = compute_nodes[0].clone();
        
        // 3. Route deployment (Compute provider does execution)
        let deploy_request = Request::new("deploy", json!({
            "workload": workload
        }));
        let response = self.coordinator
            .primals
            .get(&selected)
            .ok_or(Error::PrimalNotFound)?
            .request(deploy_request)
            .await?;
        
        Ok(response.parse_deployment_handle()?)
    }
    
    pub async fn monitor_deployment(&self,
        handle: &DeploymentHandle
    ) -> Result<WorkloadStatus> {
        // Query Compute provider for status (don't compute ourselves)
        let status_request = Request::new("status", json!({
            "handle": handle
        }));
        let response = self.coordinator
            .route_request("workload_monitoring", status_request)
            .await?;
        
        Ok(response.parse_status()?)
    }
}
```

---

## 🔄 Evolution Path (Inspired by Gaming System)

### Phase 1: Specific (Current - Gaming Example)

```rust
// Old: Hardcoded to specific gaming systems
pub enum GamingSystem {
    Steam(SteamIntegration),
    EpicGames(EpicGamesIntegration),
    Custom(CustomIntegration),
}
```

### Phase 2: Generic

```rust
// Current: Generic gaming platform trait
pub trait GamingPlatform {
    fn get_games(&self) -> Result<Vec<Game>>;
    fn launch_game(&self, game_id: &str) -> Result<()>;
}
```

### Phase 3: Agnostic

```rust
// Future: Capability-based discovery
pub struct GamingCoordinator {
    platforms: HashMap<PlatformId, Box<dyn GamingPlatform>>,
}

impl GamingCoordinator {
    pub async fn discover_platforms(&self) -> Vec<Platform> {
        // Auto-discover via capability advertisement
    }
}
```

### Applied to Primals

**Phase 1: Specific** (Start Here)
```rust
// Concrete implementations for known primals
pub struct SecurityProviderBridge { /* ... */ }
pub struct ComputeProviderBridge { /* ... */ }

// Direct usage
let security_provider = SecurityProviderBridge::new(connection);
let signature = security_provider.request_signature(data).await?;
```

**Phase 2: Generic** (Next)
```rust
// Generic trait for any primal
pub trait PrimalBridge { /* ... */ }

impl PrimalBridge for SecurityProviderBridge { /* ... */ }
impl PrimalBridge for ComputeProviderBridge { /* ... */ }

// Generic usage
let primal: Box<dyn PrimalBridge> = Box::new(SecurityProviderBridge::new());
let response = primal.request(request).await?;
```

**Phase 3: Agnostic** (Future)
```rust
// Capability-based discovery and routing
let coordinator = PrimalCoordinator::new();

// Auto-discover available primals
coordinator.discover_all().await?;

// Route by capability (don't care which primal)
let response = coordinator.route_request(
    "signature_generation",
    request
).await?;
```

---

## 🎯 Implementation Roadmap

### Milestone 1: Specific Implementations (1-2 weeks)

**Goal**: Get Security Provider and Compute provider integration working

```
Tasks:
1. Create SecurityProviderBridge struct
   - Connect via P2P
   - Request/response protocol
   - Signature operations
   - Genesis coordination

2. Enhance ComputeProviderBridge
   - Workload deployment
   - Progress monitoring
   - Result retrieval
   - Resource negotiation

3. Integration tests
   - End-to-end Genesis
   - Compute deployment
   - Error handling
```

### Milestone 2: Generic Abstraction (1 week)

**Goal**: Abstract to PrimalBridge trait

```
Tasks:
1. Define PrimalBridge trait
   - Core operations
   - Capability discovery
   - Health checks

2. Implement trait for existing bridges
   - SecurityProviderBridge
   - ComputeProviderBridge

3. Create PrimalCoordinator
   - Registry management
   - Request routing
   - Event distribution

4. Refactor consumers to use trait
```

### Milestone 3: Agnostic Coordination (1-2 weeks)

**Goal**: Capability-based discovery and routing

```
Tasks:
1. Capability advertisement protocol
   - Primal announces capabilities
   - Version negotiation
   - Feature flags

2. Dynamic discovery
   - Network scanning
   - Service registration
   - Health monitoring

3. Load balancing and failover
   - Multiple primal instances
   - Automatic failover
   - Performance metrics

4. Plugin architecture
   - Runtime primal loading
   - Configuration-based enablement
   - Hot reloading
```

---

## 📊 Data Structures

### Primal Metadata

```rust
pub struct PrimalMetadata {
    /// Primal name (e.g., "Security Provider", "Compute provider")
    pub name: String,
    
    /// Version (semver)
    pub version: String,
    
    /// Domain specialization
    pub domain: PrimalDomain,
    
    /// List of capabilities
    pub capabilities: Vec<String>,
    
    /// API endpoint
    pub endpoint: Endpoint,
}

pub enum PrimalDomain {
    Security,       // Security Provider
    Compute,        // Compute provider
    Communication,  // Songbird
    Storage,        // Future: Could have data primal
    Analytics,      // Future: Could have analytics primal
    Custom(String), // Extensible
}
```

### Capability System

```rust
pub struct Capabilities {
    /// List of capability identifiers
    capabilities: HashSet<String>,
    
    /// Capability metadata
    metadata: HashMap<String, CapabilityMetadata>,
}

pub struct CapabilityMetadata {
    /// Human-readable description
    pub description: String,
    
    /// Required parameters
    pub parameters: Vec<Parameter>,
    
    /// Return type
    pub return_type: TypeInfo,
    
    /// Performance hints
    pub performance: PerformanceHints,
}

impl Capabilities {
    pub fn has(&self, capability: &str) -> bool {
        self.capabilities.contains(capability)
    }
    
    pub fn discover_compatible(&self, 
        required: &[String]
    ) -> bool {
        required.iter().all(|c| self.has(c))
    }
}
```

### Request/Response Protocol

```rust
pub struct Request {
    /// Request ID for tracking
    pub id: RequestId,
    
    /// Operation name
    pub operation: String,
    
    /// Parameters (JSON-serializable)
    pub params: serde_json::Value,
    
    /// Timeout
    pub timeout: Option<Duration>,
}

pub struct Response {
    /// Corresponding request ID
    pub request_id: RequestId,
    
    /// Success/failure
    pub status: ResponseStatus,
    
    /// Result data
    pub data: serde_json::Value,
    
    /// Metadata (timing, etc.)
    pub metadata: ResponseMetadata,
}

pub enum ResponseStatus {
    Success,
    Error { code: ErrorCode, message: String },
    Timeout,
}
```

---

## 🔐 Security Considerations

### 1. Primal Authentication
- Each primal must authenticate via Security Provider
- Mutual TLS for all inter-primal communication
- Capability-based access control

### 2. Request Validation
- All requests validated before routing
- Parameter type checking
- Rate limiting per primal

### 3. Isolation
- Primals cannot directly access each other
- All communication through Songbird
- Audit logging of all coordination

---

## 📈 Performance Considerations

### 1. Connection Pooling
- Maintain persistent connections to primals
- Reuse connections across requests
- Health checks and reconnection

### 2. Caching
- Cache capability discovery results
- TTL-based invalidation
- Refresh on health check

### 3. Async All The Way
- No blocking operations
- Concurrent request handling
- Backpressure handling

---

## 🧪 Testing Strategy

### Unit Tests
```rust
#[tokio::test]
async fn test_security_provider_signature_request() {
    let bridge = MockSecurityProviderBridge::new();
    let signature = bridge.request_signature(b"test").await.unwrap();
    assert!(signature.is_valid());
}

#[tokio::test]
async fn test_capability_discovery() {
    let coordinator = PrimalCoordinator::new();
    coordinator.register(
        "security".into(),
        Box::new(MockSecurityProviderBridge::new())
    ).unwrap();
    
    let capable = coordinator
        .discover_by_capability("signature_generation")
        .await
        .unwrap();
    
    assert_eq!(capable.len(), 1);
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_genesis_coordination() {
    let coordinator = setup_test_coordinator().await;
    
    let identity = coordinator
        .conduct_genesis("new_node".into())
        .await
        .unwrap();
    
    assert!(identity.is_valid());
    assert!(identity.has_lineage());
}

#[tokio::test]
async fn test_compute_deployment() {
    let coordinator = setup_test_coordinator().await;
    
    let handle = coordinator
        .deploy_compute(test_workload())
        .await
        .unwrap();
    
    let status = coordinator
        .monitor_deployment(&handle)
        .await
        .unwrap();
    
    assert_eq!(status, WorkloadStatus::Running);
}
```

---

## 🚀 Future Extensions

### Additional Primals

As the ecosystem grows, new primals can be added without changing Songbird:

```rust
// Future: Storage Primal
pub struct StoragePrimal {
    // Distributed storage coordination
}

impl PrimalBridge for StoragePrimal {
    // Implements standard interface
}

// Future: Analytics Primal
pub struct AnalyticsPrimal {
    // Data analysis and insights
}

impl PrimalBridge for AnalyticsPrimal {
    // Implements standard interface
}
```

### Enhanced Coordination Patterns

```rust
// Multi-step workflows
let workflow = Workflow::new()
    .step("generate_keys", "Security Provider")
    .step("allocate_compute", "Compute provider")
    .step("store_results", "Storage")
    .execute();

// Event-driven coordination
coordinator.on_event("workload_complete", |event| {
    // Automatically trigger next step
});

// Parallel coordination
coordinator.parallel()
    .task("security", sign_operation)
    .task("compute_provider", compute_operation)
    .await_all()
    .unwrap();
```

---

## ✅ Success Criteria

### Technical
- [ ] Security Provider bridge operational
- [ ] Compute provider bridge operational
- [ ] PrimalBridge trait abstraction complete
- [ ] Capability-based discovery working
- [ ] All tests passing

### Architectural
- [ ] Clean separation of concerns
- [ ] No domain overlap
- [ ] Extensible to new primals
- [ ] Follows evolution pattern (specific → generic → agnostic)

### User Experience
- [ ] Simple API for coordination
- [ ] Clear error messages
- [ ] Performance acceptable (< 100ms overhead)
- [ ] Transparent failover

---

## 📚 References

- **Compute Bridge**: [crates/songbird-compute-bridge/](../crates/songbird-compute-bridge/) - Existing bridge pattern
- **Federation**: [crates/songbird-network-federation/](../crates/songbird-network-federation/) - Coordination patterns
- **Security provider genesis handoff** (historical document, Dec 2025) — genesis requirements; filename retained in older check-ins only

---

**Version**: 1.0  
**Status**: Specification - Ready for Implementation  
**Next**: Begin Milestone 1 (Specific Implementations)

🎵 **Songbird: The Universal Signal and Coordinator**

