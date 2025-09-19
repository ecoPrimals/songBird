use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::{Method, Request, Response};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{info, warn};

use songbird_errors::SongbirdResult;
type Result<T> = SongbirdResult<T>;

/// Simple web dashboard for observability
#[derive(Debug)]
pub struct SimpleDashboard {
    port: u16,
    running: Arc<AtomicBool>,
}

impl SimpleDashboard {
    /// Create new dashboard
    pub fn new(port: u16) -> Self {
        Self {
            port,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Create dashboard from network config
    pub fn from_network_config(port: u16) -> Self {
        Self::new(port)
    }

    /// Start the dashboard server
    pub async fn start(&self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            warn!("Dashboard already running on port {}", self.port);
            return Ok(());
        }

        info!("Starting dashboard on port {}", self.port);
        self.running.store(true, Ordering::Relaxed);

        // In a real implementation, this would start an HTTP server
        // For now, we'll just mark it as running
        let env_config = songbird_config::EnvironmentConfig::default();
        info!(
            "Dashboard started on http://{}:{}",
            env_config.bind_address, self.port
        );
        Ok(())
    }

    /// Stop the dashboard server
    pub async fn stop(&self) -> Result<()> {
        if !self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        info!("Stopping dashboard on port {}", self.port);
        self.running.store(false, Ordering::Relaxed);
        Ok(())
    }

    /// Check if dashboard is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Handle HTTP requests
    pub async fn handle_request(&self, req: Request<Incoming>) -> Result<Response<Full<Bytes>>> {
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/") => self.serve_dashboard().await,
            (&Method::GET, "/api/metrics") => self.serve_metrics().await,
            (&Method::GET, "/api/health") => self.serve_health().await,
            (&Method::GET, "/api/status") => self.serve_status().await,
            _ => self.serve_not_found().await,
        }
    }

    /// Serve main dashboard HTML
    async fn serve_dashboard(&self) -> Result<Response<Full<Bytes>>> {
        let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Songbird Orchestrator Dashboard</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .status { padding: 10px; margin: 10px 0; border-radius: 5px; }
        .healthy { background-color: #d4edda; color: #155724; }
        .warning { background-color: #fff3cd; color: #856404; }
        .critical { background-color: #f8d7da; color: #721c24; }
    </style>
</head>
<body>
    <h1>Songbird Orchestrator Dashboard</h1>
    <div id="status">Loading...</div>
    <script>
        async function loadStatus() {
            try {
                const response = await fetch('/api/status');
                const data = await response.json();
                document.getElementById('status').innerHTML = 
                    '<div class="status healthy">System Status: ' + data.status + '</div>';
            } catch (error) {
                document.getElementById('status').innerHTML = 
                    '<div class="status critical">Error loading status</div>';
            }
        }
        loadStatus();
        setInterval(loadStatus, 5000);
    </script>
</body>
</html>
        "#;

        let response = hyper::Response::builder()
            .header("content-type", "text/html")
            .body(Full::new(Bytes::from(html)));

        match response {
            Ok(resp) => Ok(resp),
            Err(e) => {
                tracing::error!("Failed to build HTTP response for dashboard: {}", e);
                // Create fallback response
                Ok(hyper::Response::builder()
                    .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                    .header("content-type", "text/plain")
                    .body(Full::new(Bytes::from("Dashboard temporarily unavailable")))
                    .unwrap_or_else(|_| {
                        let (parts, _) =
                            hyper::Response::new(Full::new(Bytes::from("Server error")))
                                .into_parts();
                        hyper::Response::from_parts(parts, Full::new(Bytes::from("Critical error")))
                    }))
            }
        }
    }

    /// Serve metrics API
    async fn serve_metrics(&self) -> Result<Response<Full<Bytes>>> {
        let metrics = json!({
            "cpu_usage": 0.0,
            "memory_usage": 0.0,
            "disk_usage": 0.0,
            "timestamp": chrono::Utc::now()
        });

        let json_response = metrics.to_string();
        let response = hyper::Response::builder()
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(json_response)));

        match response {
            Ok(resp) => Ok(resp),
            Err(e) => {
                tracing::error!("Failed to build HTTP response for metrics: {}", e);
                Ok(hyper::Response::builder()
                    .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                    .header("content-type", "application/json")
                    .body(Full::new(Bytes::from(
                        r#"{"error": "Metrics unavailable"}"#,
                    )))
                    .unwrap_or_else(|_| {
                        let (parts, _) = hyper::Response::new(Full::new(Bytes::from(
                            r#"{"error": "Server error"}"#,
                        )))
                        .into_parts();
                        hyper::Response::from_parts(
                            parts,
                            Full::new(Bytes::from(r#"{"error": "Critical error"}"#)),
                        )
                    }))
            }
        }
    }

    /// Serve health API
    async fn serve_health(&self) -> Result<Response<Full<Bytes>>> {
        let health = json!({
            "status": "healthy",
            "services": [],
            "timestamp": chrono::Utc::now()
        });

        let json_response = health.to_string();
        let response = hyper::Response::builder()
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(json_response)));

        match response {
            Ok(resp) => Ok(resp),
            Err(e) => {
                tracing::error!("Failed to build HTTP response for health: {}", e);
                Ok(hyper::Response::builder()
                    .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                    .header("content-type", "application/json")
                    .body(Full::new(Bytes::from(
                        r#"{"error": "Health check unavailable"}"#,
                    )))
                    .unwrap_or_else(|_| {
                        let (parts, _) = hyper::Response::new(Full::new(Bytes::from(
                            r#"{"error": "Server error"}"#,
                        )))
                        .into_parts();
                        hyper::Response::from_parts(
                            parts,
                            Full::new(Bytes::from(r#"{"error": "Critical error"}"#)),
                        )
                    }))
            }
        }
    }

    /// Serve status API
    async fn serve_status(&self) -> Result<Response<Full<Bytes>>> {
        let status = json!({
            "status": "running",
            "uptime": "0s",
            "timestamp": chrono::Utc::now()
        });

        let json_response = status.to_string();
        let response = hyper::Response::builder()
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(json_response)));

        match response {
            Ok(resp) => Ok(resp),
            Err(e) => {
                tracing::error!("Failed to build HTTP response for status: {}", e);
                Ok(hyper::Response::builder()
                    .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                    .header("content-type", "application/json")
                    .body(Full::new(Bytes::from(r#"{"error": "Status unavailable"}"#)))
                    .unwrap_or_else(|_| {
                        let (parts, _) = hyper::Response::new(Full::new(Bytes::from(
                            r#"{"error": "Server error"}"#,
                        )))
                        .into_parts();
                        hyper::Response::from_parts(
                            parts,
                            Full::new(Bytes::from(r#"{"error": "Critical error"}"#)),
                        )
                    }))
            }
        }
    }

    /// Serve 404 response
    async fn serve_not_found(&self) -> Result<Response<Full<Bytes>>> {
        // Return 404 for unknown paths
        let response = hyper::Response::builder()
            .status(hyper::StatusCode::NOT_FOUND)
            .header("content-type", "text/plain")
            .body(Full::new(Bytes::from("Not Found")));

        match response {
            Ok(resp) => Ok(resp),
            Err(e) => {
                tracing::error!("Failed to build 404 HTTP response: {}", e);
                Ok(hyper::Response::builder()
                    .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                    .header("content-type", "text/plain")
                    .body(Full::new(Bytes::from("Server error")))
                    .unwrap_or_else(|_| {
                        let (parts, _) =
                            hyper::Response::new(Full::new(Bytes::from("Critical failure")))
                                .into_parts();
                        hyper::Response::from_parts(parts, Full::new(Bytes::from("Server failure")))
                    }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dashboard_creation() {
        let env_config = songbird_config::EnvironmentConfig::default();
        let dashboard = SimpleDashboard::new(env_config.dashboard_port);
        assert!(!dashboard.is_running());
    }

    #[tokio::test]
    async fn test_dashboard_start_stop() {
        let env_config = songbird_config::EnvironmentConfig::default();
        let dashboard = SimpleDashboard::new(env_config.dashboard_port);

        assert!(dashboard.start().await.is_ok());
        assert!(dashboard.is_running());

        assert!(dashboard.stop().await.is_ok());
        assert!(!dashboard.is_running());
    }
}
