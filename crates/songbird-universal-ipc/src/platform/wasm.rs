// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! WASM in-process IPC implementation
//!
//! **Platform**: WebAssembly (browser, Node.js, Deno, any WASM runtime).
//!
//! ## IPC strategy (capability discovery)
//!
//! Today there is **no global primal registry** in WASM; `listen` / `connect` therefore fail with
//! [`IpcError::RegistryError`] until one is implemented.
//! Planned directions:
//!
//! - **Browser main thread ↔ Web Worker:** `postMessage` / `MessageChannel` for structured
//!   cloning of control frames; pair with async coordination in the embedder.
//! - **Shared memory (where policy allows):** `SharedArrayBuffer` + `Atomics` for low-latency
//!   buffers after cross-origin isolation headers are set; treat as an optional fast path, not a
//!   default.
//! - **Same-module primals:** in-process `mpsc`-style queues keyed by logical endpoint ID (the
//!   shape `create_endpoint` already assigns).
//!
//! ## Current behavior
//!
//! - `create_endpoint` succeeds and returns [`NativeEndpoint::InProcess`](crate::endpoint::NativeEndpoint).
//! - `listen` / `connect` return errors until a registry backs endpoint lookup.
//! - `cleanup` is a no-op success for in-process IDs.

use crate::endpoint::NativeEndpoint;
use crate::error::{IpcError, IpcResult};
use crate::platform::{AsyncStreamImpl, PlatformListenerImpl};
use std::sync::atomic::{AtomicU16, Ordering};
use tracing::{debug, info};

/// WASM in-process IPC implementation
pub struct WasmPlatformIPC;

/// Logical endpoint ID counter (increments for each primal)
static ENDPOINT_ID_COUNTER: AtomicU16 = AtomicU16::new(1);

impl WasmPlatformIPC {
    /// Create a native endpoint for the given primal name.
    pub async fn create_endpoint(&self, primal_name: &str) -> IpcResult<NativeEndpoint> {
        let endpoint_id = ENDPOINT_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

        debug!("Creating WASM in-process endpoint for '{primal_name}': ID {endpoint_id}");

        info!(
            "WASM in-process endpoint (same runtime): {primal_name} (ID: {endpoint_id}, zero IPC overhead until cross-worker bridge)"
        );

        Ok(NativeEndpoint::InProcess(endpoint_id))
    }

    /// Create a listener on the native endpoint.
    pub async fn listen(&self, endpoint: &NativeEndpoint) -> IpcResult<PlatformListenerImpl> {
        match endpoint {
            NativeEndpoint::InProcess(id) => Err(IpcError::RegistryError(format!(
                "WASM primal listen not available (endpoint ID {id}): implement a global in-process registry or a Worker postMessage acceptor before listen()"
            ))),
            _ => Err(IpcError::PlatformError("WasmIPC requires InProcess endpoint".to_string())),
        }
    }

    /// Connect to a native endpoint.
    pub async fn connect(&self, endpoint: &NativeEndpoint) -> IpcResult<AsyncStreamImpl> {
        match endpoint {
            NativeEndpoint::InProcess(id) => Err(IpcError::RegistryError(format!(
                "WASM primal connect not available (endpoint ID {id}): register senders/receivers in a shared registry or use postMessage/SharedArrayBuffer worker bridge"
            ))),
            _ => Err(IpcError::PlatformError("WasmIPC requires InProcess endpoint".to_string())),
        }
    }

    /// Cleanup endpoint.
    pub async fn cleanup(&self, endpoint: &NativeEndpoint) -> IpcResult<()> {
        match endpoint {
            NativeEndpoint::InProcess(id) => {
                debug!("WASM in-process cleanup (no-op until registry exists): ID {id}");
                Ok(())
            }
            _ => Err(IpcError::PlatformError("WasmIPC requires InProcess endpoint".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wasm_create_endpoint() {
        let ipc = WasmPlatformIPC;
        let endpoint = ipc.create_endpoint("test-primal").await.unwrap();

        match endpoint {
            NativeEndpoint::InProcess(id) => {
                assert!(id > 0);
            }
            _ => panic!("Expected InProcess endpoint"),
        }
    }

    #[tokio::test]
    async fn test_wasm_unique_ids() {
        let ipc = WasmPlatformIPC;

        let endpoint1 = ipc.create_endpoint("primal1").await.unwrap();
        let endpoint2 = ipc.create_endpoint("primal2").await.unwrap();
        let endpoint3 = ipc.create_endpoint("primal3").await.unwrap();

        let NativeEndpoint::InProcess(id1) = endpoint1 else {
            panic!("Expected InProcess endpoint");
        };
        let NativeEndpoint::InProcess(id2) = endpoint2 else {
            panic!("Expected InProcess endpoint");
        };
        let NativeEndpoint::InProcess(id3) = endpoint3 else {
            panic!("Expected InProcess endpoint");
        };

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[tokio::test]
    async fn test_wasm_listen_returns_registry_error() {
        let ipc = WasmPlatformIPC;
        let ep = ipc.create_endpoint("x").await.unwrap();
        let result = ipc.listen(&ep).await;
        assert!(result.is_err(), "listen should fail on WASM");
        let err = result.err().expect("checked is_err");
        match err {
            IpcError::RegistryError(msg) => {
                assert!(msg.contains("WASM primal listen"));
            }
            e => panic!("expected RegistryError, got {e:?}"),
        }
    }
}
