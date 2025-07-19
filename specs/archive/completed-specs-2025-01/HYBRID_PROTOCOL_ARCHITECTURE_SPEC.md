---
description: ENFORCE hybrid protocol architecture with specialized protocols for different use cases
globs: ["songbird/src/**/*.rs", "songbird/crates/**/*.rs"]
---

# Hybrid Protocol Architecture Specification

## Context
- When designing multi-protocol orchestration with different protocols for different use cases
- When optimizing for both external client compatibility and internal service performance
- When extending existing MCP protocol capabilities for streaming
- When building reactive coordination systems

## Strategic Architecture Overview

### 🎯 **Hybrid Protocol Strategy**

**External Communication Layer (Client-Facing)**
- **WebSocket + JSON**: Web UIs, external services, language-agnostic clients
- **HTTP/REST**: Traditional API access, monitoring, health checks
- **Server-Sent Events**: Real-time updates to web clients

**Internal Communication Layer (Service-to-Service)**
- **tarpc**: Pure Rust, high-performance RPC for internal services
- **MCP Protocol Extensions**: Streaming capabilities, AI agent coordination
- **Event System**: Reactive coordination and cross-primal communication

**Protocol Selection Matrix**:
```
Use Case                    | Protocol Choice          | Rationale
---------------------------|-------------------------|---------------------------
Web UI ↔ Songbird          | WebSocket JSON          | Browser compatibility, real-time
External API ↔ Songbird     | HTTP/REST + JSON        | Universal compatibility
Songbird ↔ Squirrel         | tarpc                   | High performance, type safety
Songbird ↔ NestGate         | tarpc                   | Efficient data operations
Songbird ↔ BearDog          | tarpc                   | Secure, fast auth checks
Songbird ↔ Toadstool        | tarpc                   | Resource coordination
AI Agent Streaming          | MCP Protocol Extensions | Existing protocol, streaming
Cross-Primal Events         | Event System            | Reactive, pub/sub pattern
```

## Architecture Components

### External Communication Layer
```rust
pub struct ExternalCommunicationLayer {
    websocket_json_server: Arc<WebSocketJsonServer>,
    http_rest_server: Arc<HttpRestServer>,
    sse_broadcaster: Arc<ServerSentEventsBroadcaster>,
    client_session_manager: Arc<ClientSessionManager>,
}

impl ExternalCommunicationLayer {
    pub async fn new(config: ExternalCommConfig) -> Result<Self>;
    
    // WebSocket JSON endpoints
    pub async fn handle_websocket_connection(&self, socket: WebSocket) -> Result<()>;
    pub async fn broadcast_to_clients(&self, message: JsonMessage) -> Result<()>;
    pub async fn send_to_client(&self, client_id: &str, message: JsonMessage) -> Result<()>;
    
    // HTTP REST endpoints
    pub async fn handle_http_request(&self, request: HttpRequest) -> Result<HttpResponse>;
    pub async fn register_rest_endpoint(&self, path: &str, handler: RestHandler) -> Result<()>;
    
    // Server-Sent Events
    pub async fn create_sse_stream(&self, client_id: &str) -> Result<SseStream>;
    pub async fn push_sse_event(&self, event: SseEvent) -> Result<()>;
}
```

### Internal Communication Layer
```rust
pub struct InternalCommunicationLayer {
    tarpc_server: Arc<TarpcServer>,
    tarpc_client_pool: Arc<TarpcClientPool>,
    mcp_stream_manager: Arc<McpStreamManager>,
    service_discovery: Arc<ServiceDiscovery>,
}

impl InternalCommunicationLayer {
    pub async fn new(config: InternalCommConfig) -> Result<Self>;
    
    // tarpc service-to-service communication
    pub async fn call_service<T>(&self, service_id: &str, method: &str, params: T) -> Result<T>;
    pub async fn register_service_handler(&self, service: Arc<dyn TarpcService>) -> Result<()>;
    
    // MCP protocol extensions
    pub async fn create_mcp_stream(&self, agent_id: &str) -> Result<McpStream>;
    pub async fn handle_mcp_message(&self, stream_id: &str, message: McpMessage) -> Result<()>;
    
    // Service discovery integration
    pub async fn discover_service(&self, service_type: &str) -> Result<Vec<ServiceEndpoint>>;
}
```

### Protocol Translation Bridge
```rust
pub struct ProtocolBridge {
    external_layer: Arc<ExternalCommunicationLayer>,
    internal_layer: Arc<InternalCommunicationLayer>,
    translation_engine: Arc<TranslationEngine>,
    event_coordinator: Arc<EventCoordinator>,
}

impl ProtocolBridge {
    pub async fn new(config: BridgeConfig) -> Result<Self>;
    
    // External → Internal translation
    pub async fn translate_websocket_to_tarpc(&self, ws_message: JsonMessage) -> Result<TarpcCall>;
    pub async fn translate_http_to_tarpc(&self, http_req: HttpRequest) -> Result<TarpcCall>;
    
    // Internal → External translation  
    pub async fn translate_tarpc_to_websocket(&self, tarpc_response: TarpcResponse) -> Result<JsonMessage>;
    pub async fn translate_event_to_sse(&self, event: SystemEvent) -> Result<SseEvent>;
    
    // MCP stream bridging
    pub async fn bridge_mcp_to_websocket(&self, mcp_stream: McpStream, ws_client: &str) -> Result<()>;
}
```

## Protocol-Specific Implementations

### WebSocket JSON Server
```rust
pub struct WebSocketJsonServer {
    server: Arc<WebSocketServer>,
    client_registry: Arc<ClientRegistry>,
    message_validator: Arc<JsonMessageValidator>,
    rate_limiter: Arc<RateLimiter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonMessage {
    pub id: String,
    pub message_type: JsonMessageType,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub client_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JsonMessageType {
    // Client requests
    ServiceCall { service: String, method: String },
    Subscribe { topic: String, filter: Option<String> },
    Unsubscribe { subscription_id: String },
    
    // Server responses
    ServiceResponse { request_id: String, result: serde_json::Value },
    ServiceError { request_id: String, error: String },
    
    // Real-time updates
    Event { event_type: String, data: serde_json::Value },
    Notification { message: String, severity: String },
    
    // Connection management
    Ping,
    Pong,
    ConnectionAck,
}

impl WebSocketJsonServer {
    pub async fn handle_client_message(&self, client_id: &str, message: JsonMessage) -> Result<()>;
    pub async fn broadcast_message(&self, message: JsonMessage, filter: Option<ClientFilter>) -> Result<()>;
    pub async fn send_to_client(&self, client_id: &str, message: JsonMessage) -> Result<()>;
}
```

### tarpc Service Layer
```rust
// Define tarpc service interfaces for each primal
#[tarpc::service]
pub trait SquirrelService {
    async fn execute_mcp_command(agent_id: String, command: McpCommand) -> Result<McpResponse>;
    async fn create_agent_stream(agent_id: String, capabilities: Vec<String>) -> Result<StreamId>;
    async fn get_agent_status(agent_id: String) -> Result<AgentStatus>;
}

#[tarpc::service]
pub trait NestGateService {
    async fn provision_storage(request: StorageRequest) -> Result<StorageResponse>;
    async fn get_storage_metrics(volume_id: String) -> Result<StorageMetrics>;
    async fn stream_data(stream_id: String, data: Vec<u8>) -> Result<()>;
}

#[tarpc::service]
pub trait BearDogService {
    async fn authenticate_request(auth_token: String) -> Result<AuthResult>;
    async fn validate_security_policy(policy: SecurityPolicy) -> Result<ValidationResult>;
    async fn encrypt_data(data: Vec<u8>, key_id: String) -> Result<Vec<u8>>;
}

#[tarpc::service]
pub trait ToadstoolService {
    async fn deploy_workload(deployment: WorkloadDeployment) -> Result<DeploymentResult>;
    async fn get_resource_usage(resource_id: String) -> Result<ResourceMetrics>;
    async fn scale_service(service_id: String, scale_params: ScaleParams) -> Result<ScaleResult>;
}

// tarpc client pool for efficient connection management
pub struct TarpcClientPool {
    squirrel_clients: Arc<RwLock<Vec<SquirrelServiceClient>>>,
    nestgate_clients: Arc<RwLock<Vec<NestGateServiceClient>>>,
    beardog_clients: Arc<RwLock<Vec<BearDogServiceClient>>>,
    toadstool_clients: Arc<RwLock<Vec<ToadstoolServiceClient>>>,
    connection_manager: Arc<ConnectionManager>,
}
```

### MCP Protocol Extensions
```rust
pub struct McpStreamManager {
    active_streams: Arc<RwLock<HashMap<String, McpStream>>>,
    stream_registry: Arc<StreamRegistry>,
    message_router: Arc<MessageRouter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpStream {
    pub stream_id: String,
    pub agent_id: String,
    pub capabilities: Vec<String>,
    pub stream_type: McpStreamType,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpStreamType {
    // Bidirectional streaming
    AgentInteraction,
    ToolExecution,
    DataProcessing,
    
    // Unidirectional streaming
    LogStream,
    MetricsStream,
    EventStream,
    
    // Custom extensions
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMessage {
    pub message_id: String,
    pub stream_id: String,
    pub message_type: McpMessageType,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpMessageType {
    // Standard MCP messages
    Initialize,
    ToolCall { tool_name: String, parameters: serde_json::Value },
    ToolResult { call_id: String, result: serde_json::Value },
    
    // Streaming extensions
    StreamStart { stream_config: StreamConfig },
    StreamData { chunk: Vec<u8> },
    StreamEnd { reason: String },
    
    // Error handling
    Error { error_code: String, message: String },
}

impl McpStreamManager {
    pub async fn create_stream(&self, agent_id: &str, stream_type: McpStreamType) -> Result<McpStream>;
    pub async fn send_message(&self, stream_id: &str, message: McpMessage) -> Result<()>;
    pub async fn close_stream(&self, stream_id: &str) -> Result<()>;
}
```

### Event System Integration
```rust
pub struct ReactiveEventCoordinator {
    event_bus: Arc<EventBus>,
    protocol_bridge: Arc<ProtocolBridge>,
    subscription_manager: Arc<SubscriptionManager>,
    event_transformers: Arc<RwLock<HashMap<String, Arc<dyn EventTransformer>>>>,
}

impl ReactiveEventCoordinator {
    pub async fn new(config: EventCoordinatorConfig) -> Result<Self>;
    
    // Cross-primal event coordination
    pub async fn coordinate_primal_event(&self, event: PrimalEvent) -> Result<()>;
    pub async fn broadcast_system_event(&self, event: SystemEvent) -> Result<()>;
    
    // Protocol-specific event handling
    pub async fn handle_websocket_event(&self, event: WebSocketEvent) -> Result<()>;
    pub async fn handle_tarpc_event(&self, event: TarpcEvent) -> Result<()>;
    pub async fn handle_mcp_event(&self, event: McpEvent) -> Result<()>;
    
    // Reactive coordination patterns
    pub async fn create_event_workflow(&self, workflow: EventWorkflow) -> Result<WorkflowId>;
    pub async fn trigger_workflow(&self, workflow_id: WorkflowId, trigger_event: SystemEvent) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimalEvent {
    // Squirrel events
    AgentStarted { agent_id: String, capabilities: Vec<String> },
    AgentStopped { agent_id: String, reason: String },
    ToolExecuted { agent_id: String, tool_name: String, result: serde_json::Value },
    
    // NestGate events
    StorageProvisioned { volume_id: String, capacity: u64 },
    StorageReleased { volume_id: String },
    DataTransferCompleted { transfer_id: String, bytes_transferred: u64 },
    
    // BearDog events
    AuthenticationSuccess { user_id: String, session_id: String },
    AuthenticationFailure { attempt_id: String, reason: String },
    SecurityPolicyViolation { policy_id: String, violation_type: String },
    
    // Toadstool events
    WorkloadDeployed { deployment_id: String, resources: ResourceAllocation },
    WorkloadScaled { deployment_id: String, old_scale: u32, new_scale: u32 },
    ResourceExhausted { resource_type: String, usage_percent: f64 },
    
    // BiomeOS events
    BiomeCreated { biome_id: String, team_id: String },
    BiomeDestroyed { biome_id: String, reason: String },
    BiomeResourcesUpdated { biome_id: String, resources: ResourceAllocation },
}
```

## Implementation Strategy

### Phase 1: Foundation (Week 1-2)
1. **Protocol Layer Separation**
   - Implement external communication layer with WebSocket JSON
   - Set up internal communication layer with tarpc
   - Create protocol bridge for translation
   - Basic event system integration

2. **Core Service Interfaces**
   - Define tarpc service traits for each primal
   - Implement basic WebSocket JSON message handling
   - Create MCP protocol extension framework
   - Set up service discovery integration

### Phase 2: Enhanced Capabilities (Week 3-4)
1. **Streaming and Real-time**
   - Implement MCP streaming extensions
   - Add Server-Sent Events for web clients
   - Create reactive event coordination
   - Add connection lifecycle management

2. **Performance Optimization**
   - Implement tarpc connection pooling
   - Add WebSocket connection management
   - Optimize protocol translation paths
   - Add performance monitoring

### Phase 3: Advanced Features (Week 5-6)
1. **Cross-Primal Coordination**
   - Implement event-driven workflows
   - Add distributed coordination patterns
   - Create cross-primal transaction support
   - Add conflict resolution mechanisms

2. **BiomeOS Integration**
   - Add team-scoped protocol isolation
   - Implement resource quota enforcement
   - Create biome lifecycle coordination
   - Add multi-tenant security

### Phase 4: Production Readiness (Week 7-8)
1. **Reliability and Monitoring**
   - Add circuit breakers and failover
   - Implement comprehensive metrics
   - Add distributed tracing
   - Create alerting and monitoring

2. **Security and Compliance**
   - Implement end-to-end encryption
   - Add authentication and authorization
   - Create audit logging
   - Add security policy enforcement

## Configuration Examples

### Hybrid Protocol Configuration
```rust
pub struct HybridProtocolConfig {
    pub external_layer: ExternalCommConfig,
    pub internal_layer: InternalCommConfig,
    pub protocol_bridge: BridgeConfig,
    pub event_coordination: EventCoordinatorConfig,
}

pub struct ExternalCommConfig {
    pub websocket_json: WebSocketJsonConfig,
    pub http_rest: HttpRestConfig,
    pub sse: ServerSentEventsConfig,
}

pub struct InternalCommConfig {
    pub tarpc: TarpcConfig,
    pub mcp_extensions: McpExtensionsConfig,
    pub service_discovery: ServiceDiscoveryConfig,
}

// Example configuration
let config = HybridProtocolConfig {
    external_layer: ExternalCommConfig {
        websocket_json: WebSocketJsonConfig {
            bind_address: "0.0.0.0:8080".to_string(),
            max_connections: 10000,
            max_message_size: 1024 * 1024, // 1MB
            ping_interval: Duration::from_secs(30),
            json_validation: true,
            rate_limit: RateLimitConfig {
                requests_per_second: 100,
                burst_size: 200,
            },
        },
        http_rest: HttpRestConfig {
            bind_address: "0.0.0.0:8081".to_string(),
            max_request_size: 10 * 1024 * 1024, // 10MB
            request_timeout: Duration::from_secs(30),
            cors_enabled: true,
        },
        sse: ServerSentEventsConfig {
            bind_address: "0.0.0.0:8082".to_string(),
            max_connections: 5000,
            heartbeat_interval: Duration::from_secs(15),
        },
    },
    internal_layer: InternalCommConfig {
        tarpc: TarpcConfig {
            bind_address: "0.0.0.0:9090".to_string(),
            max_connections: 1000,
            connection_timeout: Duration::from_secs(10),
            max_message_size: 100 * 1024 * 1024, // 100MB for internal use
            compression_enabled: true,
        },
        mcp_extensions: McpExtensionsConfig {
            max_streams: 1000,
            stream_timeout: Duration::from_secs(300),
            max_stream_size: 1024 * 1024 * 1024, // 1GB
        },
        service_discovery: ServiceDiscoveryConfig {
            discovery_interval: Duration::from_secs(30),
            health_check_interval: Duration::from_secs(10),
        },
    },
    protocol_bridge: BridgeConfig {
        translation_timeout: Duration::from_secs(5),
        max_concurrent_translations: 10000,
        cache_translations: true,
    },
    event_coordination: EventCoordinatorConfig {
        max_event_queue_size: 100000,
        event_processing_timeout: Duration::from_secs(30),
        enable_event_persistence: true,
    },
};
```

## Performance Targets

### External Communication (WebSocket JSON)
- **Concurrent Connections**: 10K+ WebSocket connections
- **Message Throughput**: 50K JSON messages/second
- **Latency**: <50ms for simple operations
- **Memory Usage**: <1KB per connection

### Internal Communication (tarpc)
- **RPC Throughput**: 100K calls/second
- **Latency**: <1ms for local calls, <5ms for remote
- **Memory Usage**: <100MB for connection pools
- **Reliability**: 99.9% success rate

### MCP Streaming
- **Stream Throughput**: 1GB/second sustained
- **Concurrent Streams**: 1K+ active streams
- **Latency**: <10ms for stream data
- **Memory Usage**: <10MB per active stream

### Event Coordination
- **Event Throughput**: 1M events/second
- **Coordination Latency**: <100ms for cross-primal
- **Memory Usage**: <1GB for event buffers
- **Reliability**: 99.99% event delivery

## Security Considerations

### External Layer Security
- **Authentication**: JWT tokens, OAuth2 integration
- **Authorization**: Role-based access control
- **Rate Limiting**: Per-client and per-endpoint limits
- **Input Validation**: JSON schema validation

### Internal Layer Security
- **Mutual TLS**: All tarpc connections encrypted
- **Service Authentication**: Certificate-based auth
- **Message Signing**: Tamper-proof internal messages
- **Network Isolation**: Internal network segmentation

### Cross-Layer Security
- **Token Translation**: Secure token mapping
- **Audit Logging**: All protocol translations logged
- **Encryption**: End-to-end encryption where needed
- **Access Control**: Protocol-level permissions

## Integration Examples

### Web Client Integration
```javascript
// WebSocket JSON client example
const ws = new WebSocket('wss://songbird.example.com/ws');

ws.onopen = () => {
    // Subscribe to agent events
    ws.send(JSON.stringify({
        id: 'sub-1',
        message_type: 'Subscribe',
        payload: {
            topic: 'agent_events',
            filter: { agent_type: 'data_analyst' }
        }
    }));
};

ws.onmessage = (event) => {
    const message = JSON.parse(event.data);
    switch (message.message_type) {
        case 'Event':
            console.log('Agent event:', message.payload);
            break;
        case 'ServiceResponse':
            console.log('Service response:', message.payload);
            break;
    }
};

// Call a service
function callSquirrelService(method, params) {
    const requestId = 'req-' + Date.now();
    ws.send(JSON.stringify({
        id: requestId,
        message_type: 'ServiceCall',
        payload: {
            service: 'squirrel',
            method: method,
            params: params
        }
    }));
}
```

### Internal Service Integration
```rust
// tarpc service implementation
#[derive(Clone)]
pub struct SongbirdSquirrelService {
    agent_manager: Arc<AgentManager>,
    mcp_stream_manager: Arc<McpStreamManager>,
}

#[tarpc::server]
impl SquirrelService for SongbirdSquirrelService {
    async fn execute_mcp_command(
        self,
        _: tarpc::context::Context,
        agent_id: String,
        command: McpCommand,
    ) -> Result<McpResponse> {
        let agent = self.agent_manager.get_agent(&agent_id).await?;
        let response = agent.execute_command(command).await?;
        Ok(response)
    }
    
    async fn create_agent_stream(
        self,
        _: tarpc::context::Context,
        agent_id: String,
        capabilities: Vec<String>,
    ) -> Result<StreamId> {
        let stream = self.mcp_stream_manager.create_stream(
            &agent_id,
            McpStreamType::AgentInteraction,
        ).await?;
        Ok(stream.stream_id)
    }
}
```

## Version History

- v1.0.0: Initial hybrid protocol specification
- v1.1.0: Added MCP streaming extensions
- v1.2.0: Enhanced event coordination
- v1.3.0: BiomeOS integration requirements

<version>1.3.0</version> 