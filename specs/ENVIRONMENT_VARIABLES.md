# Environment Variables

This document describes all environment variables that can be used to configure Songbird Orchestrator.

## **🔒 Security Configuration**

### OAuth2 Configuration
- `SONGBIRD_OAUTH_CLIENT_ID` - OAuth2 client ID (default: "songbird-orchestrator")
- `SONGBIRD_OAUTH_CLIENT_SECRET` - OAuth2 client secret (required for production)
- `SONGBIRD_OAUTH_AUTH_ENDPOINT` - OAuth2 authorization endpoint
- `SONGBIRD_OAUTH_TOKEN_ENDPOINT` - OAuth2 token endpoint  
- `SONGBIRD_OAUTH_USERINFO_ENDPOINT` - OAuth2 user info endpoint (optional)
- `SONGBIRD_OAUTH_REDIRECT_URI` - OAuth2 redirect URI (auto-generated if not set)
- `SONGBIRD_OAUTH_SCOPES` - Comma-separated list of OAuth2 scopes (default: "openid,profile,email")

### Zero Trust Configuration
- `SONGBIRD_ZERO_TRUST_ENABLED` - Enable zero trust security (default: true)
- `SONGBIRD_ZERO_TRUST_MAX_ATTEMPTS` - Max auth attempts before blocking (default: 5)
- `SONGBIRD_ZERO_TRUST_WINDOW` - Auth attempt window in seconds (default: 300)

## **🌐 Network Configuration**

### Bind Addresses
- `SONGBIRD_BIND_ADDRESS` - Main bind address (default: "localhost")
- `SONGBIRD_PORT` - Main service port (default: "8080")

### Discovery Endpoints
- `SONGBIRD_DISCOVERY_ENDPOINTS` - Comma-separated list of discovery endpoints
- `SONGBIRD_TEST_ENDPOINTS` - Comma-separated list of test endpoints for connectivity checks

### Federation
- `SONGBIRD_FEDERATION_ENABLED` - Enable federation (default: true)
- `SONGBIRD_FEDERATION_HEARTBEAT` - Heartbeat interval in seconds (default: 30)
- `SONGBIRD_FEDERATION_TIMEOUT` - Node timeout in seconds (default: 120)

## **📊 Observability**

### Metrics
- `SONGBIRD_METRICS_ENABLED` - Enable metrics collection (default: true)
- `SONGBIRD_METRICS_PORT` - Metrics server port (default: "9090")
- `SONGBIRD_METRICS_ADDRESS` - Metrics bind address (default: "localhost")

### Logging
- `SONGBIRD_LOG_LEVEL` - Log level (debug, info, warn, error) (default: "info")
- `SONGBIRD_LOG_FORMAT` - Log format (json, text) (default: "text")

## **🏗️ Deployment Configuration**

### Container Orchestration
- `SONGBIRD_DEPLOYMENT_STRATEGY` - Deployment strategy (docker, kubernetes, systemd, standalone)
- `SONGBIRD_NAMESPACE` - Kubernetes namespace (default: "songbird")
- `SONGBIRD_CONTAINER_REGISTRY` - Container registry URL

### Environment Detection
- `SONGBIRD_ENVIRONMENT` - Deployment environment (development, staging, production)
- `SONGBIRD_CLOUD_PLATFORM` - Cloud platform (aws, gcp, azure, local)

## **🔧 Service Configuration**

### HTTP Server
- `SONGBIRD_HTTP_TIMEOUT` - HTTP request timeout in seconds (default: 30)
- `SONGBIRD_HTTP_MAX_CONNECTIONS` - Maximum HTTP connections (default: 1000)
- `SONGBIRD_HTTP_KEEP_ALIVE` - HTTP keep-alive timeout in seconds (default: 60)

### Load Balancer
- `SONGBIRD_LB_STRATEGY` - Load balancing strategy (round_robin, least_connections, weighted)
- `SONGBIRD_LB_HEALTH_CHECK_INTERVAL` - Health check interval in seconds (default: 10)

### Proxy
- `SONGBIRD_PROXY_ENABLED` - Enable proxy functionality (default: true)
- `SONGBIRD_PROXY_BUFFER_SIZE` - Proxy buffer size in bytes (default: 8192)

## **📁 File System**

### Data Directories
- `SONGBIRD_DATA_DIR` - Data directory path (default: "./data")
- `SONGBIRD_CONFIG_DIR` - Configuration directory path (default: "./config")
- `SONGBIRD_LOG_DIR` - Log directory path (default: "./logs")

### Certificates
- `SONGBIRD_TLS_CERT_PATH` - TLS certificate file path
- `SONGBIRD_TLS_KEY_PATH` - TLS private key file path
- `SONGBIRD_CA_CERT_PATH` - Certificate Authority file path

## **🚀 Performance Tuning**

### Concurrency
- `SONGBIRD_WORKER_THREADS` - Number of worker threads (default: CPU cores)
- `SONGBIRD_MAX_CONCURRENT_REQUESTS` - Maximum concurrent requests (default: 10000)

### Memory
- `SONGBIRD_MEMORY_LIMIT` - Memory limit in MB (default: 512)
- `SONGBIRD_CACHE_SIZE` - Cache size in MB (default: 100)

## **🔄 Backward Compatibility**

The following legacy environment variables are still supported but deprecated:

- `ORCHESTRATOR_PORT` → Use `SONGBIRD_PORT`
- `FEDERATION_ENABLED` → Use `SONGBIRD_FEDERATION_ENABLED`
- `BIND_ADDRESS` → Use `SONGBIRD_BIND_ADDRESS`

## **⚡ Quick Start Examples**

### Development Environment
```bash
export SONGBIRD_ENVIRONMENT=development
export SONGBIRD_LOG_LEVEL=debug
export SONGBIRD_BIND_ADDRESS=localhost
export SONGBIRD_PORT=8080
```

### Production Environment
```bash
export SONGBIRD_ENVIRONMENT=production
export SONGBIRD_LOG_LEVEL=info
export SONGBIRD_BIND_ADDRESS=0.0.0.0
export SONGBIRD_PORT=8080
export SONGBIRD_OAUTH_CLIENT_SECRET=your-production-secret
export SONGBIRD_ZERO_TRUST_ENABLED=true
```

### High Availability Setup
```bash
export SONGBIRD_FEDERATION_ENABLED=true
export SONGBIRD_DISCOVERY_ENDPOINTS=http://node1:8080,http://node2:8080,http://node3:8080
export SONGBIRD_LB_STRATEGY=least_connections
export SONGBIRD_METRICS_ENABLED=true
```

## **🛡️ Security Best Practices**

1. **Never commit secrets to version control**
2. **Use production-grade OAuth2 providers**
3. **Enable zero trust in production environments**
4. **Restrict bind addresses appropriately**
5. **Use TLS certificates for production**
6. **Monitor authentication attempts**
7. **Regularly rotate OAuth2 client secrets**

## **🐛 Troubleshooting**

### Common Issues

**OAuth2 Authentication Fails**
- Check `SONGBIRD_OAUTH_CLIENT_SECRET` is set
- Verify OAuth2 endpoints are accessible
- Ensure redirect URI matches configuration

**Federation Not Working**
- Verify `SONGBIRD_FEDERATION_ENABLED=true`
- Check discovery endpoints are reachable
- Ensure firewall allows required ports

**Metrics Not Available**
- Check `SONGBIRD_METRICS_ENABLED=true`
- Verify metrics port is not in use
- Ensure metrics address is accessible

### Debug Mode
```bash
export SONGBIRD_LOG_LEVEL=debug
export SONGBIRD_ENVIRONMENT=development
```

This will provide detailed logging for troubleshooting configuration issues. 