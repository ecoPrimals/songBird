// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Load Balancer Error Path Tests
//!
//! Focused test suite for load balancer error conditions and edge cases.
//! These tests specifically target scenarios that are under-covered:
//! - Error conditions (no endpoints, all unavailable, etc.)
//! - Edge cases (single endpoint, very large counts, etc.)
//! - Concurrent access patterns
//!
//! Coverage Goal: Add 50 tests to increase Universal crate coverage

use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::load_balancer::{LoadBalancer, LoadBalancingStrategy};
use std::sync::Arc;

mod availability_and_health_updates;
mod concurrency;
mod counter_and_strategy_types;
mod empty_and_unavailable;
mod endpoint_formats;
mod health_and_strategy;
mod round_robin_sequence;
mod single_endpoint_and_scale;
mod url_edge_cases;
