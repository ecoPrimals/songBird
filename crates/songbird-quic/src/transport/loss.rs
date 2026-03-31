// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! RFC 9002: Loss detection and recovery for QUIC.
//!
//! Tracks sent packets and determines which are lost based on:
//! - Packet reordering threshold (`kPacketThreshold` = 3)
//! - Time threshold (`kTimeThreshold` = 9/8 of max(`smoothed_rtt`, `latest_rtt`))
//! - Probe Timeout (`PTO`) for tail loss

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

/// Default packet reordering threshold before declaring loss.
const PACKET_THRESHOLD: u64 = 3;

/// Default initial RTT estimate (333ms per RFC 9002).
const INITIAL_RTT: Duration = Duration::from_millis(333);

/// Minimum PTO (1ms granularity).
const MIN_PTO: Duration = Duration::from_millis(1);

/// Metadata about a sent packet (for loss detection).
#[derive(Debug, Clone)]
pub struct SentPacket {
    /// Packet number.
    pub pn: u64,
    /// Time the packet was sent.
    pub sent_time: Instant,
    /// Whether this packet is ack-eliciting.
    pub ack_eliciting: bool,
    /// Payload size in bytes (for congestion control).
    pub size: usize,
    /// Whether this packet has been acknowledged.
    pub acked: bool,
    /// Whether this packet has been declared lost.
    pub lost: bool,
}

/// RTT estimator (RFC 9002 Section 5.3).
#[derive(Debug, Clone)]
pub struct RttEstimator {
    /// Latest RTT measurement.
    latest_rtt: Duration,
    /// Smoothed RTT (exponential moving average).
    smoothed_rtt: Duration,
    /// RTT variance.
    rttvar: Duration,
    /// Minimum RTT observed.
    min_rtt: Duration,
    /// Whether any RTT sample has been collected.
    has_sample: bool,
}

impl Default for RttEstimator {
    fn default() -> Self {
        Self {
            latest_rtt: INITIAL_RTT,
            smoothed_rtt: INITIAL_RTT,
            rttvar: INITIAL_RTT / 2,
            min_rtt: Duration::MAX,
            has_sample: false,
        }
    }
}

impl RttEstimator {
    /// Current smoothed RTT.
    #[must_use]
    pub const fn smoothed_rtt(&self) -> Duration {
        self.smoothed_rtt
    }

    /// Current RTT variance.
    #[must_use]
    pub const fn rttvar(&self) -> Duration {
        self.rttvar
    }

    /// Minimum observed RTT.
    #[must_use]
    pub const fn min_rtt(&self) -> Duration {
        self.min_rtt
    }

    /// Latest RTT sample.
    #[must_use]
    pub const fn latest_rtt(&self) -> Duration {
        self.latest_rtt
    }

    /// Update with a new RTT sample (RFC 9002 Section 5.3).
    pub fn update(&mut self, rtt_sample: Duration, ack_delay: Duration) {
        self.latest_rtt = rtt_sample;

        if rtt_sample < self.min_rtt {
            self.min_rtt = rtt_sample;
        }

        if !self.has_sample {
            self.smoothed_rtt = rtt_sample;
            self.rttvar = rtt_sample / 2;
            self.has_sample = true;
            return;
        }

        let adjusted_rtt =
            rtt_sample.checked_sub(ack_delay).filter(|r| *r > self.min_rtt).unwrap_or(rtt_sample);

        let abs_diff = if self.smoothed_rtt > adjusted_rtt {
            self.smoothed_rtt.saturating_sub(adjusted_rtt)
        } else {
            adjusted_rtt.saturating_sub(self.smoothed_rtt)
        };
        self.rttvar = (self.rttvar * 3 + abs_diff) / 4;
        self.smoothed_rtt = (self.smoothed_rtt * 7 + adjusted_rtt) / 8;
    }
}

/// Loss detector for a single packet number space.
#[derive(Debug)]
pub struct LossDetector {
    /// Sent packets indexed by packet number.
    sent_packets: BTreeMap<u64, SentPacket>,
    /// Largest acknowledged packet number.
    largest_acked_pn: Option<u64>,
    /// RTT estimator.
    rtt: RttEstimator,
    /// PTO count (exponential backoff counter).
    pto_count: u32,
    /// Time of the last ack-eliciting packet sent.
    time_of_last_ack_eliciting: Option<Instant>,
}

impl Default for LossDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl LossDetector {
    /// Create a new loss detector.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sent_packets: BTreeMap::new(),
            largest_acked_pn: None,
            rtt: RttEstimator::default(),
            pto_count: 0,
            time_of_last_ack_eliciting: None,
        }
    }

    /// RTT estimator reference.
    #[must_use]
    pub const fn rtt(&self) -> &RttEstimator {
        &self.rtt
    }

    /// Record a sent packet.
    pub fn on_packet_sent(&mut self, packet: SentPacket) {
        if packet.ack_eliciting {
            self.time_of_last_ack_eliciting = Some(packet.sent_time);
        }
        self.sent_packets.insert(packet.pn, packet);
    }

    /// Process an ACK frame. Returns newly acknowledged packet numbers
    /// and a list of lost packet numbers.
    pub fn on_ack_received(
        &mut self,
        largest_acked: u64,
        ack_delay: Duration,
        ack_ranges: &[(u64, u64)],
        now: Instant,
    ) -> (Vec<u64>, Vec<u64>) {
        let mut newly_acked = Vec::new();

        // Mark acked packets
        for &(start, end) in ack_ranges {
            for pn in start..=end {
                if let Some(sent) = self.sent_packets.get_mut(&pn)
                    && !sent.acked
                {
                    sent.acked = true;
                    newly_acked.push(pn);
                }
            }
        }

        // Update RTT if the largest acked packet is newly acked
        if let Some(sent) = self.sent_packets.get(&largest_acked)
            && sent.acked
        {
            let rtt_sample = now.duration_since(sent.sent_time);
            self.rtt.update(rtt_sample, ack_delay);
        }

        let prev_largest = self.largest_acked_pn;
        self.largest_acked_pn =
            Some(self.largest_acked_pn.map_or(largest_acked, |prev| prev.max(largest_acked)));

        // Reset PTO on new ack
        if Some(largest_acked) > prev_largest {
            self.pto_count = 0;
        }

        // Detect lost packets
        let lost = self.detect_lost_packets(now);

        (newly_acked, lost)
    }

    /// Detect packets that should be declared lost (RFC 9002 Section 6.1).
    fn detect_lost_packets(&mut self, now: Instant) -> Vec<u64> {
        let Some(largest_acked) = self.largest_acked_pn else {
            return vec![];
        };

        let base = self.rtt.smoothed_rtt.max(self.rtt.latest_rtt);
        let time_threshold = base * 9u32 / 8u32;
        let loss_time = now.checked_sub(time_threshold).unwrap_or(now);

        let mut lost_pns = Vec::new();

        for (pn, sent) in &mut self.sent_packets {
            if sent.acked || sent.lost {
                continue;
            }
            if *pn > largest_acked {
                continue;
            }

            // Packet threshold: lost if at least PACKET_THRESHOLD packets
            // with higher PN have been acked.
            let pn_gap = largest_acked - *pn;
            if pn_gap >= PACKET_THRESHOLD {
                sent.lost = true;
                lost_pns.push(*pn);
                continue;
            }

            // Time threshold: lost if sent before loss_time.
            if sent.sent_time <= loss_time {
                sent.lost = true;
                lost_pns.push(*pn);
            }
        }

        lost_pns
    }

    /// Compute the Probe Timeout (PTO) (RFC 9002 Section 6.2).
    #[must_use]
    pub fn pto(&self) -> Duration {
        let pto = self.rtt.smoothed_rtt + self.rtt.rttvar.max(MIN_PTO) * 4;
        pto * 2u32.saturating_pow(self.pto_count)
    }

    /// Increment PTO count (called when PTO fires without receiving ack).
    pub fn on_pto_timeout(&mut self) {
        self.pto_count += 1;
    }

    /// Number of unacknowledged packets in flight.
    #[must_use]
    pub fn in_flight_count(&self) -> usize {
        self.sent_packets.values().filter(|p| !p.acked && !p.lost).count()
    }

    /// Total bytes in flight (for congestion control).
    #[must_use]
    pub fn bytes_in_flight(&self) -> usize {
        self.sent_packets.values().filter(|p| !p.acked && !p.lost).map(|p| p.size).sum()
    }

    /// Remove acknowledged and lost packets that are no longer needed.
    pub fn drain_completed(&mut self) {
        self.sent_packets.retain(|_, p| !p.acked && !p.lost);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sent(pn: u64, now: Instant) -> SentPacket {
        SentPacket {
            pn,
            sent_time: now,
            ack_eliciting: true,
            size: 100,
            acked: false,
            lost: false,
        }
    }

    #[test]
    fn rtt_initial_values() {
        let rtt = RttEstimator::default();
        assert_eq!(rtt.smoothed_rtt(), INITIAL_RTT);
        assert_eq!(rtt.min_rtt(), Duration::MAX);
    }

    #[test]
    fn rtt_first_sample() {
        let mut rtt = RttEstimator::default();
        let sample = Duration::from_millis(100);
        rtt.update(sample, Duration::ZERO);
        assert_eq!(rtt.smoothed_rtt(), sample);
        assert_eq!(rtt.rttvar(), sample / 2);
        assert_eq!(rtt.min_rtt(), sample);
    }

    #[test]
    fn rtt_subsequent_samples() {
        let mut rtt = RttEstimator::default();
        rtt.update(Duration::from_millis(100), Duration::ZERO);
        rtt.update(Duration::from_millis(120), Duration::ZERO);
        // Smoothed should be between 100 and 120
        assert!(rtt.smoothed_rtt() > Duration::from_millis(100));
        assert!(rtt.smoothed_rtt() < Duration::from_millis(120));
    }

    #[test]
    fn loss_detector_packet_threshold() {
        let now = Instant::now();
        let mut ld = LossDetector::new();

        for pn in 0..5 {
            ld.on_packet_sent(make_sent(pn, now));
        }

        // ACK packets 3, 4 (gap of 3+ from packet 0)
        let (acked, lost) =
            ld.on_ack_received(4, Duration::ZERO, &[(3, 4)], now + Duration::from_millis(50));
        assert_eq!(acked, vec![3, 4]);
        // Packet 0 and 1 should be lost (gap >= PACKET_THRESHOLD from 4)
        assert!(lost.contains(&0));
        assert!(lost.contains(&1));
    }

    #[test]
    fn pto_calculation() {
        let ld = LossDetector::new();
        let pto = ld.pto();
        // Initial: smoothed_rtt(333ms) + 4*rttvar(166ms) = 333 + 664 = ~997ms
        assert!(pto > Duration::from_millis(900));
        assert!(pto < Duration::from_millis(1100));
    }

    #[test]
    fn pto_backoff() {
        let mut ld = LossDetector::new();
        let base = ld.pto();
        ld.on_pto_timeout();
        assert_eq!(ld.pto(), base * 2);
        ld.on_pto_timeout();
        assert_eq!(ld.pto(), base * 4);
    }

    #[test]
    fn in_flight_tracking() {
        let now = Instant::now();
        let mut ld = LossDetector::new();
        assert_eq!(ld.in_flight_count(), 0);
        assert_eq!(ld.bytes_in_flight(), 0);

        ld.on_packet_sent(make_sent(0, now));
        ld.on_packet_sent(make_sent(1, now));
        assert_eq!(ld.in_flight_count(), 2);
        assert_eq!(ld.bytes_in_flight(), 200);

        ld.on_ack_received(0, Duration::ZERO, &[(0, 0)], now + Duration::from_millis(10));
        assert_eq!(ld.in_flight_count(), 1);
    }

    #[test]
    fn drain_completed() {
        let now = Instant::now();
        let mut ld = LossDetector::new();
        ld.on_packet_sent(make_sent(0, now));
        ld.on_packet_sent(make_sent(1, now));
        ld.on_ack_received(0, Duration::ZERO, &[(0, 0)], now);
        ld.drain_completed();
        assert_eq!(ld.in_flight_count(), 1);
    }

    #[test]
    fn ack_resets_pto_count() {
        let now = Instant::now();
        let mut ld = LossDetector::new();
        ld.on_packet_sent(make_sent(0, now));
        ld.on_pto_timeout();
        ld.on_pto_timeout();
        assert_eq!(ld.pto_count, 2);
        ld.on_ack_received(0, Duration::ZERO, &[(0, 0)], now + Duration::from_millis(10));
        assert_eq!(ld.pto_count, 0);
    }
}
