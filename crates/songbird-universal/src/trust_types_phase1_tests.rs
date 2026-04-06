// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Phase 1 Trust Parsing Tests
//!
//! Tests for flexible `trust_level` parsing (integer + string)

#[cfg(test)]
mod tests {
    use super::super::*;
    use serde_json::json;
    use songbird_types::TrustLevel;

    #[test]
    fn test_trust_response_deserialize_integer() {
        // security provider Phase 1: Integer format
        let json = json!({
            "decision": "auto_accept",
            "trust_level": 1,  // Integer!
            "reason": "same_genetic_family"
        });

        let response: TrustEvaluationResponse = serde_json::from_value(json).unwrap();
        assert_eq!(response.trust_level, TrustLevel::Limited);
        assert_eq!(response.decision, "auto_accept");
    }

    #[test]
    fn test_trust_response_deserialize_string() {
        // Backward compatible: String format
        let json = json!({
            "decision": "auto_accept",
            "trust_level": "limited",  // String!
            "reason": "same_genetic_family"
        });

        let response: TrustEvaluationResponse = serde_json::from_value(json).unwrap();
        assert_eq!(response.trust_level, TrustLevel::Limited);
        assert_eq!(response.decision, "auto_accept");
    }

    #[test]
    fn test_trust_response_deserialize_beardog_alias() {
        // security provider aliases
        let json = json!({
            "decision": "auto_accept",
            "trust_level": "basic",  // security provider alias for Limited
            "reason": "same_genetic_family"
        });

        let response: TrustEvaluationResponse = serde_json::from_value(json).unwrap();
        assert_eq!(response.trust_level, TrustLevel::Limited);
    }

    #[test]
    fn test_trust_response_all_levels_integer() {
        // Test all trust levels as integers
        let levels = vec![
            (0, TrustLevel::None),
            (1, TrustLevel::Limited),
            (2, TrustLevel::Elevated),
            (3, TrustLevel::Highest),
        ];

        for (int_val, expected_level) in levels {
            let json = json!({
                "decision": "auto_accept",
                "trust_level": int_val,
                "reason": "test"
            });

            let response: TrustEvaluationResponse = serde_json::from_value(json).unwrap();
            assert_eq!(response.trust_level, expected_level);
        }
    }

    #[test]
    fn test_trust_response_all_levels_string() {
        // Test all trust levels as strings
        let levels = vec![
            ("none", TrustLevel::None),
            ("limited", TrustLevel::Limited),
            ("elevated", TrustLevel::Elevated),
            ("highest", TrustLevel::Highest),
        ];

        for (str_val, expected_level) in levels {
            let json = json!({
                "decision": "auto_accept",
                "trust_level": str_val,
                "reason": "test"
            });

            let response: TrustEvaluationResponse = serde_json::from_value(json).unwrap();
            assert_eq!(response.trust_level, expected_level);
        }
    }

    #[test]
    fn test_trust_response_serialize_always_integer() {
        // Serialization should always produce integers
        let response = TrustEvaluationResponse {
            decision: "auto_accept".to_string(),
            trust_level: TrustLevel::Limited,
            reason: "test".to_string(),
            suggested_action: None,
            metadata: None,
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["trust_level"], json!(1)); // Should be integer!
    }

    #[test]
    fn test_beardog_phase1_full_response() {
        // Full security provider Phase 1 response format
        let json = json!({
            "decision": "auto_accept",
            "trust_level": 1,
            "reason": "same_genetic_family",
            "suggested_action": "proceed",
            "metadata": {
                "policy_version": 1,
                "evaluation_method": "genetic_family_match",
                "timestamp": "2026-01-07T00:00:00Z"
            }
        });

        let response: TrustEvaluationResponse = serde_json::from_value(json).unwrap();
        assert_eq!(response.trust_level, TrustLevel::Limited);
        assert_eq!(response.decision, "auto_accept");
        assert_eq!(response.reason, "same_genetic_family");
        assert!(response.is_auto_accept());
        assert!(response.metadata.is_some());
    }

    #[test]
    fn test_trust_response_invalid_integer() {
        // Out of range integer should fail
        let json = json!({
            "decision": "reject",
            "trust_level": 99,
            "reason": "test"
        });

        let result: Result<TrustEvaluationResponse, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_trust_response_invalid_string() {
        // Unknown string should fail
        let json = json!({
            "decision": "reject",
            "trust_level": "super_mega_trust",
            "reason": "test"
        });

        let result: Result<TrustEvaluationResponse, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }
}
