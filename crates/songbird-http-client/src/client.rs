//! HTTP/HTTPS client implementation

use crate::beardog_client::BearDogClient;
use crate::error::{Error, Result};
use crate::tls::{handshake::TlsHandshake, record::TlsRecordLayer};
use crate::types::HttpResponse;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Request, Uri};
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tracing::{debug, info, trace};

/// Songbird HTTP client
#[derive(Debug, Clone)]
pub struct SongbirdHttpClient {
    beardog: Arc<BearDogClient>,
}

impl SongbirdHttpClient {
    /// Create a new Songbird HTTP client
    ///
    /// # Arguments
    ///
    /// * `beardog_socket` - Path to BearDog Unix socket
    pub fn new(beardog_socket: impl Into<String>) -> Self {
        Self {
            beardog: Arc::new(BearDogClient::new(beardog_socket)),
        }
    }

    /// Make an HTTP/HTTPS request
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP method (GET, POST, etc.)
    /// * `url` - Full URL
    /// * `headers` - Request headers
    /// * `body` - Optional request body
    pub async fn request(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: Option<serde_json::Value>,
    ) -> Result<HttpResponse> {
        info!("🌐 HTTP {} {}", method, url);

        // Parse URL
        let uri: Uri = url.parse().map_err(|e| Error::InvalidUrl(format!("{}", e)))?;
        
        let scheme = uri.scheme_str().ok_or_else(|| Error::InvalidUrl("Missing scheme".to_string()))?;
        let host = uri.host().ok_or_else(|| Error::InvalidUrl("Missing host".to_string()))?;
        let port = uri.port_u16().unwrap_or(if scheme == "https" { 443 } else { 80 });

        debug!("Connecting to {}:{}", host, port);

        // Establish TCP connection
        let addr = format!("{}:{}", host, port);
        let tcp_stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| Error::Connection(format!("Failed to connect to {}: {}", addr, e)))?;

        // For HTTPS, perform TLS handshake
        if scheme == "https" {
            return self.https_request(tcp_stream, host, &uri, method, headers, body).await;
        }

        // For HTTP, use plain connection
        self.http_request(tcp_stream, &uri, method, headers, body).await
    }

    /// Make HTTPS request with TLS
    async fn https_request(
        &self,
        mut tcp_stream: TcpStream,
        host: &str,
        uri: &Uri,
        method: &str,
        headers: HashMap<String, String>,
        body: Option<serde_json::Value>,
    ) -> Result<HttpResponse> {
        debug!("🔒 Performing TLS handshake with {}", host);

        // Perform TLS handshake
        let handshake = TlsHandshake::new(self.beardog.clone());
        let session_keys = handshake.handshake(&mut tcp_stream, host).await?;

        debug!("✅ TLS handshake complete");

        // Create TLS record layer
        let mut record_layer = TlsRecordLayer::new(self.beardog.clone(), session_keys);

        // Build HTTP request
        let http_request = self.build_http_request(uri, method, &headers, body.as_ref())?;
        trace!("HTTP request: {} bytes", http_request.len());

        // Send HTTP request over TLS
        record_layer.write_application_data(&mut tcp_stream, &http_request).await?;

        // Read HTTP response over TLS
        let response_data = record_layer.read_application_data(&mut tcp_stream).await?;
        trace!("HTTP response: {} bytes", response_data.len());

        // Parse HTTP response
        self.parse_http_response(&response_data)
    }

    /// Make HTTP request without TLS
    async fn http_request(
        &self,
        tcp_stream: TcpStream,
        uri: &Uri,
        method: &str,
        headers: HashMap<String, String>,
        body: Option<serde_json::Value>,
    ) -> Result<HttpResponse> {
        debug!("📡 Making HTTP request (no TLS)");

        let io = TokioIo::new(tcp_stream);

        // Create HTTP client
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

        // Spawn connection task
        tokio::spawn(async move {
            if let Err(err) = conn.await {
                tracing::error!("Connection error: {:?}", err);
            }
        });

        // Build request
        let mut req_builder = Request::builder()
            .method(method)
            .uri(uri);

        // Add headers
        for (key, value) in headers {
            req_builder = req_builder.header(&key, &value);
        }

        // Build body
        let body_bytes = if let Some(b) = body {
            Bytes::from(serde_json::to_vec(&b)?)
        } else {
            Bytes::new()
        };

        let request = req_builder.body(Full::new(body_bytes))?;

        // Send request
        let response = sender.send_request(request).await?;

        // Read response
        let status = response.status().as_u16();
        let response_headers: HashMap<String, String> = response
            .headers()
            .iter()
            .filter_map(|(k, v)| {
                v.to_str().ok().map(|s| (k.as_str().to_string(), s.to_string()))
            })
            .collect();

        let body_bytes = response.into_body().collect().await?.to_bytes();
        let body: serde_json::Value = if body_bytes.is_empty() {
            serde_json::json!("")
        } else {
            serde_json::from_slice(&body_bytes).unwrap_or_else(|_| {
                serde_json::json!(String::from_utf8_lossy(&body_bytes).to_string())
            })
        };

        Ok(HttpResponse {
            status,
            headers: response_headers,
            body,
        })
    }

    /// Build HTTP request bytes
    fn build_http_request(
        &self,
        uri: &Uri,
        method: &str,
        headers: &HashMap<String, String>,
        body: Option<&serde_json::Value>,
    ) -> Result<Vec<u8>> {
        let mut request = Vec::new();

        // Request line
        let path = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
        request.extend_from_slice(format!("{} {} HTTP/1.1\r\n", method, path).as_bytes());

        // Host header
        if let Some(host) = uri.host() {
            request.extend_from_slice(format!("Host: {}\r\n", host).as_bytes());
        }

        // Headers
        for (key, value) in headers {
            request.extend_from_slice(format!("{}: {}\r\n", key, value).as_bytes());
        }

        // Body
        if let Some(b) = body {
            let body_bytes = serde_json::to_vec(b)?;
            request.extend_from_slice(format!("Content-Length: {}\r\n", body_bytes.len()).as_bytes());
            request.extend_from_slice(b"\r\n");
            request.extend_from_slice(&body_bytes);
        } else {
            request.extend_from_slice(b"\r\n");
        }

        Ok(request)
    }

    /// Parse HTTP response bytes
    fn parse_http_response(&self, data: &[u8]) -> Result<HttpResponse> {
        let response = String::from_utf8_lossy(data);
        let mut lines = response.lines();

        // Status line
        let status_line = lines.next().ok_or_else(|| Error::InvalidResponse("Empty response".to_string()))?;
        let status = status_line
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse::<u16>().ok())
            .ok_or_else(|| Error::InvalidResponse("Invalid status line".to_string()))?;

        // Headers
        let mut headers = HashMap::new();
        let mut body_start = 0;
        
        for (idx, line) in lines.enumerate() {
            if line.is_empty() {
                body_start = idx + 2; // +2 for status line and empty line
                break;
            }
            
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(
                    key.trim().to_lowercase(),
                    value.trim().to_string(),
                );
            }
        }

        // Body
        let body_lines: Vec<&str> = response.lines().skip(body_start).collect();
        let body_str = body_lines.join("\n");
        
        let body: serde_json::Value = if body_str.is_empty() {
            serde_json::json!("")
        } else {
            serde_json::from_str(&body_str).unwrap_or_else(|_| {
                serde_json::json!(body_str)
            })
        };

        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }

    /// Convenience method for GET requests
    pub async fn get(&self, url: &str) -> Result<HttpResponse> {
        self.request("GET", url, HashMap::new(), None).await
    }

    /// Convenience method for POST requests
    pub async fn post(&self, url: &str, body: serde_json::Value) -> Result<HttpResponse> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        self.request("POST", url, headers, Some(body)).await
    }

    /// Convenience method for PUT requests
    pub async fn put(&self, url: &str, body: serde_json::Value) -> Result<HttpResponse> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        self.request("PUT", url, headers, Some(body)).await
    }

    /// Convenience method for DELETE requests
    pub async fn delete(&self, url: &str) -> Result<HttpResponse> {
        self.request("DELETE", url, HashMap::new(), None).await
    }

    /// Convenience method for PATCH requests
    pub async fn patch(&self, url: &str, body: serde_json::Value) -> Result<HttpResponse> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        self.request("PATCH", url, headers, Some(body)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let _client = SongbirdHttpClient::new("/tmp/beardog.sock");
        // Client created successfully if we got here
    }

    #[test]
    fn test_build_http_request() {
        let client = SongbirdHttpClient::new("/tmp/beardog.sock");
        let uri: Uri = "http://example.com/test".parse().unwrap();
        let headers = HashMap::new();
        
        let request = client.build_http_request(&uri, "GET", &headers, None).unwrap();
        let request_str = String::from_utf8_lossy(&request);
        
        assert!(request_str.contains("GET /test HTTP/1.1"));
        assert!(request_str.contains("Host: example.com"));
    }

    #[test]
    fn test_parse_http_response() {
        let client = SongbirdHttpClient::new("/tmp/beardog.sock");
        let response_data = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"result\":\"ok\"}";
        
        let response = client.parse_http_response(response_data).unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.headers.get("content-type"), Some(&"application/json".to_string()));
    }
}

