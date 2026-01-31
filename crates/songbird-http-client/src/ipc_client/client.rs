//! IPC HTTP Client - Self-Delegation Pattern
//!
//! Pure Rust HTTP client that delegates to Songbird's own HTTP service via IPC.
//!
//! ## Architecture: Tower Atomic Self-Delegation
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────┐
//! │  Application Code (Discovery, Config, etc.)              │
//! │  "I need to make an HTTP request"                        │
//! └─────────────────────┬────────────────────────────────────┘
//!                       │
//!                       │ IpcHttpClient::new()
//!                       │ client.get("https://...").await?
//!                       │
//! ┌─────────────────────▼────────────────────────────────────┐
//! │  IpcHttpClient (THIS FILE)                               │
//! │  - Matches reqwest::Client API                           │
//! │  - Delegates via JSON-RPC over Unix socket               │
//! │  - Zero C dependencies                                   │
//! └─────────────────────┬────────────────────────────────────┘
//!                       │
//!                       │ JSON-RPC: {"method": "http.request", ...}
//!                       │ Socket: /primal/songbird
//!                       │
//! ┌─────────────────────▼────────────────────────────────────┐
//! │  Songbird IPC Handler                                    │
//! │  (src/ipc/handlers/http.rs)                             │
//! └─────────────────────┬────────────────────────────────────┘
//!                       │
//! ┌─────────────────────▼────────────────────────────────────┐
//! │  SongbirdHttpClient                                      │
//! │  - Pure Rust TLS 1.3                                    │
//! │  - Tower Atomic with BearDog                            │
//! └──────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use songbird_http_client::IpcHttpClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create client (connects to Songbird via IPC)
//!     let client = IpcHttpClient::new().await?;
//!
//!     // Make HTTP GET request
//!     let response = client.get("https://api.github.com/repos/rust-lang/rust").await?;
//!     
//!     println!("Status: {}", response.status());
//!     println!("Body: {}", response.text().await?);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Migration from reqwest
//!
//! ```rust,ignore
//! // BEFORE (reqwest - C dependencies)
//! use reqwest::Client;
//!
//! let client = Client::new();
//! let response = client.get(url).send().await?;
//! let text = response.text().await?;
//!
//! // AFTER (IpcHttpClient - Pure Rust via IPC)
//! use songbird_http_client::IpcHttpClient;
//!
//! let client = IpcHttpClient::new().await?;
//! let response = client.get(url).await?;
//! let text = response.text().await?;
//! ```
//!
//! ## Benefits
//!
//! - ✅ **Pure Rust**: Zero C dependencies (TRUE ecoBin compliant)
//! - ✅ **Self-Delegation**: Reuses Songbird's own HTTP client
//! - ✅ **Tower Atomic**: BearDog crypto via IPC (no ring/openssl)
//! - ✅ **Simple Migration**: Drop-in replacement for `reqwest::Client`
//! - ✅ **Maintained**: Songbird HTTP client is actively developed

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// Platform-agnostic IPC transport
#[cfg(windows)]
use tokio::net::TcpStream as PlatformStream;
#[cfg(unix)]
use tokio::net::UnixStream as PlatformStream;

use super::multipart::Form;

/// IPC HTTP Client - Pure Rust via Songbird self-delegation
///
/// Provides a `reqwest`-like API that delegates HTTP requests to Songbird's
/// own Pure Rust HTTP client via JSON-RPC over Unix sockets.
///
/// # Examples
///
/// ```no_run
/// # use songbird_http_client::IpcHttpClient;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = IpcHttpClient::new().await?;
/// let response = client.get("https://example.com").await?;
/// assert_eq!(response.status(), 200);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct IpcHttpClient {
    socket_path: PathBuf,
    request_id: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl IpcHttpClient {
    /// Create new IPC HTTP client
    ///
    /// Connects to Songbird's IPC socket for HTTP delegation.
    ///
    /// # Socket Discovery
    ///
    /// Checks environment variables in priority order:
    /// 1. `SONGBIRD_SOCKET`
    /// 2. `SONGBIRD_IPC_SOCKET`
    /// 3. `/run/user/{uid}/songbird-{family_id}.sock` (runtime dir)
    /// 4. `/tmp/songbird-{family_id}.sock` (fallback)
    ///
    /// # Errors
    ///
    /// Returns error if socket path cannot be determined.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use songbird_http_client::IpcHttpClient;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = IpcHttpClient::new().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new() -> Result<Self> {
        let socket_path = Self::discover_socket_path()?;
        Ok(Self {
            socket_path,
            request_id: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1)),
        })
    }

    /// Discover Songbird IPC socket path
    ///
    /// Uses environment-aware discovery with sensible defaults.
    fn discover_socket_path() -> Result<PathBuf> {
        // Priority 1: Explicit socket path
        if let Ok(path) = std::env::var("SONGBIRD_SOCKET") {
            return Ok(PathBuf::from(path));
        }
        if let Ok(path) = std::env::var("SONGBIRD_IPC_SOCKET") {
            return Ok(PathBuf::from(path));
        }

        // Priority 2: Runtime directory (XDG standard)
        let family_id = std::env::var("SONGBIRD_FAMILY_ID")
            .or_else(|_| std::env::var("FAMILY_ID"))
            .unwrap_or_else(|_| "default".to_string());

        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            let path = PathBuf::from(format!("{}/songbird-{}.sock", runtime_dir, family_id));
            if path.exists() {
                return Ok(path);
            }
        }

        // Priority 3: User runtime dir fallback
        if let Ok(uid) = std::env::var("UID") {
            let path = PathBuf::from(format!("/run/user/{}/songbird-{}.sock", uid, family_id));
            if path.exists() {
                return Ok(path);
            }
        }

        // Priority 4: /tmp fallback (development/testing)
        let fallback = PathBuf::from(format!("/tmp/songbird-{}.sock", family_id));
        Ok(fallback)
    }

    /// Platform-agnostic connection helper
    ///
    /// Connects to Songbird IPC using platform-specific transport:
    /// - Unix/macOS/Android: Unix domain sockets
    /// - Windows: TCP localhost (future: named pipes via universal IPC)
    #[cfg(unix)]
    async fn connect_platform(path: &PathBuf) -> std::io::Result<PlatformStream> {
        PlatformStream::connect(path).await
    }

    #[cfg(windows)]
    async fn connect_platform(address: &PathBuf) -> std::io::Result<PlatformStream> {
        // On Windows, interpret as TCP address
        let addr_str = address.to_string_lossy();
        PlatformStream::connect(addr_str.as_ref()).await
    }

    #[cfg(not(any(unix, windows)))]
    async fn connect_platform(address: &PathBuf) -> std::io::Result<tokio::net::TcpStream> {
        let addr_str = address.to_string_lossy();
        tokio::net::TcpStream::connect(addr_str.as_ref()).await
    }

    /// Make HTTP GET request
    ///
    /// # Errors
    ///
    /// Returns error if request fails or socket is unavailable.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use songbird_http_client::IpcHttpClient;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = IpcHttpClient::new().await?;
    /// let response = client.get("https://api.github.com/").await?;
    /// println!("Status: {}", response.status());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, url: impl AsRef<str>) -> Result<Response> {
        self.request("GET", url.as_ref(), None, None).await
    }

    /// Make HTTP POST request
    ///
    /// # Errors
    ///
    /// Returns error if request fails or socket is unavailable.
    pub async fn post(&self, url: impl AsRef<str>) -> RequestBuilder {
        RequestBuilder::new(self.clone(), "POST", url.as_ref().to_string())
    }

    /// Make HTTP PUT request
    ///
    /// # Errors
    ///
    /// Returns error if request fails or socket is unavailable.
    pub async fn put(&self, url: impl AsRef<str>) -> RequestBuilder {
        RequestBuilder::new(self.clone(), "PUT", url.as_ref().to_string())
    }

    /// Make HTTP DELETE request
    ///
    /// # Errors
    ///
    /// Returns error if request fails or socket is unavailable.
    pub async fn delete(&self, url: impl AsRef<str>) -> Result<Response> {
        self.request("DELETE", url.as_ref(), None, None).await
    }

    /// Make HTTP request with full control
    ///
    /// Internal method used by convenience methods.
    async fn request(
        &self,
        method: &str,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<Vec<u8>>,
    ) -> Result<Response> {
        // Connect to Songbird IPC (platform-agnostic)
        let mut stream = Self::connect_platform(&self.socket_path)
            .await
            .context(format!("Failed to connect to Songbird IPC: {:?}", self.socket_path))?;

        // Prepare request
        let request_id = self.request_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let body_base64 = body.as_ref().map(|b| BASE64.encode(b));

        let request = json!({
            "jsonrpc": "2.0",
            "method": "http.request",
            "params": {
                "method": method,
                "url": url,
                "headers": headers.unwrap_or_default(),
                "body": body_base64
            },
            "id": request_id
        });

        // Send request
        let request_bytes = serde_json::to_vec(&request)?;
        stream.write_all(&request_bytes).await?;
        stream.write_all(b"\n").await?; // Line-delimited JSON

        // Read response
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).await?;

        // Parse JSON-RPC response
        let response: Value =
            serde_json::from_slice(&buffer).context("Failed to parse JSON-RPC response")?;

        // Extract result or error
        if let Some(error) = response.get("error") {
            return Err(anyhow::anyhow!("HTTP request failed: {:?}", error));
        }

        let result = response
            .get("result")
            .ok_or_else(|| anyhow::anyhow!("Missing result in JSON-RPC response"))?;

        // Parse HTTP response
        let status = result
            .get("status")
            .and_then(|s| s.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing status in response"))?
            as u16;

        let headers: HashMap<String, String> = result
            .get("headers")
            .and_then(|h| serde_json::from_value(h.clone()).ok())
            .unwrap_or_default();

        let body_base64 = result.get("body").and_then(|b| b.as_str()).unwrap_or("");

        let body = if !body_base64.is_empty() {
            BASE64.decode(body_base64)?
        } else {
            Vec::new()
        };

        Ok(Response {
            status,
            headers,
            body,
        })
    }
}

/// HTTP Response
///
/// Simplified response type matching `reqwest::Response` API surface.
#[derive(Debug)]
pub struct Response {
    status: u16,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Response {
    /// Get HTTP status code
    #[must_use]
    pub fn status(&self) -> u16 {
        self.status
    }

    /// Check if status code indicates success (2xx)
    #[must_use]
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Get response headers
    #[must_use]
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Consume response and get body as text
    ///
    /// # Errors
    ///
    /// Returns error if body is not valid UTF-8.
    pub async fn text(self) -> Result<String> {
        String::from_utf8(self.body).context("Response body is not valid UTF-8")
    }

    /// Consume response and get body as JSON
    ///
    /// # Errors
    ///
    /// Returns error if body is not valid JSON.
    pub async fn json<T: serde::de::DeserializeOwned>(self) -> Result<T> {
        serde_json::from_slice(&self.body).context("Failed to parse JSON response")
    }

    /// Consume response and get raw bytes
    #[must_use]
    pub async fn bytes(self) -> Vec<u8> {
        self.body
    }
}

/// Request Builder
///
/// Allows building requests with headers and body before sending.
pub struct RequestBuilder {
    client: IpcHttpClient,
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
    multipart_form: Option<Form>,
}

impl RequestBuilder {
    fn new(client: IpcHttpClient, method: &str, url: String) -> Self {
        Self {
            client,
            method: method.to_string(),
            url,
            headers: HashMap::new(),
            body: None,
            multipart_form: None,
        }
    }

    /// Add a header
    #[must_use]
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set request body as JSON
    ///
    /// # Errors
    ///
    /// Returns error if JSON serialization fails.
    pub fn json<T: serde::Serialize>(mut self, body: &T) -> Result<Self> {
        let json_bytes = serde_json::to_vec(body)?;
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        self.body = Some(json_bytes);
        Ok(self)
    }

    /// Set request body as bytes
    #[must_use]
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    /// Set request body as multipart/form-data
    ///
    /// # Arguments
    ///
    /// * `form` - Multipart form to send
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use songbird_http_client::{IpcHttpClient, multipart};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = IpcHttpClient::new().await?;
    ///
    ///     let form = multipart::Form::new()
    ///         .text("service_name", "my-service")
    ///         .part("binary", multipart::Part::bytes(vec![1, 2, 3])
    ///             .file_name("service.bin"));
    ///
    ///     let response = client.post("https://api.example.com/deploy")
    ///         .await
    ///         .multipart(form)
    ///         .send()
    ///         .await?;
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn multipart(mut self, form: Form) -> Self {
        self.multipart_form = Some(form);
        self
    }

    /// Send the request
    ///
    /// # Errors
    ///
    /// Returns error if request fails.
    pub async fn send(self) -> Result<Response> {
        // If multipart form is present, encode it
        let (body, headers) = if let Some(form) = self.multipart_form {
            let (encoded_body, boundary) = form.encode();
            let content_type = format!("multipart/form-data; boundary={}", boundary);

            let mut headers = self.headers.clone();
            headers.insert("Content-Type".to_string(), content_type);

            (Some(encoded_body), headers)
        } else {
            (self.body, self.headers)
        };

        self.client.request(&self.method, &self.url, Some(headers), body).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_discovery() {
        // Test with explicit path
        std::env::set_var("SONGBIRD_SOCKET", "/tmp/test.sock");
        let path = IpcHttpClient::discover_socket_path().unwrap();
        assert_eq!(path, PathBuf::from("/tmp/test.sock"));
        std::env::remove_var("SONGBIRD_SOCKET");

        // Test with family ID
        std::env::set_var("SONGBIRD_FAMILY_ID", "test");
        let path = IpcHttpClient::discover_socket_path().unwrap();
        assert!(path.to_string_lossy().contains("songbird-test.sock"));
        std::env::remove_var("SONGBIRD_FAMILY_ID");
    }

    #[tokio::test]
    #[ignore] // Requires running Songbird instance
    async fn test_http_get() {
        let client = IpcHttpClient::new().await.unwrap();
        let response = client.get("https://httpbin.org/get").await.unwrap();
        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    #[ignore] // Requires running Songbird instance
    async fn test_http_post_json() {
        let client = IpcHttpClient::new().await.unwrap();
        let body = json!({"test": "data"});

        let response = client
            .post("https://httpbin.org/post")
            .await
            .json(&body)
            .unwrap()
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
    }
}
