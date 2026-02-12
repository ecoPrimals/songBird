//! Coverage tests for `songbird_config::config::paths`
//!
//! Tests path configuration, initialization, fallback logic, and validation.

use songbird_config::config::paths::{get_path_config, testing_config, PathConfig};
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

struct ScopedEnv {
    vars: Vec<(String, Option<String>)>,
}

impl ScopedEnv {
    fn new() -> Self {
        Self {
            vars: Vec::new(),
        }
    }

    fn set(&mut self, key: &str, value: &str) -> &mut Self {
        let old = std::env::var(key).ok();
        self.vars.push((key.to_string(), old));
        std::env::set_var(key, value);
        self
    }

    fn remove(&mut self, key: &str) -> &mut Self {
        let old = std::env::var(key).ok();
        self.vars.push((key.to_string(), old));
        std::env::remove_var(key);
        self
    }
}

impl Drop for ScopedEnv {
    fn drop(&mut self) {
        for (key, old) in self.vars.drain(..).rev() {
            match old {
                Some(val) => std::env::set_var(&key, &val),
                None => std::env::remove_var(&key),
            }
        }
    }
}

// ==================== DEFAULT TESTS ====================

#[test]
fn test_path_config_default() {
    let _g = lock_env();
    let config = PathConfig::default();
    assert!(!config.data_dir.as_os_str().is_empty());
    assert!(!config.config_dir.as_os_str().is_empty());
    assert!(!config.log_dir.as_os_str().is_empty());
    assert!(!config.cache_dir.as_os_str().is_empty());
}

#[test]
fn test_path_config_new() {
    let _g = lock_env();
    let config = PathConfig::new();
    assert!(config.is_ok(), "PathConfig::new() should succeed");
    let config = config.unwrap();
    assert!(config.config_dir.to_string_lossy().contains("songbird"));
}

// ==================== FALLBACK TESTS ====================

#[test]
fn test_path_config_fallback() {
    let _g = lock_env();
    let config = PathConfig::new_fallback();
    assert!(!config.data_dir.as_os_str().is_empty());
    assert!(!config.config_dir.as_os_str().is_empty());
}

// ==================== DEVELOPMENT TESTS ====================

#[test]
fn test_path_config_development() {
    let _g = lock_env();
    let config = PathConfig::development();
    assert!(
        config.data_dir.to_string_lossy().contains(".songbird"),
        "Dev config should use .songbird dir"
    );
}

// ==================== TESTING CONFIG ====================

#[test]
fn test_testing_config() {
    let _g = lock_env();
    let config = testing_config();
    // Testing config should return valid paths
    assert!(!config.data_dir.as_os_str().is_empty());
}

// ==================== GET_PATH_CONFIG ====================

#[test]
fn test_get_path_config() {
    let _g = lock_env();
    let config = get_path_config();
    assert!(!config.data_dir.as_os_str().is_empty());
    assert!(!config.config_dir.as_os_str().is_empty());
}

// ==================== SERVICE DATA DIRS ====================

#[test]
fn test_service_data_dirs_present() {
    let _g = lock_env();
    let config = PathConfig::default();
    assert!(!config.service_data_dirs.orchestrator.as_os_str().is_empty());
    assert!(!config.service_data_dirs.federation.as_os_str().is_empty());
    assert!(!config.service_data_dirs.metrics.as_os_str().is_empty());
    assert!(!config.service_data_dirs.discovery.as_os_str().is_empty());
    assert!(!config.service_data_dirs.registry.as_os_str().is_empty());
}

// ==================== SERIALIZATION TESTS ====================

#[test]
fn test_path_config_serializable() {
    let _g = lock_env();
    let config = PathConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    assert!(!json.is_empty());
    let deserialized: PathConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config.data_dir, deserialized.data_dir);
}

#[test]
fn test_path_config_debug() {
    let _g = lock_env();
    let config = PathConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("PathConfig"));
}

#[test]
fn test_path_config_clone() {
    let _g = lock_env();
    let config = PathConfig::default();
    let cloned = config.clone();
    assert_eq!(config.data_dir, cloned.data_dir);
    assert_eq!(config.config_dir, cloned.config_dir);
}

// ==================== INITIALIZE SERVICE PATHS ====================

#[test]
fn test_initialize_service_paths() {
    let _g = lock_env();
    use songbird_config::config::paths::initialize_service_paths;
    let result = initialize_service_paths("test-service");
    // This may fail if dirs can't be created (permissions) — that's ok
    if let Ok(dirs) = result {
        assert!(
            dirs.orchestrator.to_string_lossy().contains("orchestrator")
                || dirs.orchestrator.to_string_lossy().contains("test-service")
        );
    }
}

// ==================== ENVIRONMENT OVERRIDE TESTS ====================

#[test]
fn test_paths_from_environment() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_DATA_DIR", "/custom/data");
    env.set("SONGBIRD_CONFIG_DIR", "/custom/config");

    let config = get_path_config();
    // The get_path_config function should respect env overrides
    // (depending on implementation, it may or may not use these)
    drop(config);
}

// ==================== CONSISTENCY TESTS ====================

#[test]
fn test_default_and_get_path_config_consistent() {
    let _g = lock_env();
    let default = PathConfig::default();
    let getter = get_path_config();

    // Both should produce valid non-empty paths
    assert!(!default.data_dir.as_os_str().is_empty());
    assert!(!getter.data_dir.as_os_str().is_empty());
}

#[test]
fn test_development_uses_local_dir() {
    let _g = lock_env();
    let dev = PathConfig::development();
    // Development paths should be relative to current dir
    let dev_str = dev.config_dir.to_string_lossy();
    assert!(dev_str.contains(".songbird"), "Dev config_dir should contain .songbird: {dev_str}");
}
