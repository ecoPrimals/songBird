// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! JSON-RPC parsing for STUN port-pattern payloads (`our_pattern`).

use serde_json::Value;

/// Parse a `PortPattern` from JSON-RPC params
pub fn parse_port_pattern(value: &Value) -> songbird_stun::PortPattern {
    let pattern_type = value.get("pattern").and_then(|v| v.as_str()).unwrap_or("unknown");

    match pattern_type {
        "sequential" => {
            let step =
                i32::try_from(value.get("step").and_then(serde_json::Value::as_i64).unwrap_or(1))
                    .unwrap_or(1);
            let last_port = value
                .get("last_port")
                .and_then(serde_json::Value::as_u64)
                .map_or(0, |p| u16::try_from(p).unwrap_or(0));
            let predicted_next = value
                .get("predicted_next")
                .and_then(serde_json::Value::as_u64)
                .map_or(0, |p| u16::try_from(p).unwrap_or(0));
            let confidence =
                value.get("confidence").and_then(serde_json::Value::as_f64).unwrap_or(0.5);

            songbird_stun::PortPattern::Sequential {
                step,
                last_port,
                predicted_next,
                confidence,
            }
        }
        "random" => {
            let observed = value
                .get("observed_ports")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_u64().map(|p| u16::try_from(p).unwrap_or(0)))
                        .collect()
                })
                .unwrap_or_default();

            songbird_stun::PortPattern::Random {
                observed,
            }
        }
        _ => songbird_stun::PortPattern::Unknown,
    }
}
