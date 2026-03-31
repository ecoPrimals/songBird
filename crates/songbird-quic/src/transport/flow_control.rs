// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Connection-level and stream-level flow control (RFC 9000 Section 4).
//!
//! Flow control prevents a sender from overwhelming a receiver.
//! `QUIC` has both connection-level and stream-level flow control.

use crate::error::{QuicError, Result};

/// Tracks flow control state for a single direction (send or receive).
#[derive(Debug, Clone)]
pub struct FlowController {
    /// Maximum amount of data allowed.
    limit: u64,
    /// Amount of data consumed so far.
    consumed: u64,
    /// Whether we are currently blocked (waiting for peer to increase limit).
    blocked: bool,
}

impl FlowController {
    /// Create a new flow controller with the given initial limit.
    #[must_use]
    pub const fn new(limit: u64) -> Self {
        Self {
            limit,
            consumed: 0,
            blocked: false,
        }
    }

    /// Current limit.
    #[must_use]
    pub const fn limit(&self) -> u64 {
        self.limit
    }

    /// Amount consumed.
    #[must_use]
    pub const fn consumed(&self) -> u64 {
        self.consumed
    }

    /// Available window (limit - consumed).
    #[must_use]
    pub const fn available(&self) -> u64 {
        self.limit.saturating_sub(self.consumed)
    }

    /// Whether we are blocked.
    #[must_use]
    pub const fn is_blocked(&self) -> bool {
        self.blocked
    }

    /// Try to consume `amount` bytes. Returns error if exceeds limit.
    ///
    /// # Errors
    ///
    /// Returns [`QuicError::Stream`](crate::error::QuicError::Stream) if `amount` would exceed the flow limit.
    pub fn consume(&mut self, amount: u64) -> Result<()> {
        let new_consumed = self.consumed + amount;
        if new_consumed > self.limit {
            self.blocked = true;
            return Err(QuicError::Stream(format!(
                "Flow control limit exceeded: {new_consumed} > {}",
                self.limit
            )));
        }
        self.consumed = new_consumed;
        if self.consumed == self.limit {
            self.blocked = true;
        }
        Ok(())
    }

    /// Update the limit (e.g., from a `MAX_DATA` or `MAX_STREAM_DATA` frame).
    /// Only increases are accepted.
    pub fn update_limit(&mut self, new_limit: u64) {
        if new_limit > self.limit {
            self.limit = new_limit;
            self.blocked = false;
        }
    }

    /// Percentage of limit consumed (0.0 to 1.0).
    #[must_use]
    pub fn utilization(&self) -> f64 {
        if self.limit == 0 {
            return 0.0;
        }
        #[expect(
            clippy::cast_precision_loss,
            reason = "utilization ratio; u64 flow counts fit f64 mantissa for control-plane metrics"
        )]
        let ratio = self.consumed as f64 / self.limit as f64;
        ratio
    }

    /// Whether we should send a `MAX_DATA`/`MAX_STREAM_DATA` update.
    /// Triggers when utilization exceeds a threshold.
    #[must_use]
    pub fn should_send_update(&self) -> bool {
        self.utilization() > 0.5
    }
}

/// Connection-level flow control.
#[derive(Debug)]
pub struct ConnectionFlowControl {
    /// Sending direction: how much data we can send.
    pub send: FlowController,
    /// Receiving direction: how much data we allow the peer to send.
    pub recv: FlowController,
}

impl ConnectionFlowControl {
    /// Create with initial limits.
    #[must_use]
    pub const fn new(send_limit: u64, recv_limit: u64) -> Self {
        Self {
            send: FlowController::new(send_limit),
            recv: FlowController::new(recv_limit),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state() {
        let fc = FlowController::new(1000);
        assert_eq!(fc.limit(), 1000);
        assert_eq!(fc.consumed(), 0);
        assert_eq!(fc.available(), 1000);
        assert!(!fc.is_blocked());
    }

    #[test]
    fn consume_within_limit() {
        let mut fc = FlowController::new(100);
        fc.consume(50).unwrap();
        assert_eq!(fc.consumed(), 50);
        assert_eq!(fc.available(), 50);
        assert!(!fc.is_blocked());
    }

    #[test]
    fn consume_up_to_limit_blocks() {
        let mut fc = FlowController::new(100);
        fc.consume(100).unwrap();
        assert_eq!(fc.available(), 0);
        assert!(fc.is_blocked());
    }

    #[test]
    fn consume_exceeds_limit_errors() {
        let mut fc = FlowController::new(100);
        assert!(fc.consume(101).is_err());
        assert!(fc.is_blocked());
    }

    #[test]
    fn update_limit_increases_only() {
        let mut fc = FlowController::new(100);
        fc.consume(100).unwrap();
        assert!(fc.is_blocked());

        fc.update_limit(200);
        assert_eq!(fc.limit(), 200);
        assert_eq!(fc.available(), 100);
        assert!(!fc.is_blocked());

        // Decrease is ignored
        fc.update_limit(50);
        assert_eq!(fc.limit(), 200);
    }

    #[test]
    fn utilization() {
        let mut fc = FlowController::new(100);
        assert!((fc.utilization() - 0.0).abs() < f64::EPSILON);
        fc.consume(50).unwrap();
        assert!((fc.utilization() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn should_send_update_threshold() {
        let mut fc = FlowController::new(100);
        assert!(!fc.should_send_update());
        fc.consume(51).unwrap();
        assert!(fc.should_send_update());
    }

    #[test]
    fn zero_limit_utilization() {
        let fc = FlowController::new(0);
        assert!((fc.utilization() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn connection_flow_control() {
        let cfc = ConnectionFlowControl::new(1000, 2000);
        assert_eq!(cfc.send.limit(), 1000);
        assert_eq!(cfc.recv.limit(), 2000);
    }
}
