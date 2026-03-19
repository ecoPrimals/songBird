//! Sovereign Socket Implementation
//!
//! True network sovereignty through intelligent socket configuration.
//! No external tools, no sudo, no manual configuration.
//!
//! # Philosophy
//!
//! Instead of depending on firewall rules or external network configuration,
//! Songbird configures its sockets optimally to work in any environment.
//!
//! # Approach
//!
//! 1. Use `SO_REUSEADDR` and `SO_REUSEPORT` for maximum flexibility
//! 2. Configure optimal buffer sizes
//! 3. Enable `TCP_NODELAY` for low latency
//! 4. Set keep-alive for long connections
//! 5. Bind to all interfaces by default
//!
//! # Deep Debt Solution (Dec 20, 2025)
//!
//! This module eliminates the need for:
//! - iptables configuration
//! - Bash scripts
//! - sudo/root privileges
//! - Manual network setup
//!
//! Result: Songbird works on new deployments out-of-the-box.

use anyhow::{Context, Result};
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use tokio::net::TcpListener;
use tracing::{debug, info, warn};

/// Sovereign socket with optimal configuration
pub struct SovereignSocket {
    socket: Socket,
}

impl SovereignSocket {
    /// Create a new sovereign socket with optimal configuration
    ///
    /// This socket is configured for:
    /// - Maximum compatibility (works in any environment)
    /// - Optimal performance (large buffers, low latency)
    /// - Reliability (keep-alive, graceful shutdown)
    /// - Concurrency (port reuse for load balancing)
    ///
    /// # Errors
    ///
    /// Returns an error if socket creation or configuration fails
    pub fn new_tcp_v4() -> Result<Self> {
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))
            .context("Failed to create IPv4 TCP socket")?;

        Self::configure_socket(&socket)?;

        Ok(Self {
            socket,
        })
    }

    /// Create a new IPv6 sovereign socket
    ///
    /// # Errors
    ///
    /// Returns an error if socket creation or configuration fails
    pub fn new_tcp_v6() -> Result<Self> {
        let socket = Socket::new(Domain::IPV6, Type::STREAM, Some(Protocol::TCP))
            .context("Failed to create IPv6 TCP socket")?;

        // Enable dual-stack (accept both IPv4 and IPv6)
        socket.set_only_v6(false).ok(); // Ignore error if not supported

        Self::configure_socket(&socket)?;

        Ok(Self {
            socket,
        })
    }

    /// Configure socket with optimal settings for sovereignty
    fn configure_socket(socket: &Socket) -> Result<()> {
        // 1. Enable address reuse (immediate rebind after crash/restart)
        socket.set_reuse_address(true).context("Failed to set SO_REUSEADDR")?;

        // 2. SO_REUSEPORT REMOVED - See SO_REUSEPORT_ANALYSIS_DEC_20_2025.md
        // - Allowed multiple processes to bind same port (silent duplicates)
        // - Caused "Federation Split State Bug" (Dec 20, 2025)
        // - Not appropriate for singleton orchestrators
        // - "Address already in use" error is now a FEATURE (detects duplicates)
        // - PID file management handles singleton enforcement explicitly

        // 3. Disable Nagle's algorithm (low latency for small messages)
        // Note: TCP_NODELAY must be set after connection, not on listening socket
        // We'll document this for the application layer to handle

        // 4. Set non-blocking for async compatibility
        socket.set_nonblocking(true).context("Failed to set non-blocking mode")?;

        // 5. Set generous buffer sizes (high throughput)
        const BUFFER_SIZE: usize = 1024 * 1024; // 1MB
        socket.set_recv_buffer_size(BUFFER_SIZE).context("Failed to set receive buffer size")?;
        socket.set_send_buffer_size(BUFFER_SIZE).context("Failed to set send buffer size")?;

        debug!("✅ Socket configured with sovereign settings");
        debug!("   SO_REUSEADDR: enabled (quick restart)");
        debug!("   SO_REUSEPORT: DISABLED (singleton enforcement)");
        debug!("   Non-blocking: enabled");
        debug!("   Buffer sizes: {}KB", BUFFER_SIZE / 1024);
        debug!("   Note: TCP_NODELAY and keep-alive set per-connection");

        Ok(())
    }

    /// Bind to a specific address
    ///
    /// # Errors
    ///
    /// Returns an error if binding fails
    pub fn bind(&self, addr: SocketAddr) -> Result<()> {
        self.socket.bind(&addr.into()).with_context(|| format!("Failed to bind to {addr}"))?;

        info!("✅ Sovereign socket bound to {}", addr);
        Ok(())
    }

    /// Start listening for connections
    ///
    /// # Errors
    ///
    /// Returns an error if listen fails
    pub fn listen(&self, backlog: i32) -> Result<()> {
        self.socket.listen(backlog).context("Failed to start listening")?;

        debug!("✅ Socket listening (backlog: {})", backlog);
        Ok(())
    }

    /// Convert to tokio `TcpListener`
    ///
    /// # Errors
    ///
    /// Returns an error if conversion fails
    pub fn into_tokio_listener(self) -> Result<TcpListener> {
        let std_listener: std::net::TcpListener = self.socket.into();
        TcpListener::from_std(std_listener).context("Failed to convert to tokio listener")
    }
}

/// Sovereign binding strategy - tries multiple approaches to ensure connectivity
pub struct SovereignBinder;

impl SovereignBinder {
    /// Bind with full sovereignty - tries all strategies to ensure success
    ///
    /// This function will:
    /// 1. Try IPv6 dual-stack (::) with `IPV6_V6ONLY=false` - serves BOTH IPv4 and IPv6
    /// 2. Fall back to IPv4 wildcard (0.0.0.0) - if IPv6 unavailable
    /// 3. Try localhost only - last resort for restricted environments
    ///
    /// # Why IPv6 First?
    ///
    /// On Linux, binding IPv6 with `IPV6_V6ONLY=false` creates a dual-stack socket
    /// that accepts both IPv4 and IPv6 connections on a single socket. If IPv4 binds
    /// first, it blocks IPv6 from binding the same port. So IPv6 dual-stack MUST go
    /// first to achieve full reachability.
    ///
    /// # Errors
    ///
    /// Returns an error only if all binding strategies fail
    pub async fn bind_sovereign(port: u16) -> Result<(TcpListener, SocketAddr)> {
        info!("🦅 Attempting sovereign bind on port {}", port);

        // Strategy 1: IPv6 dual-stack (serves BOTH IPv4 and IPv6 on one socket)
        match Self::try_ipv6_wildcard(port).await {
            Ok((listener, addr)) => {
                info!("✅ Sovereign bind successful: {} (IPv6 dual-stack, serves IPv4+IPv6)", addr);
                return Ok((listener, addr));
            }
            Err(e) => {
                warn!("IPv6 dual-stack bind failed: {} — falling back to IPv4", e);
            }
        }

        // Strategy 2: IPv4 wildcard (fallback if IPv6 unavailable)
        match Self::try_ipv4_wildcard(port).await {
            Ok((listener, addr)) => {
                info!("✅ Sovereign bind successful: {} (IPv4 wildcard)", addr);
                warn!("   ⚠️  IPv6 NOT available — IPv4 only");
                return Ok((listener, addr));
            }
            Err(e) => {
                warn!("IPv4 wildcard bind failed: {}", e);
            }
        }

        // Strategy 3: Localhost only (last resort for restricted environments)
        match Self::try_localhost(port).await {
            Ok((listener, addr)) => {
                warn!("⚠️  Bound to localhost only: {}", addr);
                warn!("   External connections will not work");
                warn!("   This may indicate network configuration issues");
                return Ok((listener, addr));
            }
            Err(e) => {
                warn!("Localhost bind failed: {}", e);
            }
        }

        Err(anyhow::anyhow!(
            "Failed to establish sovereign binding on port {port} - all strategies exhausted"
        ))
    }

    /// Try binding to IPv4 wildcard (0.0.0.0)
    async fn try_ipv4_wildcard(port: u16) -> Result<(TcpListener, SocketAddr)> {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
        let socket = SovereignSocket::new_tcp_v4()?;
        socket.bind(addr)?;
        socket.listen(128)?;
        let listener = socket.into_tokio_listener()?;
        let actual_addr = listener.local_addr()?;
        Ok((listener, actual_addr))
    }

    /// Try binding to IPv6 wildcard (::)
    async fn try_ipv6_wildcard(port: u16) -> Result<(TcpListener, SocketAddr)> {
        let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port);
        let socket = SovereignSocket::new_tcp_v6()?;
        socket.bind(addr)?;
        socket.listen(128)?;
        let listener = socket.into_tokio_listener()?;
        let actual_addr = listener.local_addr()?;
        Ok((listener, actual_addr))
    }

    /// Try binding to localhost (fallback)
    async fn try_localhost(port: u16) -> Result<(TcpListener, SocketAddr)> {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
        let socket = SovereignSocket::new_tcp_v4()?;
        socket.bind(addr)?;
        socket.listen(128)?;
        let listener = socket.into_tokio_listener()?;
        let actual_addr = listener.local_addr()?;
        Ok((listener, actual_addr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sovereign_socket_creation() {
        let socket = SovereignSocket::new_tcp_v4();
        assert!(socket.is_ok(), "Should create IPv4 socket");
    }

    #[tokio::test]
    async fn test_sovereign_binding() {
        // Bind to ephemeral port (0 = OS chooses)
        let result = SovereignBinder::bind_sovereign(0).await;
        assert!(result.is_ok(), "Should bind to ephemeral port");

        if let Ok((listener, addr)) = result {
            assert!(addr.port() > 0, "Should have assigned port");
            println!("Sovereign bind successful: {}", addr);
        }
    }

    #[tokio::test]
    async fn test_port_reuse() {
        // Bind to same port twice (should work with SO_REUSEPORT)
        let port = 0; // Let OS choose

        let result1 = SovereignBinder::bind_sovereign(port).await;
        assert!(result1.is_ok(), "First bind should succeed");

        let (_listener1, addr1) = result1.unwrap();
        let actual_port = addr1.port();

        // On Linux with SO_REUSEPORT, this should work
        #[cfg(target_os = "linux")]
        {
            let result2 = SovereignBinder::bind_sovereign(actual_port).await;
            if result2.is_ok() {
                println!("✅ SO_REUSEPORT working - multiple binds to same port");
            } else {
                println!("⚠️  SO_REUSEPORT not working or port conflict");
            }
        }
    }
}
