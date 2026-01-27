//! Connection management for HTTP and HTTPS
//!
//! This module provides connection handlers for both plain HTTP and HTTPS (TLS) requests.

pub mod http;
pub mod https;

pub use http::HttpConnection;
pub use https::HttpsConnection;

