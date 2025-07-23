//! Reverse proxy functionality and configuration generation

use songbird_errors::SongbirdError;
use std::collections::HashMap;
use tracing::info;

use super::config::NetworkConfig;

/// Proxy configuration generator
pub struct ProxyConfigGenerator {
    config: NetworkConfig,
}

impl ProxyConfigGenerator {
    /// Create new proxy configuration generator
    pub fn new(config: NetworkConfig) -> Self {
        Self { config }
    }

    /// Generate Nginx configuration
    pub fn generate_nginx_config(&self) -> Result<String, SongbirdError> {
        let mut config = String::new();

        // Basic Nginx configuration
        config.push_str("events {\n");
        config.push_str(&format!(
            "    worker_connections {};\n",
            self.config.max_upstream_connections
        ));
        config.push_str("}\n\n");

        config.push_str("http {\n");
        config.push_str("    include       /etc/nginx/mime.types;\n");
        config.push_str("    default_type  application/octet-stream;\n\n");

        // Logging
        if self.config.access_logging_enabled {
            config.push_str(&format!(
                "    access_log /var/log/nginx/access.log {};\n",
                self.config.access_log_format
            ));
        }
        config.push_str(&format!(
            "    error_log /var/log/nginx/error.log {};\n",
            self.config.error_log_level
        ));

        // Basic settings
        if self.config.sendfile_enabled {
            config.push_str("    sendfile on;\n");
        }
        if self.config.tcp_nodelay_enabled {
            config.push_str("    tcp_nodelay on;\n");
        }
        if self.config.keep_alive_enabled {
            config.push_str(&format!(
                "    keepalive_timeout {};\n",
                self.config.keep_alive_timeout.as_secs()
            ));
        }

        // Compression
        if self.config.compression_enabled {
            config.push_str("    gzip on;\n");
            config.push_str("    gzip_types text/plain text/css application/json application/javascript text/xml application/xml;\n");
        }

        // Rate limiting
        if self.config.rate_limiting_enabled {
            config.push_str("    limit_req_zone $binary_remote_addr zone=api:10m rate=");
            config.push_str(&format!(
                "{}r/m;\n",
                self.config.rate_limit.requests_per_minute
            ));
        }

        // Upstream servers
        if self.config.load_balancing_enabled && !self.config.upstream_servers.is_empty() {
            config.push_str("    upstream backend {\n");
            match self.config.load_balancing_strategy {
                super::config::LoadBalancingStrategy::LeastConnections => {
                    config.push_str("        least_conn;\n");
                }
                super::config::LoadBalancingStrategy::IpHash => {
                    config.push_str("        ip_hash;\n");
                }
                super::config::LoadBalancingStrategy::RoundRobin => {
                    // Default, no directive needed
                }
            }
            for server in &self.config.upstream_servers {
                info!("Adding upstream server to Nginx config: {}", server);
                config.push_str(&format!("        server {server};\n"));
            }
            config.push_str("    }\n\n");
        }

        // Server block
        config.push_str("    server {\n");
        config.push_str(&format!(
            "        listen {};\n",
            self.config.reverse_proxy_port
        ));

        if self.config.ssl_termination_enabled {
            config.push_str(&format!("        listen {} ssl;\n", 443));
            config.push_str(&format!(
                "        ssl_certificate {}/cert.pem;\n",
                self.config.ssl_cert_dir
            ));
            config.push_str(&format!(
                "        ssl_certificate_key {}/key.pem;\n",
                self.config.ssl_cert_dir
            ));
        }

        config.push_str(&format!(
            "        server_name {};\n",
            self.config.default_domain
        ));

        // CORS headers
        if self.config.cors_enabled {
            config.push_str("        add_header 'Access-Control-Allow-Origin' '*' always;\n");
            config.push_str(
                "        add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS' always;\n",
            );
            config.push_str("        add_header 'Access-Control-Allow-Headers' 'DNT,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range' always;\n");
        }

        // Security headers
        if self.config.security_headers_enabled {
            config.push_str("        add_header X-Frame-Options DENY always;\n");
            config.push_str("        add_header X-Content-Type-Options nosniff always;\n");
            config.push_str(&format!(
                "        add_header Content-Security-Policy '{}' always;\n",
                self.config.csp_header
            ));
        }

        // Custom headers
        for (name, value) in &self.config.custom_headers {
            config.push_str(&format!("        add_header '{name}' '{value}' always;\n"));
        }

        // Rate limiting
        if self.config.rate_limiting_enabled {
            config.push_str(&format!(
                "        limit_req zone=api burst={};\n",
                self.config.rate_limit.burst_size
            ));
        }

        // Proxy settings
        config.push_str("        location / {\n");
        if self.config.load_balancing_enabled {
            config.push_str("            proxy_pass http://backend;\n");
        } else {
            let proxy_endpoint =
                songbird_config::config::hardcoded_elimination::replace::orchestrator_endpoint();
            config.push_str(&format!("            proxy_pass {proxy_endpoint};\n"));
        }

        config.push_str("            proxy_set_header Host $host;\n");
        config.push_str("            proxy_set_header X-Real-IP $remote_addr;\n");
        config
            .push_str("            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;\n");
        config.push_str("            proxy_set_header X-Forwarded-Proto $scheme;\n");

        config.push_str(&format!(
            "            proxy_connect_timeout {}s;\n",
            self.config.connection_timeout.as_secs()
        ));
        config.push_str(&format!(
            "            proxy_send_timeout {}s;\n",
            self.config.request_timeout.as_secs()
        ));
        config.push_str(&format!(
            "            proxy_read_timeout {}s;\n",
            self.config.request_timeout.as_secs()
        ));

        config.push_str(&format!(
            "            client_max_body_size {};\n",
            self.config.max_request_size
        ));
        config.push_str("        }\n");

        config.push_str("    }\n");
        config.push_str("}\n");

        Ok(config)
    }

    /// Generate HAProxy configuration
    pub fn generate_haproxy_config(&self) -> Result<String, SongbirdError> {
        let mut config = String::new();

        // Global section
        config.push_str("global\n");
        config.push_str("    daemon\n");
        config.push_str(&format!(
            "    maxconn {}\n",
            self.config.max_upstream_connections
        ));
        config.push_str(&format!(
            "    log stdout local0 {}\n",
            self.config.error_log_level
        ));

        // Defaults
        config.push_str("\ndefaults\n");
        config.push_str("    mode http\n");
        config.push_str("    option httplog\n");
        config.push_str("    option dontlognull\n");
        config.push_str(&format!(
            "    timeout connect {}s\n",
            self.config.connection_timeout.as_secs()
        ));
        config.push_str(&format!(
            "    timeout client {}s\n",
            self.config.request_timeout.as_secs()
        ));
        config.push_str(&format!(
            "    timeout server {}s\n",
            self.config.request_timeout.as_secs()
        ));

        // Frontend
        config.push_str("\nfrontend http_frontend\n");
        config.push_str(&format!("    bind *:{}\n", self.config.reverse_proxy_port));

        if self.config.ssl_termination_enabled {
            config.push_str(&format!(
                "    bind *:443 ssl crt {}/cert.pem\n",
                self.config.ssl_cert_dir
            ));
        }

        // CORS headers
        if self.config.cors_enabled {
            config.push_str("    http-response set-header Access-Control-Allow-Origin *\n");
            config.push_str(
                "    http-response set-header Access-Control-Allow-Methods GET,POST,OPTIONS\n",
            );
        }

        // Rate limiting (simplified)
        if self.config.rate_limiting_enabled {
            config.push_str(
                "    stick-table type ip size 100k expire 30s store http_req_rate(10s)\n",
            );
            config.push_str("    http-request track-sc0 src\n");
            config.push_str(&format!(
                "    http-request reject if {{ sc_http_req_rate(0) gt {} }}\n",
                self.config.rate_limit.requests_per_minute / 6
            )); // Convert per minute to per 10 seconds
        }

        config.push_str("    default_backend http_backend\n");

        // Backend
        config.push_str("\nbackend http_backend\n");
        match self.config.load_balancing_strategy {
            super::config::LoadBalancingStrategy::RoundRobin => {
                config.push_str("    balance roundrobin\n");
            }
            super::config::LoadBalancingStrategy::LeastConnections => {
                config.push_str("    balance leastconn\n");
            }
            super::config::LoadBalancingStrategy::IpHash => {
                config.push_str("    balance source\n");
            }
        }

        if self.config.load_balancing_enabled && !self.config.upstream_servers.is_empty() {
            for (i, server) in self.config.upstream_servers.iter().enumerate() {
                info!("Adding HAProxy server #{}: {}", i + 1, server);
                config.push_str(&format!("    server server{i} {server} check\n"));
            }
        } else {
            let default_server = format!(
                "{}:8080",
                songbird_config::config::hardcoded_elimination::replace::bind_address()
            );
            config.push_str(&format!("    server default {default_server} check\n"));
        }

        Ok(config)
    }

    /// Generate Traefik configuration
    pub fn generate_traefik_config(&self) -> Result<String, SongbirdError> {
        let mut config = String::new();

        // Static configuration
        config.push_str("[global]\n");
        config.push_str("  checkNewVersion = false\n");
        config.push_str("  sendAnonymousUsage = false\n\n");

        // Entry points
        config.push_str("[entryPoints]\n");
        config.push_str("  [entryPoints.web]\n");
        config.push_str(&format!(
            "    address = \":{}\"\n",
            self.config.reverse_proxy_port
        ));

        if self.config.ssl_termination_enabled {
            config.push_str("  [entryPoints.websecure]\n");
            config.push_str("    address = \":443\"\n");
        }

        // Providers
        config.push_str("\n[providers]\n");
        config.push_str("  [providers.file]\n");
        config.push_str("    directory = \"/etc/traefik/dynamic\"\n");
        config.push_str("    watch = true\n");

        // API and dashboard
        if self.config.monitoring_enabled {
            config.push_str("\n[api]\n");
            config.push_str("  dashboard = true\n");
            config.push_str("  insecure = true\n");
        }

        // Access logs
        if self.config.access_logging_enabled {
            config.push_str("\n[accessLog]\n");
            config.push_str("  filePath = \"/var/log/traefik/access.log\"\n");
        }

        // Log level
        config.push_str("\n[log]\n");
        config.push_str(&format!(
            "  level = \"{}\"\n",
            self.config.error_log_level.to_uppercase()
        ));

        Ok(config)
    }

    /// Get proxy type recommendations
    pub fn get_proxy_recommendations(&self) -> HashMap<String, String> {
        let mut recommendations = HashMap::new();

        if self.config.load_balancing_enabled {
            recommendations.insert(
                "load_balancing".to_string(),
                "Consider using HAProxy for advanced load balancing features".to_string(),
            );
        }

        if self.config.ssl_termination_enabled {
            recommendations.insert(
                "ssl_termination".to_string(),
                "Nginx and HAProxy both provide excellent SSL termination".to_string(),
            );
        }

        if self.config.monitoring_enabled {
            recommendations.insert(
                "monitoring".to_string(),
                "Traefik provides built-in dashboard and metrics".to_string(),
            );
        }

        if self.config.rate_limiting_enabled {
            recommendations.insert(
                "rate_limiting".to_string(),
                "HAProxy provides more granular rate limiting options".to_string(),
            );
        }

        recommendations
    }
}
