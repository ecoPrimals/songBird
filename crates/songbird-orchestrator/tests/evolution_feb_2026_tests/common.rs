// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Shared fixtures for evolution tests (e.g. serializing env mutations).

use std::sync::Mutex;

/// File-local mutex to serialize tests that modify process-wide env vars.
pub static ENV_LOCK: Mutex<()> = Mutex::new(());
