//! Tor transport layer using Arti (Pure Rust Tor)
//!
//! **Phase 1A**: Outbound connections only (stable API)
//! **Phase 1B**: Onion service creation (when API stable)
//!
//! ## Usage
//!
//! ```no_run
//! use songbird_onion_relay::TorTransport;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Bootstrap Tor client (10-30s, once at startup)
//! let transport = TorTransport::new().await?;
//! println!("✅ Tor bootstrapped in {:?}", transport.bootstrap_time());
//!
//! // Connect to onion service
//! let mut stream = transport.connect("abc123...xyz.onion", 80).await?;
//!
//! // Send/receive data
//! use tokio::io::{AsyncReadExt, AsyncWriteExt};
//! stream.write_all(b"HELLO").await?;
//! let mut buf = [0u8; 1024];
//! let n = stream.read(&mut buf).await?;
//! # Ok(())
//! # }
//! ```

use arti_client::{TorClient, TorClientConfig};
use tor_rtcompat::PreferredRuntime;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::timeout;
use tracing::{debug, info, warn};

use crate::error::{OnionRelayError, Result};

/// Tor transport for sovereign bootstrap
///
/// Uses Arti (Pure Rust Tor) to connect to onion services without
/// requiring port forwarding or external infrastructure.
///
/// ## Lifecycle
///
/// 1. **Bootstrap**: Download consensus, connect to relays (10-30s)
/// 2. **Connect**: Create circuits to .onion addresses (2-5s each)
/// 3. **Reuse**: Client stays connected, circuits are pooled
///
/// ## Performance
///
/// - Bootstrap: 10-30s (one-time)
/// - Connect: 2-5s per onion
/// - Latency: 300-800ms (3-hop Tor)
/// - Memory: ~50MB
pub struct TorTransport {
    client: TorClient<PreferredRuntime>,
    bootstrap_time: Duration,
}

impl TorTransport {
    /// Bootstrap new Tor client
    ///
    /// This connects to the Tor network by:
    /// 1. Downloading consensus documents
    /// 2. Building directory information
    /// 3. Establishing connections to relays
    ///
    /// Takes 10-30 seconds depending on network speed.
    /// Should be done once at startup and reused.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Network is unreachable
    /// - Tor is blocked by firewall
    /// - Bootstrap timeout (60s)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use songbird_onion_relay::TorTransport;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let transport = TorTransport::new().await?;
    /// println!("Bootstrapped in {:?}", transport.bootstrap_time());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new() -> Result<Self> {
        info!("🧅 Bootstrapping Tor client (this may take 10-30s)...");
        let start = std::time::Instant::now();
        
        // Create default config (uses public Tor network)
        let config = TorClientConfig::default();
        
        // Bootstrap with timeout
        let client = timeout(
            Duration::from_secs(60),
            TorClient::create_bootstrapped(config)
        )
        .await
        .map_err(|_| OnionRelayError::Tor("Bootstrap timeout after 60s".to_string()))?
        .map_err(|e| OnionRelayError::Tor(format!("Bootstrap failed: {}", e)))?;
        
        let bootstrap_time = start.elapsed();
        info!("✅ Tor bootstrapped successfully in {:?}", bootstrap_time);
        
        Ok(Self {
            client,
            bootstrap_time,
        })
    }
    
    /// Connect to an onion service
    ///
    /// Creates a Tor circuit to the specified .onion address.
    /// The connection goes through 3 Tor relays for anonymity.
    ///
    /// # Arguments
    ///
    /// * `onion_addr` - Onion address (e.g., "abc123def456...xyz.onion")
    /// * `port` - Port number
    ///
    /// # Returns
    ///
    /// Bidirectional stream to the onion service.
    /// Implements `AsyncRead` and `AsyncWrite`.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Onion service is offline
    /// - Connection timeout (30s)
    /// - Circuit creation fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use songbird_onion_relay::TorTransport;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let transport = TorTransport::new().await?;
    /// let stream = transport.connect("abc123...xyz.onion", 80).await?;
    /// // Use stream for signaling...
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(&self, onion_addr: &str, port: u16) -> Result<TorStream> {
        let addr_short = &onion_addr[..16.min(onion_addr.len())];
        debug!("🧅 Connecting to {}...:{}", addr_short, port);
        
        let stream = timeout(
            Duration::from_secs(30),
            self.client.connect((onion_addr, port))
        )
        .await
        .map_err(|_| OnionRelayError::Tor(format!(
            "Connection timeout to {}", addr_short
        )))?
        .map_err(|e| OnionRelayError::Tor(format!(
            "Connect failed to {}: {}", addr_short, e
        )))?;
        
        info!("✅ Connected to {}", addr_short);
        Ok(TorStream { stream })
    }
    
    /// Get bootstrap duration (for metrics)
    ///
    /// Returns how long the initial Tor bootstrap took.
    /// Useful for tracking performance and detecting issues.
    pub fn bootstrap_time(&self) -> Duration {
        self.bootstrap_time
    }
    
    /// Check if Tor client is still connected
    ///
    /// Note: Arti doesn't expose connection state directly.
    /// If bootstrap succeeded, the client is usable.
    /// Circuits are created on-demand and may fail individually.
    pub fn is_connected(&self) -> bool {
        // Arti manages connection state internally
        // If we got here, bootstrap succeeded
        true
    }
}

/// Bidirectional stream to an onion service
///
/// Wraps Arti's `DataStream` and implements standard async I/O traits.
/// Use with `tokio::io::{AsyncReadExt, AsyncWriteExt}` for convenience methods.
///
/// ## Example
///
/// ```no_run
/// use tokio::io::{AsyncReadExt, AsyncWriteExt};
/// # use songbird_onion_relay::TorTransport;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let transport = TorTransport::new().await?;
/// let mut stream = transport.connect("abc.onion", 80).await?;
///
/// // Write
/// stream.write_all(b"GET / HTTP/1.0\r\n\r\n").await?;
///
/// // Read
/// let mut response = Vec::new();
/// stream.read_to_end(&mut response).await?;
/// # Ok(())
/// # }
/// ```
pub struct TorStream {
    stream: arti_client::DataStream,
}

// Implement AsyncRead
impl AsyncRead for TorStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

// Implement AsyncWrite
impl AsyncWrite for TorStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        std::pin::Pin::new(&mut self.stream).poll_write(cx, buf)
    }
    
    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.stream).poll_flush(cx)
    }
    
    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tor_transport_creation() {
        // This just tests the structure, doesn't actually bootstrap
        // (That would require network and take 10-30s)
    }
    
    #[tokio::test]
    #[ignore = "Requires network and Tor consensus download (10-30s)"]
    async fn test_tor_bootstrap_real() {
        let result = TorTransport::new().await;
        assert!(result.is_ok(), "Bootstrap should succeed");
        
        let transport = result.unwrap();
        println!("✅ Bootstrap time: {:?}", transport.bootstrap_time());
        
        // Should take between 5s and 60s
        assert!(transport.bootstrap_time() >= Duration::from_secs(5));
        assert!(transport.bootstrap_time() < Duration::from_secs(60));
        assert!(transport.is_connected());
    }
    
    #[tokio::test]
    #[ignore = "Requires network and real onion service"]
    async fn test_tor_connect_real() {
        // First bootstrap
        let transport = match TorTransport::new().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("⚠️ Bootstrap failed (expected if no network): {}", e);
                return;
            }
        };
        
        // Try to connect to a test onion
        // Note: This will fail without a real onion service
        // Replace with actual test onion when available
        let test_onion = "thisisafake56characteronionaddressxxxxxxxxxxxxxxxxx.onion";
        let result = transport.connect(test_onion, 80).await;
        
        match result {
            Ok(_) => println!("✅ Connected to test onion"),
            Err(e) => println!("⚠️ Expected error (no test onion available): {}", e),
        }
    }
    
    #[tokio::test]
    async fn test_stream_implements_async_io() {
        // Compile-time test that TorStream implements the right traits
        fn assert_async_read<T: AsyncRead + Unpin>() {}
        fn assert_async_write<T: AsyncWrite + Unpin>() {}
        
        assert_async_read::<TorStream>();
        assert_async_write::<TorStream>();
    }
}
