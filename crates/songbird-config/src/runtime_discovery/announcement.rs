// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Multicast UDP listener for JSON capability announcements (slow path).

use songbird_types::{SongbirdError, SongbirdResult};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info};

use super::constants::{
    MIN_TIMEOUT_FOR_SLOW_DISCOVERY_PATHS, MULTICAST_ADDR_OCTETS, MULTICAST_PORT,
};
use super::types::{DiscoveredService, DiscoveryMethod};

pub async fn wait_for_announcement(
    discovery_timeout: Duration,
    capability: &str,
) -> SongbirdResult<DiscoveredService> {
    debug!("Waiting for announcement for capability '{}'", capability);

    if discovery_timeout < MIN_TIMEOUT_FOR_SLOW_DISCOVERY_PATHS {
        return Err(SongbirdError::discovery(format!(
            "No announcement received for capability '{capability}' (timeout below slow-path minimum)"
        )));
    }

    let (tx, mut rx) = tokio::sync::mpsc::channel::<DiscoveredService>(10);

    let capability_clone = capability.to_string();
    let timeout_duration = discovery_timeout;

    tokio::spawn(async move {
        use std::net::{Ipv4Addr, SocketAddrV4};
        use tokio::net::UdpSocket;

        debug!(
            "Announcement listener started for capability '{}' (timeout: {:?})",
            capability_clone, timeout_duration
        );

        let multicast_addr = Ipv4Addr::from(MULTICAST_ADDR_OCTETS);

        match UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, MULTICAST_PORT)).await {
            Ok(socket) => {
                if let Err(e) = socket.join_multicast_v4(multicast_addr, Ipv4Addr::UNSPECIFIED) {
                    debug!("Failed to join multicast group: {}", e);
                    drop(tx);
                    return;
                }

                let mut buf = [0u8; 1024];
                let start = std::time::Instant::now();

                while start.elapsed() < timeout_duration {
                    match tokio::time::timeout(
                        Duration::from_millis(100),
                        socket.recv_from(&mut buf),
                    )
                    .await
                    {
                        Ok(Ok((len, addr))) => {
                            if let Ok(announcement) = std::str::from_utf8(&buf[..len])
                                && let Ok(json) =
                                    serde_json::from_str::<serde_json::Value>(announcement)
                                && let Some(caps) =
                                    json.get("capabilities").and_then(|c| c.as_array())
                            {
                                let has_capability =
                                    caps.iter().any(|c| c.as_str() == Some(&capability_clone));

                                if has_capability
                                    && let Some(endpoint) =
                                        json.get("endpoint").and_then(|e| e.as_str())
                                {
                                    debug!(
                                        "Received matching announcement from {} for '{}'",
                                        addr, capability_clone
                                    );

                                    let service = DiscoveredService {
                                        capability: capability_clone.clone(),
                                        endpoint: endpoint.to_string(),
                                        discovered_via: DiscoveryMethod::Announcement,
                                        health_score: 1.0,
                                        last_seen: std::time::SystemTime::now(),
                                    };

                                    let _ = tx.send(service).await;
                                    return;
                                }
                            }
                        }
                        _ => {
                            tokio::task::yield_now().await;
                        }
                    }
                }

                debug!("Announcement listener timeout for '{}'", capability_clone);
            }
            Err(e) => {
                debug!("Failed to bind UDP socket for announcements: {}", e);
            }
        }

        drop(tx);
    });

    match timeout(discovery_timeout, rx.recv()).await {
        Ok(Some(service)) => {
            info!(
                "Received announcement for capability '{}' from '{}'",
                capability, service.endpoint
            );
            Ok(service)
        }
        Ok(None) => {
            debug!("Announcement channel closed without receiving service");
            Err(SongbirdError::discovery(format!(
                "No announcement received for capability '{capability}'"
            )))
        }
        Err(_) => {
            debug!("Announcement wait timed out for capability '{}'", capability);
            Err(SongbirdError::discovery(format!(
                "Timeout waiting for announcement for capability '{capability}'"
            )))
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::super::constants::MIN_TIMEOUT_FOR_SLOW_DISCOVERY_PATHS;
    use super::wait_for_announcement;
    use songbird_types::SongbirdError;
    use std::time::Duration;

    #[tokio::test]
    async fn wait_for_announcement_errors_when_timeout_below_slow_path_minimum() {
        let err = wait_for_announcement(Duration::from_millis(1), "sb-cap-too-fast")
            .await
            .expect_err("below-min timeout must fail fast");
        assert!(matches!(err, SongbirdError::Discovery { .. }), "{err:?}");
        let text = err.to_string();
        assert!(text.contains("sb-cap-too-fast") || text.contains("timeout"));
    }

    #[tokio::test]
    async fn min_timeout_constant_matches_slow_path_contract() {
        assert!(MIN_TIMEOUT_FOR_SLOW_DISCOVERY_PATHS >= Duration::from_millis(50));
    }

    #[tokio::test(start_paused = true)]
    async fn wait_for_announcement_times_out_when_no_sender() {
        let cap = "sb_ann_pause_cap";
        let h = tokio::spawn(wait_for_announcement(Duration::from_millis(60), cap));
        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_millis(200)).await;
        let err = h.await.expect("join").expect_err("expected timeout or closed channel");
        assert!(matches!(err, SongbirdError::Discovery { .. }), "{err:?}");
        assert!(err.to_string().contains(cap), "{err}");
    }
}
