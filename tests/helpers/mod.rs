// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals
//! Test helpers for Songbird integration testing
//!
//! This module provides mock servers, test utilities, and common patterns
//! for testing Unix socket communication, BTSP, and HTTP gateway functionality.

pub mod btsp_mock;
// http_mock archived: required warp; unused in current tests.
pub mod scoped_env;
pub mod test_utils;

pub use btsp_mock::SecurityProviderMock;
#[deprecated(note = "use SecurityProviderMock")]
pub use btsp_mock::BearDogMock;
pub use scoped_env::ScopedEnv;
pub use test_utils::*;
