// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Hand-crafted malformed JSON-RPC inputs for [`JsonRpcRequest`] deserialization (fuzz-style).

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::JsonRpcRequest;
use serde_json::json;

#[test]
fn deserialize_rejects_malformed_json() {
    assert!(serde_json::from_str::<JsonRpcRequest>("{").is_err());
    assert!(serde_json::from_str::<JsonRpcRequest>("not json").is_err());
    assert!(serde_json::from_str::<JsonRpcRequest>("").is_err());
}

#[test]
fn deserialize_rejects_missing_jsonrpc_field() {
    let s = r#"{"method":"m","id":1}"#;
    assert!(serde_json::from_str::<JsonRpcRequest>(s).is_err());
}

#[test]
fn deserialize_rejects_missing_method_field() {
    let s = r#"{"jsonrpc":"2.0","id":1}"#;
    assert!(serde_json::from_str::<JsonRpcRequest>(s).is_err());
}

#[test]
fn deserialize_accepts_various_id_types() {
    let cases = [
        r#"{"jsonrpc":"2.0","method":"a","id":"s"}"#,
        r#"{"jsonrpc":"2.0","method":"a","id":true}"#,
        r#"{"jsonrpc":"2.0","method":"a","id":[1,2]}"#,
        r#"{"jsonrpc":"2.0","method":"a","id":{"x":1}}"#,
        r#"{"jsonrpc":"2.0","method":"a","id":42}"#,
    ];
    for s in cases {
        let r: JsonRpcRequest = serde_json::from_str(s).unwrap();
        assert_eq!(r.jsonrpc, "2.0");
        assert_eq!(r.method, "a");
        assert!(r.id.is_some(), "id should be present for: {s}");
    }
}

#[test]
fn deserialize_null_id_treated_as_notification() {
    let s = r#"{"jsonrpc":"2.0","method":"a","id":null}"#;
    let r: JsonRpcRequest = serde_json::from_str(s).unwrap();
    assert!(r.is_notification(), "null id is indistinguishable from absent id via serde");
}

#[test]
fn deserialize_notification_omits_id() {
    let s = r#"{"jsonrpc":"2.0","method":"notify.event"}"#;
    let r: JsonRpcRequest = serde_json::from_str(s).unwrap();
    assert!(r.is_notification());
    assert!(r.id.is_none());
}

#[test]
fn deserialize_nested_json_deep_structure() {
    let mut inner = json!({});
    for _ in 0..120 {
        inner = json!({ "k": inner });
    }
    let v = json!({
        "jsonrpc": "2.0",
        "method": "deep",
        "id": 1,
        "params": inner
    });
    let s = v.to_string();
    let r: JsonRpcRequest = serde_json::from_str(&s).unwrap();
    assert_eq!(r.method, "deep");
    assert!(r.params.is_some());
}

#[test]
fn deserialize_very_long_method_name() {
    let long = "x".repeat(50_000);
    let v = json!({
        "jsonrpc": "2.0",
        "method": long,
        "id": 0
    });
    let s = v.to_string();
    let r: JsonRpcRequest = serde_json::from_str(&s).unwrap();
    assert_eq!(r.method.len(), 50_000);
}

#[test]
fn deserialize_unicode_method_name() {
    let s = r#"{"jsonrpc":"2.0","method":"ping.тест.😀","id":1}"#;
    let r: JsonRpcRequest = serde_json::from_str(s).unwrap();
    assert_eq!(r.method, "ping.тест.😀");
}
