# 🌐 Songbird Native RPC Specification

**Date**: January 2025  
**Status**: **DESIGN SPECIFICATION** - Ready for Implementation  
**Priority**: **P0 CRITICAL** - Transport system foundation  
**Scope**: Pure Rust bidirectional RPC for universal orchestration  

---

## 🎯 **Executive Summary**

Songbird requires a **pure Rust bidirectional RPC system** to match the transport sophistication of other primals in the ecosystem. While Compute Provider, Security Provider, and AI provider have implemented advanced RPC systems, Songbird remains limited to HTTP/WebSocket, creating a transport bottleneck in the universal orchestration layer.

### **🏆 Design Goals**

1. **Pure Rust RPC** - No gRPC, no protobuf dependencies (uses tarpc + custom JSON RPC)
2. **True Bidirectional** - Both client and server can initiate requests
3. **Zero-Copy Streaming** - Memory-efficient real-time data flow
4. **Universal Orchestration** - Designed for service mesh coordination
5. **Multi-Fallback Robust** - Multiple transport fallbacks (tarpc → WebSocket → HTTP)
6. **AI-First Integration** - Native support for MCP protocol extensions
7. **Ecosystem Native** - Perfect integration with other primals via tarpc

---

## 🏗️ **Architecture Overview**

### **Core Components**

```rust
// Core RPC system architecture - ACTUAL IMPLEMENTATION
pub struct SongbirdRPC {
    /// tarpc connection pool for high-performance primals
    tarpc_pool: ConnectionPool<TarpcConnection>,
    
    /// Custom JSON RPC for universal compatibility
    json_rpc_pool: ConnectionPool<JsonRpcConnection>,
    
    /// Message router for capability-based routing
    message_router: UniversalMessageRouter,
    
    /// Stream manager for real-time data flows  
    stream_manager: BidirectionalStreamManager,
    
    /// Health monitor for connection status
    health_monitor: ConnectionHealthMonitor,
    
    /// Security layer integrated with Security Provider tunnels
    security_layer: SecurityTunnelIntegration,
}
```

### **Protocol Stack (CORRECTED)**

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
│              Songbird Universal Adapters                   │
├─────────────────────────────────────────────────────────────┤
│                    Service Mesh Layer                       │
│     Message Routing | Stream Management | Load Balancing    │
├─────────────────────────────────────────────────────────────┤
│                    Transport Layer                          │
│  Primary: tarpc | Fallback-1: WebSocket+JSON | Fallback-2: HTTP│
├─────────────────────────────────────────────────────────────┤
│                    Security Layer                           │
│  Security Provider Tunnel Integration | TLS | Custom Encryption      │
├─────────────────────────────────────────────────────────────┤
│                    Network Layer                            │
│              TCP/UDP | IPv4/IPv6 | Custom Protocols        │
└─────────────────────────────────────────────────────────────┘
```

---

## 📋 **Implementation Roadmap**

### **Phase 1: Core RPC Foundation (Week 1-2)**

#### **1.1 tarpc Service Definition (Based on Security Provider Implementation)**
```rust
/// Universal RPC service for Songbird orchestration
/// Based on proven security_provider-tunnel tarpc implementation
#[tarpc::service]
pub trait SongbirdOrchestrator {
    /// Route capability request to appropriate primal
    async fn route_capability(
        request: CapabilityRequest
    ) -> Result<CapabilityResponse, OrchestrationError>;
    
    /// Establish secure tunnel to primal (Security Provider integration)
    async fn establish_tunnel(
        target_primal: PrimalType,
        tunnel_config: TunnelConfig
    ) -> Result<TunnelHandle, TunnelError>;
    
    /// Bidirectional streaming for real-time coordination
    async fn coordinate_stream(
        coordination_request: CoordinationRequest
    ) -> Result<CoordinationStream, StreamError>;
    
    /// Health check and service discovery
    async fn discover_services(
        query: ServiceQuery
    ) -> Result<Vec<ServiceInfo>, DiscoveryError>;
    
    /// Load balancing decision
    async fn select_best_provider(
        capability: String,
        requirements: ServiceRequirements
    ) -> Result<ServiceSelection, LoadBalancingError>;
}
```

#### **1.2 Custom JSON RPC Format (Universal Fallback)**
```rust
/// Universal RPC message format - for non-tarpc primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdRpcRequest {
    /// Unique message identifier
    pub id: uuid::Uuid,
    
    /// Target primal and capability
    pub target: PrimalCapabilityTarget,
    
    /// Request method (maps to tarpc service methods)
    pub method: String,
    
    /// JSON parameters
    pub params: serde_json::Value,
    
    /// Routing and performance hints
    pub hints: RoutingHints,
    
    /// Security context
    pub security: SecurityContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdRpcResponse {
    /// Request correlation ID
    pub id: uuid::Uuid,
    
    /// Success indicator
    pub success: bool,
    
    /// Response payload
    pub result: Option<serde_json::Value>,
    
    /// Error details (if failed)
    pub error: Option<RpcError>,
    
    /// Performance and routing metadata
    pub metadata: ResponseMetadata,
}
```

#### **1.2 Message Protocol Design**
```rust
/// Universal RPC message format - replacing HTTP requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcMessage {
    /// Unique message identifier
    pub id: MessageId,
    
    /// Message type (request, response, stream, event)
    pub message_type: MessageType,
    
    /// Source primal identification
    pub source: PrimalIdentifier,
    
    /// Target primal or capability
    pub target: RoutingTarget,
    
    /// Message payload with capability context
    pub payload: RpcPayload,
    
    /// Streaming context for bidirectional flows
    pub stream_context: Option<StreamContext>,
    
    /// Security context for authentication
    pub security_context: SecurityContext,
    
    /// Performance tracking metadata
    pub metadata: MessageMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Traditional request-response
    Request,
    Response,
    
    /// Bidirectional streaming
    StreamOpen,
    StreamData,
    StreamClose,
    
    /// Real-time events
    Event,
    Notification,
    
    /// System control
    Heartbeat,
    HealthCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingTarget {
    /// Specific primal by ID
    PrimalId(Uuid),
    
    /// Capability-based routing
    Capability(PrimalCapability),
    
    /// Load-balanced service group
    ServiceGroup(String),
    
    /// Broadcast to all primals
    Broadcast,
}
```

#### **1.2 Connection Management**
```rust
/// Bidirectional connection with multiple primals
pub struct PrimalConnection {
    /// Connection identifier
    pub id: ConnectionId,
    
    /// Remote primal information
    pub primal_info: PrimalInfo,
    
    /// Underlying transport (TCP, WebSocket, etc.)
    pub transport: Box<dyn Transport + Send + Sync>,
    
    /// Security layer for this connection
    pub security: Box<dyn SecurityLayer + Send + Sync>,
    
    /// Active streams on this connection
    pub streams: StreamRegistry,
    
    /// Connection health metrics
    pub health: ConnectionHealth,
    
    /// Message queue for outbound messages
    pub outbound_queue: MessageQueue,
}

pub trait Transport: Send + Sync {
    /// Send message to remote primal
    async fn send(&mut self, message: RpcMessage) -> Result<()>;
    
    /// Receive message from remote primal
    async fn receive(&mut self) -> Result<RpcMessage>;
    
    /// Open bidirectional stream
    async fn open_stream(&mut self, stream_id: StreamId) -> Result<RpcStream>;
    
    /// Health check for connection
    async fn health_check(&self) -> Result<TransportHealth>;
}
```

### **Phase 2: Bidirectional Streaming (Week 2-3)**

#### **2.1 Stream Management**
```rust
/// Bidirectional stream for real-time communication
pub struct RpcStream {
    /// Stream identifier
    pub id: StreamId,
    
    /// Stream direction capabilities
    pub direction: StreamDirection,
    
    /// Data sender (tokio channel)
    pub sender: UnboundedSender<StreamData>,
    
    /// Data receiver (tokio channel)  
    pub receiver: UnboundedReceiver<StreamData>,
    
    /// Stream health and metrics
    pub health: StreamHealth,
    
    /// Security context for this stream
    pub security: StreamSecurity,
}

#[derive(Debug, Clone)]
pub enum StreamDirection {
    /// Client-initiated stream
    ClientToServer,
    
    /// Server-initiated stream (true bidirectional)
    ServerToClient,
    
    /// Bidirectional data flow
    Bidirectional,
}

/// Stream data with zero-copy optimization
#[derive(Debug, Clone)]
pub struct StreamData {
    /// Stream identifier this data belongs to
    pub stream_id: StreamId,
    
    /// Zero-copy data payload
    pub data: Bytes,
    
    /// Data type and context
    pub data_type: StreamDataType,
    
    /// Timestamp for performance tracking
    pub timestamp: Instant,
}
```

#### **2.2 AI-First Integration**
```rust
/// AI-first communication patterns
pub struct AICollaborationStream {
    /// Human context stream
    pub human_context: RpcStream,
    
    /// AI response stream
    pub ai_response: RpcStream,
    
    /// Evidence and reasoning stream
    pub evidence_stream: RpcStream,
    
    /// Collaboration metadata
    pub collaboration_context: CollaborationContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationContext {
    /// Human participant information
    pub human_participant: HumanContext,
    
    /// AI agent information
    pub ai_agent: AIAgentContext,
    
    /// Collaboration mode
    pub mode: CollaborationMode,
    
    /// Real-time interaction state
    pub interaction_state: InteractionState,
}
```

### **Phase 3: Multi-Fallback System (Week 3-4)**

#### **3.1 Transport Negotiation**
```rust
/// Automatic transport negotiation and fallback
pub struct TransportNegotiator {
    /// Available transports in priority order
    pub available_transports: Vec<Box<dyn TransportFactory>>,
    
    /// Current active transport per connection
    pub active_transports: HashMap<ConnectionId, TransportType>,
    
    /// Fallback strategy configuration
    pub fallback_strategy: FallbackStrategy,
}

#[derive(Debug, Clone)]
pub enum TransportType {
    /// Primary: Native Rust RPC
    NativeRPC {
        compression: Option<CompressionType>,
        encryption: EncryptionType,
    },
    
    /// Fallback 1: WebSocket
    WebSocket {
        subprotocol: Option<String>,
        compression: bool,
    },
    
    /// Fallback 2: HTTP/2
    HTTP2 {
        keep_alive: bool,
        multiplexing: bool,
    },
    
    /// Emergency: HTTP/1.1
    HTTP1 {
        connection_pooling: bool,
    },
}

pub trait TransportFactory: Send + Sync {
    /// Attempt to create transport connection
    async fn create_transport(&self, target: &PrimalInfo) -> Result<Box<dyn Transport>>;
    
    /// Check if this transport can reach the target
    async fn can_connect(&self, target: &PrimalInfo) -> bool;
    
    /// Get transport priority (lower = higher priority)
    fn priority(&self) -> u8;
}
```

#### **3.2 Robust Connection Management**
```rust
/// Connection resilience and automatic recovery
pub struct ConnectionResilience {
    /// Connection health monitoring
    pub health_monitor: HealthMonitor,
    
    /// Automatic reconnection logic
    pub reconnection_manager: ReconnectionManager,
    
    /// Circuit breaker for failing connections
    pub circuit_breaker: CircuitBreaker,
    
    /// Load balancing across connections
    pub load_balancer: ConnectionLoadBalancer,
}

#[derive(Debug, Clone)]
pub struct FallbackStrategy {
    /// Maximum retry attempts per transport
    pub max_retries_per_transport: u32,
    
    /// Timeout before trying next transport
    pub transport_timeout: Duration,
    
    /// Whether to cache successful transport choices
    pub cache_successful_transports: bool,
    
    /// Fallback escalation policy
    pub escalation_policy: EscalationPolicy,
}
```

### **Phase 4: Ecosystem Integration (Week 4-5)**

#### **4.1 Universal Primal Protocol**
```rust
/// Standard protocol for all primal-to-primal communication
pub trait PrimalRPCProtocol {
    /// Capability advertisement
    async fn advertise_capabilities(&self) -> Result<Vec<PrimalCapability>>;
    
    /// Health check endpoint
    async fn health_check(&self) -> Result<PrimalHealth>;
    
    /// Handle universal RPC request
    async fn handle_rpc_request(&self, request: RpcMessage) -> Result<RpcMessage>;
    
    /// Open bidirectional stream
    async fn open_stream(&self, stream_request: StreamRequest) -> Result<RpcStream>;
    
    /// Handle system events
    async fn handle_system_event(&self, event: SystemEvent) -> Result<()>;
}

/// Integration with other primal RPC systems
pub struct PrimalRPCAdapter {
    /// Compute Provider RPC integration
    pub compute_provider_adapter: ComputeProviderRpcAdapter,
    
    /// Security Provider security integration
    pub security_provider_adapter: SecurityProviderRpcAdapter,
    
    /// AI provider MCP integration
    pub ai_provider_adapter: AiProviderMcpAdapter,
    
    /// Generic adapter for community primals
    pub generic_adapter: GenericRPCAdapter,
}
```

#### **4.2 Service Mesh Integration**
```rust
/// Service mesh capabilities for universal orchestration
pub struct ServiceMeshRPC {
    /// Service discovery via RPC
    pub discovery_rpc: ServiceDiscoveryRPC,
    
    /// Load balancing decisions via RPC
    pub load_balancer_rpc: LoadBalancerRPC,
    
    /// Health monitoring via RPC
    pub health_monitor_rpc: HealthMonitorRPC,
    
    /// Configuration distribution via RPC
    pub config_sync_rpc: ConfigSyncRPC,
}

/// Real-time service mesh events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceMeshEvent {
    /// New service registered
    ServiceRegistered {
        service_info: ServiceInfo,
        capabilities: Vec<PrimalCapability>,
    },
    
    /// Service health changed
    HealthChanged {
        service_id: Uuid,
        old_health: ServiceHealth,
        new_health: ServiceHealth,
    },
    
    /// Load balancing decision made
    LoadBalancingDecision {
        request_id: Uuid,
        selected_service: Uuid,
        routing_decision: RoutingDecision,
    },
    
    /// Configuration update
    ConfigurationUpdate {
        config_key: String,
        old_value: Option<serde_json::Value>,
        new_value: serde_json::Value,
    },
}
```

---

## 🚀 **Performance Targets**

### **Latency Targets**
- **Inter-primal RPC**: <1ms (Compute Provider compatibility)
- **Stream establishment**: <5ms
- **Message throughput**: 100K+ messages/second
- **Concurrent connections**: 10K+ connections

### **Memory Efficiency**
- **Zero-copy streaming**: 80% memory reduction vs HTTP
- **Connection pooling**: Reuse TCP connections
- **Message batching**: Reduce system call overhead

### **Reliability Targets**
- **Connection uptime**: 99.9%
- **Message delivery**: 99.99% (with retries)
- **Failover time**: <100ms
- **Recovery time**: <1s

---

## 🔒 **Security Integration**

### **Security Provider Integration**
```rust
/// Security layer integration with Security Provider
pub struct SecurityProviderSecurityLayer {
    /// Authentication provider
    pub auth_provider: Arc<dyn AuthenticationProvider>,
    
    /// Encryption provider
    pub encryption_provider: Arc<dyn EncryptionProvider>,
    
    /// Authorization policy engine
    pub authz_engine: Arc<dyn AuthorizationEngine>,
    
    /// Audit logging
    pub audit_logger: Arc<dyn AuditLogger>,
}

/// Security context for each RPC message
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// Authentication token
    pub auth_token: Option<AuthToken>,
    
    /// Encryption metadata
    pub encryption: EncryptionMetadata,
    
    /// Authorization permissions
    pub permissions: Vec<Permission>,
    
    /// Audit trail information
    pub audit_trail: AuditTrail,
}
```

---

## 📊 **Implementation Timeline**

| Phase | Duration | Deliverables | Dependencies |
|-------|----------|--------------|---------------|
| **Phase 1** | 2 weeks | Core RPC foundation | None |
| **Phase 2** | 1 week | Bidirectional streaming | Phase 1 |
| **Phase 3** | 1 week | Multi-fallback system | Phase 1 |
| **Phase 4** | 1 week | Ecosystem integration | All previous |
| **Testing** | 1 week | Integration testing | All phases |

**Total Timeline: 6 weeks to production**

---

## 🧪 **Testing Strategy**

### **Unit Testing**
- Message serialization/deserialization
- Connection management
- Stream lifecycle
- Security integration

### **Integration Testing**
- Real primal-to-primal communication
- Fallback transport negotiation
- Load balancing under stress
- Security policy enforcement

### **Performance Testing**
- Latency benchmarks vs HTTP/WebSocket
- Throughput testing with concurrent streams
- Memory usage profiling
- Connection pool efficiency

---

## 🎯 **Migration Strategy**

### **Phase 1: Parallel Implementation**
- Implement RPC alongside existing HTTP/WebSocket
- Feature flag for gradual rollout
- Maintain backward compatibility

### **Phase 2: Primal-by-Primal Migration**
- Start with Compute Provider (already has RPC)
- Move to AI provider (MCP integration)
- Integrate with Security Provider (security focus)
- Finally migrate Storage Provider

### **Phase 3: HTTP/WebSocket Deprecation**
- Mark old transport as deprecated
- Provide migration timeline
- Remove legacy code after ecosystem migration

---

## 🏆 **Success Criteria**

### **Technical Metrics**
- ✅ Sub-millisecond inter-primal latency
- ✅ 100K+ messages/second throughput
- ✅ 99.9% connection uptime
- ✅ <100ms failover time

### **Ecosystem Integration**
- ✅ All 6 primals using native RPC
- ✅ Community primal integration examples
- ✅ Zero-downtime migration completed
- ✅ Performance improvements measured

### **Developer Experience**
- ✅ Simple API for new primals
- ✅ Comprehensive documentation
- ✅ Testing framework provided
- ✅ Migration guides complete

---

## 📚 **References and Prior Art**

### **Compute Provider RPC System**
- **Status**: Production-ready bidirectional RPC
- **Lessons**: Zero-copy optimization, compression strategies
- **Integration**: Direct adapter for Compute Provider communication

### **Industry Standards**
- **Inspiration**: Apache Arrow Flight (columnar data)
- **Performance**: gRPC benchmarks for comparison
- **Security**: TLS 1.3 and modern cryptography

---

**Status**: Ready for Implementation  
**Next Steps**: Begin Phase 1 development with core RPC foundation  
**Success Measure**: Match Compute Provider RPC performance while adding universal orchestration capabilities 