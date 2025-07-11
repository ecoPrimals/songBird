# 🎼 Songbird Orchestrator - Internal Monitoring Architecture

## 🏗️ Pure Rust Monitoring System

Songbird Orchestrator features a **comprehensive built-in observability system** that is pure Rust and requires **zero external dependencies**. This internal monitoring system provides enterprise-grade observability without needing external services like Prometheus, Grafana, or Jaeger.

## 🎯 Design Philosophy

### **Self-Contained & Local-First**
- **Pure Rust Implementation**: All monitoring components written in Rust
- **Zero External Dependencies**: No requirement for external monitoring services
- **Local Data Storage**: Metrics stored in memory with configurable history
- **Embedded Dashboard**: Optional built-in web dashboard
- **Real-time Monitoring**: Live metrics collection and health monitoring

### **Optional External Integration**
- **Prometheus Export**: Optional feature for compatibility (disabled by default)
- **OpenTelemetry Support**: Optional external observability integration
- **Standards Compliance**: Can export data in standard formats when needed

## 📊 Internal Monitoring Components

### 1. **ObservabilityEngine** (`src/observability/mod.rs`)
**Core monitoring orchestrator that manages all observability features**

```rust
pub struct ObservabilityEngine {
    config: ObservabilityConfig,
    metrics_collector: Arc<MetricsCollector>,
    health_monitor: Arc<HealthMonitor>,
    dashboard: Option<Arc<SimpleDashboard>>,
    // ... internal state
}
```

**Key Features:**
- Centralized observability management
- Event-driven architecture with real-time notifications
- Configurable collection intervals and retention
- Automatic service registration/unregistration
- Built-in cluster status aggregation

### 2. **MetricsCollector** (`src/observability/metrics.rs`)
**Pure Rust metrics collection and storage**

**System Metrics:**
- CPU usage and load averages
- Memory usage and allocation patterns
- Disk I/O and storage utilization
- Network interface statistics
- Process-level metrics (PID, threads, handles)

**Application Metrics:**
- Active service count and health status
- Request rates and response times
- Error rates and failure patterns
- Load balancing distribution
- Circuit breaker activations

**Storage & Retrieval:**
- In-memory time-series storage
- Configurable retention policies
- Efficient metric aggregation
- Real-time metric snapshots

### 3. **HealthMonitor** (`src/observability/health.rs`)
**Comprehensive health monitoring and status tracking**

**Service Health:**
- Individual service health status
- Health check automation with configurable intervals
- Health trend analysis and alerting
- Service dependency health tracking

**Node Health:**
- System resource health thresholds
- Network connectivity monitoring
- Service discovery health
- Federation health (if enabled)

**Cluster Health:**
- Overall cluster status calculation
- Health aggregation across all components
- Automatic health status determination (Healthy/Degraded/Unhealthy)

### 4. **SimpleDashboard** (`src/observability/dashboard.rs`)
**Built-in web dashboard for real-time monitoring**

**Features:**
- Modern responsive web interface
- Real-time metrics visualization
- Interactive service health status
- System resource monitoring
- REST API for programmatic access

**Endpoints:**
- `GET /` - Dashboard web interface
- `GET /api/metrics` - Current metrics snapshot
- `GET /api/health` - Cluster health status
- `GET /api/services` - Service status overview
- `GET /api/prometheus` - Prometheus format export (if enabled)

## 🔧 Configuration

### **Default Configuration (Pure Rust)**
```rust
ObservabilityConfig {
    enabled: true,                    // Enable internal monitoring
    metrics_interval_secs: 30,        // Collect metrics every 30s
    health_check_interval_secs: 60,   // Health checks every 60s
    enable_dashboard: false,          // Dashboard disabled by default
    dashboard_port: 8081,             // Dashboard port when enabled
    export_prometheus: false,         // Prometheus export DISABLED by default
    max_metric_history: 1000,         // Keep 1000 data points in memory
    enable_system_metrics: true,      // Collect system metrics
    enable_service_metrics: true,     // Collect service metrics
}
```

### **Environment Variables**
```bash
# Core observability settings
SONGBIRD_OBSERVABILITY_ENABLED=true
SONGBIRD_METRICS_INTERVAL_SECS=30
SONGBIRD_HEALTH_CHECK_INTERVAL_SECS=60

# Dashboard settings
SONGBIRD_ENABLE_DASHBOARD=false
SONGBIRD_DASHBOARD_PORT=8081

# Optional external integrations (disabled by default)
SONGBIRD_EXPORT_PROMETHEUS=false

# Metric storage settings
SONGBIRD_MAX_METRIC_HISTORY=1000
SONGBIRD_ENABLE_SYSTEM_METRICS=true
SONGBIRD_ENABLE_SERVICE_METRICS=true
```

## 🚀 Usage Examples

### **Basic Internal Monitoring**
```rust
use songbird_orchestrator::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create orchestrator with default internal monitoring
    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await?;
    
    // Start orchestrator (automatically starts internal monitoring)
    orchestrator.start().await?;
    
    // Get real-time cluster status
    let cluster_status = orchestrator.get_cluster_status().await?;
    println!("Cluster Health: {:?}", cluster_status.overall_status);
    println!("Active Services: {}", cluster_status.running_services());
    
    // Subscribe to real-time observability events
    let mut events = orchestrator.subscribe_observability_events();
    while let Ok(event) = events.recv().await {
        match event {
            ObservabilityEvent::MetricsCollected { timestamp, duration_ms } => {
                println!("Metrics collected in {}ms at {}", duration_ms, timestamp);
            }
            ObservabilityEvent::HealthCheckCompleted { service_id, is_healthy, .. } => {
                println!("Service {} health: {}", service_id, is_healthy);
            }
            _ => {}
        }
    }
    
    Ok(())
}
```

### **Enable Built-in Dashboard**
```rust
let mut config = OrchestratorConfig::default();
config.observability.enable_dashboard = true;
config.observability.dashboard_port = 8081;

let orchestrator = Orchestrator::new(config).await?;
orchestrator.start().await?;

// Dashboard available at http://localhost:8081
if let Some(dashboard_url) = orchestrator.get_dashboard_url() {
    println!("📊 Dashboard: {}", dashboard_url);
    println!("📈 Metrics API: {}/api/metrics", dashboard_url);
    println!("🏥 Health API: {}/api/health", dashboard_url);
}
```

### **Optional Prometheus Export**
```rust
// Only enable if you need Prometheus compatibility
let mut config = OrchestratorConfig::default();
config.observability.export_prometheus = true;

let orchestrator = Orchestrator::new(config).await?;
orchestrator.start().await?;

// Export metrics in Prometheus format (optional)
let prometheus_metrics = orchestrator
    .observability()
    .export_prometheus()
    .await?;

println!("Prometheus format metrics:\n{}", prometheus_metrics);
```

## 📈 Metrics Available

### **System Metrics**
```
songbird_cpu_usage_percent          # CPU utilization percentage
songbird_memory_usage_ratio         # Memory usage ratio (0.0-1.0)
songbird_memory_total_bytes         # Total system memory
songbird_memory_available_bytes     # Available system memory
songbird_disk_usage_bytes          # Disk usage per mount point
songbird_network_bytes_sent        # Network bytes sent
songbird_network_bytes_received    # Network bytes received
songbird_load_average_1m           # 1-minute load average
songbird_load_average_5m           # 5-minute load average
songbird_load_average_15m          # 15-minute load average
```

### **Application Metrics**
```
songbird_active_services           # Number of active services
songbird_healthy_services          # Number of healthy services
songbird_request_rate              # Requests per second
songbird_error_rate                # Error rate percentage
songbird_response_time_ms          # Average response time
songbird_circuit_breaker_trips     # Circuit breaker activations
songbird_uptime_seconds            # System uptime
```

### **Service-Level Metrics**
```
songbird_service_health{service_id} # Per-service health status
songbird_service_requests{service_id} # Per-service request count
songbird_service_errors{service_id}   # Per-service error count
songbird_service_response_time{service_id} # Per-service response time
```

## 🔄 Real-Time Events

### **ObservabilityEvent Types**
```rust
pub enum ObservabilityEvent {
    MetricsCollected { 
        timestamp: DateTime<Utc>,
        duration_ms: u64,
    },
    HealthCheckCompleted {
        service_id: String,
        is_healthy: bool,
        response_time_ms: u64,
    },
    DashboardStarted {
        port: u16,
    },
    SystemAlert {
        message: String,
        severity: String,
    },
}
```

## 🎯 Benefits of Internal Monitoring

### **✅ Advantages**
- **Zero External Dependencies**: No need for Prometheus, Grafana, or other tools
- **Pure Rust Performance**: Native performance with minimal overhead
- **Self-Contained**: Everything needed for monitoring is built-in
- **Real-Time**: Live metrics and health monitoring
- **Local Data**: No external data storage requirements
- **Easy Deployment**: No additional services to configure or maintain
- **Security**: All monitoring data stays within your application
- **Customizable**: Full control over metrics collection and retention

### **🔧 When to Use External Integration**
- **Existing Infrastructure**: You already have Prometheus/Grafana setup
- **Multi-System Monitoring**: Need to aggregate metrics from multiple systems
- **Long-Term Storage**: Need metrics retention beyond memory limits
- **Advanced Visualization**: Need complex dashboards beyond built-in UI
- **Compliance Requirements**: Specific monitoring tool requirements

## 🏗️ Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Songbird Orchestrator                    │
├─────────────────────────────────────────────────────────────┤
│                 ObservabilityEngine                        │
├─────────────────┬─────────────────┬─────────────────────────┤
│ MetricsCollector│  HealthMonitor  │    SimpleDashboard      │
│                 │                 │                         │
│ • System Metrics│ • Service Health│ • Web Interface         │
│ • App Metrics   │ • Node Health   │ • REST APIs             │
│ • Time Series   │ • Cluster Health│ • Real-time Updates     │
│ • Aggregation   │ • Health Events │ • Responsive UI         │
└─────────────────┴─────────────────┴─────────────────────────┘
                           │
                    ┌──────┴──────┐
                    │   Optional  │
                    │ External    │
                    │ Exports     │
                    └─────────────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
         Prometheus    OpenTelemetry   Custom
         Format        Export          Integrations
```

## 🎉 Summary

Songbird Orchestrator's **internal monitoring system** provides:

✅ **Complete Observability**: System, application, and service-level metrics
✅ **Pure Rust Implementation**: No external dependencies required
✅ **Real-Time Monitoring**: Live metrics collection and health tracking
✅ **Built-in Dashboard**: Optional web interface for visualization
✅ **Event-Driven Architecture**: Real-time notifications and alerts
✅ **Configurable Retention**: In-memory storage with configurable limits
✅ **Optional External Export**: Prometheus/OpenTelemetry compatibility when needed

**Result**: Enterprise-grade monitoring that is self-contained, performant, and requires zero external infrastructure while maintaining compatibility with existing monitoring ecosystems when needed. 