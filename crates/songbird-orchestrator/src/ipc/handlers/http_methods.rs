// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! HTTP/HTTPS JSON-RPC convenience methods (Tower Atomic Pure Rust TLS 1.3).

use super::IpcHandlers;

impl IpcHandlers {
    /// Handle `http.request` RPC call (Pure Rust TLS 1.3, v5.27.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn http_request(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        self.http_handler.handle_request(params).await
    }

    /// Handle `http.get` convenience method (v5.27.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn http_get(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        self.http_handler.handle_get(params).await
    }

    /// Handle `http.post` convenience method (v5.27.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn http_post(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        self.http_handler.handle_post(params).await
    }

    /// Handle `http.put` convenience method (v5.27.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn http_put(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        self.http_handler.handle_put(params).await
    }

    /// Handle `http.delete` convenience method (v5.27.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn http_delete(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        self.http_handler.handle_delete(params).await
    }
}
