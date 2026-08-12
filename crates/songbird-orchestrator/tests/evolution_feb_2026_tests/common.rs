// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Shared fixtures for evolution tests (e.g. serializing env mutations).

/// Serialize tests that modify process-wide env vars.
pub fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    songbird_process_env::test_env_lock()
}
