// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_universal::types::*;

// ============================================================================
// DISCOVERY FILTERS TESTS
// ============================================================================

#[test]
fn test_discovery_filters_default() {
    let filters = DiscoveryFilters::default();
    assert_eq!(filters.capability_types.len(), 0);
    assert_eq!(filters.security_levels.len(), 0);
    assert_eq!(filters.geographic_regions.len(), 0);
    assert!(filters.performance_requirements.is_none());
}

#[test]
fn test_discovery_filters_with_criteria() {
    let filters = DiscoveryFilters {
        capability_types: vec!["compute".to_string(), "storage".to_string()],
        security_levels: vec![SecurityLevel::High],
        geographic_regions: vec!["us-west".to_string()],
        performance_requirements: Some(QosMetrics::default()),
    };

    assert_eq!(filters.capability_types.len(), 2);
    assert_eq!(filters.security_levels.len(), 1);
    assert_eq!(filters.geographic_regions.len(), 1);
    assert!(filters.performance_requirements.is_some());
}

#[test]
fn test_discovery_filters_clone() {
    let filters1 = DiscoveryFilters {
        capability_types: vec!["ai".to_string()],
        security_levels: vec![SecurityLevel::Maximum],
        geographic_regions: vec!["eu-central".to_string()],
        performance_requirements: None,
    };

    let filters2 = filters1.clone();
    assert_eq!(filters1.capability_types, filters2.capability_types);
}
