// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Connection management for HTTP and HTTPS
//!
//! This module provides connection handlers for both plain HTTP and HTTPS (TLS) requests.

pub mod http;
pub mod https;

pub use http::HttpConnection;
pub use https::HttpsConnection;
