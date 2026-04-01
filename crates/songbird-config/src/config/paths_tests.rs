// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;
use songbird_process_env;

#[test]
fn from_env_reader_uses_injected_env_for_dirs() {
    let pc = PathConfig::from_env_reader(|k| {
        if k == "HOME" {
            Ok("/tmp/sb_paths_home_test".into())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    assert!(pc.config_dir.to_string_lossy().contains("songbird"));
    assert!(pc.service_data_dirs.orchestrator.to_string_lossy().contains("orchestrator"));
}

#[test]
fn get_service_path_unknown_type_errors() {
    let pc = testing_config();
    let err = pc.get_service_path("svc", "invalid_type").expect_err("bad type");
    assert!(matches!(err, SongbirdError::Configuration { .. }), "{err:?}");
}

#[test]
fn get_service_path_creates_data_subdir() {
    let base = tempfile::tempdir().expect("tempdir");
    let pc = PathConfig {
        data_dir: base.path().join("data"),
        config_dir: base.path().join("config"),
        log_dir: base.path().join("logs"),
        cache_dir: base.path().join("cache"),
        runtime_dir: base.path().join("runtime"),
        service_data_dirs: ServiceDataDirs {
            orchestrator: base.path().join("o"),
            federation: base.path().join("f"),
            metrics: base.path().join("m"),
            discovery: base.path().join("d"),
            registry: base.path().join("r"),
        },
    };
    let p = pc.get_service_path("mysvc", "data").expect("service path");
    assert!(p.exists());
    assert!(p.to_string_lossy().contains("mysvc"));
}

#[test]
fn validate_paths_fails_when_root_missing() {
    let pc = PathConfig {
        data_dir: PathBuf::from("/nonexistent/songbird_test_data_xyz"),
        config_dir: PathBuf::from("/nonexistent/songbird_test_cfg_xyz"),
        log_dir: PathBuf::from("/nonexistent/songbird_test_log_xyz"),
        cache_dir: PathBuf::from("/nonexistent/songbird_test_cache_xyz"),
        runtime_dir: PathBuf::from("/nonexistent/songbird_test_rt_xyz"),
        service_data_dirs: ServiceDataDirs {
            orchestrator: PathBuf::from("/tmp"),
            federation: PathBuf::from("/tmp"),
            metrics: PathBuf::from("/tmp"),
            discovery: PathBuf::from("/tmp"),
            registry: PathBuf::from("/tmp"),
        },
    };
    assert!(pc.validate_paths().is_err());
}

#[test]
fn create_directories_makes_all_base_paths() {
    let base = tempfile::tempdir().expect("tempdir");
    let pc = PathConfig {
        data_dir: base.path().join("data"),
        config_dir: base.path().join("config"),
        log_dir: base.path().join("logs"),
        cache_dir: base.path().join("cache"),
        runtime_dir: base.path().join("runtime"),
        service_data_dirs: ServiceDataDirs {
            orchestrator: base.path().join("data").join("orchestrator"),
            federation: base.path().join("data").join("federation"),
            metrics: base.path().join("data").join("metrics"),
            discovery: base.path().join("data").join("discovery"),
            registry: base.path().join("data").join("registry"),
        },
    };
    pc.create_directories().expect("create dirs");
    assert!(pc.data_dir.exists());
    assert!(pc.service_data_dirs.registry.exists());
}

#[test]
fn get_temp_path_creates_under_temp() {
    let op = format!("unit_op_{}", std::process::id());
    let p = PathConfig::get_temp_path(&op).expect("temp path");
    assert!(p.exists());
    assert!(p.to_string_lossy().contains(&op));
}

#[test]
fn get_secure_path_under_data() {
    let base = tempfile::tempdir().expect("tempdir");
    let pc = PathConfig {
        data_dir: base.path().join("data"),
        config_dir: base.path().join("config"),
        log_dir: base.path().join("logs"),
        cache_dir: base.path().join("cache"),
        runtime_dir: base.path().join("runtime"),
        service_data_dirs: ServiceDataDirs {
            orchestrator: base.path().join("o"),
            federation: base.path().join("f"),
            metrics: base.path().join("m"),
            discovery: base.path().join("d"),
            registry: base.path().join("r"),
        },
    };
    let s = pc.get_secure_path("keys").expect("secure");
    assert!(s.exists());
    assert!(s.to_string_lossy().contains("secure"));
}

#[test]
fn initialize_service_paths_static_helper() {
    let base = tempfile::tempdir().expect("tempdir");
    let dirs = PathConfig::initialize_service_paths(base.path()).expect("init");
    assert!(dirs.registry.exists());
}

#[test]
fn initialize_service_paths_top_level_creates_under_tmp() {
    let name = format!("sb_svc_test_{}", std::process::id());
    let dirs = initialize_service_paths(&name).expect("top-level init");
    assert!(dirs.discovery.exists());
}

#[test]
fn get_fallback_data_dir_with_xdg_data_home() {
    let base = tempfile::tempdir().expect("tempdir");
    let prev = std::env::var_os("XDG_DATA_HOME");
    songbird_process_env::set_var("XDG_DATA_HOME", base.path());
    let p = PathConfig::get_fallback_data_dir().expect("xdg data");
    if let Some(v) = prev {
        songbird_process_env::set_var("XDG_DATA_HOME", v);
    } else {
        songbird_process_env::remove_var("XDG_DATA_HOME");
    }
    assert!(p.ends_with("songbird"));
}

#[test]
fn get_fallback_config_dir_with_xdg_config_home() {
    let base = tempfile::tempdir().expect("tempdir");
    let prev = std::env::var_os("XDG_CONFIG_HOME");
    songbird_process_env::set_var("XDG_CONFIG_HOME", base.path());
    let p = PathConfig::get_fallback_config_dir().expect("xdg config");
    if let Some(v) = prev {
        songbird_process_env::set_var("XDG_CONFIG_HOME", v);
    } else {
        songbird_process_env::remove_var("XDG_CONFIG_HOME");
    }
    assert!(p.ends_with("songbird"));
}

#[test]
fn testing_config_has_distinct_top_level_dirs() {
    let c = testing_config();
    assert_ne!(c.data_dir, c.config_dir);
    assert_ne!(c.cache_dir, c.log_dir);
}

#[test]
fn get_service_path_config_and_runtime_under_base() {
    let base = tempfile::tempdir().expect("tempdir");
    let pc = PathConfig {
        data_dir: base.path().join("data"),
        config_dir: base.path().join("config"),
        log_dir: base.path().join("logs"),
        cache_dir: base.path().join("cache"),
        runtime_dir: base.path().join("runtime"),
        service_data_dirs: ServiceDataDirs {
            orchestrator: base.path().join("o"),
            federation: base.path().join("f"),
            metrics: base.path().join("m"),
            discovery: base.path().join("d"),
            registry: base.path().join("r"),
        },
    };
    let cfg_path = pc.get_service_path("alpha", "config").expect("config path");
    let rt_path = pc.get_service_path("alpha", "runtime").expect("runtime path");
    assert!(cfg_path.starts_with(&pc.config_dir));
    assert!(rt_path.starts_with(&pc.runtime_dir));
}

#[test]
fn from_env_reader_joins_service_subdirs_to_config_dir() {
    let pc = PathConfig::from_env_reader(|k| {
        if k == "HOME" {
            Ok("/tmp/sb_paths_join_test".into())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    let expected_base = pc.config_dir.clone();
    assert_eq!(pc.service_data_dirs.federation, expected_base.join("federation"));
    assert_eq!(pc.service_data_dirs.registry, expected_base.join("registry"));
}

#[test]
fn validate_paths_succeeds_when_all_exist() {
    let base = tempfile::tempdir().expect("tempdir");
    let pc = PathConfig {
        data_dir: base.path().join("data"),
        config_dir: base.path().join("config"),
        log_dir: base.path().join("logs"),
        cache_dir: base.path().join("cache"),
        runtime_dir: base.path().join("runtime"),
        service_data_dirs: ServiceDataDirs {
            orchestrator: base.path().join("o"),
            federation: base.path().join("f"),
            metrics: base.path().join("m"),
            discovery: base.path().join("d"),
            registry: base.path().join("r"),
        },
    };
    pc.create_directories().expect("create");
    assert!(pc.validate_paths().is_ok());
}

#[test]
fn get_service_path_log_type_under_log_dir() {
    let base = tempfile::tempdir().expect("tempdir");
    let pc = PathConfig {
        data_dir: base.path().join("data"),
        config_dir: base.path().join("config"),
        log_dir: base.path().join("logs"),
        cache_dir: base.path().join("cache"),
        runtime_dir: base.path().join("runtime"),
        service_data_dirs: ServiceDataDirs {
            orchestrator: base.path().join("o"),
            federation: base.path().join("f"),
            metrics: base.path().join("m"),
            discovery: base.path().join("d"),
            registry: base.path().join("r"),
        },
    };
    let p = pc.get_service_path("worker", "log").expect("log path");
    assert!(p.starts_with(&pc.log_dir));
    assert!(p.to_string_lossy().contains("worker"));
}

#[test]
fn get_service_path_cache_type_under_cache_dir() {
    let base = tempfile::tempdir().expect("tempdir");
    let pc = PathConfig {
        data_dir: base.path().join("data"),
        config_dir: base.path().join("config"),
        log_dir: base.path().join("logs"),
        cache_dir: base.path().join("cache"),
        runtime_dir: base.path().join("runtime"),
        service_data_dirs: ServiceDataDirs {
            orchestrator: base.path().join("o"),
            federation: base.path().join("f"),
            metrics: base.path().join("m"),
            discovery: base.path().join("d"),
            registry: base.path().join("r"),
        },
    };
    let p = pc.get_service_path("indexer", "cache").expect("cache path");
    assert!(p.starts_with(&pc.cache_dir));
}

#[test]
fn development_config_nests_under_dot_songbird() {
    let dev = PathConfig::development();
    let s = dev.config_dir.to_string_lossy();
    assert!(s.contains(".songbird"));
    assert!(s.contains("config"));
    assert_eq!(dev.service_data_dirs.orchestrator, dev.data_dir.join("orchestrator"));
}
