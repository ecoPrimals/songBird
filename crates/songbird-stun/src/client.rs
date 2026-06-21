// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! STUN client implementation
//!
//! **Pure Rust | Async | Zero Unsafe Code | Concurrent Racing**

use crate::error::{StunError, StunResult};
use crate::protocol::{
    infer_port_pattern_from_mapped_ports, local_bind_addr_for_peer, resolve_stun_server,
};
use crate::transaction::BindingTransaction;
use crate::types::{NatType, PortPattern, PublicEndpoint};

use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// STUN client for NAT traversal
///
/// Pure Rust implementation of RFC 5389 STUN protocol.
///
/// ## Credential Tier
///
/// When credentials are provided, they MUST be beacon-tier (mitochondrial)
/// per `DARK_FOREST_BEACON_GENETICS_STANDARD.md`. STUN servers can observe
/// the USERNAME attribute; using nuclear/lineage credentials here would
/// expose authorization material in NAT traversal traffic.
#[derive(Debug)]
pub struct StunClient {
    /// Request timeout
    timeout: Duration,

    /// Optional beacon-tier credentials for authenticated STUN requests.
    /// When set, binding requests include the USERNAME attribute.
    credentials: Option<crate::types::StunCredentials>,
}

impl StunClient {
    /// Create a new STUN client (unauthenticated)
    #[must_use]
    pub fn new() -> Self {
        let timeout_config = songbird_config::timeouts::TimeoutConfig::from_env();

        Self {
            timeout: timeout_config.connect,
            credentials: None,
        }
    }

    /// Create STUN client with custom timeout (unauthenticated)
    #[must_use]
    pub const fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
            credentials: None,
        }
    }

    /// Attach beacon-tier credentials for authenticated STUN requests.
    ///
    /// Per `DARK_FOREST_BEACON_GENETICS_STANDARD.md`, these MUST be
    /// mitochondrial/beacon-tier credentials — never nuclear/lineage.
    #[must_use]
    pub fn with_credentials(mut self, credentials: crate::types::StunCredentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Create a binding transaction, optionally authenticated.
    fn new_transaction(&self) -> BindingTransaction {
        match self.credentials {
            Some(ref creds) => BindingTransaction::with_credentials(creds),
            None => BindingTransaction::new(),
        }
    }

    /// Discover public address via STUN server
    ///
    /// # Arguments
    ///
    /// * `stun_server` - STUN server address (e.g. [`songbird_types::constants::DEFAULT_STUN_SERVER_1`])
    ///
    /// # Returns
    ///
    /// Public IP address and port as seen by the STUN server.
    ///
    /// # Errors
    ///
    /// Returns an error if the STUN server is unreachable or returns invalid response.
    ///
    /// # Privacy Note
    ///
    /// STUN servers can observe your public IP/port and connection timing.
    /// Prefer genetic lineage relay when sovereignty > convenience.
    pub async fn discover_public_address(&self, stun_server: &str) -> StunResult<SocketAddr> {
        info!("🔍 Discovering public address via STUN: {}", stun_server);

        let server_addr = resolve_stun_server(stun_server).await?;

        debug!("  Resolved STUN server: {}", server_addr);

        let bind_addr = local_bind_addr_for_peer(server_addr);

        let local_socket = UdpSocket::bind(bind_addr)
            .await
            .map_err(|e| StunError::Network(format!("Failed to bind UDP socket: {e}")))?;

        debug!("  Local socket bound: {}", local_socket.local_addr()?);

        let txn = self.new_transaction();
        let request_bytes = txn.encode_request();

        debug!("  Sending STUN binding request ({} bytes)", request_bytes.len());

        local_socket
            .send_to(&request_bytes, server_addr)
            .await
            .map_err(|e| StunError::Network(format!("Failed to send STUN request: {e}")))?;

        let mut buf = vec![0u8; 2048];

        let (recv_len, recv_addr) = timeout(self.timeout, local_socket.recv_from(&mut buf))
            .await
            .map_err(|_| StunError::Timeout(self.timeout))?
            .map_err(|e| StunError::Network(format!("Failed to receive STUN response: {e}")))?;

        debug!("  Received STUN response ({} bytes from {})", recv_len, recv_addr);

        let public_addr = txn.parse_response(&buf[..recv_len])?;

        info!("✅ Discovered public address: {}", public_addr);

        Ok(public_addr)
    }

    /// 🏁 Race multiple STUN servers concurrently; first successful response wins.
    ///
    /// # Errors
    ///
    /// Returns an error if all servers fail or timeout.
    ///
    /// ```rust,no_run
    /// use songbird_stun::StunClient;
    /// use songbird_types::constants::DEFAULT_STUN_SERVER_1;
    /// # async fn ex() -> Result<(), Box<dyn std::error::Error>> {
    /// let addr = StunClient::new()
    ///     .discover_public_address_racing(&[DEFAULT_STUN_SERVER_1])
    ///     .await?;
    /// # let _ = addr;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover_public_address_racing(
        &self,
        stun_servers: &[&str],
    ) -> StunResult<SocketAddr> {
        if stun_servers.is_empty() {
            return Err(StunError::Config(String::from("No STUN servers provided")));
        }

        info!("🏁 Racing {} STUN servers concurrently", stun_servers.len());

        let mut tasks = Vec::with_capacity(stun_servers.len());

        for (idx, server) in stun_servers.iter().enumerate() {
            let server = server.to_string();
            let timeout_duration = self.timeout;

            let client = Self::with_timeout(timeout_duration);

            let task = tokio::spawn(async move {
                debug!("🏁 Server {}: Attempting {}", idx, server);

                match timeout(timeout_duration, client.discover_public_address(&server)).await {
                    Ok(Ok(addr)) => {
                        info!("🏆 Server {}: SUCCESS! Discovered {}", idx, addr);
                        Ok((idx, addr))
                    }
                    Ok(Err(e)) => {
                        warn!("⚠️  Server {}: Failed - {}", idx, e);
                        Err(e)
                    }
                    Err(_) => {
                        warn!("⏱️  Server {}: Timeout after {:?}", idx, timeout_duration);
                        Err(StunError::Timeout(timeout_duration))
                    }
                }
            });

            tasks.push(task);
        }

        let mut last_error = None;

        while !tasks.is_empty() {
            let (result, _idx, remaining) = futures_util::future::select_all(tasks).await;

            match result {
                Ok(Ok((server_idx, addr))) => {
                    info!(
                        "✅ STUN racing complete! Server {} won (tried {} total)",
                        server_idx,
                        stun_servers.len()
                    );
                    return Ok(addr);
                }
                Ok(Err(e)) => {
                    last_error = Some(e);
                    tasks = remaining;
                }
                Err(e) => {
                    warn!("🔥 STUN task panic: {}", e);
                    tasks = remaining;
                }
            }
        }

        let error_msg = last_error.map_or_else(
            || format!("All {} STUN servers failed with unknown errors", stun_servers.len()),
            |e| format!("All {} STUN servers failed. Last error: {}", stun_servers.len(), e),
        );

        Err(StunError::AllServersFailed(error_msg))
    }

    /// Perform a STUN binding request on an **existing** UDP socket.
    ///
    /// Unlike [`discover_public_address`](Self::discover_public_address) which binds a
    /// fresh ephemeral socket per call, this method reuses the caller-provided socket.
    /// This is critical for symmetric-NAT detection: the same local endpoint must be
    /// probed against multiple STUN servers to observe whether the NAT assigns different
    /// external ports per destination.
    ///
    /// # Errors
    ///
    /// Returns an error if the STUN server is unreachable, unresolvable, or
    /// returns an invalid response.
    pub async fn discover_on_socket(
        &self,
        socket: &UdpSocket,
        stun_server: &str,
    ) -> StunResult<SocketAddr> {
        let server_addr = resolve_stun_server(stun_server).await?;

        let txn = self.new_transaction();
        let request_bytes = txn.encode_request();

        socket
            .send_to(&request_bytes, server_addr)
            .await
            .map_err(|e| StunError::Network(format!("Failed to send STUN request: {e}")))?;

        let mut buf = vec![0u8; 2048];

        let (recv_len, _recv_addr) = timeout(self.timeout, socket.recv_from(&mut buf))
            .await
            .map_err(|_| StunError::Timeout(self.timeout))?
            .map_err(|e| StunError::Network(format!("Failed to receive STUN response: {e}")))?;

        txn.parse_response(&buf[..recv_len])
    }

    /// Discover public endpoint **with** NAT type detection via two STUN servers.
    ///
    /// Binds a single local UDP socket and probes two servers. If the NAT
    /// assigns the same external port for both, it is likely cone-type;
    /// different ports indicate symmetric NAT.
    ///
    /// When only one server is provided, falls back to single-probe (returns
    /// `NatType::Unknown`).
    ///
    /// # Errors
    ///
    /// Returns an error if the primary STUN server is unreachable.
    pub async fn discover_public_endpoint(&self, stun_server: &str) -> StunResult<PublicEndpoint> {
        self.discover_public_endpoint_multi(&[stun_server]).await
    }

    /// Discover public endpoint using multiple STUN servers for NAT classification.
    ///
    /// Requires at least one server. Two or more enables NAT type detection
    /// via same-socket dual-probe (RFC 5780 simplified).
    ///
    /// # Errors
    ///
    /// Returns an error if all STUN servers fail.
    pub async fn discover_public_endpoint_multi(
        &self,
        stun_servers: &[&str],
    ) -> StunResult<PublicEndpoint> {
        if stun_servers.is_empty() {
            return Err(StunError::Config(String::from("No STUN servers provided")));
        }

        let first_server_addr = resolve_stun_server(stun_servers[0]).await?;
        let bind_addr = local_bind_addr_for_peer(first_server_addr);
        let socket = UdpSocket::bind(bind_addr)
            .await
            .map_err(|e| StunError::Network(format!("Failed to bind shared probe socket: {e}")))?;

        let addr1 = self.discover_on_socket(&socket, stun_servers[0]).await?;

        if stun_servers.len() < 2 {
            return Ok(PublicEndpoint {
                address: addr1,
                nat_type: NatType::Unknown,
            });
        }

        let nat_type = match self.discover_on_socket(&socket, stun_servers[1]).await {
            Ok(addr2) => classify_nat_from_dual_probes(addr1, addr2),
            Err(e) => {
                warn!("Second STUN probe failed (NAT type unknown): {e}");
                NatType::Unknown
            }
        };

        Ok(PublicEndpoint {
            address: addr1,
            nat_type,
        })
    }

    /// Test multiple STUN servers in parallel, use first success
    ///
    /// # Arguments
    ///
    /// * `stun_servers` - List of STUN server addresses
    ///
    /// # Returns
    ///
    /// First successful public address discovery.
    ///
    /// # Strategy
    ///
    /// Attempts all servers in parallel for fastest response.
    /// Useful when server availability is uncertain.
    ///
    /// # Errors
    ///
    /// Returns an error if no STUN servers provided or all fail.
    pub async fn discover_public_address_parallel(
        &self,
        stun_servers: &[String],
    ) -> StunResult<SocketAddr> {
        if stun_servers.is_empty() {
            return Err(StunError::Config(String::from("No STUN servers provided")));
        }

        info!("🔍 Discovering public address via {} STUN servers (parallel)", stun_servers.len());

        let mut handles = Vec::new();
        for server in stun_servers {
            let server = server.clone();
            let timeout = self.timeout;

            let handle = tokio::spawn(async move {
                let client = Self::with_timeout(timeout);
                client.discover_public_address(&server).await
            });

            handles.push(handle);
        }

        let (result, index, _remaining) = futures_util::future::select_all(handles).await;

        match result {
            Ok(Ok(addr)) => {
                info!("✅ First success from STUN server #{}", index);
                Ok(addr)
            }
            Ok(Err(e)) => {
                warn!("All STUN servers failed");
                Err(e)
            }
            Err(e) => Err(StunError::Network(format!("Task join error: {e}"))),
        }
    }

    /// Probe STUN server N times to detect NAT port allocation pattern
    ///
    /// Sends multiple STUN binding requests from a single socket to observe
    /// how the NAT allocates external ports. Pattern detection enables
    /// port prediction for coordinated hole punching.
    ///
    /// # Arguments
    ///
    /// * `stun_server` - STUN server to probe (e.g. [`songbird_types::constants::DEFAULT_STUN_SERVER_1`])
    /// * `probes` - Number of probes to send (recommended: 5–8)
    ///
    /// # Returns
    ///
    /// Detected [`PortPattern`] — `Sequential` if ports are predictable,
    /// `Random` if not, `Unknown` if probing failed.
    ///
    /// # Privacy Note
    ///
    /// Each probe reveals timing information to the STUN server.
    /// Use a self-hosted STUN server for sovereign operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the STUN server is unreachable.
    ///
    /// # Panics
    ///
    /// Panics if `ports` is empty when computing the last port (internal logic error).
    pub async fn probe_port_pattern(
        &self,
        stun_server: &str,
        probes: usize,
    ) -> StunResult<PortPattern> {
        let probes = probes.max(2);

        info!("🔍 Probing port pattern: {} probes to {}", probes, stun_server);

        let server_addr = resolve_stun_server(stun_server).await?;

        let mut ports = Vec::with_capacity(probes);

        for i in 0..probes {
            let bind_addr = local_bind_addr_for_peer(server_addr);

            let socket = UdpSocket::bind(bind_addr).await.map_err(|e| {
                StunError::Network(format!("Failed to bind UDP socket for probe {i}: {e}"))
            })?;

            let txn = self.new_transaction();
            let request_bytes = txn.encode_request();

            socket
                .send_to(&request_bytes, server_addr)
                .await
                .map_err(|e| StunError::Network(format!("Failed to send STUN probe {i}: {e}")))?;

            let mut buf = vec![0u8; 2048];
            match timeout(self.timeout, socket.recv_from(&mut buf)).await {
                Ok(Ok((recv_len, _))) => {
                    if let Ok(addr) = txn.parse_response(&buf[..recv_len]) {
                        debug!("  Probe {}: port {}", i + 1, addr.port());
                        ports.push(addr.port());
                    }
                }
                Ok(Err(e)) => {
                    warn!("  Probe {} recv error: {}", i + 1, e);
                }
                Err(_) => {
                    warn!("  Probe {} timed out", i + 1);
                }
            }
        }

        if ports.len() < 2 {
            info!("⚠️ Only {} successful probes — insufficient for pattern detection", ports.len());
            return Ok(PortPattern::Unknown);
        }

        let pattern = infer_port_pattern_from_mapped_ports(&ports);
        match &pattern {
            PortPattern::Sequential {
                step,
                confidence,
                predicted_next,
                ..
            } => {
                info!(
                    "✅ Sequential pattern detected: step={}, confidence={:.0}%, predicted_next={}",
                    step,
                    confidence * 100.0,
                    predicted_next
                );
            }
            PortPattern::Random {
                observed,
            } => {
                info!("⚠️ Random pattern detected: {} ports observed", observed.len(),);
            }
            PortPattern::Unknown => {}
        }

        Ok(pattern)
    }
}

/// Classify NAT behavior from two STUN binding results obtained via the
/// **same local socket** against two different servers.
///
/// Same external port → cone-type NAT (good for direct punch).
/// Different external ports → symmetric NAT (relay-assisted punch needed).
/// Different public IPs → unusual multi-homed topology.
#[must_use]
pub fn classify_nat_from_dual_probes(addr1: SocketAddr, addr2: SocketAddr) -> NatType {
    if addr1.ip() != addr2.ip() {
        NatType::Unknown
    } else if addr1.port() == addr2.port() {
        NatType::PortRestrictedCone
    } else {
        NatType::Symmetric
    }
}

impl Default for StunClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;
