// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Allocation handling, session forwarding, and privacy masking for the relay server.

use crate::error::{LineageRelayError, Result};
use crate::relay::RelayAuthority;
use crate::relay_protocol::{AllocationRequest, AllocationResponse, RelayProtocol};
use crate::types::MaskingLevel;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::{RelayServerStats, RelaySessionState};

/// Handle single packet
pub(super) async fn handle_packet(
    socket: &Arc<UdpSocket>,
    sessions: &Arc<RwLock<HashMap<Uuid, RelaySessionState>>>,
    authority: &Arc<dyn RelayAuthority>,
    stats: &Arc<RwLock<RelayServerStats>>,
    relay_addr: SocketAddr,
    data: &[u8],
    src_addr: SocketAddr,
) -> Result<()> {
    match RelayProtocol::parse(data)? {
        RelayProtocol::AllocateRequest(req) => {
            handle_allocate(socket, sessions, authority, stats, relay_addr, req, src_addr).await
        }
        RelayProtocol::DataPacket {
            session_id,
            data,
        } => forward_packet(socket, sessions, stats, session_id, &data, src_addr).await,
        RelayProtocol::Refresh {
            session_id,
        } => refresh_session(sessions, session_id, src_addr).await,
        RelayProtocol::Deallocate {
            session_id,
        } => deallocate_session(sessions, session_id, src_addr).await,
        RelayProtocol::AllocateResponse(_) => {
            // Server doesn't handle responses (client-only message)
            Ok(())
        }
    }
}

/// Handle allocation request
async fn handle_allocate(
    socket: &Arc<UdpSocket>,
    sessions: &Arc<RwLock<HashMap<Uuid, RelaySessionState>>>,
    authority: &Arc<dyn RelayAuthority>,
    stats: &Arc<RwLock<RelayServerStats>>,
    relay_addr: SocketAddr,
    request: AllocationRequest,
    src_addr: SocketAddr,
) -> Result<()> {
    debug!("🔐 Allocation request from {} for {}", request.requester, request.target_addr);

    // Verify lineage authorization
    let auth_result = authority.authorize_relay(&request.relay_node, &request.requester).await;

    let response = match auth_result {
        Ok(auth) => {
            if auth.authorized {
                // Authorized - create session
                let session_id = Uuid::new_v4();

                let session = RelaySessionState {
                    session_id,
                    requester_addr: src_addr,
                    target_addr: request.target_addr,
                    requester_id: request.requester.clone(),
                    target_id: "unknown".into(), // Will be discovered on first packet
                    masking_level: auth.masking_level,
                    created_at: SystemTime::now(),
                    last_activity: SystemTime::now(),
                    bytes_forwarded: 0,
                    packets_forwarded: 0,
                };

                // Store session (drop sessions lock before updating stats)
                {
                    let mut sessions_guard = sessions.write().await;
                    sessions_guard.insert(session_id, session);
                    let active = sessions_guard.len() as u64;
                    drop(sessions_guard);

                    let mut stats_guard = stats.write().await;
                    stats_guard.sessions_active = active;
                    stats_guard.sessions_total += 1;
                }

                info!("✅ Allocated relay session {} for {}", session_id, request.requester);

                AllocationResponse::success(session_id, relay_addr, request.ttl_seconds)
            } else {
                // Not authorized
                warn!("🚫 Unauthorized relay request from {}", request.requester);

                {
                    let mut stats_guard = stats.write().await;
                    stats_guard.authorization_failures += 1;
                }
                AllocationResponse::unauthorized("Lineage verification failed")
            }
        }
        Err(e) => {
            // Authorization check failed
            warn!("⚠️  Authorization error: {}", e);

            {
                let mut stats_guard = stats.write().await;
                stats_guard.authorization_failures += 1;
            }
            AllocationResponse::error(format!("Authorization failed: {e}"))
        }
    };

    // Send response
    let response_msg = RelayProtocol::AllocateResponse(response);
    let encoded = response_msg.encode();
    socket.send_to(&encoded, src_addr).await.map_err(|e| {
        LineageRelayError::NetworkError(format!("Failed to send allocation response: {e}"))
    })?;

    Ok(())
}

/// Forward packet between peers
///
/// This is the CORE functionality that replaces the stub in `RelaySession.send()`
async fn forward_packet(
    socket: &Arc<UdpSocket>,
    sessions: &Arc<RwLock<HashMap<Uuid, RelaySessionState>>>,
    stats: &Arc<RwLock<RelayServerStats>>,
    session_id: Uuid,
    data: &[u8],
    src_addr: SocketAddr,
) -> Result<()> {
    let (dest_addr, masking_level, data_len) = {
        let mut sessions_guard = sessions.write().await;

        let session = sessions_guard.get_mut(&session_id).ok_or_else(|| {
            LineageRelayError::SessionNotFound(format!("Session {session_id} not found"))
        })?;

        // Determine destination (the other peer)
        let dest_addr = if src_addr == session.requester_addr {
            // From requester → to target
            session.target_addr
        } else if src_addr.ip() == session.target_addr.ip() {
            // From target → to requester
            // Note: Port might differ due to NAT, but IP should match
            session.requester_addr
        } else {
            // Unknown source - reject
            warn!(
                "🚫 Packet from unauthorized source {} (session {}, expected {} or {})",
                src_addr, session_id, session.requester_addr, session.target_addr
            );
            return Ok(()); // Silently drop (don't error, just ignore)
        };

        // Update session activity
        session.last_activity = SystemTime::now();
        session.bytes_forwarded += data.len() as u64;
        session.packets_forwarded += 1;

        let masking_level = session.masking_level;
        let data_len = data.len();
        drop(sessions_guard);

        (dest_addr, masking_level, data_len)
    };

    // Apply masking based on lineage relationship
    let masked_data = apply_masking(data, masking_level)?;

    // Forward packet
    socket
        .send_to(&masked_data, dest_addr)
        .await
        .map_err(|e| LineageRelayError::NetworkError(format!("Failed to forward packet: {e}")))?;

    // Update global stats
    {
        let mut stats_guard = stats.write().await;
        stats_guard.bytes_forwarded += data_len as u64;
        stats_guard.packets_forwarded += 1;
    }

    debug!(
        "📦 Forwarded {} bytes: {} → {} (session: {})",
        data_len, src_addr, dest_addr, session_id
    );

    Ok(())
}

/// Apply privacy masking based on lineage relationship
///
/// Closer family = less masking, distant family = more masking
#[expect(
    clippy::unnecessary_wraps,
    reason = "intentional pattern; clippy false positive for this API"
)] // Result kept for future masking errors
pub(super) fn apply_masking(data: &[u8], level: MaskingLevel) -> Result<Vec<u8>> {
    match level {
        MaskingLevel::None => {
            // Direct family (parent ↔ child): No masking
            Ok(data.to_vec())
        }
        MaskingLevel::TimingOnly => {
            // Close family (siblings): Timing jitter only
            // Future: Add random delay (not in packet data)
            Ok(data.to_vec())
        }
        MaskingLevel::SizeObfuscation => {
            // Extended family: Pad to fixed sizes
            let mut padded = data.to_vec();
            // Pad to next 1KB boundary
            let target_size = data.len().div_ceil(1024) * 1024;
            padded.resize(target_size, 0);
            Ok(padded)
        }
        MaskingLevel::Full => {
            // Distant family: Full encryption + padding
            // Future: Integrate with security provider encryption
            // For now, just pad (encryption is future enhancement)
            let mut padded = data.to_vec();
            let target_size = data.len().div_ceil(1024) * 1024;
            padded.resize(target_size, 0);
            Ok(padded)
        }
        // Legacy variants (for backward compatibility)
        MaskingLevel::Masked | MaskingLevel::SubMasked => {
            // Minimal masking (legacy default)
            Ok(data.to_vec())
        }
        MaskingLevel::FullVisibility => {
            // Full visibility (ancestor privilege - legacy)
            Ok(data.to_vec())
        }
    }
}

/// Refresh session (extend TTL)
async fn refresh_session(
    sessions: &Arc<RwLock<HashMap<Uuid, RelaySessionState>>>,
    session_id: Uuid,
    src_addr: SocketAddr,
) -> Result<()> {
    {
        let mut sessions_guard = sessions.write().await;

        if let Some(session) = sessions_guard.get_mut(&session_id) {
            // Verify refresh comes from requester or target
            if src_addr == session.requester_addr || src_addr.ip() == session.target_addr.ip() {
                session.last_activity = SystemTime::now();
                debug!("🔄 Refreshed session {}", session_id);
            } else {
                warn!("🚫 Refresh from unauthorized source: {}", src_addr);
            }
        }
    }

    Ok(())
}

/// Deallocate session (close)
async fn deallocate_session(
    sessions: &Arc<RwLock<HashMap<Uuid, RelaySessionState>>>,
    session_id: Uuid,
    src_addr: SocketAddr,
) -> Result<()> {
    let mut sessions_guard = sessions.write().await;

    if let Some(session) = sessions_guard.get(&session_id) {
        // Verify deallocation comes from requester
        if src_addr == session.requester_addr {
            sessions_guard.remove(&session_id);
            drop(sessions_guard);
            info!("🛑 Deallocated session {}", session_id);
        } else {
            warn!("🚫 Deallocation from unauthorized source: {}", src_addr);
        }
    }

    Ok(())
}
