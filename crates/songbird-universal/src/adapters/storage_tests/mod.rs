// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Storage adapter tests: metrics, HTTP discovery, and protocol selection.

use super::*;

pub(super) fn assert_protocol_debug(adapter: &StorageAdapter, expected: &str) {
    let dbg = format!("{adapter:?}");
    assert!(dbg.contains(expected), "expected Debug to contain {expected:?}, got {dbg}");
}

mod adapter_http;
mod discovery_transport;
mod storage_metrics;
mod storage_metrics_extended;
