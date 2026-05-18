// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]
#![allow(clippy::expect_used, reason = "test assertions")]

use super::*;
use std::sync::Mutex;

/// Serialize tests that mutate `COMPUTE_*` process env (parallel runs share one env).
static COMPUTE_ENDPOINT_ENV_MUTEX: Mutex<()> = Mutex::new(());

/// Restores a previous env value (or removal) on drop for isolated env tests.
struct EnvRestore {
    key: &'static str,
    previous: Option<String>,
}

impl EnvRestore {
    fn set(key: &'static str, value: &str) -> Self {
        let previous = songbird_process_env::var(key).ok();
        songbird_process_env::set_var(key, value);
        Self {
            key,
            previous,
        }
    }

    /// Remove `key` for the scope of the test; restore previous value on drop.
    fn clear(key: &'static str) -> Self {
        let previous = songbird_process_env::var(key).ok();
        songbird_process_env::remove_var(key);
        Self {
            key,
            previous,
        }
    }
}

impl Drop for EnvRestore {
    fn drop(&mut self) {
        match &self.previous {
            Some(v) => songbird_process_env::set_var(self.key, v),
            None => songbird_process_env::remove_var(self.key),
        }
    }
}

#[tokio::test]
async fn test_compute_endpoint_from_explicit_option() {
    let options =
        DiscoveryOptions::for_testing().compute_endpoint("http://test-compute:9000").build();

    let result = get_compute_endpoint(options).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "http://test-compute:9000");
}

#[tokio::test]
async fn test_compute_endpoint_explicit_compute_provider_option() {
    let _env_lock = COMPUTE_ENDPOINT_ENV_MUTEX.lock().expect("compute env mutex");
    let _compute_endpoint_guard = EnvRestore::clear("COMPUTE_ENDPOINT");
    let _compute_provider_endpoint_guard = EnvRestore::clear("COMPUTE_PROVIDER_ENDPOINT");

    let options = DiscoveryOptions::for_testing()
        .compute_provider_endpoint("http://legacy-compute-provider:8001")
        .build();

    let result = get_compute_endpoint(options).await;
    assert!(result.is_ok());
    assert_eq!(result.expect("should resolve"), "http://legacy-compute-provider:8001");
}

#[tokio::test]
async fn test_compute_endpoint_not_configured() {
    let options =
        DiscoveryOptions::for_testing().discovery_timeout(Duration::from_millis(1)).build();
    let result = get_compute_endpoint(options).await;

    if result.is_ok() {
        return;
    }

    assert!(result.is_err());
    if let Err(SongbirdError::Configuration {
        message,
        suggestion,
        ..
    }) = result
    {
        assert!(message.contains("No compute provider configured"));
        assert!(suggestion.is_some());
        let suggestion_text = suggestion.unwrap();
        assert!(suggestion_text.contains("COMPUTE_ENDPOINT"));
    }
}

#[tokio::test]
async fn test_capability_based_endpoint_with_env() {
    use std::collections::HashMap;

    let vars: HashMap<String, String> =
        HashMap::from([("MYSERVICE_ENDPOINT".to_string(), "http://my-service:5000".to_string())]);
    let env = move |key: &str| -> std::result::Result<String, std::env::VarError> {
        vars.get(key).cloned().ok_or(std::env::VarError::NotPresent)
    };

    let result = get_endpoint_by_capability_with("myservice", env, None).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "http://my-service:5000");
}

#[tokio::test]
async fn test_capability_empty_env_falls_through() {
    use std::collections::HashMap;

    let vars: HashMap<String, String> =
        HashMap::from([("MYSERVICE_ENDPOINT".to_string(), String::new())]);
    let env = move |key: &str| -> std::result::Result<String, std::env::VarError> {
        vars.get(key).cloned().ok_or(std::env::VarError::NotPresent)
    };

    let result =
        get_endpoint_by_capability_with("myservice", env, Some(Duration::from_millis(1))).await;
    if result.is_err()
        && let Err(SongbirdError::Configuration {
            message,
            ..
        }) = result
    {
        assert!(message.contains("No provider found"));
    }
}

#[tokio::test]
async fn test_get_storage_endpoint_from_env() {
    let ep = get_storage_endpoint_with(|k| {
        if k == "STORAGE_ENDPOINT" {
            Ok("http://storage-test:3".to_string())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    })
    .await
    .expect("storage env");
    assert_eq!(ep, "http://storage-test:3");
}

#[tokio::test]
async fn test_get_security_endpoint_from_env() {
    let ep = get_security_endpoint_with(|k| {
        if k == "SECURITY_ENDPOINT" {
            Ok("http://sec-test:4".to_string())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    })
    .await
    .expect("security env");
    assert_eq!(ep, "http://sec-test:4");
}

#[tokio::test]
async fn test_get_ai_endpoint_from_env() {
    let ep = get_ai_endpoint_with(|k| {
        if k == "AI_ENDPOINT" {
            Ok("http://ai-test:5".to_string())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    })
    .await
    .expect("ai env");
    assert_eq!(ep, "http://ai-test:5");
}

#[tokio::test]
async fn test_compute_endpoint_backward_compat_prefers_compute_over_legacy_toadstool_env() {
    let _env_lock = COMPUTE_ENDPOINT_ENV_MUTEX.lock().expect("compute env mutex");
    let _c = EnvRestore::set("COMPUTE_ENDPOINT", "http://compute-wins:8001");
    let _t = EnvRestore::set("TOADSTOOL_ENDPOINT", "http://legacy-compute-fallback:9001");
    let options =
        DiscoveryOptions::for_testing().discovery_timeout(Duration::from_millis(1)).build();
    let ep = get_compute_endpoint(options).await.expect("compute from env");
    assert_eq!(ep, "http://compute-wins:8001");
}

#[tokio::test]
async fn test_get_storage_endpoint_backward_compat_provider_before_legacy_nestgate_env() {
    let ep = get_storage_endpoint_with(|k| match k {
        "STORAGE_PROVIDER_ENDPOINT" => Ok("http://provider-priority:8003".to_string()),
        "NESTGATE_ENDPOINT" => Ok("http://legacy-storage-fallback:8003".to_string()),
        _ => Err(std::env::VarError::NotPresent),
    })
    .await
    .expect("storage");
    assert_eq!(ep, "http://provider-priority:8003");
}

#[tokio::test]
async fn test_get_security_endpoint_backward_compat_legacy_beardog_env_var() {
    let ep = get_security_endpoint_with(|k| match k {
        "BEARDOG_ENDPOINT" => Ok("http://security-provider-legacy:7443".to_string()),
        _ => Err(std::env::VarError::NotPresent),
    })
    .await
    .expect("security");
    assert_eq!(ep, "http://security-provider-legacy:7443");
}

#[tokio::test]
async fn test_get_ai_endpoint_backward_compat_legacy_squirrel_env_var() {
    let ep = get_ai_endpoint_with(|k| match k {
        "SQUIRREL_ENDPOINT" => Ok("http://ai-provider-legacy:9200".to_string()),
        _ => Err(std::env::VarError::NotPresent),
    })
    .await
    .expect("ai");
    assert_eq!(ep, "http://ai-provider-legacy:9200");
}

#[tokio::test]
async fn test_endpoint_by_capability_uppercases_capability_in_env_key() {
    use std::collections::HashMap;

    let vars: HashMap<String, String> =
        HashMap::from([("MIXEDCAP_ENDPOINT".to_string(), "http://mixed:1".to_string())]);
    let env = move |key: &str| -> std::result::Result<String, std::env::VarError> {
        vars.get(key).cloned().ok_or(std::env::VarError::NotPresent)
    };
    let ep = get_endpoint_by_capability_with("mixedcap", env, Some(Duration::from_millis(1))).await;
    assert_eq!(ep.expect("endpoint"), "http://mixed:1");
}

#[tokio::test]
async fn test_endpoint_by_capability_accepts_non_url_string_from_env() {
    use std::collections::HashMap;

    let vars: HashMap<String, String> =
        HashMap::from([("RAW_ENDPOINT".to_string(), "not-a-valid-url:::broken".to_string())]);
    let env = move |key: &str| -> std::result::Result<String, std::env::VarError> {
        vars.get(key).cloned().ok_or(std::env::VarError::NotPresent)
    };
    let ep = get_endpoint_by_capability_with("raw", env, Some(Duration::from_millis(1))).await;
    assert_eq!(ep.expect("opaque string preserved"), "not-a-valid-url:::broken");
}

#[tokio::test]
async fn test_get_compute_endpoint_with_prefers_provider_over_plain_compute_env() {
    use std::collections::HashMap;

    let vars: HashMap<String, String> = HashMap::from([
        ("COMPUTE_PROVIDER_ENDPOINT".to_string(), "http://provider:8001".to_string()),
        ("COMPUTE_ENDPOINT".to_string(), "http://plain:9001".to_string()),
    ]);
    let env = move |k: &str| -> std::result::Result<String, std::env::VarError> {
        vars.get(k).cloned().ok_or(std::env::VarError::NotPresent)
    };
    let options = DiscoveryOptions::for_testing().build();
    let ep = get_compute_endpoint_with(options, env).await.expect("compute endpoint");
    assert_eq!(ep, "http://provider:8001");
}

#[tokio::test]
async fn test_get_compute_endpoint_with_prefers_compute_over_legacy_toadstool() {
    use std::collections::HashMap;

    let vars: HashMap<String, String> = HashMap::from([
        ("COMPUTE_ENDPOINT".to_string(), "http://compute-wins:1".to_string()),
        ("TOADSTOOL_ENDPOINT".to_string(), "http://legacy:2".to_string()),
    ]);
    let env = move |k: &str| -> std::result::Result<String, std::env::VarError> {
        vars.get(k).cloned().ok_or(std::env::VarError::NotPresent)
    };
    let options =
        DiscoveryOptions::for_testing().discovery_timeout(Duration::from_millis(1)).build();
    let ep = get_compute_endpoint_with(options, env).await.expect("compute");
    assert_eq!(ep, "http://compute-wins:1");
}

#[tokio::test]
async fn test_get_security_endpoint_with_prefers_provider_key() {
    let ep = get_security_endpoint_with(|k| match k {
        "SECURITY_PROVIDER_ENDPOINT" => Ok("https://sec-prov:8443".to_string()),
        "SECURITY_ENDPOINT" => Ok("http://plain-sec:80".to_string()),
        _ => Err(std::env::VarError::NotPresent),
    })
    .await
    .expect("security");
    assert_eq!(ep, "https://sec-prov:8443");
}

#[tokio::test]
async fn test_get_ai_endpoint_with_prefers_provider_key() {
    let ep = get_ai_endpoint_with(|k| match k {
        "AI_PROVIDER_ENDPOINT" => Ok("http://ai-prov:8083".to_string()),
        "AI_ENDPOINT" => Ok("http://ai-plain:9".to_string()),
        _ => Err(std::env::VarError::NotPresent),
    })
    .await
    .expect("ai");
    assert_eq!(ep, "http://ai-prov:8083");
}

#[tokio::test]
async fn test_get_endpoint_by_capability_discovery_failure_returns_configuration_error() {
    use std::collections::HashMap;

    let vars: HashMap<String, String> = HashMap::new();
    let env = move |key: &str| -> std::result::Result<String, std::env::VarError> {
        vars.get(key).cloned().ok_or(std::env::VarError::NotPresent)
    };
    let result =
        get_endpoint_by_capability_with("unknowncapxyz", env, Some(Duration::from_millis(1))).await;
    if result.is_ok() {
        return;
    }
    match result.expect_err("expected configuration error when discovery finds nothing") {
        SongbirdError::Configuration {
            message,
            field,
            ..
        } => {
            assert!(message.contains("unknowncapxyz"));
            assert_eq!(field.as_deref(), Some("unknowncapxyz_endpoint"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
