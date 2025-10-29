//! Tests for universal adapter
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]

use songbird_universal::*;

#[test]
fn test_create_universal_adapter() {
    let adapter = create_universal_adapter();
    // Just test that we can create it
    assert!(format!("{adapter:?}").contains("UnifiedUniversalAdapter"));
}

#[test]
fn test_adapter_with_default_config() {
    let config = UnifiedAdapterConfig::default();
    let adapter = create_universal_adapter_with_config(config);
    assert!(format!("{adapter:?}").contains("UnifiedUniversalAdapter"));
}

#[test]
fn test_adapter_config_default() {
    let config = UnifiedAdapterConfig::default();
    // Test that default config can be created
    assert!(format!("{config:?}").contains("UnifiedAdapterConfig"));
}

#[test]
fn test_adapter_config_clone() {
    let config = UnifiedAdapterConfig::default();
    let cloned = config.clone();
    // Verify cloning works
    assert!(format!("{cloned:?}").contains("UnifiedAdapterConfig"));
}

#[tokio::test]
async fn test_adapter_creation_async() {
    let adapter = create_universal_adapter();
    // Test async creation
    assert!(format!("{adapter:?}").contains("UnifiedUniversalAdapter"));
}
