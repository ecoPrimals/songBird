// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unused_async)]

use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::{Method, Request, Response};
use serde_json::json;
use songbird_types::{SongbirdError, SongbirdResult};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{info, warn};
type Result<T> = SongbirdResult<T>;

// Helper function to convert hyper::http::Error to SongbirdError
#[allow(clippy::needless_pass_by_value)]
fn http_error_to_songbird(error: hyper::http::Error) -> SongbirdError {
    SongbirdError::Network {
        message: format!("HTTP error: {error}"),
        interface: None,
        suggestion: Some("Check HTTP request/response construction".to_string()),
    }
}

/// Simple web dashboard for observability
#[derive(Debug)]
pub struct SimpleDashboard {
    port: u16,
    running: Arc<AtomicBool>,
}

impl SimpleDashboard {
    /// Create new dashboard
    #[must_use]
    pub fn new(port: u16) -> Self {
        Self {
            port,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Create dashboard from network config
    #[must_use]
    pub fn from_network_config(port: u16) -> Self {
        Self::new(port)
    }

    /// Start the dashboard server
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
    pub async fn start(&self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            warn!("Dashboard already running on port {}", self.port);
            return Ok(());
        }

        info!("Starting dashboard on port {}", self.port);
        self.running.store(true, Ordering::Relaxed);

        // In a real implementation, this would start an HTTP server
        // For now, we'll just mark it as running
        info!("Dashboard started on http://127.0.0.1:{}", self.port);
        Ok(())
    }

    /// Stop the dashboard server
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
    pub async fn stop(&self) -> Result<()> {
        if !self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        info!("Stopping dashboard on port {}", self.port);
        self.running.store(false, Ordering::Relaxed);
        Ok(())
    }

    /// Check if dashboard is running
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Handle HTTP requests
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP response cannot be constructed
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

        let response = Response::builder()
            .header("content-type", "text/html")
            .body(Full::new(Bytes::from(html)))
            .map_err(http_error_to_songbird)?;

        Ok(response)
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
        let response = Response::builder()
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(json_response)))
            .map_err(http_error_to_songbird)?;

        Ok(response)
    }

    /// Serve health API
    async fn serve_health(&self) -> Result<Response<Full<Bytes>>> {
        let health = json!({
            "status": "healthy",
            "services": [],
            "timestamp": chrono::Utc::now()
        });

        let json_response = health.to_string();
        let response = Response::builder()
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(json_response)))
            .map_err(http_error_to_songbird)?;

        Ok(response)
    }

    /// Serve status API
    async fn serve_status(&self) -> Result<Response<Full<Bytes>>> {
        let status = json!({
            "status": "running",
            "uptime": "0s",
            "timestamp": chrono::Utc::now()
        });

        let json_response = status.to_string();
        let response = Response::builder()
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(json_response)))
            .map_err(http_error_to_songbird)?;

        Ok(response)
    }

    /// Serve 404 response
    async fn serve_not_found(&self) -> Result<Response<Full<Bytes>>> {
        let response = Response::builder()
            .status(hyper::StatusCode::NOT_FOUND)
            .header("content-type", "text/plain")
            .body(Full::new(Bytes::from("Not Found")))
            .map_err(http_error_to_songbird)?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dashboard_creation() {
        let dashboard = SimpleDashboard::new(8000);
        assert!(!dashboard.is_running());
    }

    #[tokio::test]
    async fn test_dashboard_start_stop() {
        let dashboard = SimpleDashboard::new(8000);

        assert!(dashboard.start().await.is_ok());
        assert!(dashboard.is_running());

        assert!(dashboard.stop().await.is_ok());
        assert!(!dashboard.is_running());
    }

    #[tokio::test]
    async fn test_dashboard_from_network_config() {
        let dashboard = SimpleDashboard::from_network_config(9090);
        assert!(!dashboard.is_running());
        assert_eq!(dashboard.port, 9090);
    }

    #[tokio::test]
    async fn test_dashboard_double_start() {
        let dashboard = SimpleDashboard::new(9091);
        assert!(dashboard.start().await.is_ok());
        assert!(dashboard.is_running());

        // Second start should be a no-op
        assert!(dashboard.start().await.is_ok());
        assert!(dashboard.is_running());

        assert!(dashboard.stop().await.is_ok());
    }

    #[tokio::test]
    async fn test_dashboard_double_stop() {
        let dashboard = SimpleDashboard::new(9092);
        assert!(dashboard.start().await.is_ok());
        assert!(dashboard.stop().await.is_ok());

        // Second stop should be a no-op
        assert!(dashboard.stop().await.is_ok());
        assert!(!dashboard.is_running());
    }

    #[tokio::test]
    async fn test_dashboard_stop_without_start() {
        let dashboard = SimpleDashboard::new(9093);
        // Stop without start should be a no-op
        assert!(dashboard.stop().await.is_ok());
        assert!(!dashboard.is_running());
    }

    #[tokio::test]
    async fn test_serve_dashboard() -> Result<()> {
        let dashboard = SimpleDashboard::new(9094);
        let response = dashboard.serve_dashboard().await;
        assert!(response.is_ok());

        let resp =
            response.map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;
        assert_eq!(resp.status(), hyper::StatusCode::OK);

        let content_type = resp
            .headers()
            .get("content-type")
            .ok_or_else(|| SongbirdError::configuration("Missing content-type header".to_string()))?
            .to_str()
            .map_err(|e| SongbirdError::configuration(format!("Invalid HTTP header value: {e}")))?;
        assert!(content_type.contains("text/html"));
        Ok(())
    }

    #[tokio::test]
    async fn test_serve_metrics() -> Result<()> {
        let dashboard = SimpleDashboard::new(9095);
        let response = dashboard.serve_metrics().await;
        assert!(response.is_ok());

        let resp =
            response.map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;
        assert_eq!(resp.status(), hyper::StatusCode::OK);

        let content_type = resp
            .headers()
            .get("content-type")
            .ok_or_else(|| SongbirdError::configuration("Missing content-type header".to_string()))?
            .to_str()
            .map_err(|e| SongbirdError::configuration(format!("Invalid HTTP header value: {e}")))?;
        assert!(content_type.contains("application/json"));
        Ok(())
    }

    #[tokio::test]
    async fn test_serve_health() -> Result<()> {
        let dashboard = SimpleDashboard::new(9096);
        let response = dashboard.serve_health().await;
        assert!(response.is_ok());

        let resp =
            response.map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;
        assert_eq!(resp.status(), hyper::StatusCode::OK);

        let content_type = resp
            .headers()
            .get("content-type")
            .ok_or_else(|| SongbirdError::configuration("Missing content-type header".to_string()))?
            .to_str()
            .map_err(|e| SongbirdError::configuration(format!("Invalid HTTP header value: {e}")))?;
        assert!(content_type.contains("application/json"));
        Ok(())
    }

    #[tokio::test]
    async fn test_serve_status() -> Result<()> {
        let dashboard = SimpleDashboard::new(9097);
        let response = dashboard.serve_status().await;
        assert!(response.is_ok());

        let resp =
            response.map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;
        assert_eq!(resp.status(), hyper::StatusCode::OK);

        let content_type = resp
            .headers()
            .get("content-type")
            .ok_or_else(|| SongbirdError::configuration("Missing content-type header".to_string()))?
            .to_str()
            .map_err(|e| SongbirdError::configuration(format!("Invalid HTTP header value: {e}")))?;
        assert!(content_type.contains("application/json"));
        Ok(())
    }

    #[tokio::test]
    async fn test_serve_not_found() -> Result<()> {
        let dashboard = SimpleDashboard::new(9098);
        let response = dashboard.serve_not_found().await;
        assert!(response.is_ok());

        let resp =
            response.map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;
        assert_eq!(resp.status(), hyper::StatusCode::NOT_FOUND);
        Ok(())
    }

    #[test]
    fn test_dashboard_debug() {
        let dashboard = SimpleDashboard::new(9105);
        let debug_str = format!("{dashboard:?}");
        assert!(debug_str.contains("SimpleDashboard"));
    }

    #[tokio::test]
    async fn test_is_running_initial_state() {
        let dashboard = SimpleDashboard::new(9106);
        assert!(!dashboard.is_running());
    }

    #[tokio::test]
    async fn test_port_assignment() {
        let dashboard = SimpleDashboard::new(8888);
        assert_eq!(dashboard.port, 8888);
    }
}
