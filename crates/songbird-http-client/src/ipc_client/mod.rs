//! IPC HTTP Client module
//!
//! Provides an HTTP client that delegates to Songbird's HTTP service via IPC.
//!
//! # Connection Pooling (NEW - Feb 3, 2026)
//!
//! The IPC HTTP client now supports optional connection pooling for improved performance:
//!
//! ```no_run
//! # use songbird_http_client::IpcHttpClient;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // With connection pooling (recommended)
//! let client = IpcHttpClient::builder()
//!     .with_connection_pool(20)
//!     .build()
//!     .await?;
//! # Ok(())
//! # }
//! ```

mod client;
pub mod multipart;

pub use client::{IpcHttpClient, IpcHttpClientBuilder, RequestBuilder, Response};
pub use multipart::{Form, Part};
