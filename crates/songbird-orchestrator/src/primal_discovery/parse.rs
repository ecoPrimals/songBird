// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! JSON-RPC `capabilities.list` response parsing.

/// Normalize `capabilities.list` / array-shaped results into flat capability tokens.
pub fn parse_capabilities_result(response: &serde_json::Value) -> Option<Vec<String>> {
    let result = response.get("result")?;
    if let Some(arr) = result.as_array() {
        return Some(
            arr.iter().filter_map(|v| v.as_str().map(std::string::ToString::to_string)).collect(),
        );
    }
    if let Some(obj) = result.as_object()
        && let Some(arr) = obj.get("capabilities").and_then(|c| c.as_array())
    {
        return Some(
            arr.iter().filter_map(|v| v.as_str().map(std::string::ToString::to_string)).collect(),
        );
    }
    None
}
