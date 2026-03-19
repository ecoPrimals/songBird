//! JSON-RPC 2.0 Client for Unix Socket IPC
//!
//! This module provides a protocol-agnostic JSON-RPC 2.0 client implementation
//! for inter-primal communication over Unix domain sockets.
//!
//! ## Philosophy
//!
//! This client embodies Songbird's core principles:
//! - **Protocol Agnostic**: Works with any primal providing JSON-RPC 2.0
//! - **Zero Hardcoding**: No assumptions about specific primals
//! - **Capability-Based**: Discovers and communicates via capabilities
//! - **Port-Free**: Uses Unix sockets (no TCP ports)
//!
//! ## Design
//!
//! - Async-first (tokio-based)
//! - Compatible with `IpcHttpClient` interface patterns
//! - Clean error handling via `SongbirdError`
//! - Timeout support
//! - Connection pooling for Unix sockets
//!
//! ## Usage
//!
//! ```rust,no_run
//! use songbird_universal::JsonRpcClient;
//! use serde_json::json;
//!
//! # async fn example() -> songbird_types::SongbirdResult<()> {
//! // Create client for Unix socket endpoint
//! let client = JsonRpcClient::new("unix:///tmp/beardog-nat0-tower1.sock")?;
//!
//! // Call a method
//! let request = json!({
//!     "jsonrpc": "2.0",
//!     "method": "evaluate_trust",
//!     "params": {"peer_id": "tower2", "family": "nat0"},
//!     "id": 1
//! });
//!
//! let response = client.call(request).await?;
//! println!("Response: {:?}", response);
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use songbird_types::{SongbirdError, SongbirdResult};
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
// Platform-agnostic IPC transport
#[cfg(windows)]
use tokio::net::TcpStream as PlatformStream;
#[cfg(unix)]
use tokio::net::UnixStream as PlatformStream;
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// JSON-RPC 2.0 Client for Unix Socket IPC
///
/// Provides async JSON-RPC 2.0 communication over Unix domain sockets.
/// Automatically handles connection management, request/response formatting,
/// and error handling.
#[derive(Debug, Clone)]
pub struct JsonRpcClient {
    /// Unix socket path (without unix:// prefix)
    socket_path: PathBuf,
    /// Request timeout duration
    timeout: Duration,
    /// Next request ID (for correlation)
    next_id: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
    id: u64,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: u64,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Deserialize, Serialize)]
struct JsonRpcError {
    code: i64,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl JsonRpcClient {
    /// Create a new JSON-RPC client for the given Unix socket endpoint
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Unix socket path (e.g., "<unix:///tmp/beardog.sock>" or "/tmp/beardog.sock")
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - New client instance or error
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use songbird_universal::JsonRpcClient;
    ///
    /// # fn example() -> songbird_types::SongbirdResult<()> {
    /// let client = JsonRpcClient::new("unix:///tmp/beardog.sock")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the socket path is empty or configuration is invalid.
    pub fn new(endpoint: &str) -> SongbirdResult<Self> {
        // Parse endpoint - strip unix:// prefix if present
        let socket_path =
            endpoint.strip_prefix("unix://").map_or_else(|| PathBuf::from(endpoint), PathBuf::from);

        // Validate socket path
        if socket_path.to_string_lossy().is_empty() {
            return Err(SongbirdError::configuration("Empty socket path"));
        }

        info!("📡 JSON-RPC client initialized for socket: {}", socket_path.display());

        // ✅ DEEP DEBT EVOLUTION (Feb 3, 2026): Use TimeoutConfig
        // Replaces hardcoded Duration::from_secs(5) with configurable timeout
        let timeout_config = songbird_config::timeouts::TimeoutConfig::from_env();

        Ok(Self {
            socket_path,
            timeout: timeout_config.request,
            next_id: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1)),
        })
    }

    /// Set request timeout duration
    ///
    /// # Arguments
    ///
    /// * `timeout` - Request timeout duration
    ///
    /// # Returns
    ///
    /// * `Self` - Updated client (builder pattern)
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Platform-agnostic connection helper
    #[cfg(unix)]
    async fn connect_platform(path: &PathBuf) -> std::io::Result<PlatformStream> {
        PlatformStream::connect(path).await
    }

    #[cfg(windows)]
    async fn connect_platform(path: &PathBuf) -> std::io::Result<PlatformStream> {
        let addr = path.to_string_lossy();
        PlatformStream::connect(addr.as_ref()).await
    }

    #[cfg(not(any(unix, windows)))]
    async fn connect_platform(path: &PathBuf) -> std::io::Result<tokio::net::TcpStream> {
        let addr = path.to_string_lossy();
        tokio::net::TcpStream::connect(addr.as_ref()).await
    }

    /// Call a JSON-RPC method with automatic request ID generation
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails or the response cannot be parsed.
    ///
    /// # Arguments
    ///
    /// * `method` - Method name (e.g., "`evaluate_trust`")
    /// * `params` - Method parameters (can be object or array)
    ///
    /// # Returns
    ///
    /// * `Result<Value>` - Method result or error
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use songbird_universal::JsonRpcClient;
    /// use serde_json::json;
    ///
    /// # async fn example() -> songbird_types::SongbirdResult<()> {
    /// let client = JsonRpcClient::new("unix:///tmp/beardog.sock")?;
    ///
    /// let result = client.call_method(
    ///     "evaluate_trust",
    ///     Some(json!({"peer_id": "tower2", "family": "nat0"}))
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn call_method(&self, method: &str, params: Option<Value>) -> SongbirdResult<Value> {
        // Generate request ID
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        // Build request
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id,
        };

        debug!("📤 JSON-RPC request: method={}, id={}", request.method, request.id);

        // Send request and get response
        self.send_request(&request).await
    }

    /// Call with pre-built JSON-RPC request (for compatibility with existing code)
    ///
    /// # Errors
    ///
    /// Returns an error if the request is invalid or the RPC call fails.
    ///
    /// # Arguments
    ///
    /// * `request` - Pre-built JSON request value
    ///
    /// # Returns
    ///
    /// * `Result<Value>` - Full response value (including "result" key)
    pub async fn call(&self, request: Value) -> SongbirdResult<Value> {
        // Parse request to validate format
        let method = request
            .get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SongbirdError::configuration("Missing 'method' field in request"))?;

        let params = request.get("params").cloned();

        // Use call_method for actual RPC
        let result = self.call_method(method, params).await?;

        // Return full response format (for compatibility)
        Ok(json!({
            "jsonrpc": "2.0",
            "result": result,
            "id": request.get("id").cloned().unwrap_or_else(|| json!(1))
        }))
    }

    /// Send a JSON-RPC request and receive response
    ///
    /// # Arguments
    ///
    /// * `request` - JSON-RPC request to send
    ///
    /// # Returns
    ///
    /// * `Result<Value>` - Response result or error
    async fn send_request(&self, request: &JsonRpcRequest) -> SongbirdResult<Value> {
        // Serialize request (uses From<serde_json::Error> for SongbirdError)
        let request_json = serde_json::to_string(request)?;
        let request_bytes = format!("{request_json}\n"); // JSON-RPC over newline-delimited stream

        debug!("🔌 Connecting to Unix socket: {}", self.socket_path.display());

        // Connect with timeout (platform-agnostic)
        let stream = timeout(self.timeout, Self::connect_platform(&self.socket_path))
            .await
            .map_err(|_| {
                SongbirdError::network(format!(
                    "Connection timeout to {}",
                    self.socket_path.display()
                ))
            })
            .and_then(|r| {
                r.map_err(|e| SongbirdError::network(format!("Failed to connect: {e}")))
            })?;

        // Split into reader and writer
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Send request with timeout
        timeout(self.timeout, writer.write_all(request_bytes.as_bytes()))
            .await
            .map_err(|_| SongbirdError::network("Write timeout"))
            .and_then(|r| r.map_err(|e| SongbirdError::network(format!("Write failed: {e}"))))?;

        debug!("📤 Sent request: {}", request_json);

        // Read response with timeout
        let mut response_line = String::new();
        timeout(self.timeout, reader.read_line(&mut response_line))
            .await
            .map_err(|_| SongbirdError::network("Read timeout"))
            .and_then(|r| r.map_err(|e| SongbirdError::network(format!("Read failed: {e}"))))?;

        debug!("📥 Received response: {}", response_line.trim());

        // Parse response (uses From<serde_json::Error> for SongbirdError)
        let response: JsonRpcResponse = serde_json::from_str(&response_line)?;

        // Validate response ID matches request ID
        if response.id != request.id {
            warn!("⚠️ Response ID mismatch: expected {}, got {}", request.id, response.id);
        }

        // Check for errors
        if let Some(error) = response.error {
            return Err(SongbirdError::protocol(format!(
                "JSON-RPC error {}: {}",
                error.code, error.message
            )));
        }

        // Return result
        response
            .result
            .ok_or_else(|| SongbirdError::protocol("Missing 'result' in successful response"))
    }

    /// Get the socket path this client is connected to
    #[must_use]
    pub const fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    /// Get the configured timeout
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        self.timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_unix_prefix() {
        let client = JsonRpcClient::new("unix:///tmp/test.sock").unwrap();
        assert_eq!(client.socket_path(), &PathBuf::from("/tmp/test.sock"));
    }

    #[test]
    fn test_new_without_prefix() {
        let client = JsonRpcClient::new("/tmp/test.sock").unwrap();
        assert_eq!(client.socket_path(), &PathBuf::from("/tmp/test.sock"));
    }

    #[test]
    fn test_empty_path_error() {
        let result = JsonRpcClient::new("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty socket path"));
    }

    #[test]
    fn test_with_timeout() {
        let client =
            JsonRpcClient::new("/tmp/test.sock").unwrap().with_timeout(Duration::from_secs(10));
        assert_eq!(client.timeout(), Duration::from_secs(10));
    }

    #[test]
    fn test_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "test_method".to_string(),
            params: Some(json!({"key": "value"})),
            id: 1,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"test_method\""));
        assert!(json.contains("\"id\":1"));
    }

    #[test]
    fn test_response_deserialization_success() {
        let json = r#"{"jsonrpc":"2.0","result":{"status":"ok"},"id":1}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, 1);
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_response_deserialization_error() {
        let json =
            r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid Request"},"id":1}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, 1);
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Invalid Request");
    }
}
