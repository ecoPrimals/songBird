// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Health check related constants

use std::time::Duration;

/// Default health check interval
pub const DEFAULT_CHECK_INTERVAL: Duration = Duration::from_secs(30);

/// Default health check timeout
pub const DEFAULT_CHECK_TIMEOUT: Duration = Duration::from_secs(5);
