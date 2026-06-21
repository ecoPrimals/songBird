// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Tor network consensus
//!
//! Fetches and parses the Tor network consensus document from directory authorities.

use crate::directory::authorities::DIRECTORY_AUTHORITIES;
use crate::directory::{CircuitPath, DirectoryAuthority, RelayInfo};
use crate::error::{Error, Result};
use crate::http_fetch;
use songbird_crypto_provider::CryptoProvider;
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};

/// Tor network consensus
#[derive(Debug, Clone)]
pub struct Consensus {
    /// When this consensus is valid from
    pub valid_after: SystemTime,
    /// When this consensus is fresh until
    pub fresh_until: SystemTime,
    /// When this consensus is valid until
    pub valid_until: SystemTime,
    /// List of relays in the network
    pub relays: Vec<RelayInfo>,
}

impl Consensus {
    /// Fetch consensus from directory authorities
    ///
    /// Tries multiple authorities until successful.
    ///
    /// # Errors
    /// Returns error if all authorities fail or consensus parse fails.
    pub async fn fetch(security_provider: &CryptoProvider) -> Result<Self> {
        info!("Fetching Tor network consensus");

        // Try up to 3 authorities
        for authority in DIRECTORY_AUTHORITIES.iter().take(3) {
            debug!("Trying directory authority: {}", authority.nickname);

            match Self::fetch_from_authority(authority, security_provider).await {
                Ok(consensus) => {
                    info!(
                        authority = authority.nickname,
                        relays = consensus.relays.len(),
                        "Successfully fetched consensus"
                    );
                    return Ok(consensus);
                }
                Err(e) => {
                    warn!(
                        authority = authority.nickname,
                        error = %e,
                        "Failed to fetch from authority, trying next"
                    );
                }
            }
        }

        Err(Error::Consensus(String::from("Failed to fetch consensus from all authorities")))
    }

    /// Fetch consensus from specific authority
    async fn fetch_from_authority(
        authority: &DirectoryAuthority,
        _security_provider: &CryptoProvider,
    ) -> Result<Self> {
        let url = authority.consensus_url();

        debug!("Fetching consensus from: {}", url);

        // Pure Rust HTTP GET (all directory authorities serve plain HTTP)
        let body = http_fetch::get(&url, Duration::from_secs(30)).await?;

        debug!("Consensus downloaded, size: {} bytes", body.len());

        // Parse consensus
        Self::parse(&body)
    }

    /// Parse consensus document
    fn parse(data: &str) -> Result<Self> {
        use crate::directory::parser::parse_consensus;

        debug!("Parsing consensus document");

        // Parse relay entries
        let relays = parse_consensus(data)?;

        debug!("Parsed {} relays from consensus", relays.len());

        // Parse timestamps from consensus header
        // Format: "valid-after YYYY-MM-DD HH:MM:SS"
        let valid_after = Self::parse_timestamp(data, "valid-after");
        let fresh_until = Self::parse_timestamp(data, "fresh-until");
        let valid_until = Self::parse_timestamp(data, "valid-until");

        // Fallback to reasonable defaults if timestamps not found
        let now = SystemTime::now();

        Ok(Self {
            valid_after: valid_after.unwrap_or(now),
            fresh_until: fresh_until.unwrap_or(now + Duration::from_secs(3600)),
            valid_until: valid_until.unwrap_or(now + Duration::from_secs(7200)),
            relays,
        })
    }

    /// Parse a timestamp line from the consensus document
    ///
    /// Looks for lines like: `valid-after 2026-02-08 12:00:00`
    /// Parses the date/time and converts to `SystemTime`.
    fn parse_timestamp(data: &str, keyword: &str) -> Option<SystemTime> {
        for line in data.lines() {
            if let Some(rest) = line.strip_prefix(keyword) {
                let ts_str = rest.trim();
                // Format: "YYYY-MM-DD HH:MM:SS"
                if let Some(unix) = parse_datetime_to_unix(ts_str) {
                    return Some(SystemTime::UNIX_EPOCH + Duration::from_secs(unix));
                }
            }
        }
        None
    }

    /// Select a circuit path (guard -> middle -> exit/hsdir)
    ///
    /// # Errors
    /// Returns error if not enough relays or no suitable path found.
    pub fn select_path(&self) -> Result<CircuitPath> {
        // Path selection is heuristic (guard/middle/exit flags), not weighted or privacy-tuned yet.

        // For now, require at least 3 relays
        if self.relays.len() < 3 {
            return Err(Error::Consensus(String::from("Not enough relays in consensus")));
        }

        // Select guard (must have GUARD flag)
        let guard = self
            .relays
            .iter()
            .find(|r| r.is_guard())
            .ok_or_else(|| Error::Consensus(String::from("No suitable guard found")))?
            .clone();

        // Select middle (must be fast + stable)
        let middle = self
            .relays
            .iter()
            .find(|r| r.is_middle() && r.fingerprint != guard.fingerprint)
            .ok_or_else(|| Error::Consensus(String::from("No suitable middle found")))?
            .clone();

        // Select exit/hsdir (must have HSDIR flag for onion services)
        let exit = self
            .relays
            .iter()
            .find(|r| {
                r.is_hsdir()
                    && r.fingerprint != guard.fingerprint
                    && r.fingerprint != middle.fingerprint
            })
            .ok_or_else(|| Error::Consensus(String::from("No suitable hsdir found")))?
            .clone();

        Ok(CircuitPath {
            guard,
            middle,
            exit,
        })
    }

    /// Check if consensus is still fresh
    #[must_use]
    pub fn is_fresh(&self) -> bool {
        SystemTime::now() < self.fresh_until
    }

    /// Check if consensus is still valid
    #[must_use]
    pub fn is_valid(&self) -> bool {
        SystemTime::now() < self.valid_until
    }

    /// Fetch ntor key for a relay from its descriptor
    ///
    /// Returns the 32-byte ntor-onion-key if found.
    ///
    /// # Errors
    /// Returns error if descriptor fetch or parse fails.
    pub async fn fetch_relay_ntor_key(relay: &RelayInfo) -> Result<[u8; 32]> {
        // Use STANDARD_NO_PAD since Tor omits base64 padding
        use base64::{Engine, engine::general_purpose::STANDARD_NO_PAD as BASE64};

        // Convert fingerprint to hex
        let fp_hex: String =
            relay.fingerprint.iter().fold(String::new(), |s, b| format!("{s}{b:02X}"));

        // Directory authorities that serve descriptors (DirPort)
        // moria1 (MIT) is reliable and responds quickly
        let dir_servers = [
            ("128.31.0.39", 9131),   // moria1 (MIT)
            ("194.109.206.212", 80), // dizum
            ("199.58.81.140", 80),   // longclaw
        ];

        let mut last_error = None;

        for (host, port) in &dir_servers {
            let url = format!("http://{host}:{port}/tor/server/fp/{fp_hex}");
            debug!("Trying directory server {}:{} for {}", host, port, relay.nickname);

            match http_fetch::get(&url, Duration::from_secs(10)).await {
                Ok(body) => {
                    debug!("Descriptor body length: {} bytes", body.len());
                    // Parse ntor-onion-key from descriptor
                    // Format: ntor-onion-key <base64-encoded-32-byte-key>
                    // Note: There's also ntor-onion-key-crosscert, skip that
                    for line in body.lines() {
                        if line.starts_with("ntor-onion-key ") && !line.contains("crosscert") {
                            let Some(key_str) = line.strip_prefix("ntor-onion-key ") else {
                                continue;
                            };
                            let key_str = key_str.trim();
                            debug!("Found ntor-onion-key line: {}", key_str);
                            match BASE64.decode(key_str) {
                                Ok(key_bytes) if key_bytes.len() == 32 => {
                                    let mut key = [0u8; 32];
                                    key.copy_from_slice(&key_bytes);
                                    info!("Fetched ntor key for {}", relay.nickname);
                                    return Ok(key);
                                }
                                Ok(key_bytes) => {
                                    last_error = Some(format!(
                                        "ntor key wrong size: {} (expected 32)",
                                        key_bytes.len()
                                    ));
                                }
                                Err(e) => {
                                    last_error = Some(format!("Failed to decode ntor key: {e}"));
                                }
                            }
                        }
                    }
                    if last_error.is_none() {
                        last_error = Some(format!(
                            "No ntor-onion-key found in descriptor ({} bytes)",
                            body.len()
                        ));
                    }
                }
                Err(e) => {
                    debug!("Request to {}:{} failed: {}", host, port, e);
                    last_error = Some(format!("Request failed: {e}"));
                }
            }
        }

        Err(Error::Consensus(format!(
            "Failed to fetch ntor key for {}: {}",
            relay.nickname,
            last_error.unwrap_or_else(|| String::from("Unknown error"))
        )))
    }

    /// Fetch ntor keys for path relays
    ///
    /// Updates the path with ntor keys fetched from relay descriptors.
    ///
    /// # Errors
    /// Returns error if guard ntor key cannot be fetched.
    pub async fn fetch_path_ntor_keys(path: &mut CircuitPath) -> Result<()> {
        info!("Fetching ntor keys for circuit path");

        // Fetch for guard
        if path.guard.ntor_key.is_none() {
            match Self::fetch_relay_ntor_key(&path.guard).await {
                Ok(key) => path.guard.ntor_key = Some(key),
                Err(e) => warn!("Failed to fetch guard ntor key: {}", e),
            }
        }

        // Fetch for middle
        if path.middle.ntor_key.is_none() {
            match Self::fetch_relay_ntor_key(&path.middle).await {
                Ok(key) => path.middle.ntor_key = Some(key),
                Err(e) => warn!("Failed to fetch middle ntor key: {}", e),
            }
        }

        // Fetch for exit
        if path.exit.ntor_key.is_none() {
            match Self::fetch_relay_ntor_key(&path.exit).await {
                Ok(key) => path.exit.ntor_key = Some(key),
                Err(e) => warn!("Failed to fetch exit ntor key: {}", e),
            }
        }

        // Check if we got keys for at least the guard (required for first hop)
        if path.guard.ntor_key.is_none() {
            return Err(Error::Consensus(String::from("Failed to fetch guard ntor key")));
        }

        Ok(())
    }
}

/// Parse "YYYY-MM-DD HH:MM:SS" to Unix timestamp (pure Rust, zero deps)
///
/// Returns None if the format is invalid.
fn parse_datetime_to_unix(s: &str) -> Option<u64> {
    // Days before each month (non-leap year)
    const DAYS_BEFORE_MONTH: [u64; 13] = [0, 0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

    // Expected: "2026-02-08 12:00:00"
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }

    let date_parts: Vec<&str> = parts[0].split('-').collect();
    let time_parts: Vec<&str> = parts[1].split(':').collect();

    if date_parts.len() != 3 || time_parts.len() != 3 {
        return None;
    }

    let year: u64 = date_parts[0].parse().ok()?;
    let month: u64 = date_parts[1].parse().ok()?;
    let day: u64 = date_parts[2].parse().ok()?;
    let hour: u64 = time_parts[0].parse().ok()?;
    let minute: u64 = time_parts[1].parse().ok()?;
    let second: u64 = time_parts[2].parse().ok()?;

    // Validate ranges
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }

    // Calculate days from epoch (1970-01-01)
    let mut days: u64 = 0;

    // Years
    for y in 1970..year {
        days += if is_leap_year(y) {
            366
        } else {
            365
        };
    }

    // Months
    days += DAYS_BEFORE_MONTH[usize::try_from(month).unwrap_or(0)];
    if month > 2 && is_leap_year(year) {
        days += 1;
    }

    // Days (1-indexed)
    days += day - 1;

    Some(days * 86400 + hour * 3600 + minute * 60 + second)
}

/// Check if a year is a leap year
const fn is_leap_year(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::directory::{RelayFlags, RelayInfo};
    use crate::error::Error;
    use std::net::{IpAddr, Ipv4Addr};
    use std::time::{Duration, SystemTime};

    fn sample_relay(name: &str, fp: u8, flags: RelayFlags) -> RelayInfo {
        let mut fingerprint = [0u8; 20];
        fingerprint[0] = fp;
        RelayInfo {
            nickname: name.to_string(),
            fingerprint,
            address: IpAddr::V4(Ipv4Addr::new(10, 0, 0, fp)),
            or_port: 443,
            dir_port: None,
            flags,
            bandwidth: 50_000,
            ntor_key: None,
            version: None,
        }
    }

    #[test]
    fn test_parse_empty() {
        // Empty string will fail since parser expects "r " prefix
        let result = Consensus::parse("");
        // Parser fails on empty input (expects at least "r " line)
        // This is expected behavior - empty consensus is invalid
        assert!(result.is_err());
    }

    #[test]
    fn test_is_fresh() {
        let consensus = Consensus {
            valid_after: SystemTime::now() - Duration::from_secs(3600),
            fresh_until: SystemTime::now() + Duration::from_secs(3600),
            valid_until: SystemTime::now() + Duration::from_secs(7200),
            relays: vec![],
        };

        assert!(consensus.is_fresh());
        assert!(consensus.is_valid());
    }

    #[test]
    fn test_parse_datetime_to_unix_valid() {
        // 2026-02-08 00:00:00 UTC
        let ts = parse_datetime_to_unix("2026-02-08 00:00:00");
        assert!(ts.is_some());
        let unix = ts.expect("valid datetime");
        // Verify it's in a reasonable range (around Feb 2026)
        assert!(unix > 1_700_000_000); // After ~Nov 2023
        assert!(unix < 1_900_000_000); // Before ~Feb 2030
    }

    #[test]
    fn test_parse_datetime_epoch() {
        let ts = parse_datetime_to_unix("1970-01-01 00:00:00");
        assert_eq!(ts, Some(0));
    }

    #[test]
    fn test_parse_datetime_with_time() {
        let ts = parse_datetime_to_unix("1970-01-01 01:00:00");
        assert_eq!(ts, Some(3600));
    }

    #[test]
    fn test_parse_datetime_invalid_format() {
        assert!(parse_datetime_to_unix("not-a-date").is_none());
        assert!(parse_datetime_to_unix("2026-13-01 00:00:00").is_none()); // month 13
        assert!(parse_datetime_to_unix("2026-02-08").is_none()); // no time
    }

    #[test]
    fn test_parse_timestamp_from_consensus() {
        let doc = "network-status-version 3\nvote-status consensus\nvalid-after 2026-02-08 12:00:00\nfresh-until 2026-02-08 13:00:00\nvalid-until 2026-02-08 15:00:00\n";

        let va = Consensus::parse_timestamp(doc, "valid-after");
        let fu = Consensus::parse_timestamp(doc, "fresh-until");
        let vu = Consensus::parse_timestamp(doc, "valid-until");

        assert!(va.is_some());
        assert!(fu.is_some());
        assert!(vu.is_some());

        // fresh-until should be after valid-after
        assert!(fu.expect("fresh-until") > va.expect("valid-after"));
        // valid-until should be after fresh-until
        assert!(vu.expect("valid-until") > fu.expect("fresh-until"));
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2025));
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
    }

    #[test]
    fn select_path_errors_when_not_enough_relays() {
        let c = Consensus {
            valid_after: SystemTime::UNIX_EPOCH,
            fresh_until: SystemTime::UNIX_EPOCH + Duration::from_secs(3600),
            valid_until: SystemTime::UNIX_EPOCH + Duration::from_secs(7200),
            relays: vec![sample_relay("a", 1, RelayFlags::empty())],
        };
        let err = c.select_path().expect_err("need >=3 relays");
        assert!(matches!(err, Error::Consensus(_)));
    }

    #[test]
    fn select_path_succeeds_with_three_flagged_relays() {
        let guard_f = RelayFlags::GUARD
            | RelayFlags::FAST
            | RelayFlags::STABLE
            | RelayFlags::VALID
            | RelayFlags::RUNNING;
        let mid_f = RelayFlags::FAST | RelayFlags::STABLE | RelayFlags::VALID | RelayFlags::RUNNING;
        let exit_f = RelayFlags::HSDIR | RelayFlags::VALID | RelayFlags::RUNNING;

        let c = Consensus {
            valid_after: SystemTime::UNIX_EPOCH,
            fresh_until: SystemTime::UNIX_EPOCH + Duration::from_secs(3600),
            valid_until: SystemTime::UNIX_EPOCH + Duration::from_secs(7200),
            relays: vec![
                sample_relay("g", 1, guard_f),
                sample_relay("m", 2, mid_f),
                sample_relay("e", 3, exit_f),
            ],
        };

        let path = c.select_path().expect("path");
        assert_eq!(path.guard.nickname, "g");
        assert_eq!(path.middle.nickname, "m");
        assert_eq!(path.exit.nickname, "e");
    }

    #[test]
    fn consensus_not_valid_when_past_valid_until() {
        let past = SystemTime::UNIX_EPOCH + Duration::from_secs(10);
        let c = Consensus {
            valid_after: past,
            fresh_until: past,
            valid_until: past,
            relays: vec![],
        };
        assert!(!c.is_valid());
        assert!(!c.is_fresh());
    }

    #[test]
    fn parse_internal_roundtrip_header_timestamps() {
        let doc = "network-status-version 3\nvalid-after 2026-02-08 12:00:00\nfresh-until 2026-02-08 13:00:00\nvalid-until 2026-02-08 15:00:00\n\nr X AAAAAAAAAAAAAAAAAAAAAAAAAAA AAAAAAAAAAAAAAAAAAAAAAAAAAA 2026-02-07 00:00:00 1.2.3.4 443 0\ns Fast Running Valid\nw Bandwidth=1000\n";
        let parsed = Consensus::parse(doc).expect("parse");
        assert!(!parsed.relays.is_empty());
        assert!(parsed.valid_after <= parsed.fresh_until);
        assert!(parsed.fresh_until <= parsed.valid_until);
    }
}
