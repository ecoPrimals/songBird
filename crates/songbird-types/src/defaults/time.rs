// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Zero-dependency UTC timestamp formatting.
//!
//! Provides RFC 3339 timestamps using only `std::time::SystemTime` — no `chrono` or `time` crate
//! needed. Use [`rfc3339_now`] anywhere a simple UTC timestamp string is required.

use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current UTC time formatted as an RFC 3339 string (e.g. `"2026-07-24T22:33:00Z"`).
///
/// # Errors
///
/// Never panics. If the system clock is before the Unix epoch, returns the epoch itself.
#[must_use]
pub fn rfc3339_now() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    epoch_secs_to_rfc3339(secs)
}

/// Formats a Unix epoch timestamp (seconds) as an RFC 3339 UTC string.
#[must_use]
pub fn epoch_secs_to_rfc3339(secs: u64) -> String {
    let (days, day_rem) = (secs / 86400, secs % 86400);
    let hours = day_rem / 3600;
    let mins = (day_rem % 3600) / 60;
    let s = day_rem % 60;

    let (y, m, d) = days_to_ymd(days);
    format!("{y:04}-{m:02}-{d:02}T{hours:02}:{mins:02}:{s:02}Z")
}

fn days_to_ymd(mut days: u64) -> (i32, u32, u32) {
    let mut y = 1970i32;
    loop {
        let leap = is_leap(y);
        let year_days: u64 = if leap {
            366
        } else {
            365
        };
        if days < year_days {
            break;
        }
        days -= year_days;
        y += 1;
    }
    let leap = is_leap(y);
    let month_lengths: [u32; 12] = [
        31,
        if leap {
            29
        } else {
            28
        },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 1u32;
    for ml in month_lengths {
        if days < u64::from(ml) {
            break;
        }
        days -= u64::from(ml);
        m += 1;
    }
    #[expect(clippy::cast_possible_truncation, reason = "days remaining in month ≤ 30")]
    let d = days as u32 + 1;
    (y, m, d)
}

const fn is_leap(y: i32) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_zero() {
        assert_eq!(epoch_secs_to_rfc3339(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn known_date() {
        // 2026-07-24T12:00:00Z = 1784894400
        assert_eq!(epoch_secs_to_rfc3339(1_784_894_400), "2026-07-24T12:00:00Z");
    }

    #[test]
    fn leap_year_feb29() {
        // 2024-02-29T00:00:00Z = 1709164800
        assert_eq!(epoch_secs_to_rfc3339(1_709_164_800), "2024-02-29T00:00:00Z");
    }

    #[test]
    fn rfc3339_now_format() {
        let ts = rfc3339_now();
        assert!(ts.ends_with('Z'));
        assert_eq!(ts.len(), 20);
        assert_eq!(&ts[4..5], "-");
        assert_eq!(&ts[7..8], "-");
        assert_eq!(&ts[10..11], "T");
    }
}
