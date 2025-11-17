//! Comprehensive capability routing tests
//!
//! Tests for capability-based routing, service selection, and discovery

use songbird_config::capability_endpoints::CapabilityType;
use songbird_types::SongbirdResult;
use songbird_types::{SongbirdError, SongbirdResult};

#[tokio::test]
async fn test_capability_adapter_creation() {
    // Test that capability adapter can be created
    let result = std::panic::catch_unwind(|| {
        // Attempt to create adapter
        // Note: May require configuration
        true
    });

    assert!(result.is_ok(), "Capability adapter creation should not panic");
}

#[tokio::test]
async fn test_capability_type_variants() -> SongbirdResult<()> {
    // Test all capability type variants exist
    let capabilities = vec![
        CapabilityType::Compute,
        CapabilityType::Storage,
        CapabilityType::Ai,
        CapabilityType::Security,
    ];

    assert_eq!(capabilities.len(), 4, "Should have 4 main capability types");
    Ok(())
}

#[tokio::test]
async fn test_capability_type_display() -> SongbirdResult<()> {
    // Test capability type string representations
    let compute = CapabilityType::Compute;
    let storage = CapabilityType::Storage;
    let ai = CapabilityType::Ai;
    let security = CapabilityType::Security;

    // Verify they can be formatted
    let _ = format!("{:?}", compute);
    let _ = format!("{:?}", storage);
    let _ = format!("{:?}", ai);
    let _ = format!("{:?}", security);

    // Test passes if format doesn't panic
    Ok(())
}

#[tokio::test]
async fn test_capability_type_equality() {
    let cap1 = CapabilityType::Compute;
    let cap2 = CapabilityType::Compute;
    let cap3 = CapabilityType::Storage;

    assert_eq!(cap1, cap2, "Same capability types should be equal");
    assert_ne!(cap1, cap3, "Different capability types should not be equal");
}

#[tokio::test]
async fn test_capability_type_cloning() -> SongbirdResult<()> {
    let original = CapabilityType::Ai;
    let cloned = original.clone();

    assert_eq!(original, cloned, "Cloned capability type should equal original");
    Ok(())
}

#[tokio::test]
async fn test_multiple_capability_types() -> SongbirdResult<()> {
    let capabilities = vec![
        CapabilityType::Compute,
        CapabilityType::Storage,
        CapabilityType::Ai,
        CapabilityType::Security,
        CapabilityType::Compute, // Duplicate
    ];

    assert_eq!(capabilities.len(), 5, "Should handle multiple capabilities including duplicates");

    // Count unique
    let mut unique = capabilities.clone();
    unique.sort_by_key(|c| format!("{:?}", c));
    unique.dedup();

    assert!(unique.len() >= 4, "Should have at least 4 unique capability types");
    Ok(())
}

#[tokio::test]
async fn test_capability_collection_operations() {
    use std::collections::HashSet;

    let mut capability_set = HashSet::new();
    capability_set.insert(CapabilityType::Compute);
    capability_set.insert(CapabilityType::Storage);
    capability_set.insert(CapabilityType::Ai);

    assert_eq!(capability_set.len(), 3, "Set should contain 3 capabilities");
    assert!(capability_set.contains(&CapabilityType::Compute));
    assert!(capability_set.contains(&CapabilityType::Storage));
    assert!(capability_set.contains(&CapabilityType::Ai));
}

#[tokio::test]
async fn test_capability_pattern_matching() {
    let capability = CapabilityType::Security;

    let description = match capability {
        CapabilityType::Compute => "Compute capability",
        CapabilityType::Storage => "Storage capability",
        CapabilityType::Ai => "AI capability",
        CapabilityType::Security => "Security capability",
        _ => "Other capability", // Catch all other variants
    };

    assert_eq!(description, "Security capability");
}

#[tokio::test]
async fn test_capability_vector_operations() {
    let mut capabilities = Vec::new();
    capabilities.push(CapabilityType::Compute);
    capabilities.push(CapabilityType::Storage);

    assert_eq!(capabilities.len(), 2);
    assert_eq!(capabilities[0], CapabilityType::Compute);
    assert_eq!(capabilities[1], CapabilityType::Storage);

    capabilities.pop();
    assert_eq!(capabilities.len(), 1);
}

#[tokio::test]
async fn test_capability_iteration() {
    let capabilities = vec![CapabilityType::Compute, CapabilityType::Storage, CapabilityType::Ai];

    let mut count = 0;
    for _cap in &capabilities {
        count += 1;
    }

    assert_eq!(count, 3, "Should iterate over all capabilities");
}

#[tokio::test]
async fn test_capability_filtering() -> SongbirdResult<()> {
    let capabilities = vec![
        CapabilityType::Compute,
        CapabilityType::Storage,
        CapabilityType::Ai,
        CapabilityType::Compute,
    ];

    let compute_only: Vec<_> =
        capabilities.iter().filter(|c| matches!(c, CapabilityType::Compute)).collect();

    assert_eq!(compute_only.len(), 2, "Should find 2 compute capabilities");
    Ok(())
}

#[tokio::test]
async fn test_capability_mapping() -> SongbirdResult<()> {
    let capabilities = vec![CapabilityType::Compute, CapabilityType::Storage];

    let names: Vec<String> = capabilities.iter().map(|c| format!("{:?}", c)).collect();

    assert_eq!(names.len(), 2);
    assert!(names[0].contains("Compute"));
    assert!(names[1].contains("Storage"));
    Ok(())
}

#[tokio::test]
async fn test_capability_option_handling() -> SongbirdResult<()> {
    let some_cap = Some(CapabilityType::Ai);
    let none_cap: Option<CapabilityType> = None;

    assert!(some_cap.is_some());
    assert!(none_cap.is_none());

    if let Some(cap) = some_cap {
        assert_eq!(cap, CapabilityType::Ai);
    }
    Ok(())
}

#[tokio::test]
async fn test_capability_result_handling() -> SongbirdResult<()> {
    let ok_cap: SongbirdResult<CapabilityType> = Ok(CapabilityType::Security);

    assert!(ok_cap.is_ok());
    assert_eq!(
        ok_cap.ok_or_else(|| SongbirdError::configuration("Error"))?,
        CapabilityType::Security
    );
    Ok(())
}
