// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Multicast and timing constants for announcement-based discovery.

use std::time::Duration;

/// Multicast address octets for peer-to-peer service announcements.
pub const MULTICAST_ADDR_OCTETS: [u8; 4] = [239, 255, 255, 250];

pub const MULTICAST_PORT: u16 = 9091;

/// Below this threshold, `RuntimeDiscoveryEngine` skips mDNS daemon startup and the multicast
/// announcement listener so unit tests using ~1ms timeouts do not pay real network or `mdns-sd`
/// shutdown latency.
pub const MIN_TIMEOUT_FOR_SLOW_DISCOVERY_PATHS: Duration = Duration::from_millis(50);
