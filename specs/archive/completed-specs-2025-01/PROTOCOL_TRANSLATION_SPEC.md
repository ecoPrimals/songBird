---
description: ENFORCE universal protocol translation for seamless multi-protocol communication
globs: ["songbird/src/**/*.rs", "songbird/crates/**/*.rs"]
---

# Protocol Translation Specification

## Context
- When implementing universal protocol translation for multi-protocol orchestration
- When enabling seamless communication between different protocol types
- When bridging legacy protocols with modern communication standards
- When optimizing protocol selection based on context and performance

## Requirements

### Universal Protocol Support
- Support for HTTP/REST, WebSocket JSON, tarpc, Server-Sent Events, and TCP/UDP
- Bidirectional protocol translation with message fidelity preservation
- Dynamic protocol negotiation based on capabilities and preferences
- Protocol-specific optimization and feature mapping

### Message Translation Engine
- Lossless message translation between protocol formats
- Message schema validation and transformation
- Support for streaming and batch message processing
- Error handling and recovery across protocol boundaries

### Performance Optimization
- Protocol selection optimization based on latency and throughput requirements
- Connection pooling and reuse across protocol types
- Efficient message serialization and deserialization
- Memory-efficient streaming for large messages

### Security and Authentication
- Security context preservation across protocol translations
- Authentication token translation and validation
- Encryption and decryption handling for secure protocols
- Access control enforcement at protocol boundaries

## Architecture

### Core Protocol Router
```rust
pub struct ProtocolRouter {
    translators: Arc<RwLock<HashMap<ProtocolPair, Arc<dyn ProtocolTranslator>>>>,
    protocol_registry: Arc<ProtocolRegistry>,
    optimization_engine: Arc<OptimizationEngine>,
    security_manager: Arc<SecurityManager>,
    metrics_collector: Arc<MetricsCollector>,
}

impl ProtocolRouter {
    pub async fn new(config: ProtocolRouterConfig) -> Result<Self>;
    
    // Core translation operations
    pub async fn translate_request(&self, request: UniversalRequest) -> Result<UniversalResponse>;
    pub async fn translate_stream(&self, stream: ProtocolStream) -> Result<ProtocolStream>;
    pub async fn negotiate_protocol(&self, client_prefs: &[ProtocolType], server_caps: &[ProtocolType]) -> Result<ProtocolType>;
    
    // Protocol management
    pub async fn register_translator(&self, translator: Arc<dyn ProtocolTranslator>) -> Result<()>;
    pub async fn get_supported_protocols(&self) -> Result<Vec<ProtocolType>>;
    pub async fn get_translation_capabilities(&self, from: ProtocolType, to: ProtocolType) -> Result<TranslationCapabilities>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProtocolPair {
    pub from: ProtocolType,
    pub to: ProtocolType,
}
```

### Universal Message Format
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalRequest {
    pub id: String,
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: MessageBody,
    pub metadata: MessageMetadata,
    pub security_context: Option<SecurityContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalResponse {
    pub id: String,
    pub status: ResponseStatus,
    pub headers: HashMap<String, String>,
    pub body: MessageBody,
    pub metadata: MessageMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageBody {
    Empty,
    Text(String),
    Binary(Vec<u8>),
    Json(serde_json::Value),
    Protobuf(Vec<u8>),
    Stream(StreamHandle),
    Multipart(Vec<MessagePart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub timestamp: DateTime<Utc>,
    pub source_protocol: ProtocolType,
    pub target_protocol: ProtocolType,
    pub content_type: Option<String>,
    pub content_length: Option<usize>,
    pub encoding: Option<String>,
    pub compression: Option<CompressionType>,
    pub custom_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseStatus {
    Success(u16),
    Error(u16),
    Redirect(u16),
    Custom(String),
}
```

### Protocol Translator Interface
```rust
#[async_trait]
pub trait ProtocolTranslator: Send + Sync {
    async fn translate_request(&self, request: UniversalRequest) -> Result<UniversalRequest>;
    async fn translate_response(&self, response: UniversalResponse) -> Result<UniversalResponse>;
    async fn translate_stream(&self, stream: ProtocolStream) -> Result<ProtocolStream>;
    
    fn source_protocol(&self) -> ProtocolType;
    fn target_protocol(&self) -> ProtocolType;
    fn translation_capabilities(&self) -> TranslationCapabilities;
    fn supports_streaming(&self) -> bool;
    fn supports_bidirectional(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct TranslationCapabilities {
    pub supports_headers: bool,
    pub supports_metadata: bool,
    pub supports_streaming: bool,
    pub supports_binary: bool,
    pub supports_compression: bool,
    pub supports_security_context: bool,
    pub max_message_size: Option<usize>,
    pub supported_content_types: Vec<String>,
}
```

### Specific Protocol Translators
```rust
// HTTP to WebSocket Translator
pub struct HttpToWebSocketTranslator {
    websocket_manager: Arc<WebSocketManager>,
    connection_pool: Arc<ConnectionPool>,
    upgrade_handler: Arc<UpgradeHandler>,
}

// WebSocket JSON to tarpc Translator
pub struct WebSocketJsonToTarpcTranslator {
    tarpc_client_pool: Arc<TarpcClientPool>,
    message_mapper: Arc<MessageMapper>,
    stream_manager: Arc<StreamManager>,
}

// tarpc to HTTP Translator
pub struct TarpcToHttpTranslator {
    http_client: Arc<HttpClient>,
    serde_mapper: Arc<SerdeMapper>,
    response_converter: Arc<ResponseConverter>,
}

// Server-Sent Events to WebSocket Translator
pub struct SseToWebSocketTranslator {
    event_stream_manager: Arc<EventStreamManager>,
    websocket_broadcaster: Arc<WebSocketBroadcaster>,
}

// TCP to HTTP Translator
pub struct TcpToHttpTranslator {
    tcp_listener: Arc<TcpListener>,
    http_adapter: Arc<HttpAdapter>,
    protocol_detector: Arc<ProtocolDetector>,
}

// UDP to tarpc Translator
pub struct UdpToTarpcTranslator {
    udp_socket: Arc<UdpSocket>,
    tarpc_bridge: Arc<TarpcBridge>,
    message_framer: Arc<MessageFramer>,
}
```

### Protocol Registry
```rust
pub struct ProtocolRegistry {
    protocols: Arc<RwLock<HashMap<ProtocolType, ProtocolInfo>>>,
    handlers: Arc<RwLock<HashMap<ProtocolType, Arc<dyn ProtocolHandler>>>>,
    capabilities: Arc<RwLock<HashMap<ProtocolType, ProtocolCapabilities>>>,
}

#[derive(Debug, Clone)]
pub struct ProtocolInfo {
    pub protocol_type: ProtocolType,
    pub name: String,
    pub version: String,
    pub description: String,
    pub default_port: Option<u16>,
    pub security_supported: bool,
    pub streaming_supported: bool,
    pub bidirectional_supported: bool,
}

#[derive(Debug, Clone)]
pub struct ProtocolCapabilities {
    pub supported_content_types: Vec<String>,
    pub supported_methods: Vec<String>,
    pub supported_headers: Vec<String>,
    pub max_message_size: Option<usize>,
    pub compression_types: Vec<CompressionType>,
    pub security_types: Vec<SecurityType>,
}

#[async_trait]
pub trait ProtocolHandler: Send + Sync {
    async fn handle_request(&self, request: UniversalRequest) -> Result<UniversalResponse>;
    async fn handle_stream(&self, stream: ProtocolStream) -> Result<ProtocolStream>;
    async fn validate_message(&self, message: &UniversalRequest) -> Result<ValidationResult>;
    
    fn protocol_type(&self) -> ProtocolType;
    fn capabilities(&self) -> ProtocolCapabilities;
}
```

### Optimization Engine
```rust
pub struct OptimizationEngine {
    performance_analyzer: Arc<PerformanceAnalyzer>,
    protocol_selector: Arc<ProtocolSelector>,
    routing_optimizer: Arc<RoutingOptimizer>,
    cache_manager: Arc<CacheManager>,
}

impl OptimizationEngine {
    pub async fn select_optimal_protocol(&self, request: &UniversalRequest, available_protocols: &[ProtocolType]) -> Result<ProtocolType>;
    pub async fn optimize_translation_path(&self, from: ProtocolType, to: ProtocolType) -> Result<Vec<ProtocolType>>;
    pub async fn get_performance_metrics(&self, protocol: ProtocolType) -> Result<PerformanceMetrics>;
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub latency: Duration,
    pub throughput: f64,
    pub error_rate: f64,
    pub resource_usage: ResourceUsage,
    pub connection_count: u32,
}

#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub cpu_usage: f64,
    pub memory_usage: usize,
    pub network_bandwidth: f64,
}
```

### Security Manager
```rust
pub struct SecurityManager {
    auth_providers: Arc<RwLock<HashMap<ProtocolType, Arc<dyn AuthProvider>>>>,
    encryption_handlers: Arc<RwLock<HashMap<ProtocolType, Arc<dyn EncryptionHandler>>>>,
    policy_enforcer: Arc<PolicyEnforcer>,
}

#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn authenticate(&self, request: &UniversalRequest) -> Result<SecurityContext>;
    async fn validate_token(&self, token: &str) -> Result<TokenValidation>;
    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenPair>;
    
    fn supported_auth_types(&self) -> Vec<AuthType>;
    fn protocol_type(&self) -> ProtocolType;
}

#[async_trait]
pub trait EncryptionHandler: Send + Sync {
    async fn encrypt_message(&self, message: &UniversalRequest) -> Result<UniversalRequest>;
    async fn decrypt_message(&self, message: &UniversalRequest) -> Result<UniversalRequest>;
    async fn negotiate_encryption(&self, client_caps: &[EncryptionType]) -> Result<EncryptionType>;
    
    fn supported_encryption_types(&self) -> Vec<EncryptionType>;
    fn protocol_type(&self) -> ProtocolType;
}
```

## Implementation Tasks

### Phase 1: Core Translation Framework (Week 1-2)
1. **Universal Message Format**
   - Define universal request/response structures
   - Implement message serialization/deserialization
   - Create message validation framework
   - Add message metadata and context

2. **Protocol Registry**
   - Protocol information management
   - Protocol capability registration
   - Protocol handler framework
   - Basic protocol detection

### Phase 2: Basic Translators (Week 3-4)
1. **HTTP/WebSocket Translation**
   - HTTP to WebSocket translator
   - WebSocket to HTTP translator
   - Connection upgrade handling
   - Message format conversion

2. **gRPC Integration**
   - gRPC to HTTP translator
   - HTTP to gRPC translator
   - Protobuf message handling
   - Streaming support

### Phase 3: Advanced Protocols (Week 5-6)
1. **Server-Sent Events**
   - SSE to WebSocket translator
   - Event stream management
   - Real-time event bridging
   - Connection lifecycle management

2. **TCP/UDP Support**
   - TCP to HTTP translator
   - UDP to gRPC translator
   - Raw socket handling
   - Protocol detection and framing

### Phase 4: Optimization and Security (Week 7-8)
1. **Performance Optimization**
   - Protocol selection optimization
   - Connection pooling
   - Message caching
   - Compression handling

2. **Security Integration**
   - Authentication translation
   - Encryption handling
   - Security context preservation
   - Access control enforcement

## Configuration

### Protocol Router Configuration
```rust
pub struct ProtocolRouterConfig {
    pub enabled_protocols: Vec<ProtocolType>,
    pub translation_rules: Vec<TranslationRule>,
    pub optimization_config: OptimizationConfig,
    pub security_config: SecurityConfig,
    pub performance_config: PerformanceConfig,
}

#[derive(Debug, Clone)]
pub struct TranslationRule {
    pub source_protocol: ProtocolType,
    pub target_protocol: ProtocolType,
    pub conditions: Vec<TranslationCondition>,
    pub transformations: Vec<MessageTransformation>,
    pub priority: u32,
}

#[derive(Debug, Clone)]
pub enum TranslationCondition {
    PathPattern(String),
    HeaderPresent(String),
    HeaderValue(String, String),
    ContentType(String),
    MessageSize(usize),
    Custom(String),
}

pub struct OptimizationConfig {
    pub enable_protocol_selection: bool,
    pub enable_connection_pooling: bool,
    pub enable_message_caching: bool,
    pub enable_compression: bool,
    pub performance_monitoring: bool,
}
```

### Protocol-Specific Configuration
```rust
pub struct HttpConfig {
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub request_timeout: Duration,
    pub max_request_size: usize,
    pub compression_enabled: bool,
}

pub struct WebSocketConfig {
    pub max_connections: usize,
    pub ping_interval: Duration,
    pub close_timeout: Duration,
    pub max_message_size: usize,
    pub compression_enabled: bool,
}

pub struct TarpcConfig {
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub max_message_size: usize,
    pub compression_enabled: bool,
    pub tls_config: Option<TlsConfig>,
}

pub struct WebSocketJsonConfig {
    pub max_connections: usize,
    pub ping_interval: Duration,
    pub close_timeout: Duration,
    pub max_message_size: usize,
    pub compression_enabled: bool,
    pub json_validation: bool,
}
```

## Integration Points

### BiomeOS Integration
- Protocol translation for biome services
- Team-scoped protocol policies
- Resource usage tracking per protocol
- Performance analytics and optimization

### Primal Integration
- **Squirrel**: MCP protocol translation and AI agent communication
- **NestGate**: Storage protocol optimization and data transfer
- **BearDog**: Security policy enforcement and encrypted communication
- **ToadStool**: Runtime protocol adaptation and container communication

### External Integration
- API gateway integration
- Service mesh integration
- Load balancer integration
- Monitoring and observability

## Performance Requirements

### Latency Targets
- Protocol translation: < 5ms overhead
- Message serialization: < 1ms per message
- Connection establishment: < 100ms
- Stream processing: < 10ms per chunk

### Throughput Targets
- HTTP translation: 10K requests/second
- WebSocket translation: 100K messages/second
- WebSocket JSON translation: 50K messages/second
- tarpc translation: 5K RPC calls/second
- Streaming: 1GB/second sustained

### Resource Utilization
- Memory usage: < 100MB per active translation
- CPU usage: < 5% per active translation
- Network bandwidth: 95% efficiency
- Connection overhead: < 1KB per connection

## Security Considerations

### Authentication Translation
- Token format conversion
- Authentication context preservation
- Multi-protocol authentication support
- Session management across protocols

### Encryption Handling
- TLS termination and resumption
- End-to-end encryption preservation
- Protocol-specific encryption
- Key management and rotation

### Access Control
- Protocol-level access control
- Message-level authorization
- Rate limiting per protocol
- Audit logging for security events

## Testing Strategy

### Unit Testing
- Message translation accuracy
- Protocol handler functionality
- Security context preservation
- Performance optimization logic

### Integration Testing
- End-to-end protocol translation
- Cross-protocol communication
- Security policy enforcement
- Performance under load

### Performance Testing
- Translation latency benchmarks
- Throughput optimization
- Memory usage efficiency
- Scalability testing

## Examples

### Basic Protocol Translation
```rust
let config = ProtocolRouterConfig {
    enabled_protocols: vec![
        ProtocolType::HTTP,
        ProtocolType::WebSocket,
        ProtocolType::WebSocketJson,
        ProtocolType::Tarpc,
    ],
    translation_rules: vec![
        TranslationRule {
            source_protocol: ProtocolType::HTTP,
            target_protocol: ProtocolType::WebSocket,
            conditions: vec![
                TranslationCondition::HeaderPresent("Upgrade".to_string()),
                TranslationCondition::HeaderValue("Upgrade".to_string(), "websocket".to_string()),
            ],
            transformations: vec![
                MessageTransformation::WebSocketUpgrade,
            ],
            priority: 1,
        },
    ],
    // ... other configs
};

let router = ProtocolRouter::new(config).await?;
```

### Request Translation
```rust
let http_request = UniversalRequest {
    id: "req-123".to_string(),
    method: "GET".to_string(),
    path: "/api/v1/users".to_string(),
    headers: HashMap::from([
        ("Content-Type".to_string(), "application/json".to_string()),
        ("Authorization".to_string(), "Bearer token123".to_string()),
    ]),
    body: MessageBody::Empty,
    metadata: MessageMetadata {
        timestamp: Utc::now(),
        source_protocol: ProtocolType::HTTP,
        target_protocol: ProtocolType::Tarpc,
        content_type: Some("application/json".to_string()),
        content_length: Some(0),
        encoding: None,
        compression: None,
        custom_fields: HashMap::new(),
    },
    security_context: Some(SecurityContext {
        user_id: "user123".to_string(),
        roles: vec!["user".to_string()],
        permissions: vec!["read_users".to_string()],
        token: Some("token123".to_string()),
    }),
};

let tarpc_request = router.translate_request(http_request).await?;
```

### Custom Protocol Translator
```rust
pub struct CustomProtocolTranslator {
    source_protocol: ProtocolType,
    target_protocol: ProtocolType,
    message_mapper: Arc<MessageMapper>,
}

#[async_trait]
impl ProtocolTranslator for CustomProtocolTranslator {
    async fn translate_request(&self, request: UniversalRequest) -> Result<UniversalRequest> {
        let mut translated = request.clone();
        
        // Custom translation logic
        match (self.source_protocol, self.target_protocol) {
            (ProtocolType::HTTP, ProtocolType::Custom(ref name)) => {
                // Convert HTTP headers to custom format
                translated.headers = self.transform_headers(&request.headers)?;
                
                // Transform message body
                translated.body = self.transform_body(&request.body)?;
                
                // Update metadata
                translated.metadata.target_protocol = self.target_protocol;
            }
            _ => return Err(Error::UnsupportedTranslation),
        }
        
        Ok(translated)
    }
    
    async fn translate_response(&self, response: UniversalResponse) -> Result<UniversalResponse> {
        // Response translation logic
        Ok(response)
    }
    
    async fn translate_stream(&self, stream: ProtocolStream) -> Result<ProtocolStream> {
        // Stream translation logic
        Ok(stream)
    }
    
    fn source_protocol(&self) -> ProtocolType {
        self.source_protocol
    }
    
    fn target_protocol(&self) -> ProtocolType {
        self.target_protocol.clone()
    }
    
    fn translation_capabilities(&self) -> TranslationCapabilities {
        TranslationCapabilities {
            supports_headers: true,
            supports_metadata: true,
            supports_streaming: false,
            supports_binary: true,
            supports_compression: false,
            supports_security_context: true,
            max_message_size: Some(10 * 1024 * 1024), // 10MB
            supported_content_types: vec![
                "application/json".to_string(),
                "application/xml".to_string(),
            ],
        }
    }
    
    fn supports_streaming(&self) -> bool {
        false
    }
    
    fn supports_bidirectional(&self) -> bool {
        true
    }
}
```

### Protocol Optimization
```rust
// Select optimal protocol based on request characteristics
let request = UniversalRequest {
    // ... request details
};

let available_protocols = vec![
    ProtocolType::HTTP,
    ProtocolType::WebSocket,
    ProtocolType::GRPC,
];

let optimal_protocol = router.optimization_engine
    .select_optimal_protocol(&request, &available_protocols)
    .await?;

println!("Selected protocol: {:?}", optimal_protocol);

// Get performance metrics
let metrics = router.optimization_engine
    .get_performance_metrics(optimal_protocol)
    .await?;

println!("Protocol performance: latency={:?}, throughput={:.2}/s", 
         metrics.latency, metrics.throughput);
```

## Monitoring and Observability

### Translation Metrics
- Translation latency and throughput
- Protocol selection effectiveness
- Message transformation accuracy
- Error rates per protocol pair

### Performance Metrics
- Connection pool utilization
- Memory usage per protocol
- CPU usage during translation
- Network bandwidth efficiency

### Security Metrics
- Authentication success rates
- Encryption/decryption performance
- Security policy violations
- Access control effectiveness

### Alerting Rules
- High translation latency
- Translation failures
- Security violations
- Resource exhaustion

## Version History

- v1.0.0: Initial specification
- v1.1.0: Added streaming support
- v1.2.0: Enhanced security integration
- v1.3.0: BiomeOS integration requirements

<version>1.3.0</version> 