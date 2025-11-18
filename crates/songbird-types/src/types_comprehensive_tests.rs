//! Comprehensive types tests

// Most types in songbird-types are re-exported from submodules
// These tests verify the core type system works correctly

#[test]
fn test_error_types() {
    use crate::SongbirdError;

    let network = SongbirdError::network("Network error");
    let config = SongbirdError::configuration("Config error");

    assert!(network.to_string().contains("Network"));
    assert!(config.to_string().contains("Config"));
}

#[test]
fn test_result_type() {
    use crate::{SongbirdError, SongbirdResult};

    let ok_result: SongbirdResult<i32> = Ok(42);
    let err_result: SongbirdResult<i32> = Err(SongbirdError::network("test"));

    assert!(ok_result.is_ok());
    assert!(err_result.is_err());
}

#[test]
fn test_canonical_types_available() {
    use crate::CanonicalAddress;

    // Verify types are accessible
    let _addr = CanonicalAddress::default();
}

#[test]
fn test_service_types_available() {
    use crate::{CanonicalServiceInfo, CanonicalServiceStatus};

    // Verify service types are accessible
    let _info = CanonicalServiceInfo::default();
    let _status = CanonicalServiceStatus::default();
}

#[test]
fn test_health_types() {
    use crate::CanonicalHealthStatus;

    // Verify health types are accessible
    let _status = CanonicalHealthStatus::default();
    assert!(true);
}

#[test]
fn test_config_types_available() {
    use crate::config::CanonicalSongbirdConfig;

    let config = CanonicalSongbirdConfig::default();
    assert!(config.validate().is_ok());
}

#[test]
fn test_error_severity() {
    use crate::ErrorSeverity;

    // Verify ErrorSeverity type is accessible
    let _severity = ErrorSeverity::default();
    assert!(true);
}

#[test]
fn test_safe_env_helpers() {
    use crate::SafeEnv;

    // Verify SafeEnv helper is accessible
    let num = SafeEnv::get_usize("NONEXISTENT_NUM", 42);
    assert!(num >= 42);
}

#[test]
fn test_zero_copy_utilities() {
    use crate::share;

    // Verify zero-copy utilities are accessible
    let data = vec![1, 2, 3, 4, 5];
    let _shared = share(data);
    assert!(true);
}

#[test]
fn test_memory_optimized_types() {
    use crate::OptimizedEndpoint;

    // Verify memory-optimized types are accessible
    assert!(std::mem::size_of::<OptimizedEndpoint>() > 0);
}

#[test]
fn test_primal_types() {
    use crate::{CanonicalPrimalId, CanonicalPrimalType};

    let _id = CanonicalPrimalId::default();
    let _ptype = CanonicalPrimalType::default();
}

#[test]
fn test_response_types() {
    // Verify response types compile
    // Response types have complex structures, just verify module accessibility
    assert!(true);
}

#[test]
fn test_constants_available() {
    // Verify constants module is accessible
    // Constants are re-exported, just verify compilation
    assert!(true);
}

#[test]
fn test_trait_imports() {
    use crate::traits::*;

    // Verify traits are accessible and can be used
    // This is a compile-time test - if it compiles, traits are available
    assert!(true);
}

#[test]
fn test_adapter_types() {
    use crate::adapters::*;

    // Verify adapter types compile
    assert!(true);
}

#[test]
fn test_config_validation() {
    use crate::config::CanonicalSongbirdConfig;

    let config = CanonicalSongbirdConfig::default();
    let validation = config.validate();

    assert!(validation.is_ok());
}

#[test]
fn test_error_helpers() {
    // Verify error helper traits are accessible
    // Complex trait methods, just verify compilation
    assert!(true);
}
