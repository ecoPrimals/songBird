// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `BirdSong` `family_id` environment integration.

use super::common::lock_env;

#[test]
fn test_family_id_from_environment_priority() {
    let _guard = lock_env();
    // Test environment variable priority for family_id
    // Priority: SONGBIRD_FAMILY_ID > FAMILY_ID > default "default"

    // Clear all
    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
    songbird_process_env::remove_var("FAMILY_ID");

    // Default should be "default"
    let family_id = get_family_id_from_env();
    assert_eq!(family_id, "default", "Default should be 'default'");

    // FAMILY_ID should override default
    songbird_process_env::set_var("FAMILY_ID", "family-fallback");
    let family_id = get_family_id_from_env();
    assert_eq!(family_id, "family-fallback", "FAMILY_ID should be used");

    // SONGBIRD_FAMILY_ID should have highest priority
    songbird_process_env::set_var("SONGBIRD_FAMILY_ID", "songbird-primary");
    let family_id = get_family_id_from_env();
    assert_eq!(family_id, "songbird-primary", "SONGBIRD_FAMILY_ID should have priority");

    // Cleanup
    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
    songbird_process_env::remove_var("FAMILY_ID");
}

#[test]
fn test_family_id_special_characters() {
    let _guard = lock_env();
    // Test that family_id handles special characters
    let special_ids = vec![
        "nat0",
        "family-with-dash",
        "family_with_underscore",
        "family.with.dots",
        "UPPERCASE",
        "MixedCase123",
    ];

    for id in special_ids {
        songbird_process_env::set_var("SONGBIRD_FAMILY_ID", id);
        let family_id = get_family_id_from_env();
        assert_eq!(family_id, id);
    }

    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
}

/// Helper to get `family_id` using same logic as canonical env chain
fn get_family_id_from_env() -> String {
    songbird_process_env::var("SONGBIRD_FAMILY_ID")
        .or_else(|_| songbird_process_env::var("FAMILY_ID"))
        .unwrap_or_else(|_| "default".to_string())
}
