// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for [`crate::app::SongbirdOrchestrator`] helpers (broadcast discovery, lifecycle-related pure logic).

use std::sync::{Mutex, OnceLock};

use crate::app::SongbirdOrchestrator;

static BROADCAST_ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

fn broadcast_env_lock() -> std::sync::MutexGuard<'static, ()> {
    BROADCAST_ENV_MUTEX.get_or_init(|| Mutex::new(())).lock().expect("lock")
}

#[test]
fn discover_broadcast_addresses_respects_env_override() {
    let _g = broadcast_env_lock();
    let prev = std::env::var("SONGBIRD_BROADCAST_ADDRESSES").ok();
    songbird_process_env::set_var("SONGBIRD_BROADCAST_ADDRESSES", "127.0.0.1:9999");

    let addrs = SongbirdOrchestrator::discover_broadcast_addresses(&[]);
    assert!(
        addrs.iter().any(|a| a.to_string() == "127.0.0.1:9999"),
        "expected env override in {addrs:?}"
    );

    match prev {
        Some(v) => songbird_process_env::set_var("SONGBIRD_BROADCAST_ADDRESSES", v),
        None => songbird_process_env::remove_var("SONGBIRD_BROADCAST_ADDRESSES"),
    }
}

#[test]
fn discover_broadcast_addresses_merges_config_and_fallbacks() {
    let _g = broadcast_env_lock();
    let prev = std::env::var("SONGBIRD_BROADCAST_ADDRESSES").ok();
    songbird_process_env::remove_var("SONGBIRD_BROADCAST_ADDRESSES");

    let configured = vec!["10.0.0.1:2300".to_string()];
    let addrs = SongbirdOrchestrator::discover_broadcast_addresses(&configured);
    assert!(
        addrs.iter().any(|a| a.ip().to_string() == "10.0.0.1"),
        "configured address missing: {addrs:?}"
    );
    assert!(
        addrs.iter().any(|a| a.ip().to_string() == "192.168.1.255"),
        "expected subnet broadcast fallback: {addrs:?}"
    );

    match prev {
        Some(v) => songbird_process_env::set_var("SONGBIRD_BROADCAST_ADDRESSES", v),
        None => {}
    }
}

#[test]
fn discover_broadcast_addresses_empty_config_uses_defaults_when_no_env() {
    let _g = broadcast_env_lock();
    let prev = std::env::var("SONGBIRD_BROADCAST_ADDRESSES").ok();
    songbird_process_env::remove_var("SONGBIRD_BROADCAST_ADDRESSES");

    let addrs = SongbirdOrchestrator::discover_broadcast_addresses(&[]);
    assert!(!addrs.is_empty(), "expected default subnet broadcast fallbacks: {addrs:?}");
    assert!(
        addrs.iter().any(|a| a.ip().to_string() == "192.168.1.255")
            || addrs.iter().any(|a| a.ip().to_string() == "192.168.0.255"),
        "expected common /24 broadcast fallbacks: {addrs:?}"
    );

    match prev {
        Some(v) => songbird_process_env::set_var("SONGBIRD_BROADCAST_ADDRESSES", v),
        None => {}
    }
}
