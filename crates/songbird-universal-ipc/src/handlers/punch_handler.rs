//! Hole Punch JSON-RPC Handler
//!
//! Provides JSON-RPC methods for initiating and monitoring UDP hole punch
//! attempts to establish direct P2P connections across NAT boundaries.
//!
//! ## Methods
//!
//! - `punch.request` - Initiate hole punch attempt to a peer
//! - `punch.status` - Check status of ongoing punch attempts
//!
//! ## TRUE PRIMAL Architecture
//!
//! This handler coordinates with the `HolePunchCoordinator` from
//! `songbird-onion-relay` and `BeaconMesh` for fallback routing.

use serde_json::{json, Value};
use songbird_onion_relay::HolePunchCoordinator;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Status of a punch attempt
#[derive(Debug, Clone)]
pub struct PunchAttempt {
    /// Target node ID
    pub target_node_id: String,
    /// Current status
    pub status: PunchStatus,
    /// Number of attempts made
    pub attempts: u32,
    /// Max attempts before giving up
    pub max_attempts: u32,
    /// When the punch was started
    pub started: Instant,
    /// Connected address if successful
    pub connected_address: Option<SocketAddr>,
    /// Measured latency if successful
    pub latency: Option<Duration>,
}

/// Punch attempt status
#[derive(Debug, Clone, PartialEq)]
pub enum PunchStatus {
    /// Punch in progress
    InProgress,
    /// Punch succeeded - direct connection established
    Succeeded,
    /// Punch failed - will use relay fallback
    Failed { reason: String },
}

/// Punch handler for JSON-RPC integration
///
/// Manages hole punch attempts and provides status information
/// via JSON-RPC methods.
///
/// ## Design Principles
///
/// - **Coordinated**: Works with `HolePunchCoordinator`
/// - **Fallback-Aware**: Reports relay fallback on failure
/// - **Safe**: All operations use safe Rust
/// - **Async**: Modern async/await patterns
#[derive(Clone)]
pub struct PunchHandler {
    /// Active punch attempts
    attempts: Arc<RwLock<HashMap<String, PunchAttempt>>>,
    /// Hole punch coordinator (optional - may not be initialized)
    coordinator: Arc<RwLock<Option<Arc<HolePunchCoordinator>>>>,
    /// Default max attempts
    default_max_attempts: u32,
}

impl PunchHandler {
    /// Create a new punch handler
    pub fn new() -> Self {
        Self {
            attempts: Arc::new(RwLock::new(HashMap::new())),
            coordinator: Arc::new(RwLock::new(None)),
            default_max_attempts: 20,
        }
    }

    /// Create with an existing coordinator
    pub fn with_coordinator(coordinator: Arc<HolePunchCoordinator>) -> Self {
        Self {
            attempts: Arc::new(RwLock::new(HashMap::new())),
            coordinator: Arc::new(RwLock::new(Some(coordinator))),
            default_max_attempts: 20,
        }
    }

    /// Set the hole punch coordinator
    pub async fn set_coordinator(&self, coordinator: Arc<HolePunchCoordinator>) {
        *self.coordinator.write().await = Some(coordinator);
    }

    /// Handle `punch.request` method - Initiate hole punch
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "punch.request",
    ///   "params": {
    ///     "target_node_id": "pixel-xyz789",
    ///     "timeout_seconds": 10
    ///   },
    ///   "id": 1
    /// }
    /// ```
    ///
    /// # Response Example (Success)
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "result": {
    ///     "success": true,
    ///     "target_node_id": "pixel-xyz789",
    ///     "connected_address": "198.51.100.25:54321",
    ///     "latency_ms": 35,
    ///     "attempts": 5
    ///   },
    ///   "id": 1
    /// }
    /// ```
    pub async fn handle_request(&self, params: Value) -> Result<Value, String> {
        let target_node_id = params
            .get("target_node_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing target_node_id parameter")?
            .to_string();

        let timeout_seconds = params
            .get("timeout_seconds")
            .and_then(|v| v.as_u64())
            .unwrap_or(10);

        let max_attempts = params
            .get("max_attempts")
            .and_then(|v| v.as_u64())
            .unwrap_or(self.default_max_attempts as u64) as u32;

        info!(
            "🥊 Starting hole punch to {} (timeout: {}s, max: {} attempts)",
            &target_node_id[..8.min(target_node_id.len())],
            timeout_seconds,
            max_attempts
        );

        // Record the attempt
        let attempt = PunchAttempt {
            target_node_id: target_node_id.clone(),
            status: PunchStatus::InProgress,
            attempts: 0,
            max_attempts,
            started: Instant::now(),
            connected_address: None,
            latency: None,
        };

        self.attempts
            .write()
            .await
            .insert(target_node_id.clone(), attempt);

        // Check if we have a coordinator
        let coordinator = self.coordinator.read().await.clone();

        if let Some(coord) = coordinator {
            // ✅ Use the real HolePunchCoordinator (Feb 9, 2026)
            // Event-driven: coordinator uses signaling channels internally (no polling)
            let attempts_ref = self.attempts.clone();
            let target_id = target_node_id.clone();

            tokio::spawn(async move {
                // Update attempt status
                if let Some(attempt) = attempts_ref.write().await.get_mut(&target_id) {
                    attempt.attempts = 1;
                }

                // Use the real coordinator's punch_to_peer
                match coord.punch_to_peer(&target_id).await {
                    Ok(songbird_onion_relay::coordinator::PunchResult::Direct {
                        peer_addr,
                        latency,
                        ..
                    }) => {
                        if let Some(attempt) = attempts_ref.write().await.get_mut(&target_id) {
                            attempt.status = PunchStatus::Succeeded;
                            attempt.connected_address = Some(peer_addr);
                            attempt.latency = Some(latency);
                        }
                    }
                    Ok(songbird_onion_relay::coordinator::PunchResult::Relay { attempts: punch_count }) => {
                        if let Some(attempt) = attempts_ref.write().await.get_mut(&target_id) {
                            attempt.status = PunchStatus::Failed {
                                reason: format!("fell back to relay after {} attempts", punch_count),
                            };
                            attempt.attempts = punch_count;
                        }
                    }
                    Err(e) => {
                        if let Some(attempt) = attempts_ref.write().await.get_mut(&target_id) {
                            attempt.status = PunchStatus::Failed {
                                reason: format!("{}", e),
                            };
                        }
                    }
                }
            });

            // Return immediately - caller should poll status
            Ok(json!({
                "started": true,
                "target_node_id": target_node_id,
                "status": "in_progress",
                "timeout_seconds": timeout_seconds,
                "max_attempts": max_attempts
            }))
        } else {
            // No coordinator - simulate failure
            let mut attempts = self.attempts.write().await;
            if let Some(attempt) = attempts.get_mut(&target_node_id) {
                attempt.status = PunchStatus::Failed {
                    reason: "no_coordinator".to_string(),
                };
                attempt.attempts = 0;
            }

            Ok(json!({
                "success": false,
                "target_node_id": target_node_id,
                "attempts": 0,
                "reason": "hole_punch_coordinator_not_initialized",
                "fallback": "family_relay"
            }))
        }
    }

    /// Handle `punch.status` method - Check punch status
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "punch.status",
    ///   "params": {
    ///     "target_node_id": "pixel-xyz789"
    ///   },
    ///   "id": 2
    /// }
    /// ```
    pub async fn handle_status(&self, params: Value) -> Result<Value, String> {
        let target_node_id = params
            .get("target_node_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing target_node_id parameter")?;

        let attempts = self.attempts.read().await;

        if let Some(attempt) = attempts.get(target_node_id) {
            let (status_str, reason) = match &attempt.status {
                PunchStatus::InProgress => ("in_progress", None),
                PunchStatus::Succeeded => ("succeeded", None),
                PunchStatus::Failed { reason } => ("failed", Some(reason.clone())),
            };

            let mut response = json!({
                "target_node_id": target_node_id,
                "status": status_str,
                "attempts": attempt.attempts,
                "max_attempts": attempt.max_attempts,
                "elapsed_ms": attempt.started.elapsed().as_millis() as u64
            });

            if let Some(addr) = attempt.connected_address {
                response["connected_address"] = json!(addr.to_string());
            }

            if let Some(latency) = attempt.latency {
                response["latency_ms"] = json!(latency.as_millis() as u64);
            }

            if let Some(r) = reason {
                response["reason"] = json!(r);
                if status_str == "failed" {
                    response["fallback"] = json!("family_relay");
                }
            }

            Ok(response)
        } else {
            Ok(json!({
                "target_node_id": target_node_id,
                "status": "not_found",
                "reason": "no_punch_attempt_for_this_peer"
            }))
        }
    }

    /// Record a successful punch (called by coordinator callback)
    pub async fn record_success(
        &self,
        target_node_id: &str,
        connected_address: SocketAddr,
        latency: Duration,
        attempts: u32,
    ) {
        if let Some(attempt) = self.attempts.write().await.get_mut(target_node_id) {
            attempt.status = PunchStatus::Succeeded;
            attempt.connected_address = Some(connected_address);
            attempt.latency = Some(latency);
            attempt.attempts = attempts;

            info!(
                "✅ Hole punch to {} succeeded: {} ({}ms, {} attempts)",
                &target_node_id[..8.min(target_node_id.len())],
                connected_address,
                latency.as_millis(),
                attempts
            );
        }
    }

    /// Record a failed punch (called by coordinator callback)
    pub async fn record_failure(&self, target_node_id: &str, reason: String, attempts: u32) {
        if let Some(attempt) = self.attempts.write().await.get_mut(target_node_id) {
            attempt.status = PunchStatus::Failed { reason: reason.clone() };
            attempt.attempts = attempts;

            warn!(
                "❌ Hole punch to {} failed: {} ({} attempts)",
                &target_node_id[..8.min(target_node_id.len())],
                reason,
                attempts
            );
        }
    }
}

impl Default for PunchHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_punch_handler_new() {
        let handler = PunchHandler::new();
        assert_eq!(handler.default_max_attempts, 20);
    }

    #[tokio::test]
    async fn test_punch_request_no_coordinator() {
        let handler = PunchHandler::new();
        
        let result = handler
            .handle_request(json!({
                "target_node_id": "test-peer",
                "timeout_seconds": 5
            }))
            .await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["success"], false);
        assert_eq!(response["fallback"], "family_relay");
    }

    #[tokio::test]
    async fn test_punch_status_not_found() {
        let handler = PunchHandler::new();
        
        let result = handler
            .handle_status(json!({
                "target_node_id": "unknown-peer"
            }))
            .await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["status"], "not_found");
    }

    #[tokio::test]
    async fn test_punch_record_success() {
        let handler = PunchHandler::new();
        
        // Start a punch request
        handler
            .handle_request(json!({
                "target_node_id": "test-peer",
                "timeout_seconds": 5
            }))
            .await
            .unwrap();
        
        // Record success
        handler
            .record_success(
                "test-peer",
                "1.2.3.4:5678".parse().unwrap(),
                Duration::from_millis(45),
                5,
            )
            .await;
        
        // Check status
        let result = handler
            .handle_status(json!({
                "target_node_id": "test-peer"
            }))
            .await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["status"], "succeeded");
        assert_eq!(response["connected_address"], "1.2.3.4:5678");
        assert_eq!(response["latency_ms"], 45);
    }

    #[tokio::test]
    async fn test_punch_record_failure() {
        let handler = PunchHandler::new();
        
        // Start a punch request first
        handler
            .handle_request(json!({
                "target_node_id": "test-peer",
                "timeout_seconds": 5
            }))
            .await
            .unwrap();
        
        // Record failure
        handler
            .record_failure("test-peer", "symmetric_nat_both_sides".to_string(), 20)
            .await;
        
        // Check status
        let result = handler
            .handle_status(json!({
                "target_node_id": "test-peer"
            }))
            .await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["status"], "failed");
        assert_eq!(response["reason"], "symmetric_nat_both_sides");
        assert_eq!(response["fallback"], "family_relay");
    }
}
