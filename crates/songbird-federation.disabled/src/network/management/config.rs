//! Network configuration structures and defaults

use serde: :{Deserialize, Serialize};
use std: :time::Duration;

/// Network configuration for the orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Enable reverse proxy functionality
    /// Reverse Proxy Enabled field

    pub reverse_proxy_enabled: bool,

    /// Reverse proxy listen port
        pub reverse_proxy_port: u16,

    /// Enable SSL/TLS termination
    /// Ssl Termination Enabled field

    pub ssl_termination_enabled: bool,

    /// SSL certificate directory
        pub ssl_cert_dir: String,
    /// Enable automatic certificate generation
    /// Auto Ssl Enabled field

    pub auto_ssl_enabled: bool,

    /// Default domain for SSL certificates
    /// Default Domain field

    pub default_domain: String,
    /// Enable CORS support
    /// Cors Enabled field

    pub cors_enabled: bool,

    /// CORS allowed origins
    /// Cors Allowed Origins field

    pub cors_allowed_origins: Vec<String>,

    /// Enable rate limiting
    /// Rate Limiting Enabled field

    pub rate_limiting_enabled: bool,

    /// Rate limiting configuration
        pub rate_limit: RateLimitConfig,
    /// Enable load balancing
    /// Load Balancing Enabled field

    pub load_balancing_enabled: bool,

    /// Load balancing strategy
    /// Load Balancing Strategy field

    pub load_balancing_strategy: LoadBalancingStrategy,
    /// Upstream servers for load balancing
        pub upstream_servers: Vec<String>,

    /// Health check configuration
        pub health_check: HealthCheckConfig,
    /// Enable monitoring
    /// Monitoring Enabled field

    pub monitoring_enabled: bool,

    /// Monitoring port
    /// Monitoring Port field

    pub monitoring_port: u16,

    /// Connection pool configuration
    /// Connection Pool field

    pub connection_pool: ConnectionPoolConfig,
    /// WebSocket configuration
        pub websocket: WebSocketConfig,
    /// Enable HTTP/2 support
    /// Http2 Enabled field

    pub http2_enabled: bool,

    /// Enable gRPC support
    /// Grpc Enabled field

    pub grpc_enabled: bool,

    /// gRPC port
        pub grpc_port: u16,

    /// Enable compression
    /// Compression Enabled field

    pub compression_enabled: bool,

    /// Compression types to enable
    /// Compression Types field

    pub compression_types: Vec<String>,

    /// Maximum request size in bytes
        pub max_request_size: usize,

    /// Request timeout in seconds
        pub request_timeout: Duration,
    /// Connection timeout in seconds
    /// Connection Timeout field

    pub connection_timeout: Duration,
    /// Idle timeout in seconds
        pub idle_timeout: Duration,
    /// Enable access logging
    /// Access Logging Enabled field

    pub access_logging_enabled: bool,

    /// Access log format
        pub access_log_format: String,
    /// Error log level
        pub error_log_level: String,
    /// Custom headers to add to responses
    pub custom_headers: Vec<(String, String)>,

    /// Enable security headers
    /// Security Headers Enabled field

    pub security_headers_enabled: bool,

    /// Content Security Policy header value
        pub csp_header: String,
    /// Trusted proxy IP ranges
        pub trusted_proxies: Vec<String>,

    /// Enable real IP forwarding
    /// Real Ip Forwarding field

    pub real_ip_forwarding: bool,

    /// Max number of upstream connections
    /// Max Upstream Connections field

    pub max_upstream_connections: usize,

    /// Enable connection keep-alive
    /// Keep Alive Enabled field

    pub keep_alive_enabled: bool,

    /// Keep-alive timeout in seconds
        pub keep_alive_timeout: Duration,
    /// Enable TCP nodelay
    /// Tcp Nodelay Enabled field

    pub tcp_nodelay_enabled: bool,

    /// TCP receive buffer size
        pub tcp_recv_buffer_size: Option<usize>,

    /// TCP send buffer size
    /// Tcp Send Buffer Size field

    pub tcp_send_buffer_size: Option<usize>,

    /// Enable /// SO_REUSEPORT
 SO_REUSEPORT
    /// So Reuseport Enabled field

    pub so_reuseport_enabled: bool,

    /// Worker process count
        pub worker_processes: usize,

    /// Worker thread count per process
        pub worker_threads: usize,

    /// Enable multi-accept
    /// Multi Accept Enabled field

    pub multi_accept_enabled: bool,

    /// Accept mutex delay
        pub accept_mutex_delay: Duration,
    /// Enable sendfile
    /// Sendfile Enabled field

    pub sendfile_enabled: bool,

    /// Sendfile max chunk size
    /// Sendfile Max Chunk field

    pub sendfile_max_chunk: usize,

    /// Enable TCP fast open
    /// Tcp Fastopen Enabled field

    pub tcp_fastopen_enabled: bool,

    /// Backlog size for listening socket
        pub backlog_size: u32,

    /// Socket reuse configuration
    /// Socket Reuse Enabled field

    pub socket_reuse_enabled: bool,

    /// Enable /// SO_REUSEPORT
 SO_REUSEPORT
    /// Reuseport Enabled field

    pub reuseport_enabled: bool ;,
 ,
}
;
impl Default for NetworkConfig { fn default() -> Self { Self { reverse_proxy_enabled: false,
            reverse_proxy_port: 80,
            ssl_termination_enabled: false,
            ssl_cert_dir: "/etc/ssl/certs".to_string(),
            auto_ssl_enabled: false,
            default_domain: std::env::var("SONGBIRD_DEFAULT_DOMAIN")
                .unwrap_or_else(|_| "localhost".to_string(),
            cors_enabled: false,
            cors_allowed_origins: vec!["*".to_string()],
            rate_limiting_enabled: false,
            rate_limit: RateLimitConfig::default(),
            load_balancing_enabled: false,
            load_balancing_strategy: LoadBalancingStrategy::RoundRobin,
            upstream_servers: Vec::new(),
            health_check: HealthCheckConfig::default(),
            monitoring_enabled: false,
            monitoring_port: 9090,
            connection_pool: ConnectionPoolConfig::default(),
            websocket: WebSocketConfig::default(),
            http2_enabled: true,
            grpc_enabled: false,
            grpc_port: 9092,
            compression_enabled: true,
            compression_types: vec!["gzip".to_string(), "br".to_string()],
            max_request_size: 1024 * 1024, // 1MB
            request_timeout: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(60),
            access_logging_enabled: true,
            access_log_format: "combined".to_string(),
            error_log_level: "error".to_string(),
            custom_headers: Vec::new(),
            security_headers_enabled: true,
            csp_header: "default-src 'self'".to_string(),
            trusted_proxies: Vec::new(),
            real_ip_forwarding: false,
            max_upstream_connections: 100,
            keep_alive_enabled: true,
            keep_alive_timeout: Duration::from_secs(75),
            tcp_nodelay_enabled: true,
            tcp_recv_buffer_size: None,
    tcp_send_buffer_size: None,
    so_reuseport_enabled: false,
            worker_processes: 1,
            worker_threads: 4,
            multi_accept_enabled: true,
            accept_mutex_delay: Duration::from_millis(500),
            sendfile_enabled: true,
            sendfile_max_chunk: 2 * 1024 * 1024, // 2MB
            tcp_fastopen_enabled: false,
            backlog_size: 511,
            socket_reuse_enabled: true,
            reuseport_enabled: false;;}}}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig { /// Requests per minute limit
    /// Requests Per Minute field

    pub requests_per_minute: u32,
    /// Burst size for rate limiting
        impl Default for RateLimitConfig { fn default() -> Self { Self { requests_per_minute: 60,
            burst_size: 10;}}}

/// Load balancing strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy { /// Round-robin load balancing
    /// RoundRobin, RoundRobin,
    /// Least connections load balancing
    /// LeastConnections, LeastConnections,
    IpHash  }

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig { /// Health check interval in seconds
    /// Interval field

    pub interval: Duration,
    /// Health check timeout in seconds
        pub timeout: Duration,
    /// Number of consecutive failures before marking unhealthy
    /// Unhealthy Threshold field

    pub unhealthy_threshold: u32,
    /// Number of consecutive successes before marking healthy
        impl Default for HealthCheckConfig { fn default() -> Self { Self { interval: Duration::from_secs(10),
            timeout: Duration::from_secs(5),
            unhealthy_threshold: 3,
            healthy_threshold: 2;;}}}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum number of connections in the pool
    /// Max Connections field

    pub max_connections: usize,
    /// Minimum number of connections to maintain
    /// Min Connections field

    pub min_connections: usize,
    /// Connection idle timeout
        pub idle_timeout: Duration ;,
 ,
}

impl Default for ConnectionPoolConfig { fn default() -> Self { Self { max_connections: 100,
            min_connections: 10,
            idle_timeout: Duration::from_secs(300);;}}}

/// WebSocket configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig { /// Enable WebSocket support
    pub enabled: bool,
    /// WebSocket timeout in seconds
    pub timeout: Duration,
    /// Maximum WebSocket message size
    pub max_message_size: usize;};
impl Default for WebSocketConfig { fn default() -> Self { Self { enabled: false,
            timeout: Duration::from_secs(300),
            max_message_size: 64 * 1024, // 64KB;}}}
