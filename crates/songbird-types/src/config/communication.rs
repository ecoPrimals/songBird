//! Communication Configuration - Canonical Types Types
//!
//! This module consolidates all communication-related configuration structures
//! that were previously scattered across songbird-network crate.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// **CANONICAL**: Communication Configuration - Single Source of Truth
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CanonicalCommunicationConfig {
    /// HTTP client configuration
    pub http: HttpClientConfig,
    /// WebSocket configuration
    pub websocket: WebSocketConfig,
    /// gRPC configuration
    pub grpc: GrpcConfig,
    /// JSON-RPC configuration
    /// Jsonrpc field
    pub jsonrpc: JsonRpcConfig,
    /// Performance configuration
    /// Performance field
    pub performance: PerformanceConfig,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
}

/// HTTP client configuration - consolidates `HttpClientConfig`s
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpClientConfig {
    /// Connection timeout
    pub timeout: Duration,
    /// Maximum number of connections per host
    pub max_connections_per_host: usize,
    /// Enable HTTP/2
    pub http2_enabled: bool,
    /// User agent string
    pub user_agent: String,
    /// Default headers
    pub default_headers: HashMap<String, String>,
    /// Enable compression
    pub compression_enabled: bool,
    /// Connection keep-alive timeout
    pub keep_alive_timeout: Duration,
    /// Maximum redirects to follow
    pub max_redirects: usize,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_connections_per_host: 10,
            http2_enabled: true,
            user_agent: "songbird/1.0".to_string(),
            default_headers: HashMap::new()),
            compression_enabled: true,
            keep_alive_timeout: Duration::from_secs(60),
            max_redirects: 10,
        }
    }
}

/// gRPC configuration - consolidates `GrpcConfig`s
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcConfig {
    /// Connection timeout
    pub timeout: Duration,
    /// Keep-alive interval
    /// Keep Alive Interval field
    pub keep_alive_interval: Duration,
    /// Keep-alive timeout
    pub keep_alive_timeout: Duration,
    /// Maximum message size
    pub max_message_size: usize,
    /// Enable /// TLS
    /// Tls Enabled field
    pub tls_enabled: bool,
    /// TLS configuration
    /// Tls Config field
    pub tls_config: Option<TlsConfig>,
    /// Compression algorithm
    /// Compression field
    pub compression: Option<String>,
    /// Maximum concurrent streams
    /// Max Concurrent Streams field
    pub max_concurrent_streams: u32,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            keep_alive_interval: Duration::from_secs(30),
            keep_alive_timeout: Duration::from_secs(5),
            max_message_size: 4 * 1024 * 1024, // 4MB
            tls_enabled: true,
            tls_config: None,
            compression: Some("gzip".to_string()),
            max_concurrent_streams: 100,
        }
    }
}

/// TLS configuration for secure connections
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TlsConfig {
    /// Certificate file path
    pub cert_file: Option<String>,
    /// Private key file path
    pub key_file: Option<String>,
    /// CA certificate file path
    pub ca_file: Option<String>,
    /// Verify peer certificates
    #[serde(default)]
    /// Verify Peer field
    pub verify_peer: bool,
    /// Server name for
    pub server_name: Option<String>,
}

/// WebSocket configuration - consolidates `WebSocketConfig`s
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// Connection timeout
    pub timeout: Duration,
    /// Maximum frame size
    pub max_frame_size: usize,
    /// Maximum message size
    pub max_message_size: usize,
    /// Ping interval for keep-alive
    /// Ping Interval field
    pub ping_interval: Duration,
    /// Pong timeout
    /// Pong Timeout field
    pub pong_timeout: Duration,
    /// Enable compression
    /// Compression Enabled field
    pub compression_enabled: bool,
    /// Subprotocols to support
    pub subprotocols: Vec<String>,
    /// Custom headers
    pub headers: HashMap<String, String>,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_frame_size: 64 * 1024,     // 64KB
            max_message_size: 1024 * 1024, // 1MB
            ping_interval: Duration::from_secs(30),
            pong_timeout: Duration::from_secs(10),
            compression_enabled: true,
            subprotocols: Vec::new(),
            headers: HashMap::new()),
        }
    }
}

/// Tarpc configuration for high-performance RPC - consolidates `TarpcConfig`s
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TarpcConfig {
    /// Connection timeout
    pub timeout: Duration,
    /// Maximum concurrent requests
    /// Max Concurrent Requests field
    pub max_concurrent_requests: usize,
    /// Request timeout
    pub request_timeout: Duration,
    /// Enable compression
    /// Compression Enabled field
    pub compression_enabled: bool,
    /// Transport configuration
    /// Transport field
    pub transport: TarpcTransportConfig,
    /// Serialization format
    /// Serialization field
    pub serialization: SerializationFormat,
}

impl Default for TarpcConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_concurrent_requests: 100,
            request_timeout: Duration::from_secs(60),
            compression_enabled: true,
            transport: TarpcTransportConfig::default(),
            serialization: SerializationFormat::Bincode,
        }
    }
}

/// Tarpc transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TarpcTransportConfig {
    /// Transport type
    pub transport_type: TransportType,
    /// Buffer size
    pub buffer_size: usize,
    /// Enable TCP nodelay
    pub tcp_nodelay: bool,
    /// Socket keep-alive
    pub keep_alive: bool,
}

impl Default for TarpcTransportConfig {
    fn default() -> Self {
        Self {
            transport_type: TransportType::Tcp,
            buffer_size: 8192,
            tcp_nodelay: true,
            keep_alive: true,
        }
    }
}

/// Transport type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportType {
    /// TCP transport
    Tcp,
    /// Unix domain socket
    Unix,
    /// In-process transport
    InProcess,
}

/// Serialization format for /// Tarpc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializationFormat {
    /// Bincode serialization (fastest)
    Bincode,
    /// JSON serialization (human-readable)
    Json,
    /// `MessagePack` serialization (compact)
    MessagePack,
}

/// JSON-RPC configuration - consolidates `JsonRpcConfig`s
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcConfig {
    /// Request timeout
    pub timeout: Duration,
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Enable batch requests
    /// Batch Enabled field
    pub batch_enabled: bool,
    /// JSON-RPC version
    /// Version string
    pub version: String,
    /// Enable notification support
    /// Notifications Enabled field
    pub notifications_enabled: bool,
    /// Custom method prefix
    pub method_prefix: Option<String>,
}

impl Default for JsonRpcConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_batch_size: 100,
            batch_enabled: true,
            version: "2.0".to_string(),
            notifications_enabled: true,
            method_prefix: None,
        }
    }
}

/// Hyper client configuration - consolidates `HyperClientConfig`s
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperClientConfig {
    /// Connection timeout
    pub timeout: Duration,
    /// Pool idle timeout
    pub pool_idle_timeout: Duration,
    /// Maximum idle connections per host
    pub pool_max_idle_per_host: usize,
    /// HTTP version preference
    pub http_version: HttpVersion,
    /// Enable HTTP/2 prior knowledge
    pub http2_prior_knowledge: bool,
    /// Maximum header list size
    pub max_header_list_size: usize,
    /// Enable automatic decompression
    pub auto_decompress: bool,
}

impl Default for HyperClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            pool_idle_timeout: Duration::from_secs(90),
            pool_max_idle_per_host: 10,
            http_version: HttpVersion::Http2,
            http2_prior_knowledge: false,
            max_header_list_size: 16384,
            auto_decompress: true,
        }
    }
}

/// HTTP version preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpVersion {
    /// HTTP/1.1
    Http1,
    /// HTTP/2
    Http2,
    /// HTTP/3 (QUIC)
    Http3,
}

/// Circuit breaker configuration - consolidates `CircuitBreakerConfig`s
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit
    pub failure_threshold: usize,
    /// Success threshold to close circuit
    pub success_threshold: usize,
    /// Timeout duration in open state
    pub timeout: Duration,
    /// Half-open state timeout
    pub half_open_timeout: Duration,
    /// Enable circuit breaker
    pub enabled: bool,
    /// Reset timeout multiplier
    pub reset_timeout_multiplier: f64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(30),
            half_open_timeout: Duration::from_secs(10),
            enabled: true,
            reset_timeout_multiplier: 2.0,
        }
    }
}

/// Performance configuration - consolidates `PerformanceConfig`s
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable performance optimizations
    /// Enabled field
    pub enabled: bool,
    /// Connection pooling settings
    /// Connection Pooling field
    pub connection_pooling: ConnectionPoolingConfig,
    /// Request batching settings
    /// Request Batching field
    pub request_batching: RequestBatchingConfig,
    /// Caching configuration
    /// Caching field
    pub caching: CachingConfig,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            connection_pooling: ConnectionPoolingConfig::default(),
            request_batching: RequestBatchingConfig::default(),
            caching: CachingConfig::default(),
        }
    }
}

/// Connection pooling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolingConfig {
    /// Enable connection pooling
    /// Enabled field
    pub enabled: bool,
    /// Maximum pool size
    pub max_pool_size: usize,
    /// Minimum pool size
    /// Min Pool Size field
    pub min_pool_size: usize,
    /// Connection idle timeout
    pub idle_timeout: Duration,
}

impl Default for ConnectionPoolingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_pool_size: 50,
            min_pool_size: 5,
            idle_timeout: Duration::from_secs(300),
        }
    }
}

/// Request batching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBatchingConfig {
    /// Enable request batching
    pub enabled: bool,
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Batch timeout
    pub batch_timeout: Duration,
}

impl Default for RequestBatchingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_batch_size: 10,
            batch_timeout: Duration::from_millis(100),
        }
    }
}

/// Caching configuration for communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachingConfig {
    /// Enable response caching
    /// Enabled field
    pub enabled: bool,
    /// Cache
    pub ttl: Duration,
    /// Maximum cache size
    pub max_size: usize,
    /// Cache eviction policy
    /// Eviction Policy field
    pub eviction_policy: EvictionPolicy,
}

impl Default for CachingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl: Duration::from_secs(300),
            max_size: 1000,
            eviction_policy: EvictionPolicy::Lru,
        }
    }
}

/// Cache eviction policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    /// Least Recently
    Lru,
    /// Least Frequently
    Lfu,
    /// First In, First
    Fifo,
    /// Time-based expiration
    Ttl,
}

/// Connection pool configuration - consolidates `ConnectionPoolConfig`s
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum connections per pool
    /// Max Connections field
    pub max_connections: usize,
    /// Minimum connections per pool
    /// Min Connections field
    pub min_connections: usize,
    /// Connection timeout
    /// Connection Timeout field
    pub connection_timeout: Duration,
    /// Pool timeout (time to wait for connection)
    /// Pool Timeout field
    pub pool_timeout: Duration,
    /// Idle timeout for connections
    pub idle_timeout: Duration,
    /// Maximum lifetime for connections
    pub max_lifetime: Duration,
    /// Health check interval
    /// Health Check Interval field
    pub health_check_interval: Duration,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            min_connections: 10,
            connection_timeout: Duration::from_secs(30),
            pool_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(3600),
            health_check_interval: Duration::from_secs(30),
        }
    }
}
