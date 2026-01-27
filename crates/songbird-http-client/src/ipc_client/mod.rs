//! IPC HTTP Client module
//!
//! Provides an HTTP client that delegates to Songbird's HTTP service via IPC.

mod client;
pub mod multipart;

pub use client::{IpcHttpClient, RequestBuilder, Response};
pub use multipart::{Form, Part};
