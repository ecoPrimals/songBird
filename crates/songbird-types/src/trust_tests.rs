// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for trust level flexible parsing (Phase 1)

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test assertions and harness ergonomics")]

    use crate::trust::TrustLevel;
    use serde_json::json;

    #[test]
    fn test_trust_level_deserialize_integer() {
        // security provider sends integers
        let json_0 = json!({"trust_level": 0});
        let parsed: serde_json::Value = serde_json::from_value(json_0).unwrap();
        let trust_level: TrustLevel =
            serde_json::from_value(parsed["trust_level"].clone()).unwrap();
        assert_eq!(trust_level, TrustLevel::None);

        let json_1 = json!({"trust_level": 1});
        let parsed: serde_json::Value = serde_json::from_value(json_1).unwrap();
        let trust_level: TrustLevel =
            serde_json::from_value(parsed["trust_level"].clone()).unwrap();
        assert_eq!(trust_level, TrustLevel::Limited);

        let json_2 = json!({"trust_level": 2});
        let parsed: serde_json::Value = serde_json::from_value(json_2).unwrap();
        let trust_level: TrustLevel =
            serde_json::from_value(parsed["trust_level"].clone()).unwrap();
        assert_eq!(trust_level, TrustLevel::Elevated);

        let json_3 = json!({"trust_level": 3});
        let parsed: serde_json::Value = serde_json::from_value(json_3).unwrap();
        let trust_level: TrustLevel =
            serde_json::from_value(parsed["trust_level"].clone()).unwrap();
        assert_eq!(trust_level, TrustLevel::Highest);
    }

    #[test]
    fn test_trust_level_deserialize_string_primary() {
        // Songbird primary names
        let json = json!({"trust_level": "none"});
        let parsed: serde_json::Value = serde_json::from_value(json).unwrap();
        let trust_level: TrustLevel =
            serde_json::from_value(parsed["trust_level"].clone()).unwrap();
        assert_eq!(trust_level, TrustLevel::None);

        let json = json!({"trust_level": "limited"});
        let parsed: serde_json::Value = serde_json::from_value(json).unwrap();
        let trust_level: TrustLevel =
            serde_json::from_value(parsed["trust_level"].clone()).unwrap();
        assert_eq!(trust_level, TrustLevel::Limited);

        let json = json!({"trust_level": "elevated"});
        let parsed: serde_json::Value = serde_json::from_value(json).unwrap();
        let trust_level: TrustLevel =
            serde_json::from_value(parsed["trust_level"].clone()).unwrap();
        assert_eq!(trust_level, TrustLevel::Elevated);

        let json = json!({"trust_level": "highest"});
        let parsed: serde_json::Value = serde_json::from_value(json).unwrap();
        let trust_level: TrustLevel =
            serde_json::from_value(parsed["trust_level"].clone()).unwrap();
        assert_eq!(trust_level, TrustLevel::Highest);
    }

    #[test]
    fn test_trust_level_deserialize_string_aliases() {
        // security provider aliases
        let json = json!({"trust_level": "anonymous"});
        let parsed: serde_json::Value = serde_json::from_value(json).unwrap();
        let trust_level: TrustLevel =
            serde_json::from_value(parsed["trust_level"].clone()).unwrap();
        assert_eq!(trust_level, TrustLevel::None);

        let json = json!({"trust_level": "basic"});
        let parsed: serde_json::Value = serde_json::from_value(json).unwrap();
        let trust_level: TrustLevel =
            serde_json::from_value(parsed["trust_level"].clone()).unwrap();
        assert_eq!(trust_level, TrustLevel::Limited);

        let json = json!({"trust_level": "medium"});
        let parsed: serde_json::Value = serde_json::from_value(json).unwrap();
        let trust_level: TrustLevel =
            serde_json::from_value(parsed["trust_level"].clone()).unwrap();
        assert_eq!(trust_level, TrustLevel::Elevated);

        let json = json!({"trust_level": "explicit"});
        let parsed: serde_json::Value = serde_json::from_value(json).unwrap();
        let trust_level: TrustLevel =
            serde_json::from_value(parsed["trust_level"].clone()).unwrap();
        assert_eq!(trust_level, TrustLevel::Highest);
    }

    #[test]
    fn test_trust_level_deserialize_case_insensitive() {
        // Mixed case
        let json = json!({"trust_level": "LIMITED"});
        let parsed: serde_json::Value = serde_json::from_value(json).unwrap();
        let trust_level: TrustLevel =
            serde_json::from_value(parsed["trust_level"].clone()).unwrap();
        assert_eq!(trust_level, TrustLevel::Limited);

        let json = json!({"trust_level": "Elevated"});
        let parsed: serde_json::Value = serde_json::from_value(json).unwrap();
        let trust_level: TrustLevel =
            serde_json::from_value(parsed["trust_level"].clone()).unwrap();
        assert_eq!(trust_level, TrustLevel::Elevated);
    }

    #[test]
    fn test_trust_level_deserialize_invalid_integer() {
        // Out of range
        let json = json!({"trust_level": 99});
        let parsed: serde_json::Value = serde_json::from_value(json).unwrap();
        let result: Result<TrustLevel, _> = serde_json::from_value(parsed["trust_level"].clone());
        assert!(result.is_err());
    }

    #[test]
    fn test_trust_level_deserialize_invalid_string() {
        // Unknown string
        let json = json!({"trust_level": "super_mega_trust"});
        let parsed: serde_json::Value = serde_json::from_value(json).unwrap();
        let result: Result<TrustLevel, _> = serde_json::from_value(parsed["trust_level"].clone());
        assert!(result.is_err());
    }

    #[test]
    fn test_trust_level_serialize() {
        // Serialization should always produce integers (compact)
        let trust = TrustLevel::Limited;
        let serialized = serde_json::to_value(trust).unwrap();
        assert_eq!(serialized, json!(1));

        let trust = TrustLevel::Highest;
        let serialized = serde_json::to_value(trust).unwrap();
        assert_eq!(serialized, json!(3));
    }

    #[test]
    fn test_beardog_phase1_response() {
        // Simulate security provider Phase 1 response
        let beardog_response = json!({
            "trust_level": 1,
            "trust_level_name": "limited",
            "capabilities": {
                "allowed": ["birdsong/*", "coordination/*"],
                "denied": ["data/*", "commands/*"]
            }
        });

        // Parse trust_level (should accept integer)
        let trust_level: TrustLevel =
            serde_json::from_value(beardog_response["trust_level"].clone()).unwrap();
        assert_eq!(trust_level, TrustLevel::Limited);

        // Parse trust_level_name (should accept string)
        let trust_level_name: TrustLevel =
            serde_json::from_value(beardog_response["trust_level_name"].clone()).unwrap();
        assert_eq!(trust_level_name, TrustLevel::Limited);

        // Both should match!
        assert_eq!(trust_level, trust_level_name);
    }
}
