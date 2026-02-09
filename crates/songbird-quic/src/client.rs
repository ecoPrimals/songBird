//! QUIC client implementation

use crate::config::QuicConfig;
use crate::connection::QuicConnection;
use crate::error::Result;
use quinn::Endpoint;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, info};

/// QUIC client
///
/// Connects to QUIC servers with BearDog crypto delegation
pub struct QuicClient {
    /// Quinn endpoint
    endpoint: Endpoint,
    
    /// Client configuration
    config: Arc<QuicConfig>,
}

impl QuicClient {
    /// Create new QUIC client
    ///
    /// # Arguments
    ///
    /// * `config` - QUIC configuration
    ///
    /// # Errors
    ///
    /// Returns error if endpoint creation fails
    pub async fn new(config: QuicConfig) -> Result<Self> {
        info!("Creating QUIC client");
        
        // Build client configuration
        let client_config = config.build_client_config()?;
        
        // Create endpoint (binds to random port)
        let mut endpoint = Endpoint::client("[::]:0".parse().unwrap())?;
        endpoint.set_default_client_config(client_config);
        
        debug!("QUIC client bound to {}", endpoint.local_addr()?);
        
        Ok(Self {
            endpoint,
            config: Arc::new(config),
        })
    }
    
    /// Connect to remote server
    ///
    /// # Arguments
    ///
    /// * `remote_addr` - Remote address (e.g., `[2600::27]:4433`)
    ///
    /// # Errors
    ///
    /// Returns error if connection fails
    pub async fn connect(&self, remote_addr: &str) -> Result<QuicConnection> {
        let addr: SocketAddr = remote_addr.parse()?;
        
        info!("Connecting to {} via QUIC", addr);
        
        // Connect with SNI hostname (use IP for now, will be configurable)
        let connection = self.endpoint.connect(addr, "songbird.local")?
            .await?;
        
        info!("✅ Connected to {} via QUIC", addr);
        
        Ok(QuicConnection::new(connection, self.config.clone()))
    }
    
    /// Connect with 0-RTT (if enabled)
    ///
    /// Faster reconnection using cached session data
    pub async fn connect_0rtt(&self, remote_addr: &str) -> Result<QuicConnection> {
        if !self.config.enable_0rtt {
            return self.connect(remote_addr).await;
        }
        
        let addr: SocketAddr = remote_addr.parse()?;
        
        info!("Connecting to {} via QUIC (0-RTT attempt)", addr);
        
        let connecting = self.endpoint.connect(addr, "songbird.local")?;
        
        // Try 0-RTT first
        match connecting.into_0rtt() {
            Ok((connection, _zero_rtt_accepted)) => {
                info!("✅ Connected to {} via QUIC (0-RTT)", addr);
                Ok(QuicConnection::new(connection, self.config.clone()))
            }
            Err(connecting) => {
                // Fall back to 1-RTT
                debug!("0-RTT not available, using 1-RTT");
                let connection = connecting.await?;
                info!("✅ Connected to {} via QUIC (1-RTT fallback)", addr);
                Ok(QuicConnection::new(connection, self.config.clone()))
            }
        }
    }
    
    /// Close client
    pub async fn close(&self) {
        info!("Closing QUIC client");
        self.endpoint.close(0u32.into(), b"client shutdown");
        self.endpoint.wait_idle().await;
    }
}

impl Drop for QuicClient {
    fn drop(&mut self) {
        debug!("QUIC client dropped");
    }
}
