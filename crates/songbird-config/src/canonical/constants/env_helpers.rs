// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Shared environment parsing helpers for canonical constants.

/// Process environment lookup (function pointer satisfies HRTB for injectable env readers).
pub fn read_process_env(key: &str) -> Result<String, std::env::VarError> {
    songbird_process_env::var(key)
}

pub fn env_parse_with<T: std::str::FromStr>(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    key: &str,
    default: T,
) -> T {
    env(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

pub fn env_get_bool_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    key: &str,
    default: bool,
) -> bool {
    env(key)
        .ok()
        .and_then(|v| match v.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            "false" | "0" | "no" | "off" => Some(false),
            _ => v.parse().ok(),
        })
        .unwrap_or(default)
}

pub fn env_get_or_default_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    key: &str,
    default: impl Into<String>,
) -> String {
    env(key).unwrap_or_else(|_| default.into())
}

pub fn env_port_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    key: &str,
    default: u16,
) -> u16 {
    env(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

pub fn env_or_default_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    key: &str,
    default: impl Into<String>,
) -> String {
    env_get_or_default_with(env, key, default)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn mock_env<'a>(
        map: &'a HashMap<&str, &str>,
    ) -> impl Fn(&str) -> Result<String, std::env::VarError> + 'a {
        move |key| map.get(key).map(ToString::to_string).ok_or(std::env::VarError::NotPresent)
    }

    #[test]
    fn env_parse_with_parses_valid_u16() {
        let map = HashMap::from([("PORT", "8080")]);
        let env = mock_env(&map);
        assert_eq!(env_parse_with(&env, "PORT", 3000u16), 8080);
    }

    #[test]
    fn env_parse_with_parses_valid_f64() {
        let map = HashMap::from([("RATIO", "3.14")]);
        let env = mock_env(&map);
        #[allow(clippy::approx_constant)]
        let expected = 3.14;
        assert!((env_parse_with(&env, "RATIO", 0.0f64) - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn env_parse_with_returns_default_for_non_numeric() {
        let map = HashMap::from([("PORT", "not_a_number")]);
        let env = mock_env(&map);
        assert_eq!(env_parse_with(&env, "PORT", 3000u16), 3000);
    }

    #[test]
    fn env_parse_with_returns_default_when_key_missing() {
        let map = HashMap::new();
        let env = mock_env(&map);
        assert_eq!(env_parse_with(&env, "MISSING", 42u32), 42);
    }

    #[test]
    fn env_get_bool_with_true_string() {
        let map = HashMap::from([("FLAG", "true")]);
        let env = mock_env(&map);
        assert!(env_get_bool_with(&env, "FLAG", false));
    }

    #[test]
    fn env_get_bool_with_one_string() {
        let map = HashMap::from([("FLAG", "1")]);
        let env = mock_env(&map);
        assert!(env_get_bool_with(&env, "FLAG", false));
    }

    #[test]
    fn env_get_bool_with_yes_string() {
        let map = HashMap::from([("FLAG", "yes")]);
        let env = mock_env(&map);
        assert!(env_get_bool_with(&env, "FLAG", false));
    }

    #[test]
    fn env_get_bool_with_on_string() {
        let map = HashMap::from([("FLAG", "on")]);
        let env = mock_env(&map);
        assert!(env_get_bool_with(&env, "FLAG", false));
    }

    #[test]
    fn env_get_bool_with_false_string() {
        let map = HashMap::from([("FLAG", "false")]);
        let env = mock_env(&map);
        assert!(!env_get_bool_with(&env, "FLAG", true));
    }

    #[test]
    fn env_get_bool_with_zero_string() {
        let map = HashMap::from([("FLAG", "0")]);
        let env = mock_env(&map);
        assert!(!env_get_bool_with(&env, "FLAG", true));
    }

    #[test]
    fn env_get_bool_with_no_string() {
        let map = HashMap::from([("FLAG", "no")]);
        let env = mock_env(&map);
        assert!(!env_get_bool_with(&env, "FLAG", true));
    }

    #[test]
    fn env_get_bool_with_off_string() {
        let map = HashMap::from([("FLAG", "off")]);
        let env = mock_env(&map);
        assert!(!env_get_bool_with(&env, "FLAG", true));
    }

    #[test]
    fn env_get_bool_with_uppercase_true() {
        let map = HashMap::from([("FLAG", "TRUE")]);
        let env = mock_env(&map);
        assert!(env_get_bool_with(&env, "FLAG", false));
    }

    #[test]
    fn env_get_bool_with_mixed_case_false() {
        let map = HashMap::from([("FLAG", "False")]);
        let env = mock_env(&map);
        assert!(!env_get_bool_with(&env, "FLAG", true));
    }

    #[test]
    fn env_get_bool_with_invalid_value_returns_default() {
        let map = HashMap::from([("FLAG", "maybe")]);
        let env = mock_env(&map);
        assert!(env_get_bool_with(&env, "FLAG", true));
        assert!(!env_get_bool_with(&env, "FLAG", false));
    }

    #[test]
    fn env_get_bool_with_missing_key_returns_default_true() {
        let map = HashMap::new();
        let env = mock_env(&map);
        assert!(env_get_bool_with(&env, "MISSING", true));
    }

    #[test]
    fn env_get_bool_with_missing_key_returns_default_false() {
        let map = HashMap::new();
        let env = mock_env(&map);
        assert!(!env_get_bool_with(&env, "MISSING", false));
    }

    #[test]
    fn env_port_with_parses_valid_port() {
        let map = HashMap::from([("PORT", "8080")]);
        let env = mock_env(&map);
        assert_eq!(env_port_with(&env, "PORT", 3000), 8080);
    }

    #[test]
    fn env_port_with_returns_default_for_non_numeric() {
        let map = HashMap::from([("PORT", "not_a_port")]);
        let env = mock_env(&map);
        assert_eq!(env_port_with(&env, "PORT", 3000), 3000);
    }

    #[test]
    fn env_port_with_returns_default_for_overflow() {
        let map = HashMap::from([("PORT", "99999")]);
        let env = mock_env(&map);
        assert_eq!(env_port_with(&env, "PORT", 3000), 3000);
    }

    #[test]
    fn env_port_with_returns_default_when_key_missing() {
        let map = HashMap::new();
        let env = mock_env(&map);
        assert_eq!(env_port_with(&env, "MISSING", 9000), 9000);
    }

    #[test]
    fn env_get_or_default_with_returns_value_when_present() {
        let map = HashMap::from([("HOST", "example.com")]);
        let env = mock_env(&map);
        assert_eq!(env_get_or_default_with(&env, "HOST", "localhost"), "example.com");
    }

    #[test]
    fn env_get_or_default_with_returns_default_when_absent() {
        let map = HashMap::new();
        let env = mock_env(&map);
        assert_eq!(env_get_or_default_with(&env, "HOST", "localhost"), "localhost");
    }
}
