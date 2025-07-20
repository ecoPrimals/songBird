# 📚 **SONGBIRD API REFERENCE**

**Version**: 0.1.0  
**Status**: Production Ready  
**Last Updated**: January 2025  

---

## 🎯 **QUICK START FOR LIVE TESTING**

### **Core APIs Available**
- **🎮 Gaming Auto-Configuration API** - One-touch gaming setup
- **🤖 AI-First Service Mesh API** - Intelligent workload orchestration  
- **🔍 Universal Primal Discovery API** - Dynamic primal integration
- **🌐 Federation Management API** - Multi-node coordination
- **🔒 Security & Authentication API** - Universal security integration
- **📊 Observability API** - Real-time monitoring and metrics

---

## 🎮 **GAMING AUTO-CONFIGURATION API**

### **Core Gaming Setup**

#### **`GamingAutoConfig::setup_one_touch()`**
**Purpose**: Automated gaming network setup with universal primal integration

**Rust Usage**:
```rust
use songbird_network::network::gaming::GamingAutoConfig;

// Create auto-configurator
let mut auto_config = GamingAutoConfig::new().await?;

// Perform one-touch setup
let result = auto_config.setup_one_touch().await?;

if result.success {
    println!("Gaming setup completed: {}", result.message);
    for step in result.next_steps {
        println!("  → {}", step);
    }
} else {
    for warning in result.warnings {
        println!("  ⚠️  {}", warning);
    }
}
```

**HTTP API Endpoint**:
```http
POST /api/gaming/setup
Content-Type: application/json

{
  "setup_type": "one_touch",
  "user_preferences": {
    "family_safe_mode": false,
    "allow_guests": true
  }
}
```

**Response**:
```json
{
  "success": true,
  "message": "Gaming setup completed successfully",
  "configuration": {
    "primal_type": "beardog",
    "endpoint": "https://beardog.example.com:8443",
    "auto_configured": true
  },
  "next_steps": [
    "Gaming network ready",
    "Auto-detection enabled"
  ],
  "warnings": []
}
```

#### **`GamingAutoConfig::configure_for_game(game_name)`**
**Purpose**: Game-specific optimization

**Rust Usage**:
```rust
// Configure for specific game
let config = auto_config.configure_for_game("StarCraft").await?;
```

**HTTP Endpoint**:
```http
POST /api/gaming/configure
{
  "game_name": "StarCraft",
  "optimization_level": "maximum"
}
```

---

## 🤖 **AI-FIRST SERVICE MESH API**

### **Workload Classification**

#### **`WorkloadClassifier::classify_workload()`**
**Purpose**: Intelligent workload analysis and routing decisions

**Rust Usage**:
```rust
use songbird_core::api::ai_workload_classification::WorkloadClassificationEngine;

let classifier = WorkloadClassificationEngine::new().await;
let request = WorkloadClassificationRequest {
    workload_id: "web-service-1".to_string(),
    characteristics: vec!["high_throughput", "low_latency"],
    resource_requirements: ResourceRequirements {
        cpu_cores: 4,
        memory_gb: 8,
        storage_gb: 100,
    },
};

let result = classifier.classify_workload(&request).await?;
```

**HTTP API**:
```http
POST /api/ai/classify
{
  "workload_id": "web-service-1", 
  "characteristics": ["high_throughput", "low_latency"],
  "resource_requirements": {
    "cpu_cores": 4,
    "memory_gb": 8,
    "storage_gb": 100
  }
}
```

**Response**:
```json
{
  "classification": {
    "primary_type": "HighThroughputService",
    "confidence": 0.95,
    "recommended_placement": "performance_optimized_cluster",
    "scaling_strategy": "horizontal_auto",
    "monitoring_profile": "intensive"
  },
  "ai_rationale": "High confidence classification based on throughput requirements...",
  "evidence": [
    "CPU requirement indicates compute-intensive workload",
    "Low latency requirement suggests real-time service"
  ]
}
```

---

## 🔍 **UNIVERSAL PRIMAL DISCOVERY API**

### **Dynamic Primal Registration**

#### **`PrimalRegistry::register_primal()`**
**Purpose**: Register any type of primal dynamically

**Rust Usage**:
```rust
use songbird_config::config::{PrimalRegistry, PrimalConfiguration};

let mut registry = PrimalRegistry::default();

let primal_config = PrimalConfiguration {
    primal_type: "custom-ai-primal".to_string(),
    display_name: "Custom AI Service".to_string(),
    enabled: true,
    endpoint: PrimalEndpoint {
        primary_url: "https://ai-service.example.com:9000".to_string(),
        fallback_urls: vec![],
        health_check_path: "/health".to_string(),
    },
    capabilities: vec![
        PrimalCapability::ModelInference { 
            models: vec!["gpt-4".to_string(), "claude-3".to_string()] 
        }
    ],
    // ... other configuration
};

registry.register_primal(primal_config);
```

**HTTP API**:
```http
POST /api/primals/register
{
  "primal_type": "custom-ai-primal",
  "display_name": "Custom AI Service",
  "endpoint": {
    "primary_url": "https://ai-service.example.com:9000",
    "health_check_path": "/health"
  },
  "capabilities": [
    {
      "type": "ModelInference",
      "models": ["gpt-4", "claude-3"]
    }
  ]
}
```

#### **`PrimalDiscovery::discover_primals()`**
**Purpose**: Auto-discover available primals on network

**Rust Usage**:
```rust
use songbird_universal_primals::discovery::PrimalDiscoveryEngine;

let discovery = PrimalDiscoveryEngine::new().await;
let discovered = discovery.discover_primals().await?;

for primal in discovered {
    println!("Found: {} at {}", primal.primal_type, primal.endpoint);
    println!("  Capabilities: {:?}", primal.capabilities);
}
```

**HTTP API**:
```http
GET /api/primals/discover
```

**Response**:
```json
{
  "discovered_primals": [
    {
      "primal_type": "beardog",
      "display_name": "BearDog Security Service",
      "endpoint": "https://beardog.local:8443",
      "capabilities": ["security", "encryption", "authentication"],
      "health_status": "healthy",
      "discovery_method": "mdns",
      "last_seen": "2025-01-19T19:30:00Z"
    }
  ],
  "discovery_stats": {
    "total_found": 3,
    "healthy_count": 3,
    "discovery_time_ms": 1250
  }
}
```

---

## 🌐 **FEDERATION MANAGEMENT API**

### **Multi-Node Coordination**

#### **`FederationManager::join_federation()`**
**Purpose**: Join or create a distributed federation

**Rust Usage**:
```rust
use songbird_federation::FederationManager;

let config = FederationConfig {
    cluster_id: "production-cluster".to_string(),
    node_id: "node-1".to_string(),
    cluster_endpoints: vec![
        "https://node-2.example.com:8080".to_string(),
        "https://node-3.example.com:8080".to_string(),
    ],
    auto_discovery: true,
    heartbeat_interval: Some(30),
    connection_timeout: 10,
    max_retries: 3,
};

let federation = FederationManager::new(config).await?;
federation.start().await?;
```

**HTTP API**:
```http
POST /api/federation/join
{
  "cluster_id": "production-cluster",
  "node_id": "node-1", 
  "cluster_endpoints": [
    "https://node-2.example.com:8080",
    "https://node-3.example.com:8080"
  ],
  "auto_discovery": true,
  "heartbeat_interval": 30
}
```

#### **`FederationManager::get_cluster_status()`**
**Purpose**: Monitor federation health and topology

**HTTP API**:
```http
GET /api/federation/status
```

**Response**:
```json
{
  "cluster_status": {
    "cluster_id": "production-cluster",
    "node_count": 3,
    "healthy_nodes": 3,
    "cluster_health": 1.0,
    "last_heartbeat": "2025-01-19T19:30:00Z"
  },
  "nodes": [
    {
      "node_id": "node-1",
      "status": "online",
      "last_seen": "2025-01-19T19:30:00Z",
      "load": 0.45,
      "services": 12
    }
  ],
  "topology": {
    "connections": 6,
    "avg_latency_ms": 15,
    "total_services": 36
  }
}
```

---

## 🔒 **SECURITY & AUTHENTICATION API**

### **Universal Security Integration**

#### **`UniversalSecurityIntegration::authenticate()`**
**Purpose**: Universal authentication across any primal type

**Rust Usage**:
```rust
use songbird_security::UniversalSecurityIntegration;

let security = UniversalSecurityIntegration::new(security_primal_config).await?;
let authenticated = security.authenticate("user123", "credentials").await?;
```

**HTTP API**:
```http
POST /api/security/authenticate
{
  "username": "user123",
  "credentials": "...",
  "primal_type": "beardog"
}
```

#### **`SecurityIntegration::create_secure_tunnel()`**
**Purpose**: Establish encrypted communication tunnels

**HTTP API**:
```http
POST /api/security/tunnel
{
  "remote_endpoint": "https://remote-service.example.com:8443",
  "encryption_level": "maximum",
  "tunnel_type": "beardog_secure"
}
```

---

## 📊 **OBSERVABILITY API**

### **Real-Time Monitoring**

#### **`ObservabilityManager::get_metrics()`**
**Purpose**: Get comprehensive system metrics

**HTTP API**:
```http
GET /api/metrics
```

**Response**:
```json
{
  "system_metrics": {
    "timestamp": 1705598400,
    "cpu_usage": 45.2,
    "memory_usage": 67.8,
    "memory_total_gb": 32,
    "storage_available_gb": 150,
    "uptime_seconds": 86400,
    "service_count": 24,
    "active_connections": 156
  },
  "federation_metrics": {
    "cluster_health": 0.98,
    "node_count": 3,
    "total_services": 72,
    "avg_response_time_ms": 12
  },
  "gaming_metrics": {
    "active_sessions": 8,
    "protocols_supported": ["ipx", "directplay", "tcp", "udp"],
    "avg_latency_ms": 2.1
  }
}
```

#### **`ObservabilityManager::get_health_status()`**
**Purpose**: Comprehensive health assessment

**HTTP API**:
```http
GET /api/health
```

**Response**:
```json
{
  "overall_health": "healthy",
  "components": {
    "gaming_bridge": {
      "status": "healthy",
      "active_protocols": 4,
      "sessions": 8
    },
    "federation": {
      "status": "healthy", 
      "cluster_health": 0.98,
      "nodes_online": 3
    },
    "primals": {
      "status": "healthy",
      "registered": 5,
      "healthy": 5
    }
  },
  "alerts": [],
  "recommendations": [
    "Consider scaling up gaming services during peak hours"
  ]
}
```

---

## 🧪 **TESTING ENDPOINTS FOR LIVE VALIDATION**

### **Gaming Tests**
```http
# Test one-touch gaming setup
POST /api/gaming/setup
{"setup_type": "one_touch"}

# Test family-safe mode
POST /api/gaming/setup  
{"setup_type": "family_safe", "family_name": "TestFamily"}

# Test game-specific optimization
POST /api/gaming/configure
{"game_name": "StarCraft", "optimization_level": "maximum"}

# Test gaming session status
GET /api/gaming/sessions
```

### **Federation Tests**
```http
# Test federation join
POST /api/federation/join
{"cluster_id": "test-cluster", "node_id": "test-node"}

# Test cluster health
GET /api/federation/status

# Test heartbeat
POST /api/federation/heartbeat
```

### **Primal Discovery Tests**
```http
# Test primal discovery
GET /api/primals/discover

# Test primal registration
POST /api/primals/register
{"primal_type": "test-primal", "endpoint": {"primary_url": "http://test:8080"}}

# Test primal health
GET /api/primals/health
```

### **Load Testing Endpoints**
```http
# Stress test AI classification
POST /api/ai/classify/batch
{"workloads": [...]} # Array of workload classification requests

# Stress test federation
GET /api/federation/load-test?duration=60&connections=100

# Stress test gaming protocols
POST /api/gaming/stress-test
{"protocol": "ipx", "sessions": 50, "duration": 120}
```

---

## 🔧 **CONFIGURATION EXAMPLES**

### **Production Configuration**
```toml
# songbird.toml
[primal_registry]
auto_discovery = true
default_timeout = 30

[[primal_registry.primals]]
primal_type = "beardog"
display_name = "BearDog Security"
enabled = true
endpoint = { primary_url = "https://beardog.prod.example.com:8443" }

[[primal_registry.primals]]
primal_type = "toadstool"
display_name = "Toadstool Compute" 
enabled = true
endpoint = { primary_url = "https://toadstool.prod.example.com:8080" }

[federation]
cluster_id = "production"
auto_discovery = true
heartbeat_interval = 30

[gaming]
family_safe_mode = false
auto_detect_games = true
protocols = ["ipx", "directplay", "tcp", "udp"]

[security]
encryption_enabled = true
authentication_required = true
audit_level = "comprehensive"
```

### **Development/Testing Configuration**
```toml
# songbird-dev.toml
[primal_registry]
auto_discovery = true
development_mode = true

[[primal_registry.primals]]
primal_type = "mock-beardog"
enabled = true
endpoint = { primary_url = "http://localhost:8443" }

[federation]
cluster_id = "dev-cluster"
development_mode = true

[gaming]
development_mode = true
mock_games = ["StarCraft", "Diablo", "AgeOfEmpires"]
```

---

## 🚨 **ERROR CODES & TROUBLESHOOTING**

### **Common Error Codes**
- **`GAMING_001`**: Gaming auto-configuration failed
- **`FEDERATION_002`**: Unable to join federation cluster
- **`PRIMAL_003`**: Primal discovery timeout
- **`SECURITY_004`**: Authentication failed
- **`AI_005`**: Workload classification error

### **Quick Fixes**
- **503 Service Unavailable**: Check primal endpoint health
- **Connection Timeout**: Verify network connectivity and firewall rules
- **Authentication Failed**: Validate primal credentials and certificates
- **Classification Error**: Check workload request format and requirements

---

## 📈 **PERFORMANCE BENCHMARKS**

### **Expected Performance**
- **Gaming Setup**: < 2 seconds (one-touch)
- **Primal Discovery**: < 5 seconds (local network)
- **Federation Join**: < 10 seconds
- **AI Classification**: < 100ms per request
- **Health Check**: < 50ms

### **Scalability Limits**
- **Concurrent Gaming Sessions**: 1000+
- **Federation Nodes**: 100+
- **Registered Primals**: Unlimited
- **AI Classifications/sec**: 10,000+

---

**🎯 This API reference enables comprehensive live testing of all Songbird capabilities!** 