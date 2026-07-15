// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! JSON-RPC `storage.list` / `storage.get` result parsing and endpoint normalization.

use anyhow::Result;
use serde_json::Value;
#[cfg(unix)]
use std::path::PathBuf;

/// Interpret a storage capability endpoint as a local Unix socket path (`unix://…` or absolute path).
#[cfg(unix)]
#[must_use]
pub fn storage_socket_path_from_endpoint(endpoint: &str) -> Option<PathBuf> {
    let t = endpoint.trim();
    if let Some(p) = t.strip_prefix("unix://") {
        return Some(PathBuf::from(p));
    }
    if t.starts_with('/') {
        return Some(PathBuf::from(t));
    }
    None
}

pub fn parse_get_value_string(result: &Value) -> Result<Option<String>> {
    if result.is_null() {
        return Ok(None);
    }
    if let Some(s) = result.as_str() {
        if s.is_empty() {
            return Ok(None);
        }
        return Ok(Some(s.to_string()));
    }
    if let Some(inner) = result.get("value") {
        if inner.is_null() {
            return Ok(None);
        }
        if let Some(s) = inner.as_str() {
            return Ok(Some(s.to_string()));
        }
        return Ok(Some(inner.to_string()));
    }
    Ok(Some(result.to_string()))
}

pub fn parse_list_keys(result: &Value) -> Result<Vec<String>> {
    if let Some(arr) = result.as_array() {
        return arr
            .iter()
            .map(|v| {
                v.as_str()
                    .map(std::string::ToString::to_string)
                    .ok_or_else(|| anyhow::anyhow!("list: expected string keys"))
            })
            .collect();
    }
    if let Some(keys) = result.get("keys").and_then(|v| v.as_array()) {
        return keys
            .iter()
            .map(|v| {
                v.as_str()
                    .map(std::string::ToString::to_string)
                    .ok_or_else(|| anyhow::anyhow!("list.keys: expected strings"))
            })
            .collect();
    }
    if let Some(entries) = result.get("entries").and_then(|v| v.as_array()) {
        let mut out = Vec::new();
        for e in entries {
            if let Some(k) = e.as_str() {
                out.push(k.to_string());
            } else if let Some(k) = e.get("key").and_then(|x| x.as_str()) {
                out.push(k.to_string());
            }
        }
        return Ok(out);
    }
    Err(anyhow::anyhow!("storage.list: unexpected result shape"))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use serde_json::json;

    #[test]
    fn parse_get_accepts_string_result() {
        let v = json!("{\"id\":\"x\"}");
        let s = parse_get_value_string(&v).unwrap();
        assert_eq!(s, Some(String::from("{\"id\":\"x\"}")));
    }

    #[test]
    fn parse_get_accepts_value_object() {
        let v = json!({ "value": "{\"k\":1}" });
        let s = parse_get_value_string(&v).unwrap();
        assert_eq!(s, Some(String::from("{\"k\":1}")));
    }

    #[test]
    fn parse_list_accepts_keys_array() {
        let v = json!({ "keys": ["a", "b"] });
        let k = parse_list_keys(&v).unwrap();
        assert_eq!(k, vec!["a", "b"]);
    }

    #[test]
    fn parse_list_accepts_plain_array() {
        let v = json!(["x"]);
        let k = parse_list_keys(&v).unwrap();
        assert_eq!(k, vec!["x"]);
    }

    #[test]
    fn parse_list_accepts_entries_objects() {
        let v = json!({ "entries": [{ "key": "k1" }] });
        let k = parse_list_keys(&v).unwrap();
        assert_eq!(k, vec!["k1"]);
    }
}
