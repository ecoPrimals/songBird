//! Simple Web Dashboard Module
//!
//! Optional lightweight web dashboard for viewing observability data

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde_json::json;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::net::TcpListener;
use tracing::{error, info, warn};

use crate::errors::{Result, SongbirdError};
use super::{health::HealthMonitor, metrics::MetricsCollector};

/// Simple dashboard implementation
pub struct SimpleDashboard {
    port: u16,
    metrics_collector: Arc<MetricsCollector>,
    health_monitor: Arc<HealthMonitor>,
    running: Arc<AtomicBool>,
}

impl SimpleDashboard {
    /// Create a new simple dashboard
    pub fn new(
        port: u16,
        metrics_collector: Arc<MetricsCollector>,
        health_monitor: Arc<HealthMonitor>,
    ) -> Self {
        Self {
            port,
            metrics_collector,
            health_monitor,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the dashboard web server
    pub async fn start(&self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            warn!("Dashboard already running");
            return Ok(());
        }

        info!("🌐 Starting Songbird Dashboard on port {}", self.port);
        self.running.store(true, Ordering::Relaxed);

        let app_state = DashboardState {
            metrics_collector: Arc::clone(&self.metrics_collector),
            health_monitor: Arc::clone(&self.health_monitor),
        };

        let app = Router::new()
            .route("/", get(dashboard_page))
            .route("/api/status", get(status_api))
            .route("/api/metrics", get(metrics_api))
            .route("/api/health", get(health_api))
            .route("/api/prometheus", get(prometheus_api))
            .with_state(app_state);

        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).await.map_err(|e| {
            SongbirdError::Network {
                message: format!("Failed to bind dashboard to {}: {}", addr, e),
            }
        })?;

        info!("✅ Songbird Dashboard available at http://localhost:{}", self.port);

        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                error!("Dashboard server error: {}", e);
            }
        });

        Ok(())
    }

    /// Stop the dashboard web server
    pub async fn stop(&self) -> Result<()> {
        if !self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        info!("🛑 Stopping Songbird Dashboard");
        self.running.store(false, Ordering::Relaxed);

        // Note: axum doesn't have a graceful shutdown mechanism built into the serve function
        // In a production implementation, we'd need to handle this differently
        Ok(())
    }
}

/// Dashboard application state
#[derive(Clone)]
pub struct DashboardState {
    metrics_collector: Arc<MetricsCollector>,
    health_monitor: Arc<HealthMonitor>,
}

/// Main dashboard page
async fn dashboard_page() -> impl IntoResponse {
    Html(DASHBOARD_HTML)
}

/// Status API endpoint
async fn status_api(State(state): State<DashboardState>) -> impl IntoResponse {
    match get_cluster_status(&state).await {
        Ok(status) => Json(status).into_response(),
        Err(e) => {
            error!("Failed to get cluster status: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response()
        }
    }
}

/// Metrics API endpoint
async fn metrics_api(State(state): State<DashboardState>) -> impl IntoResponse {
    match state.metrics_collector.get_current_snapshot().await {
        Ok(metrics) => Json(metrics).into_response(),
        Err(e) => {
            error!("Failed to get metrics: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response()
        }
    }
}

/// Health API endpoint
async fn health_api(State(state): State<DashboardState>) -> impl IntoResponse {
    match state.health_monitor.get_service_health().await {
        Ok(health) => Json(health).into_response(),
        Err(e) => {
            error!("Failed to get health status: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response()
        }
    }
}

/// Prometheus metrics endpoint
async fn prometheus_api(State(state): State<DashboardState>) -> impl IntoResponse {
    match state.metrics_collector.export_prometheus().await {
        Ok(prometheus_data) => {
            Response::builder()
                .header("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
                .body(prometheus_data)
                .unwrap_or_else(|e| {
                    error!("Failed to build Prometheus response: {}", e);
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("Error building response".to_string())
                        .unwrap_or_default()
                })
        }
        Err(e) => {
            error!("Failed to export Prometheus metrics: {}", e);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Error: {}", e))
                .unwrap_or_else(|build_err| {
                    error!("Failed to build error response: {}", build_err);
                    Response::default()
                })
        }
    }
}

/// Get cluster status for the dashboard
async fn get_cluster_status(state: &DashboardState) -> Result<serde_json::Value> {
    let metrics = state.metrics_collector.get_current_snapshot().await?;
    let service_health = state.health_monitor.get_service_health().await?;
    let health_stats = state.health_monitor.get_health_stats().await;

    Ok(json!({
        "timestamp": chrono::Utc::now(),
        "system": {
            "cpu_usage": metrics.system.cpu_usage,
            "memory_usage": (metrics.system.memory_usage * 100.0).round() / 100.0,
            "disk_usage": (metrics.system.disk_usage.usage_percentage).round() / 100.0,
            "uptime_seconds": metrics.system.uptime.as_secs(),
            "load_average": {
                "one": metrics.system.load_average.one,
                "five": metrics.system.load_average.five,
                "fifteen": metrics.system.load_average.fifteen
            }
        },
        "services": {
            "total": health_stats.total_services,
            "healthy": health_stats.healthy_services,
            "unhealthy": health_stats.unhealthy_services,
            "details": service_health
        },
        "application": {
            "active_services": metrics.songbird.active_services,
            "request_rate": metrics.songbird.request_rate,
            "error_rate": metrics.songbird.error_rate,
            "avg_response_time_ms": metrics.songbird.avg_response_time_ms
        },
        "collection": {
            "duration_ms": metrics.collection_duration_ms,
            "timestamp": metrics.timestamp
        }
    }))
}

/// Dashboard HTML template
const DASHBOARD_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>🎼 Songbird Orchestrator Dashboard</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #333;
            min-height: 100vh;
        }
        
        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
        }
        
        .header {
            text-align: center;
            margin-bottom: 40px;
            color: white;
        }
        
        .header h1 {
            font-size: 2.5rem;
            margin-bottom: 10px;
            text-shadow: 0 2px 4px rgba(0,0,0,0.3);
        }
        
        .header p {
            font-size: 1.1rem;
            opacity: 0.9;
        }
        
        .dashboard-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin-bottom: 40px;
        }
        
        .card {
            background: white;
            border-radius: 12px;
            padding: 24px;
            box-shadow: 0 8px 32px rgba(0,0,0,0.1);
            backdrop-filter: blur(10px);
            border: 1px solid rgba(255,255,255,0.2);
        }
        
        .card h2 {
            color: #4a5568;
            margin-bottom: 16px;
            font-size: 1.2rem;
            display: flex;
            align-items: center;
            gap: 8px;
        }
        
        .metric {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 8px 0;
            border-bottom: 1px solid #eee;
        }
        
        .metric:last-child {
            border-bottom: none;
        }
        
        .metric-label {
            color: #718096;
            font-weight: 500;
        }
        
        .metric-value {
            font-weight: 600;
            color: #2d3748;
        }
        
        .status-indicator {
            display: inline-block;
            width: 12px;
            height: 12px;
            border-radius: 50%;
            margin-right: 8px;
        }
        
        .status-healthy { background-color: #48bb78; }
        .status-degraded { background-color: #ed8936; }
        .status-unhealthy { background-color: #f56565; }
        .status-unknown { background-color: #a0aec0; }
        
        .services-list {
            max-height: 300px;
            overflow-y: auto;
        }
        
        .service-item {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 12px;
            margin: 8px 0;
            background: #f7fafc;
            border-radius: 8px;
            border-left: 4px solid #48bb78;
        }
        
        .service-item.unhealthy {
            border-left-color: #f56565;
        }
        
        .service-name {
            font-weight: 600;
            color: #2d3748;
        }
        
        .service-status {
            font-size: 0.875rem;
            color: #718096;
        }
        
        .loading {
            text-align: center;
            color: #718096;
            font-style: italic;
        }
        
        .error {
            background: #fed7d7;
            color: #c53030;
            padding: 12px;
            border-radius: 8px;
            margin: 16px 0;
        }
        
        .refresh-info {
            text-align: center;
            color: white;
            opacity: 0.8;
            margin-top: 20px;
        }
        
        @keyframes pulse {
            0% { opacity: 1; }
            50% { opacity: 0.5; }
            100% { opacity: 1; }
        }
        
        .updating {
            animation: pulse 1s infinite;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🎼 Songbird Orchestrator</h1>
            <p>Real-time System Monitoring & Service Health Dashboard</p>
        </div>
        
        <div class="dashboard-grid">
            <!-- System Metrics -->
            <div class="card">
                <h2>🖥️ System Metrics</h2>
                <div id="system-metrics">
                    <div class="loading">Loading system metrics...</div>
                </div>
            </div>
            
            <!-- Service Health -->
            <div class="card">
                <h2>🏥 Service Health</h2>
                <div id="service-health">
                    <div class="loading">Loading service health...</div>
                </div>
            </div>
            
            <!-- Application Metrics -->
            <div class="card">
                <h2>📊 Application Metrics</h2>
                <div id="app-metrics">
                    <div class="loading">Loading application metrics...</div>
                </div>
            </div>
            
            <!-- Services List -->
            <div class="card">
                <h2>🎵 Active Services</h2>
                <div id="services-list">
                    <div class="loading">Loading services...</div>
                </div>
            </div>
        </div>
        
        <div class="refresh-info">
            <p>Dashboard updates automatically every 5 seconds</p>
            <p>Last updated: <span id="last-updated">Never</span></p>
        </div>
    </div>

    <script>
        let updateInterval;
        let isUpdating = false;

        async function updateDashboard() {
            if (isUpdating) return;
            isUpdating = true;
            
            try {
                // Add updating indicator
                document.body.classList.add('updating');
                
                const [statusResponse, healthResponse] = await Promise.all([
                    fetch('/api/status'),
                    fetch('/api/health')
                ]);
                
                if (!statusResponse.ok || !healthResponse.ok) {
                    throw new Error('Failed to fetch data');
                }
                
                const status = await statusResponse.json();
                const health = await healthResponse.json();
                
                updateSystemMetrics(status.system);
                updateServiceHealth(status.services);
                updateAppMetrics(status.application);
                updateServicesList(health);
                
                document.getElementById('last-updated').textContent = new Date().toLocaleTimeString();
                
                // Clear any previous errors
                document.querySelectorAll('.error').forEach(el => el.remove());
                
            } catch (error) {
                console.error('Dashboard update failed:', error);
                showError('Failed to update dashboard: ' + error.message);
            } finally {
                document.body.classList.remove('updating');
                isUpdating = false;
            }
        }

        function updateSystemMetrics(system) {
            const container = document.getElementById('system-metrics');
            container.innerHTML = `
                <div class="metric">
                    <span class="metric-label">CPU Usage</span>
                    <span class="metric-value">${system.cpu_usage.toFixed(1)}%</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Memory Usage</span>
                    <span class="metric-value">${(system.memory_usage * 100).toFixed(1)}%</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Disk Usage</span>
                    <span class="metric-value">${system.disk_usage.toFixed(1)}%</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Load (1m)</span>
                    <span class="metric-value">${system.load_average.one.toFixed(2)}</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Uptime</span>
                    <span class="metric-value">${formatUptime(system.uptime_seconds)}</span>
                </div>
            `;
        }

        function updateServiceHealth(services) {
            const container = document.getElementById('service-health');
            const healthRatio = services.total > 0 ? (services.healthy / services.total) : 1;
            const statusClass = healthRatio >= 0.9 ? 'healthy' : healthRatio >= 0.7 ? 'degraded' : 'unhealthy';
            
            container.innerHTML = `
                <div class="metric">
                    <span class="metric-label">
                        <span class="status-indicator status-${statusClass}"></span>
                        Overall Status
                    </span>
                    <span class="metric-value">${(healthRatio * 100).toFixed(1)}%</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Total Services</span>
                    <span class="metric-value">${services.total}</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Healthy</span>
                    <span class="metric-value" style="color: #48bb78">${services.healthy}</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Unhealthy</span>
                    <span class="metric-value" style="color: #f56565">${services.unhealthy}</span>
                </div>
            `;
        }

        function updateAppMetrics(app) {
            const container = document.getElementById('app-metrics');
            container.innerHTML = `
                <div class="metric">
                    <span class="metric-label">Active Services</span>
                    <span class="metric-value">${app.active_services}</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Request Rate</span>
                    <span class="metric-value">${app.request_rate.toFixed(2)} req/s</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Error Rate</span>
                    <span class="metric-value">${app.error_rate.toFixed(2)} err/s</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Avg Response Time</span>
                    <span class="metric-value">${app.avg_response_time_ms.toFixed(1)} ms</span>
                </div>
            `;
        }

        function updateServicesList(health) {
            const container = document.getElementById('services-list');
            
            if (health.length === 0) {
                container.innerHTML = '<div class="loading">No services registered</div>';
                return;
            }
            
            container.innerHTML = `
                <div class="services-list">
                    ${health.map(service => `
                        <div class="service-item ${service.is_healthy ? '' : 'unhealthy'}">
                            <div>
                                <div class="service-name">${service.service_id}</div>
                                <div class="service-status">
                                    ${service.is_healthy ? '✅' : '❌'} ${service.message}
                                    ${service.response_time_ms ? ` (${service.response_time_ms}ms)` : ''}
                                </div>
                            </div>
                            <div>
                                <span class="status-indicator status-${service.is_healthy ? 'healthy' : 'unhealthy'}"></span>
                            </div>
                        </div>
                    `).join('')}
                </div>
            `;
        }

        function formatUptime(seconds) {
            const days = Math.floor(seconds / 86400);
            const hours = Math.floor((seconds % 86400) / 3600);
            const minutes = Math.floor((seconds % 3600) / 60);
            
            if (days > 0) return `${days}d ${hours}h`;
            if (hours > 0) return `${hours}h ${minutes}m`;
            return `${minutes}m`;
        }

        function showError(message) {
            const error = document.createElement('div');
            error.className = 'error';
            error.textContent = message;
            document.querySelector('.container').appendChild(error);
            
            setTimeout(() => error.remove(), 5000);
        }

        // Initialize dashboard
        updateDashboard();
        updateInterval = setInterval(updateDashboard, 5000);

        // Handle visibility changes to pause updates when tab is not visible
        document.addEventListener('visibilitychange', () => {
            if (document.hidden) {
                clearInterval(updateInterval);
            } else {
                updateDashboard();
                updateInterval = setInterval(updateDashboard, 5000);
            }
        });
    </script>
</body>
</html>
"#; 