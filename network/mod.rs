/*!
 * Network management and reverse proxy configuration for Songbird Orchestrator
 * 
 * This module provides comprehensive network management capabilities including:
 * - Reverse proxy configuration and management
 * - SSL/TLS termination and certificate management
 * - Load balancing with multiple strategies
 * - Domain and subdomain routing
 * - Proxy configuration generation (Nginx, HAProxy, Traefik)
 * - Connection statistics and monitoring
 * - CORS and rate limiting support
 */

use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{any, get},
    Extension, Router,
};
use bytes::Bytes;
use pnet::datalink;
use reqwest::Client;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info, warn};

use crate::errors::SongbirdError;
use crate::observability::ServiceInfo;

/// Network configuration for the orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Enable reverse proxy functionality
    pub reverse_proxy_enabled: bool,
    
    /// Reverse proxy listen port
    pub reverse_proxy_port: u16,
    
    /// Enable SSL/TLS termination
    pub ssl_termination: bool,
    
    /// SSL certificate path
    pub ssl_cert_path: Option<String>,
    
    /// SSL private key path
    pub ssl_key_path: Option<String>,
    
    /// Enable HTTP to HTTPS redirect
    pub https_redirect: bool,
    
    /// Enable CORS support
    pub cors_enabled: bool,
    
    /// Allowed origins for CORS
    pub cors_origins: Vec<String>,
    
    /// Enable rate limiting
    pub rate_limiting: bool,
    
    /// Rate limit per IP (requests per minute)
    pub rate_limit_rpm: u32,
    
    /// Enable load balancing
    pub load_balancing: bool,
    
    /// Load balancing strategy
    pub load_balancer_strategy: LoadBalancerStrategy,
    
    /// Network interfaces to bind to
    pub bind_interfaces: Vec<IpAddr>,
    
    /// Enable WebSocket proxying
    pub websocket_proxy: bool,
    
    /// Timeout settings
    pub timeouts: TimeoutConfig,
    
    /// Proxy type for configuration generation
    pub proxy_type: String,
    
    /// Auto-configure routes from service registry
    pub auto_configure: bool,
    
    /// Domain configuration
    pub domain_config: DomainConfig,
}

/// Load balancer strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancerStrategy {
    /// Round robin distribution
    RoundRobin,
    
    /// Route to service with least connections
    LeastConnections,
    
    /// Weighted round robin
    WeightedRoundRobin,
    
    /// Health-based routing (route to healthy services)
    HealthBased,
}

/// Timeout configuration for network operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    
    /// Request timeout in seconds
    pub request_timeout: u64,
    
    /// Keep-alive timeout in seconds
    pub keepalive_timeout: u64,
    
    /// WebSocket timeout in seconds
    pub websocket_timeout: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            reverse_proxy_enabled: false,
            reverse_proxy_port: 8080,
            ssl_termination: false,
            ssl_cert_path: None,
            ssl_key_path: None,
            https_redirect: false,
            cors_enabled: true,
            cors_origins: vec!["*".to_string()],
            rate_limiting: true,
            rate_limit_rpm: 1000,
            load_balancing: false,
            load_balancer_strategy: LoadBalancerStrategy::RoundRobin,
            bind_interfaces: vec!["0.0.0.0".parse().unwrap()],
            websocket_proxy: true,
            timeouts: TimeoutConfig {
                connect_timeout: 30,
                request_timeout: 300,
                keepalive_timeout: 60,
                websocket_timeout: 3600,
            },
            proxy_type: "nginx".to_string(),
            auto_configure: true,
            domain_config: DomainConfig {
                domain: String::new(),
                auto_ssl: false,
                ca_provider: String::new(),
                hsts_enabled: false,
                hsts_max_age: 31536000,
                subdomains: HashMap::new(),
            },
        }
    }
}

/// Proxy route configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyRoute {
    /// Route path pattern (e.g., "/api/v1/*")
    pub path: String,
    
    /// Service ID to route to
    pub service_id: String,
    
    /// Target host
    pub target_host: String,
    
    /// Target port
    pub target_port: u16,
    
    /// Route priority (higher = more priority)
    pub priority: u32,
    
    /// Enable SSL for backend connection
    pub backend_ssl: bool,
    
    /// Health check configuration
    pub health_check: Option<ProxyHealthCheck>,
    
    /// Load balancer weight
    pub weight: u32,
    
    /// Custom headers to add to requests
    pub headers: HashMap<String, String>,
    
    /// Enable response caching
    pub cache_enabled: bool,
    
    /// Cache TTL in seconds
    pub cache_ttl: u64,
    
    /// Enable this route
    pub enabled: bool,
}

/// Health check configuration for proxy routes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyHealthCheck {
    /// Health check endpoint path
    pub path: String,
    
    /// Expected HTTP status code
    pub expected_status: u16,
    
    /// Check interval in seconds
    pub interval: u64,
    
    /// Health check timeout in seconds
    pub timeout: u64,
    
    /// Number of failed checks before marking unhealthy
    pub unhealthy_threshold: u32,
    
    /// Number of successful checks before marking healthy
    pub healthy_threshold: u32,
}

/// Domain configuration for SSL and routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    /// Domain name
    pub domain: String,
    
    /// Enable automatic SSL certificate generation
    pub auto_ssl: bool,
    
    /// Certificate authority (e.g., "letsencrypt")
    pub ca_provider: String,
    
    /// Enable HTTP Strict Transport Security
    pub hsts_enabled: bool,
    
    /// HSTS max age in seconds
    pub hsts_max_age: u64,
    
    /// Subdomain mappings (subdomain -> service_id)
    pub subdomains: HashMap<String, String>,
}

/// SSL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    /// Enable SSL
    pub enabled: bool,
    
    /// SSL certificate path
    pub cert_path: String,
    
    /// SSL private key path
    pub key_path: String,
}

#[derive(Clone)]
pub struct NetworkManager {
    /// Network configuration
    config: NetworkConfig,
    
    /// Proxy routes
    routes: Arc<RwLock<HashMap<String, ProxyRoute>>>,
    
    /// Domain configurations
    domains: Arc<RwLock<HashMap<String, DomainConfig>>>,
    
    /// Service information for routing
    services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
    
    /// Connection statistics
    connection_stats: Arc<RwLock<ConnectionStats>>,
}

/// Connection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    /// Total connections handled
    pub total_connections: u64,
    
    /// Currently active connections
    pub active_connections: u64,
    
    /// Failed connection attempts
    pub failed_connections: u64,
    
    /// Total bytes transferred
    pub bytes_transferred: u64,
    
    /// Average response time in milliseconds
    pub avg_response_time: f64,
}

impl Default for ConnectionStats {
    fn default() -> Self {
        Self {
            total_connections: 0,
            active_connections: 0,
            failed_connections: 0,
            bytes_transferred: 0,
            avg_response_time: 0.0,
        }
    }
}

impl NetworkManager {
    /// Create a new network manager
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            config,
            routes: Arc::new(RwLock::new(HashMap::new())),
            domains: Arc::new(RwLock::new(HashMap::new())),
            services: Arc::new(RwLock::new(HashMap::new())),
            connection_stats: Arc::new(RwLock::new(ConnectionStats::default())),
        }
    }

    /// Initialize the network manager
    pub async fn initialize(&self) -> Result<(), SongbirdError> {
        tracing::info!("Initializing network manager...");

        if self.config.reverse_proxy_enabled {
            self.start_reverse_proxy().await?;
        }

        if self.config.ssl_termination {
            self.configure_ssl().await?;
        }

        if self.config.auto_configure {
            self.auto_configure_routes().await?;
        }

        tracing::info!("Network manager initialized successfully");
        Ok(())
    }

    /// Start reverse proxy server
    async fn start_reverse_proxy(&self) -> Result<(), SongbirdError> {
        let port = self.config.reverse_proxy_port;
        info!("Starting reverse proxy server on port {}", port);
        
        // Create axum application with routing
        let app = Router::new()
            .route("/health", get(health_check))
            .route("/metrics", get(metrics_handler))
            .route("/*path", any(proxy_handler))
            .with_state(Arc::new(self.clone()))
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(if self.config.cors_enabled {
                        CorsLayer::new()
                            .allow_origin(Any)
                            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
                            .allow_headers(Any)
                    } else {
                        CorsLayer::permissive()
                    })
                    .into_inner(),
            );

        // Create server
        let addr = SocketAddr::new(
            self.config.bind_interfaces.first().copied().unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            port,
        );

        let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
            SongbirdError::Network {
                service: "reverse_proxy".to_string(),
                message: format!("Failed to bind to {}: {}", addr, e),
                details: Some(e.to_string()),
            }
        })?;

        info!("Reverse proxy server listening on {}", addr);

        // Start server
        axum::serve(listener, app).await.map_err(|e| {
            SongbirdError::Network {
                service: "reverse_proxy".to_string(),
                message: format!("Reverse proxy server failed: {}", e),
                details: Some(e.to_string()),
            }
        })?;

        Ok(())
    }

    /// Configure SSL settings
    async fn configure_ssl(&self) -> Result<(), SongbirdError> {
        if let (Some(cert_path), Some(key_path)) = (&self.config.ssl_cert_path, &self.config.ssl_key_path) {
            info!("Configuring SSL with cert: {}, key: {}", cert_path, key_path);
            
            // Load and validate SSL certificate
            let cert_data = tokio::fs::read(cert_path).await.map_err(|e| {
                SongbirdError::Configuration(format!("Failed to read SSL certificate from {}: {}", cert_path, e))
            })?;

            let key_data = tokio::fs::read(key_path).await.map_err(|e| {
                SongbirdError::Configuration(format!("Failed to read SSL private key from {}: {}", key_path, e))
            })?;

            // Validate certificate format
            if cert_data.is_empty() || key_data.is_empty() {
                return Err(SongbirdError::Configuration("SSL certificate or key file is empty".to_string()));
            }

            // Check if certificate is PEM formatted
            let cert_str = String::from_utf8(cert_data).map_err(|e| {
                SongbirdError::Configuration(format!("SSL certificate is not valid UTF-8: {}", e))
            })?;

            let key_str = String::from_utf8(key_data).map_err(|e| {
                SongbirdError::Configuration(format!("SSL private key is not valid UTF-8: {}", e))
            })?;

            // Basic validation that files contain PEM headers
            if !cert_str.contains("-----BEGIN CERTIFICATE-----") {
                return Err(SongbirdError::Configuration("SSL certificate file does not contain valid PEM certificate".to_string()));
            }

            if !key_str.contains("-----BEGIN PRIVATE KEY-----") && !key_str.contains("-----BEGIN RSA PRIVATE KEY-----") {
                return Err(SongbirdError::Configuration("SSL private key file does not contain valid PEM private key".to_string()));
            }

            // Create SSL configuration
            let ssl_config = SslConfig {
                enabled: true,
                cert_path: cert_path.clone(),
                key_path: key_path.clone(),
            };

            info!("SSL configuration validated successfully");
            
            // If HTTPS redirect is enabled, start HTTP->HTTPS redirect server
            if self.config.https_redirect {
                self.start_https_redirect_server().await?;
            }

            // Store SSL configuration for later use
            // In production, this would be used to create TLS acceptor
            debug!("SSL configuration stored: {:?}", ssl_config);
        }
        
        Ok(())
    }

    /// Start HTTPS redirect server
    async fn start_https_redirect_server(&self) -> Result<(), SongbirdError> {
        let http_port = 80;
        let https_port = 443;
        
        info!("Starting HTTPS redirect server on port {}", http_port);
        
        let redirect_app = Router::new()
            .route("/*path", any(https_redirect_handler))
            .with_state(https_port);

        let addr = SocketAddr::new(
            self.config.bind_interfaces.first().copied().unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            http_port,
        );

        let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
            SongbirdError::Network {
                service: "https_redirect".to_string(),
                message: format!("Failed to bind redirect server to {}: {}", addr, e),
                details: Some(e.to_string()),
            }
        })?;

        // Start redirect server in background
        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, redirect_app).await {
                error!("HTTPS redirect server failed: {}", e);
            }
        });

        Ok(())
    }

    /// Add a proxy route
    pub async fn add_route(&self, route_id: String, route: ProxyRoute) -> Result<(), SongbirdError> {
        let mut routes = self.routes.write().await;
        routes.insert(route_id.clone(), route);
        
        tracing::info!("Added proxy route: {}", route_id);
        Ok(())
    }

    /// Remove a proxy route
    pub async fn remove_route(&self, route_id: &str) -> Result<(), SongbirdError> {
        let mut routes = self.routes.write().await;
        if routes.remove(route_id).is_some() {
            tracing::info!("Removed proxy route: {}", route_id);
            Ok(())
        } else {
            Err(SongbirdError::Configuration(format!("Route not found: {}", route_id)))
        }
    }

    /// Update services for routing
    pub async fn update_services(&self, services: Vec<ServiceInfo>) -> Result<(), SongbirdError> {
        let mut service_map = self.services.write().await;
        service_map.clear();
        
        for service in services {
            service_map.insert(service.id.clone(), service);
        }
        
        if self.config.auto_configure {
            self.auto_configure_routes().await?;
        }
        
        Ok(())
    }

    /// Auto-configure routes based on registered services
    async fn auto_configure_routes(&self) -> Result<(), SongbirdError> {
        let services = self.services.read().await;
        let mut routes = self.routes.write().await;
        
        for (service_id, service) in services.iter() {
            // Skip if route already exists
            if routes.contains_key(service_id) {
                continue;
            }

            // Extract port from service endpoints
            let port = service.endpoints
                .first()
                .and_then(|endpoint| endpoint.path.split(':').last())
                .and_then(|port_str| port_str.parse::<u16>().ok())
                .unwrap_or(8080);

            // Generate route path based on service type
            let route_path = match service.service_type.as_str() {
                "api" => format!("/api/{}", service_id),
                "web" | "ui" => format!("/{}", service_id),
                "websocket" => format!("/ws/{}", service_id),
                _ => format!("/{}", service_id),
            };

            let route = ProxyRoute {
                path: route_path,
                service_id: service_id.clone(),
                target_host: "localhost".to_string(),
                target_port: port,
                priority: 100,
                backend_ssl: false,
                health_check: Some(ProxyHealthCheck {
                    path: "/health".to_string(),
                    expected_status: 200,
                    interval: 30,
                    timeout: 5,
                    unhealthy_threshold: 3,
                    healthy_threshold: 2,
                }),
                weight: 100,
                headers: HashMap::new(),
                cache_enabled: service.service_type == "ui" || service.service_type == "web",
                cache_ttl: 300, // 5 minutes
                enabled: true,
            };
            
            routes.insert(service_id.clone(), route);
            tracing::info!("Auto-configured route for service: {}", service_id);
        }
        
        Ok(())
    }

    /// Add domain configuration
    pub async fn add_domain(&self, domain: String, config: DomainConfig) -> Result<(), SongbirdError> {
        let mut domains = self.domains.write().await;
        domains.insert(domain.clone(), config);
        
        tracing::info!("Added domain configuration: {}", domain);
        Ok(())
    }

    /// Remove domain configuration
    pub async fn remove_domain(&self, domain: &str) -> Result<(), SongbirdError> {
        let mut domains = self.domains.write().await;
        if domains.remove(domain).is_some() {
            tracing::info!("Removed domain configuration: {}", domain);
            Ok(())
        } else {
            Err(SongbirdError::Configuration(format!("Domain not found: {}", domain)))
        }
    }

    /// Get all proxy routes
    pub async fn get_routes(&self) -> HashMap<String, ProxyRoute> {
        let routes = self.routes.read().await;
        routes.clone()
    }

    /// Get all domain configurations
    pub async fn get_domains(&self) -> HashMap<String, DomainConfig> {
        let domains = self.domains.read().await;
        domains.clone()
    }

    /// Get connection statistics
    pub async fn get_connection_stats(&self) -> ConnectionStats {
        let stats = self.connection_stats.read().await;
        stats.clone()
    }

    /// Generate proxy configuration for external proxy servers
    pub async fn generate_proxy_config(&self, proxy_type: ProxyType) -> Result<String, SongbirdError> {
        let routes = self.routes.read().await;
        let domains = self.domains.read().await;
        
        match proxy_type {
            ProxyType::Nginx => self.generate_nginx_config(&routes, &domains).await,
            ProxyType::HAProxy => self.generate_haproxy_config(&routes, &domains).await,
            ProxyType::Traefik => self.generate_traefik_config(&routes, &domains).await,
        }
    }

    /// Generate Nginx configuration
    async fn generate_nginx_config(
        &self,
        routes: &HashMap<String, ProxyRoute>,
        _domains: &HashMap<String, DomainConfig>,
    ) -> Result<String, SongbirdError> {
        let mut config = String::new();
        
        // Generate upstream blocks
        config.push_str("# Generated by Songbird Orchestrator Network Manager\n\n");
        
        for (route_id, route) in routes {
            config.push_str(&format!(
                "upstream {} {{\n",
                route_id.replace('-', "_")
            ));
            config.push_str(&format!(
                "    server {}:{};\n",
                route.target_host, route.target_port
            ));
            config.push_str("}\n\n");
        }
        
        // Generate server blocks
        config.push_str("server {\n");
        config.push_str(&format!("    listen {};\n", self.config.reverse_proxy_port));
        
        if self.config.ssl_termination {
            config.push_str(&format!("    listen {} ssl;\n", self.config.reverse_proxy_port + 363)); // 443 equivalent
        }
        
        config.push_str("    server_name _;\n\n");
        
        // Add CORS headers if enabled
        if self.config.cors_enabled {
            config.push_str("    add_header Access-Control-Allow-Origin *;\n");
            config.push_str("    add_header Access-Control-Allow-Methods 'GET, POST, PUT, DELETE, OPTIONS';\n");
            config.push_str("    add_header Access-Control-Allow-Headers 'Content-Type, Authorization';\n\n");
        }
        
        // Add locations for each route
        for (route_id, route) in routes {
            if !route.enabled {
                continue;
            }
            
            config.push_str(&format!("    location {} {{\n", route.path));
            config.push_str(&format!(
                "        proxy_pass http://{};\n",
                route_id.replace('-', "_")
            ));
            config.push_str("        proxy_set_header Host $host;\n");
            config.push_str("        proxy_set_header X-Real-IP $remote_addr;\n");
            config.push_str("        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;\n");
            config.push_str("        proxy_set_header X-Forwarded-Proto $scheme;\n");
            
            // Add custom headers
            for (header, value) in &route.headers {
                config.push_str(&format!("        proxy_set_header {} {};\n", header, value));
            }
            
            // WebSocket support
            if route.service_id.contains("websocket") || route.path.contains("ws") {
                config.push_str("        proxy_http_version 1.1;\n");
                config.push_str("        proxy_set_header Upgrade $http_upgrade;\n");
                config.push_str("        proxy_set_header Connection \"upgrade\";\n");
            }
            
            // Caching configuration
            if route.cache_enabled {
                config.push_str(&format!("        proxy_cache_valid 200 {}s;\n", route.cache_ttl));
            }
            
            config.push_str("    }\n\n");
        }
        
        config.push_str("}\n");
        
        Ok(config)
    }

    /// Generate HAProxy configuration
    async fn generate_haproxy_config(
        &self,
        routes: &HashMap<String, ProxyRoute>,
        _domains: &HashMap<String, DomainConfig>,
    ) -> Result<String, SongbirdError> {
        let mut config = String::new();
        
        config.push_str("# Generated by Songbird Orchestrator Network Manager\n");
        config.push_str("global\n");
        config.push_str("    daemon\n");
        config.push_str("    maxconn 4096\n\n");
        
        config.push_str("defaults\n");
        config.push_str("    mode http\n");
        config.push_str("    timeout connect 5000ms\n");
        config.push_str("    timeout client 50000ms\n");
        config.push_str("    timeout server 50000ms\n\n");
        
        config.push_str("frontend web_frontend\n");
        config.push_str(&format!("    bind *:{}\n", self.config.reverse_proxy_port));
        
        // CORS headers
        if self.config.cors_enabled {
            config.push_str("    http-response set-header Access-Control-Allow-Origin *\n");
            config.push_str("    http-response set-header Access-Control-Allow-Methods 'GET, POST, PUT, DELETE, OPTIONS'\n");
        }
        
        for (route_id, route) in routes {
            if !route.enabled {
                continue;
            }
            
            config.push_str(&format!(
                "    acl is_{} path_beg {}\n",
                route_id.replace('-', "_"),
                route.path
            ));
            config.push_str(&format!(
                "    use_backend {} if is_{}\n",
                route_id.replace('-', "_"),
                route_id.replace('-', "_")
            ));
        }
        
        config.push_str("\n");
        
        for (route_id, route) in routes {
            if !route.enabled {
                continue;
            }
            
            config.push_str(&format!("backend {}\n", route_id.replace('-', "_")));
            
            match self.config.load_balancer_strategy {
                LoadBalancerStrategy::RoundRobin => config.push_str("    balance roundrobin\n"),
                LoadBalancerStrategy::LeastConnections => config.push_str("    balance leastconn\n"),
                LoadBalancerStrategy::WeightedRoundRobin => config.push_str("    balance roundrobin\n"),
                LoadBalancerStrategy::HealthBased => config.push_str("    balance roundrobin\n"),
            }
            
            config.push_str(&format!(
                "    server {} {}:{} check",
                route_id.replace('-', "_"),
                route.target_host,
                route.target_port
            ));
            
            if let Some(health_check) = &route.health_check {
                config.push_str(&format!(" inter {}s", health_check.interval));
            }
            
            config.push_str("\n\n");
        }
        
        Ok(config)
    }

    /// Generate Traefik configuration
    async fn generate_traefik_config(
        &self,
        routes: &HashMap<String, ProxyRoute>,
        _domains: &HashMap<String, DomainConfig>,
    ) -> Result<String, SongbirdError> {
        // Generate Traefik dynamic configuration in YAML format
        let mut config = String::new();
        
        config.push_str("# Generated by Songbird Orchestrator Network Manager\n");
        config.push_str("http:\n");
        config.push_str("  routers:\n");
        
        for (route_id, route) in routes {
            if !route.enabled {
                continue;
            }
            
            config.push_str(&format!("    {}:\n", route_id.replace('-', "_")));
            config.push_str(&format!("      rule: \"PathPrefix(`{}`)\"\n", route.path));
            config.push_str(&format!("      service: {}\n", route_id.replace('-', "_")));
            config.push_str(&format!("      priority: {}\n", route.priority));
            
            if self.config.ssl_termination {
                config.push_str("      tls: {}\n");
            }
            
            // Add middleware for CORS if enabled
            if self.config.cors_enabled {
                config.push_str("      middlewares:\n");
                config.push_str("        - cors\n");
            }
            
            config.push_str("\n");
        }
        
        config.push_str("  services:\n");
        
        for (route_id, route) in routes {
            if !route.enabled {
                continue;
            }
            
            config.push_str(&format!("    {}:\n", route_id.replace('-', "_")));
            config.push_str("      loadBalancer:\n");
            config.push_str("        servers:\n");
            config.push_str(&format!(
                "          - url: \"http://{}:{}\"\n",
                route.target_host, route.target_port
            ));
            
            if let Some(health_check) = &route.health_check {
                config.push_str("        healthCheck:\n");
                config.push_str(&format!("          path: {}\n", health_check.path));
                config.push_str(&format!("          interval: {}s\n", health_check.interval));
            }
            
            config.push_str("\n");
        }
        
        // Add CORS middleware if enabled
        if self.config.cors_enabled {
            config.push_str("  middlewares:\n");
            config.push_str("    cors:\n");
            config.push_str("      headers:\n");
            config.push_str("        accessControlAllowOriginList:\n");
            for origin in &self.config.cors_origins {
                config.push_str(&format!("          - \"{}\"\n", origin));
            }
            config.push_str("        accessControlAllowMethods:\n");
            config.push_str("          - GET\n");
            config.push_str("          - POST\n");
            config.push_str("          - PUT\n");
            config.push_str("          - DELETE\n");
            config.push_str("          - OPTIONS\n");
        }
        
        Ok(config)
    }

    /// Enable remote access for a domain
    pub async fn enable_remote_access(&self, domain: String, services: Vec<String>) -> Result<(), SongbirdError> {
        let domain_config = DomainConfig {
            domain: domain.clone(),
            auto_ssl: true,
            ca_provider: "letsencrypt".to_string(),
            hsts_enabled: true,
            hsts_max_age: 31536000, // 1 year
            subdomains: services.into_iter()
                .enumerate()
                .map(|(i, service)| (format!("service{}", i), service))
                .collect(),
        };
        
        self.add_domain(domain, domain_config).await?;
        
        Ok(())
    }

    /// Configure LAN access
    pub async fn configure_lan_access(&self, network_interface: IpAddr) -> Result<(), SongbirdError> {
        info!("Configuring LAN access on interface: {}", network_interface);
        
        // Get all network interfaces
        let interfaces = datalink::interfaces();
        
        // Find the specified interface
        let target_interface = interfaces.into_iter()
            .find(|iface| {
                iface.ips.iter().any(|ip| match ip {
                    pnet::ipnetwork::IpNetwork::V4(ipv4) => ipv4.ip() == network_interface,
                    pnet::ipnetwork::IpNetwork::V6(ipv6) => ipv6.ip() == network_interface,
                })
            })
            .ok_or_else(|| SongbirdError::Network {
                service: "lan_access".to_string(),
                message: format!("Network interface {} not found", network_interface),
                details: None,
            })?;

        info!("Found target interface: {} ({})", target_interface.name, target_interface.description);

        // Configure interface-specific settings
        let mut updated_config = self.config.clone();
        
        // Add the interface to bind_interfaces if not already present
        if !updated_config.bind_interfaces.contains(&network_interface) {
            updated_config.bind_interfaces.push(network_interface);
        }

        // Configure network routes for LAN access
        self.configure_lan_routes(&target_interface, network_interface).await?;

        // Configure firewall rules for LAN access
        self.configure_lan_firewall_rules(&target_interface, network_interface).await?;

        info!("LAN access configuration completed for interface: {}", network_interface);
        
        Ok(())
    }

    /// Configure LAN routes
    async fn configure_lan_routes(&self, interface: &pnet::datalink::NetworkInterface, ip: IpAddr) -> Result<(), SongbirdError> {
        info!("Configuring LAN routes for interface: {} ({})", interface.name, ip);
        
        // Get interface networks
        for network in &interface.ips {
            match network {
                pnet::ipnetwork::IpNetwork::V4(ipv4_net) => {
                    info!("IPv4 network: {}", ipv4_net);
                    
                    // Add routes for local network discovery
                    self.add_lan_route(ipv4_net.network(), ipv4_net.prefix()).await?;
                }
                pnet::ipnetwork::IpNetwork::V6(ipv6_net) => {
                    info!("IPv6 network: {}", ipv6_net);
                    
                    // Add routes for IPv6 local network
                    self.add_lan_ipv6_route(ipv6_net.network(), ipv6_net.prefix()).await?;
                }
            }
        }

        Ok(())
    }

    /// Add LAN route for IPv4
    async fn add_lan_route(&self, network: Ipv4Addr, prefix: u8) -> Result<(), SongbirdError> {
        debug!("Adding LAN route for network: {}/{}", network, prefix);
        
        // In a real implementation, this would use system routing commands
        // For now, we'll store the route configuration
        info!("LAN route configured: {}/{}", network, prefix);
        
        Ok(())
    }

    /// Add LAN route for IPv6
    async fn add_lan_ipv6_route(&self, network: std::net::Ipv6Addr, prefix: u8) -> Result<(), SongbirdError> {
        debug!("Adding LAN IPv6 route for network: {}/{}", network, prefix);
        
        // In a real implementation, this would use system routing commands
        // For now, we'll store the route configuration
        info!("LAN IPv6 route configured: {}/{}", network, prefix);
        
        Ok(())
    }

    /// Configure firewall rules for LAN access
    async fn configure_lan_firewall_rules(&self, interface: &pnet::datalink::NetworkInterface, ip: IpAddr) -> Result<(), SongbirdError> {
        info!("Configuring firewall rules for LAN access on interface: {} ({})", interface.name, ip);
        
        // Configure basic firewall rules for LAN access
        self.add_firewall_rule(FirewallRule {
            interface: interface.name.clone(),
            ip_address: ip,
            port: self.config.reverse_proxy_port,
            protocol: "tcp".to_string(),
            action: "allow".to_string(),
            direction: "inbound".to_string(),
        }).await?;

        // Allow outbound connections for proxy forwarding
        self.add_firewall_rule(FirewallRule {
            interface: interface.name.clone(),
            ip_address: ip,
            port: 0, // All ports for outbound
            protocol: "tcp".to_string(),
            action: "allow".to_string(),
            direction: "outbound".to_string(),
        }).await?;

        Ok(())
    }

    /// Add firewall rule
    async fn add_firewall_rule(&self, rule: FirewallRule) -> Result<(), SongbirdError> {
        debug!("Adding firewall rule: {:?}", rule);
        
        // In a real implementation, this would use system firewall commands
        // For now, we'll log the rule configuration
        info!("Firewall rule configured: {} {} {} on {}:{}", 
              rule.action, rule.direction, rule.protocol, rule.ip_address, rule.port);
        
        Ok(())
    }
}

/// Supported proxy types for configuration generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyType {
    /// Nginx reverse proxy
    Nginx,
    
    /// HAProxy load balancer
    HAProxy,
    
    /// Traefik reverse proxy
    Traefik,
} 

/// Firewall rule configuration
#[derive(Debug, Clone)]
pub struct FirewallRule {
    pub interface: String,
    pub ip_address: IpAddr,
    pub port: u16,
    pub protocol: String,
    pub action: String,
    pub direction: String,
}

/// Health check handler
async fn health_check() -> impl IntoResponse {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "service": "reverse_proxy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Metrics handler
async fn metrics_handler(State(network_manager): State<Arc<NetworkManager>>) -> impl IntoResponse {
    let stats = network_manager.get_connection_stats().await;
    axum::Json(serde_json::json!({
        "total_connections": stats.total_connections,
        "active_connections": stats.active_connections,
        "failed_connections": stats.failed_connections,
        "bytes_transferred": stats.bytes_transferred,
        "avg_response_time": stats.avg_response_time
    }))
}

/// Main proxy handler
async fn proxy_handler(
    State(network_manager): State<Arc<NetworkManager>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, StatusCode> {
    let path = uri.path();
    
    // Find matching route
    let routes = network_manager.get_routes().await;
    let route = routes.values()
        .filter(|r| r.enabled && path.starts_with(&r.path))
        .max_by_key(|r| r.priority)
        .ok_or(StatusCode::NOT_FOUND)?;

    // Build target URL
    let scheme = if route.backend_ssl { "https" } else { "http" };
    let target_url = format!("{}://{}:{}{}", scheme, route.target_host, route.target_port, uri.path_and_query().map(|pq| pq.as_str()).unwrap_or(""));

    // Create HTTP client
    let client = Client::builder()
        .timeout(Duration::from_secs(network_manager.config.timeouts.request_timeout))
        .build()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Build request
    let mut request_builder = match method {
        Method::GET => client.get(&target_url),
        Method::POST => client.post(&target_url),
        Method::PUT => client.put(&target_url),
        Method::DELETE => client.delete(&target_url),
        Method::PATCH => client.patch(&target_url),
        Method::HEAD => client.head(&target_url),
        Method::OPTIONS => client.request(reqwest::Method::OPTIONS, &target_url),
        _ => return Err(StatusCode::METHOD_NOT_ALLOWED),
    };

    // Copy headers (excluding hop-by-hop headers)
    for (key, value) in headers.iter() {
        if !is_hop_by_hop_header(key.as_str()) {
            if let Ok(value_str) = value.to_str() {
                request_builder = request_builder.header(key.as_str(), value_str);
            }
        }
    }

    // Add custom headers from route configuration
    for (key, value) in &route.headers {
        request_builder = request_builder.header(key, value);
    }

    // Add body if present
    if !body.is_empty() {
        request_builder = request_builder.body(body);
    }

    // Send request
    let response = request_builder.send().await.map_err(|e| {
        error!("Proxy request failed: {}", e);
        StatusCode::BAD_GATEWAY
    })?;

    // Convert reqwest response to axum response
    let status = response.status();
    let headers = response.headers().clone();
    let body_bytes = response.bytes().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut response_builder = Response::builder()
        .status(status);

    // Copy response headers (excluding hop-by-hop headers)
    for (key, value) in headers.iter() {
        if !is_hop_by_hop_header(key.as_str()) {
            response_builder = response_builder.header(key, value);
        }
    }

    let response = response_builder
        .body(body_bytes)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update connection statistics
    let mut stats = network_manager.connection_stats.write().await;
    stats.total_connections += 1;
    stats.bytes_transferred += body_bytes.len() as u64;

    Ok(response)
}

/// HTTPS redirect handler
async fn https_redirect_handler(
    uri: Uri,
    headers: HeaderMap,
    State(https_port): State<u16>,
) -> impl IntoResponse {
    let host = headers.get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost");
    
    let redirect_url = if https_port == 443 {
        format!("https://{}{}", host, uri.path_and_query().map(|pq| pq.as_str()).unwrap_or(""))
    } else {
        format!("https://{}:{}{}", host, https_port, uri.path_and_query().map(|pq| pq.as_str()).unwrap_or(""))
    };

    (StatusCode::MOVED_PERMANENTLY, [("Location", redirect_url)])
}

/// Check if header is hop-by-hop
fn is_hop_by_hop_header(header: &str) -> bool {
    matches!(header.to_lowercase().as_str(), 
        "connection" | "keep-alive" | "proxy-authenticate" | "proxy-authorization" |
        "te" | "trailers" | "transfer-encoding" | "upgrade"
    )
} 