// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use axum::http::StatusCode;
use serde_json::Value;

use super::super::types::JsonRpcError;

pub(super) fn extract_str_param(params: Option<&Value>, key: &str) -> Result<String, JsonRpcError> {
    params
        .and_then(|p| p.as_object())
        .and_then(|m| m.get(key))
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| JsonRpcError::invalid_params(format!("Missing '{key}'")))
}

pub(super) fn jsonrpc_from_compute_error(e: crate::server::compute_api::ApiError) -> JsonRpcError {
    use crate::server::compute_api::ApiError;
    match e {
        ApiError::Routing(msg) | ApiError::Execution(msg) => JsonRpcError::internal_error(msg),
        ApiError::InvalidRequest(msg) => JsonRpcError::invalid_params(msg),
        ApiError::NotFound(msg) => JsonRpcError {
            code: -32001,
            message: msg,
            data: None,
        },
    }
}

pub(super) fn jsonrpc_code_from_http_status(status: StatusCode) -> i32 {
    match status.as_u16() {
        404 => -32001,
        400..=499 => JsonRpcError::INVALID_PARAMS,
        _ => JsonRpcError::INTERNAL_ERROR,
    }
}
