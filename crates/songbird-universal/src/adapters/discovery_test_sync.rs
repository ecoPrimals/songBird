// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Single mutex for [`songbird_process_env`] overlay tests — discovery fallback tests must not run in parallel.

use std::sync::{Mutex, OnceLock};

static DISCOVERY_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// Serialize tests that set process environment variables for capability discovery.
pub(crate) fn lock_discovery_env() -> std::sync::MutexGuard<'static, ()> {
    DISCOVERY_ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
