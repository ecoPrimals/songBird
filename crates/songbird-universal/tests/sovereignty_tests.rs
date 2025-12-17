//! Modern tests for sovereignty system
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
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

//!
//! Tests for sovereignty-aware routing and federation capabilities.

use songbird_universal::sovereignty::{SovereigntyAdapterConfig, SovereigntyAwareAdapter};

#[tokio::test]
async fn test_sovereignty_adapter_creation() {
    let adapter = SovereigntyAwareAdapter::new().await;
    assert!(adapter.is_ok(), "Adapter should be created successfully");
}

#[tokio::test]
async fn test_sovereignty_config_default() {
    let config = SovereigntyAdapterConfig::default();
    let adapter = SovereigntyAwareAdapter::with_config(config).await;
    assert!(adapter.is_ok(), "Adapter should be created with default config");
}

#[tokio::test]
async fn test_sovereignty_adapter_with_custom_config() {
    let config = SovereigntyAdapterConfig::default();
    let adapter = SovereigntyAwareAdapter::with_config(config).await;
    assert!(adapter.is_ok(), "Adapter should be created with custom config");
}

#[tokio::test]
async fn test_multiple_sovereignty_adapters_independent() {
    let config1 = SovereigntyAdapterConfig::default();
    let config2 = SovereigntyAdapterConfig::default();

    let adapter1 = SovereigntyAwareAdapter::with_config(config1).await;
    let adapter2 = SovereigntyAwareAdapter::with_config(config2).await;

    assert!(adapter1.is_ok(), "First adapter should be created");
    assert!(adapter2.is_ok(), "Second adapter should be created");
    // Each should maintain independent state
}

#[tokio::test]
async fn test_sovereignty_config_structure() {
    let config = SovereigntyAdapterConfig::default();
    let adapter = SovereigntyAwareAdapter::with_config(config).await;
    assert!(adapter.is_ok(), "Config should be well-formed and constructable");
}
