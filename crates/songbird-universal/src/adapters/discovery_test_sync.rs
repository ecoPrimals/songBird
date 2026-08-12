// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Serialize [`songbird_process_env`] overlay tests — discovery fallback tests must not run in parallel.

/// Serialize tests that set or read process environment variables for capability discovery.
pub fn lock_discovery_env() -> std::sync::MutexGuard<'static, ()> {
    songbird_process_env::test_env_lock()
}
