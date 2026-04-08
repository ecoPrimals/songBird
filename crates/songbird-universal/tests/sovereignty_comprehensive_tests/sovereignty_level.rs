// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

// ============================================================================
// SOVEREIGNTY LEVEL TESTS
// ============================================================================

use crate::imports::*;

#[test]
fn test_sovereignty_level_variants() -> SongbirdResult<()> {
    let fully = SovereigntyLevel::FullySovereign;
    let highly = SovereigntyLevel::HighlySovereign;
    let moderately = SovereigntyLevel::ModeratelySovereign;
    let limited = SovereigntyLevel::LimitedSovereignty;
    let minimal = SovereigntyLevel::NonSovereign;

    // Ensure all variants are distinct
    assert_ne!(format!("{fully:?}"), format!("{:?}", highly));
    assert_ne!(format!("{highly:?}"), format!("{:?}", moderately));
    assert_ne!(format!("{moderately:?}"), format!("{:?}", limited));
    assert_ne!(format!("{limited:?}"), format!("{:?}", minimal));
    Ok(())
}

#[test]
fn test_sovereignty_level_ordering() -> SongbirdResult<()> {
    // Ensure sovereignty levels can be compared
    let levels = [
        SovereigntyLevel::FullySovereign,
        SovereigntyLevel::HighlySovereign,
        SovereigntyLevel::ModeratelySovereign,
        SovereigntyLevel::LimitedSovereignty,
        SovereigntyLevel::NonSovereign,
    ];

    // All should be valid and distinct
    for (i, level1) in levels.iter().enumerate() {
        for (j, level2) in levels.iter().enumerate() {
            if i != j {
                assert_ne!(format!("{level1:?}"), format!("{:?}", level2));
            }
        }
    }
    Ok(())
}
