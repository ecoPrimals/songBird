// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Default timeout values

use std::time::Duration;

/// Default upper bound for completing a single request.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
/// Default time to wait when establishing a connection.
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Default idle period before closing or recycling a connection.
pub const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(30);
