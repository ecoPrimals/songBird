// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `NewReno` congestion control (RFC 9002 Appendix B).
//!
//! This is the baseline congestion controller for QUIC. It uses a
//! classic `AIMD` (additive increase, multiplicative decrease) approach
//! with slow start and congestion avoidance phases.

use std::time::Instant;

/// Initial congestion window (RFC 9002: min(10*MSS, max(2*MSS, 14720))).
const INITIAL_WINDOW_PACKETS: usize = 10;

/// Minimum congestion window (2 packets).
const MINIMUM_WINDOW_PACKETS: usize = 2;

/// Maximum Segment Size for QUIC.
const MSS: usize = 1200;

/// Congestion control state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CongestionState {
    /// Exponential growth (double `cwnd` per `RTT`).
    SlowStart,
    /// Linear growth (increase `cwnd` by `MSS` per `RTT`).
    CongestionAvoidance,
    /// After loss detected, waiting for recovery.
    Recovery,
}

/// `NewReno` congestion controller.
#[derive(Debug)]
pub struct NewReno {
    state: CongestionState,
    /// Congestion window in bytes.
    cwnd: usize,
    /// Slow start threshold in bytes.
    ssthresh: usize,
    /// Bytes acknowledged since last cwnd increase.
    bytes_acked_since_increase: usize,
    /// Largest packet number sent when entering recovery.
    recovery_start_pn: Option<u64>,
    /// Bytes currently in flight.
    bytes_in_flight: usize,
}

impl Default for NewReno {
    fn default() -> Self {
        Self::new()
    }
}

impl NewReno {
    /// Create a new congestion controller with the initial window.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: CongestionState::SlowStart,
            cwnd: INITIAL_WINDOW_PACKETS * MSS,
            ssthresh: usize::MAX,
            bytes_acked_since_increase: 0,
            recovery_start_pn: None,
            bytes_in_flight: 0,
        }
    }

    /// Current congestion state.
    #[must_use]
    pub const fn state(&self) -> CongestionState {
        self.state
    }

    /// Current congestion window in bytes.
    #[must_use]
    pub const fn cwnd(&self) -> usize {
        self.cwnd
    }

    /// Slow start threshold.
    #[must_use]
    pub const fn ssthresh(&self) -> usize {
        self.ssthresh
    }

    /// Available congestion window (`cwnd` - `bytes_in_flight`).
    #[must_use]
    pub fn available_cwnd(&self) -> usize {
        self.cwnd.saturating_sub(self.bytes_in_flight)
    }

    /// Whether we can send more data.
    #[must_use]
    pub fn can_send(&self, packet_size: usize) -> bool {
        self.bytes_in_flight + packet_size <= self.cwnd
    }

    /// Notify that bytes were sent.
    pub fn on_bytes_sent(&mut self, bytes: usize) {
        self.bytes_in_flight += bytes;
    }

    /// Process a packet acknowledgement (RFC 9002 Appendix B.5).
    pub fn on_packet_acked(&mut self, acked_bytes: usize, pn: u64, _sent_time: Instant) {
        self.bytes_in_flight = self.bytes_in_flight.saturating_sub(acked_bytes);

        // Don't increase cwnd for acks of packets sent before recovery
        if let Some(recovery_pn) = self.recovery_start_pn {
            if pn <= recovery_pn {
                return;
            }
            // Exiting recovery
            self.recovery_start_pn = None;
            self.state = if self.cwnd < self.ssthresh {
                CongestionState::SlowStart
            } else {
                CongestionState::CongestionAvoidance
            };
        }

        match self.state {
            CongestionState::SlowStart => {
                self.cwnd += acked_bytes;
                if self.cwnd >= self.ssthresh {
                    self.state = CongestionState::CongestionAvoidance;
                }
            }
            CongestionState::CongestionAvoidance => {
                self.bytes_acked_since_increase += acked_bytes;
                if self.bytes_acked_since_increase >= self.cwnd {
                    self.cwnd += MSS;
                    self.bytes_acked_since_increase = 0;
                }
            }
            CongestionState::Recovery => {
                // No cwnd increase during recovery
            }
        }
    }

    /// React to a packet loss (RFC 9002 Appendix B.6).
    pub fn on_packet_lost(&mut self, lost_bytes: usize, largest_lost_pn: u64) {
        self.bytes_in_flight = self.bytes_in_flight.saturating_sub(lost_bytes);

        // Only enter recovery once per flight
        if let Some(recovery_pn) = self.recovery_start_pn
            && largest_lost_pn <= recovery_pn
        {
            return;
        }

        self.state = CongestionState::Recovery;
        self.recovery_start_pn = Some(largest_lost_pn);

        // Multiplicative decrease
        self.ssthresh = self.cwnd / 2;
        self.cwnd = self.ssthresh.max(MINIMUM_WINDOW_PACKETS * MSS);
        self.bytes_acked_since_increase = 0;
    }

    /// React to a persistent congestion event.
    pub fn on_persistent_congestion(&mut self) {
        self.cwnd = MINIMUM_WINDOW_PACKETS * MSS;
        self.ssthresh = self.cwnd;
        self.state = CongestionState::SlowStart;
        self.recovery_start_pn = None;
        self.bytes_acked_since_increase = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state() {
        let cc = NewReno::new();
        assert_eq!(cc.state(), CongestionState::SlowStart);
        assert_eq!(cc.cwnd(), INITIAL_WINDOW_PACKETS * MSS);
        assert_eq!(cc.available_cwnd(), cc.cwnd());
        assert!(cc.can_send(MSS));
    }

    #[test]
    fn slow_start_growth() {
        let mut cc = NewReno::new();
        let now = Instant::now();
        let initial_cwnd = cc.cwnd();

        cc.on_bytes_sent(MSS);
        cc.on_packet_acked(MSS, 0, now);

        assert!(cc.cwnd() > initial_cwnd);
        assert_eq!(cc.cwnd(), initial_cwnd + MSS);
        assert_eq!(cc.state(), CongestionState::SlowStart);
    }

    #[test]
    fn transition_to_congestion_avoidance() {
        let mut cc = NewReno::new();
        let now = Instant::now();

        // Force ssthresh low
        cc.ssthresh = 3 * MSS;

        // Ack enough to exceed ssthresh
        for i in 0..5 {
            cc.on_bytes_sent(MSS);
            cc.on_packet_acked(MSS, i, now);
        }

        assert_eq!(cc.state(), CongestionState::CongestionAvoidance);
    }

    #[test]
    fn loss_triggers_recovery() {
        let mut cc = NewReno::new();
        let initial_cwnd = cc.cwnd();
        cc.on_bytes_sent(5 * MSS);

        cc.on_packet_lost(MSS, 0);

        assert_eq!(cc.state(), CongestionState::Recovery);
        assert!(cc.cwnd() < initial_cwnd);
        assert_eq!(cc.ssthresh(), initial_cwnd / 2);
    }

    #[test]
    fn no_double_recovery() {
        let mut cc = NewReno::new();
        cc.on_bytes_sent(5 * MSS);

        cc.on_packet_lost(MSS, 2);
        let cwnd_after_first = cc.cwnd();

        // Loss of earlier packet shouldn't trigger another recovery
        cc.on_packet_lost(MSS, 1);
        assert_eq!(cc.cwnd(), cwnd_after_first);
    }

    #[test]
    fn persistent_congestion_resets() {
        let mut cc = NewReno::new();
        cc.on_bytes_sent(5 * MSS);
        cc.on_packet_lost(MSS, 0);

        cc.on_persistent_congestion();
        assert_eq!(cc.cwnd(), MINIMUM_WINDOW_PACKETS * MSS);
        assert_eq!(cc.state(), CongestionState::SlowStart);
    }

    #[test]
    fn can_send_respects_cwnd() {
        let mut cc = NewReno::new();
        cc.on_bytes_sent(cc.cwnd());
        assert!(!cc.can_send(MSS));
    }

    #[test]
    fn available_cwnd_tracks_in_flight() {
        let mut cc = NewReno::new();
        let initial = cc.available_cwnd();
        cc.on_bytes_sent(3 * MSS);
        assert_eq!(cc.available_cwnd(), initial - 3 * MSS);
    }

    #[test]
    fn congestion_avoidance_linear_growth() {
        let mut cc = NewReno::new();
        let now = Instant::now();
        cc.ssthresh = cc.cwnd(); // Start in CA immediately
        cc.state = CongestionState::CongestionAvoidance;

        let ca_cwnd = cc.cwnd();
        // Need to ack cwnd bytes to get +MSS
        let acks_needed = ca_cwnd / MSS;
        for i in 0..acks_needed as u64 {
            cc.on_bytes_sent(MSS);
            cc.on_packet_acked(MSS, i + 100, now);
        }
        assert_eq!(cc.cwnd(), ca_cwnd + MSS);
    }

    #[test]
    fn minimum_window_enforced() {
        let mut cc = NewReno::new();
        cc.cwnd = MINIMUM_WINDOW_PACKETS * MSS;
        cc.on_bytes_sent(MSS);
        cc.on_packet_lost(MSS, 0);
        assert!(cc.cwnd() >= MINIMUM_WINDOW_PACKETS * MSS);
    }
}
