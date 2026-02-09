//! Unix Socket JSON-RPC Client for Inter-Primal Communication
//!
//! v3.34.0: Pure Rust implementation for inter-primal RPC via Unix sockets
//!
//! ## Design Principles
//!
//! 1. **Zero HTTP Overhead**: Direct Unix socket communication
//! 2. **JSON-RPC 2.0**: Standard protocol for interoperability
//! 3. **Type-Safe**: Generic methods with serde serialization
//! 4. **Modern Async**: tokio-based async/await
//! 5. **RAII**: Automatic cleanup via Drop
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use songbird_primal_sdk::unix_rpc_client::UnixRpcClient;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize)]
//! struct EncryptRequest {
//!     data: Vec<u8>,
//! }
//!
//! #[derive(Deserialize)]
//! struct EncryptResponse {
//!     ciphertext: Vec<u8>,
//! }
//!
//! async fn encrypt_data() -> Result<Vec<u8>> {
//!     let client = UnixRpcClient::new("/tmp/beardog.sock")?;
//!     let request = EncryptRequest { data: vec![1, 2, 3] };
//!     let response: EncryptResponse = client.call("encrypt", request).await?;
//!     Ok(response.ciphertext)
//! }
//! ```

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
// Platform-agnostic IPC transport
#[cfg(windows)]
use tokio::net::TcpStream as PlatformStream;
#[cfg(unix)]
use tokio::net::UnixStream as PlatformStream;
use tracing::{debug, trace};

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize)]
struct JsonRpcRequest<P> {
    jsonrpc: String,
    method: String,
    params: P,
    id: u64,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Deserialize)]
struct JsonRpcResponse<R> {
    #[allow(dead_code)]
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<R>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    #[allow(dead_code)]
    id: u64,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[allow(dead_code)]
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

/// Unix Socket JSON-RPC Client
///
/// Provides a simple, type-safe interface for JSON-RPC 2.0 communication
/// over Unix domain sockets.
///
/// ## Features
///
/// - **Type-Safe**: Generic `call()` method with automatic serialization
/// - **Async**: Built on tokio for concurrent operations
/// - **Error Handling**: Comprehensive error propagation
/// - **Discovery**: Automatic socket path resolution
#[derive(Debug, Clone)]
pub struct UnixRpcClient {
    /// Path to the Unix socket
    socket_path: PathBuf,
    /// Request ID counter (for correlation)
    next_id: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl UnixRpcClient {
    /// Create a new Unix RPC client
    ///
    /// ## Arguments
    ///
    /// * `socket_path` - Path to the Unix socket (e.g., "/tmp/beardog.sock")
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let client = UnixRpcClient::new("/tmp/beardog.sock")?;
    /// ```
    pub fn new(socket_path: impl AsRef<Path>) -> Result<Self> {
        let socket_path = socket_path.as_ref().to_path_buf();

        // Validate socket path exists (optional, can be deferred to connect time)
        if !socket_path.exists() {
            debug!("Socket path does not exist yet: {:?}", socket_path);
        }

        Ok(Self {
            socket_path,
            next_id: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1)),
        })
    }

    /// Call a JSON-RPC method
    ///
    /// ## Type Parameters
    ///
    /// * `P` - Request parameters type (must be Serialize)
    /// * `R` - Response result type (must be DeserializeOwned)
    ///
    /// ## Arguments
    ///
    /// * `method` - The JSON-RPC method name
    /// * `params` - The method parameters
    ///
    /// ## Returns
    ///
    /// The deserialized response result
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let result: MyResponse = client.call("my_method", MyParams { ... }).await?;
    /// ```
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

    pub async fn call<P, R>(&self, method: &str, params: P) -> Result<R>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        // Generate request ID
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        trace!("Calling JSON-RPC method: {} (id: {})", method, id);

        // Build request
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id,
        };

        // Serialize request
        let request_bytes =
            serde_json::to_vec(&request).context("Failed to serialize JSON-RPC request")?;

        // Connect (platform-agnostic)
        let mut stream = Self::connect_platform(&self.socket_path)
            .await
            .with_context(|| format!("Failed to connect to IPC: {}", self.socket_path.display()))?;

        // Send request
        stream.write_all(&request_bytes).await.context("Failed to write request to Unix socket")?;

        // Add newline delimiter (some servers expect this)
        stream.write_all(b"\n").await.context("Failed to write delimiter")?;

        // Shutdown write side to signal end of request
        stream.shutdown().await.context("Failed to shutdown write side")?;

        // Read response
        let mut response_bytes = Vec::new();
        stream
            .read_to_end(&mut response_bytes)
            .await
            .context("Failed to read response from Unix socket")?;

        trace!("Received {} bytes from Unix socket", response_bytes.len());

        // Deserialize response
        let response: JsonRpcResponse<R> = serde_json::from_slice(&response_bytes)
            .context("Failed to deserialize JSON-RPC response")?;

        // Check for errors
        if let Some(error) = response.error {
            return Err(anyhow::anyhow!(
                "JSON-RPC error: {} (code: {})",
                error.message,
                error.code
            ));
        }

        // Extract result
        response.result.ok_or_else(|| anyhow::anyhow!("JSON-RPC response missing result field"))
    }

    /// Call a JSON-RPC method with no parameters
    ///
    /// Convenience method for parameterless RPC calls.
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let result: HealthStatus = client.call_no_params("health_check").await?;
    /// ```
    pub async fn call_no_params<R>(&self, method: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        self.call(method, json!({})).await
    }

    /// Get the socket path
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    // Removed unused import: use std::sync::Arc;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::net::UnixListener;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct EchoRequest {
        message: String,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct EchoResponse {
        echo: String,
    }

    /// Mock JSON-RPC server for testing (signals readiness via oneshot)
    async fn mock_server(socket_path: PathBuf, ready_tx: tokio::sync::oneshot::Sender<()>) {
        let listener = UnixListener::bind(&socket_path).unwrap();
        // Signal that the server is ready to accept connections
        let _ = ready_tx.send(());

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    tokio::spawn(async move {
                        let mut reader = BufReader::new(stream);
                        let mut request_line = String::new();

                        if reader.read_line(&mut request_line).await.is_ok() {
                            let request: serde_json::Value =
                                serde_json::from_str(&request_line).unwrap();

                            let method = request["method"].as_str().unwrap();
                            let id = request["id"].as_u64().unwrap();

                            let response = if method == "echo" {
                                let params: EchoRequest =
                                    serde_json::from_value(request["params"].clone()).unwrap();
                                json!({
                                    "jsonrpc": "2.0",
                                    "result": EchoResponse { echo: params.message },
                                    "id": id
                                })
                            } else {
                                json!({
                                    "jsonrpc": "2.0",
                                    "error": {
                                        "code": -32601,
                                        "message": "Method not found"
                                    },
                                    "id": id
                                })
                            };

                            let mut stream = reader.into_inner();
                            let response_bytes = serde_json::to_vec(&response).unwrap();
                            stream.write_all(&response_bytes).await.ok();
                        }
                    });
                }
                Err(_) => break,
            }
        }
    }

    #[tokio::test]
    async fn test_unix_rpc_client_success() -> Result<()> {
        let socket_path = PathBuf::from("/tmp/test_unix_rpc_success.sock");

        // Cleanup old socket
        let _ = std::fs::remove_file(&socket_path);

        // Start mock server with readiness signal (no sleep needed)
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let server_path = socket_path.clone();
        tokio::spawn(async move {
            mock_server(server_path, ready_tx).await;
        });

        // Wait for server to be ready (event-driven, instant)
        ready_rx.await.unwrap();

        // Create client
        let client = UnixRpcClient::new(&socket_path)?;

        // Call echo method
        let request = EchoRequest {
            message: "Hello, Unix sockets!".to_string(),
        };
        let response: EchoResponse = client.call("echo", request).await?;

        assert_eq!(response.echo, "Hello, Unix sockets!");

        // Cleanup
        std::fs::remove_file(&socket_path).ok();

        Ok(())
    }

    #[tokio::test]
    async fn test_unix_rpc_client_error() -> Result<()> {
        let socket_path = PathBuf::from("/tmp/test_unix_rpc_error.sock");

        // Cleanup old socket
        let _ = std::fs::remove_file(&socket_path);

        // Start mock server with readiness signal (no sleep needed)
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let server_path = socket_path.clone();
        tokio::spawn(async move {
            mock_server(server_path, ready_tx).await;
        });

        // Wait for server to be ready (event-driven, instant)
        ready_rx.await.unwrap();

        // Create client
        let client = UnixRpcClient::new(&socket_path)?;

        // Call non-existent method
        let result: Result<EchoResponse> = client.call("nonexistent", json!({})).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Method not found"));

        // Cleanup
        std::fs::remove_file(&socket_path).ok();

        Ok(())
    }

    #[test]
    fn test_unix_rpc_client_creation() {
        let client = UnixRpcClient::new("/tmp/test.sock");
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.socket_path(), Path::new("/tmp/test.sock"));
    }
}
