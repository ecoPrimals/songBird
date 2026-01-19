//! Test helpers for Songbird integration testing
//!
//! This module provides mock servers, test utilities, and common patterns
//! for testing Unix socket communication, BTSP, and HTTP gateway functionality.

pub mod btsp_mock;
pub mod http_mock;
pub mod scoped_env;
pub mod test_utils;

pub use btsp_mock::BearDogMock;
pub use http_mock::MockHttpApi;
pub use scoped_env::ScopedEnv;
pub use test_utils::*;
