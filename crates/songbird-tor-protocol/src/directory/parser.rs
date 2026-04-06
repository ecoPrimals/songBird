// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Consensus parsing with nom
//!
//! Parses Tor network consensus documents using nom combinator parsers.
//! Format: <https://spec.torproject.org/dir-spec/formats.html>

use crate::directory::{RelayFlags, RelayInfo};
use crate::error::{Error, Result};
use nom::{
    IResult,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{digit1, line_ending, multispace0, space1},
    combinator::{map_res, opt},
    multi::{many0, separated_list1},
    sequence::preceded,
};
use std::net::{IpAddr, Ipv4Addr};

/// Parse full consensus document
pub fn parse_consensus(input: &str) -> Result<Vec<RelayInfo>> {
    match consensus_document(input) {
        Ok((_, relays)) => Ok(relays),
        Err(e) => Err(Error::Parse(format!("Consensus parse error: {e:?}"))),
    }
}

/// Parse consensus document
fn consensus_document(input: &str) -> IResult<&str, Vec<RelayInfo>> {
    let (input, _) = multispace0(input)?;

    // Skip header by finding first relay line (starts with "\nr " at beginning of line)
    // This avoids matching "r " in header values like "valid-after 2026-02-07"
    let (input, _) = take_until("\nr ")(input)?;

    // Consume the newline, leaving "r " at start
    let (input, _) = tag("\n")(input)?;

    // Parse relay entries
    let (input, relays) = many0(relay_entry)(input)?;

    Ok((input, relays))
}

/// Parse single relay entry (r, a, s, v, pr, w, p lines)
///
/// Format (2026):
/// - r: router info (required)
/// - a: IPv6 address (optional)
/// - s: flags (required)
/// - v: version (optional)
/// - pr: protocols (optional)
/// - w: bandwidth (required)
/// - p: exit policy (optional)
fn relay_entry(input: &str) -> IResult<&str, RelayInfo> {
    let (input, _) = multispace0(input)?;

    // r line: nickname identity published IP port
    let (input, (nickname, fingerprint, address, or_port)) = r_line(input)?;

    // a line: IPv6 address (optional, skip)
    let (input, _) = opt(a_line)(input)?;

    // s line: flags (GUARD, FAST, STABLE, etc.)
    let (input, flags) = s_line(input)?;

    // v line: version (optional, skip)
    let (input, _) = opt(v_line)(input)?;

    // pr line: protocols (optional, skip)
    let (input, _) = opt(pr_line)(input)?;

    // w line: bandwidth
    let (input, bandwidth) = w_line(input)?;

    // p line: exit policy (optional, skip)
    let (input, _) = opt(p_line)(input)?;

    Ok((
        input,
        RelayInfo {
            nickname,
            fingerprint,
            address,
            or_port,
            dir_port: None,
            flags,
            bandwidth,
            ntor_key: None, // Populated when microdescriptor parsing is wired
            version: None,  // Tor version line not parsed from consensus `r` entries here
        },
    ))
}

/// Parse r line: r nickname identity digest published IP `ORPort` `DirPort`
///
/// Format: r <nickname> <identity-b64> <digest-b64> <published-date> <published-time> <IP> <ORPort> <DirPort>
fn r_line(input: &str) -> IResult<&str, (String, [u8; 20], IpAddr, u16)> {
    let (input, _) = tag("r ")(input)?;

    // Nickname
    let (input, nickname) =
        map_res(take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '-'), |s: &str| {
            Ok::<_, Error>(s.to_string())
        })(input)?;
    let (input, _) = space1(input)?;

    // Identity (base64, ~27 chars decodes to 20 bytes)
    let (input, identity_b64) =
        take_while1(|c: char| c.is_alphanumeric() || c == '+' || c == '/' || c == '=')(input)?;
    let fingerprint = base64_to_fingerprint(identity_b64)?;
    let (input, _) = space1(input)?;

    // Descriptor digest (base64, ~27 chars - skip)
    let (input, _) =
        take_while1(|c: char| c.is_alphanumeric() || c == '+' || c == '/' || c == '=')(input)?;
    let (input, _) = space1(input)?;

    // Published date (YYYY-MM-DD format, skip)
    let (input, _) = take_while1(|c: char| c.is_ascii_digit() || c == '-')(input)?;
    let (input, _) = space1(input)?;

    // Published time (HH:MM:SS format, skip)
    let (input, _) = take_while1(|c: char| c.is_ascii_digit() || c == ':')(input)?;
    let (input, _) = space1(input)?;

    // IP address
    let (input, ip_str) = take_while1(|c: char| c.is_ascii_digit() || c == '.')(input)?;
    let address = ip_str.parse::<Ipv4Addr>().map(IpAddr::V4).map_err(|_| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
    })?;
    let (input, _) = space1(input)?;

    // OR port
    let (input, or_port) = map_res(digit1, |s: &str| s.parse::<u16>())(input)?;

    // Dir port (optional, skip)
    let (input, _) = opt(preceded(space1, digit1))(input)?;

    let (input, _) = line_ending(input)?;

    Ok((input, (nickname, fingerprint, address, or_port)))
}

/// Parse s line: s Flag1 Flag2 Flag3...
/// Format: "s <flag1> <flag2> ..."
fn s_line(input: &str) -> IResult<&str, RelayFlags> {
    let (input, _) = tag("s ")(input)?; // Note: "s " with space

    let (input, flag_strs) =
        separated_list1(space1, take_while1(|c: char| c.is_alphanumeric()))(input)?;

    let (input, _) = line_ending(input)?;

    let mut flags = RelayFlags::empty();
    for flag in flag_strs {
        match flag {
            "Authority" => flags |= RelayFlags::AUTHORITY,
            "BadExit" => flags |= RelayFlags::BAD_EXIT,
            "Exit" => flags |= RelayFlags::EXIT,
            "Fast" => flags |= RelayFlags::FAST,
            "Guard" => flags |= RelayFlags::GUARD,
            "HSDir" => flags |= RelayFlags::HSDIR,
            "Running" => flags |= RelayFlags::RUNNING,
            "Stable" => flags |= RelayFlags::STABLE,
            "Valid" => flags |= RelayFlags::VALID,
            "V2Dir" => flags |= RelayFlags::V2DIR,
            _ => {} // Ignore unknown flags
        }
    }

    Ok((input, flags))
}

/// Parse a line: a [IPv6]:port (optional, skip)
fn a_line(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("a ")(input)?;
    let (input, _) = take_until("\n")(input)?;
    let (input, _) = line_ending(input)?;
    Ok((input, ()))
}

/// Parse v line: v Tor version (skip)
fn v_line(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("v ")(input)?;
    let (input, _) = take_until("\n")(input)?;
    let (input, _) = line_ending(input)?;
    Ok((input, ()))
}

/// Parse pr line: pr protocols (optional, skip)
fn pr_line(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("pr ")(input)?;
    let (input, _) = take_until("\n")(input)?;
    let (input, _) = line_ending(input)?;
    Ok((input, ()))
}

/// Parse w line: w Bandwidth=12345
fn w_line(input: &str) -> IResult<&str, u64> {
    let (input, _) = tag("w Bandwidth=")(input)?;
    let (input, bandwidth) = map_res(digit1, |s: &str| s.parse::<u64>())(input)?;
    let (input, _) = take_until("\n")(input)?;
    let (input, _) = line_ending(input)?;
    Ok((input, bandwidth))
}

/// Parse p line: p accept/reject ports (skip)
fn p_line(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("p ")(input)?;
    let (input, _) = take_until("\n")(input)?;
    let (input, _) = line_ending(input)?;
    Ok((input, ()))
}

/// Convert base64 identity to SHA1 fingerprint (20 bytes)
fn base64_to_fingerprint(
    b64: &str,
) -> std::result::Result<[u8; 20], nom::Err<nom::error::Error<&str>>> {
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    // Add padding if necessary (Tor base64 often lacks padding)
    let padded = match b64.len() % 4 {
        2 => format!("{b64}=="),
        3 => format!("{b64}="),
        _ => b64.to_string(),
    };

    let bytes = STANDARD.decode(&padded).map_err(|_| {
        nom::Err::Failure(nom::error::Error::new(b64, nom::error::ErrorKind::Digit))
    })?;

    if bytes.len() != 20 {
        return Err(nom::Err::Failure(nom::error::Error::new(b64, nom::error::ErrorKind::Digit)));
    }

    let mut fingerprint = [0u8; 20];
    fingerprint.copy_from_slice(&bytes);

    Ok(fingerprint)
}

/// Debug helper: Try to parse a single relay entry and return detailed error info
///
/// Useful for diagnosing consensus parsing issues during development
/// or when debugging relay entry format variations.
#[allow(dead_code, reason = "debug helper for development and relay parse diagnostics")]
pub fn debug_parse_relay_entry(input: &str) -> std::result::Result<(RelayInfo, &str), String> {
    // Try each step and report where it fails
    let input = input.trim_start();

    // Step 1: r line
    let (input, (nickname, fingerprint, address, or_port)) = match r_line(input) {
        Ok(r) => r,
        Err(e) => {
            return Err(format!(
                "r_line failed: {:?}\nInput was: '{}'",
                e,
                &input[..std::cmp::min(100, input.len())]
            ));
        }
    };

    // Step 2: optional a line
    let input = match opt(a_line)(input) {
        Ok((i, _)) => i,
        Err(e) => return Err(format!("a_line failed: {e:?}")),
    };

    // Step 3: s line
    let (input, flags) = match s_line(input) {
        Ok(r) => r,
        Err(e) => {
            return Err(format!(
                "s_line failed: {:?}\nInput was: '{}'",
                e,
                &input[..std::cmp::min(100, input.len())]
            ));
        }
    };

    // Step 4: optional v line
    let input = match opt(v_line)(input) {
        Ok((i, _)) => i,
        Err(e) => return Err(format!("v_line failed: {e:?}")),
    };

    // Step 5: optional pr line
    let input = match opt(pr_line)(input) {
        Ok((i, _)) => i,
        Err(e) => return Err(format!("pr_line failed: {e:?}")),
    };

    // Step 6: w line
    let (input, bandwidth) = match w_line(input) {
        Ok(r) => r,
        Err(e) => {
            return Err(format!(
                "w_line failed: {:?}\nInput was: '{}'",
                e,
                &input[..std::cmp::min(100, input.len())]
            ));
        }
    };

    // Step 7: optional p line
    let input = match opt(p_line)(input) {
        Ok((i, _)) => i,
        Err(e) => return Err(format!("p_line failed: {e:?}")),
    };

    Ok((
        RelayInfo {
            nickname,
            fingerprint,
            address,
            or_port,
            dir_port: None,
            flags,
            bandwidth,
            ntor_key: None,
            version: None,
        },
        input,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_r_line() {
        // Format: r nickname identity-b64 digest-b64 date time IP ORPort DirPort
        // Two base64 strings (both decode to 20 bytes)
        // Use 27 chars without explicit padding (Tor omits padding in consensus)
        let input = "r Test AAAAAAAAAAAAAAAAAAAAAAAAAAA AAAAAAAAAAAAAAAAAAAAAAAAAAA 2026-02-04 00:00:00 1.2.3.4 443 80\n";
        let result = r_line(input);

        if let Err(e) = &result {
            println!("Parse error: {e:?}");
        }

        assert!(result.is_ok());

        let (_, (nickname, fingerprint, _address, or_port)) = result.expect("parse should succeed");
        assert_eq!(nickname, "Test");
        assert_eq!(or_port, 443);
        assert_eq!(fingerprint.len(), 20);
    }

    #[test]
    fn test_parse_r_line_real() {
        // Real example from Tor consensus
        let input = "r lisdex AAAErLudKby6FyVrs1ko3b/Iq6k IE+F8M9BVgN0RmHuI0QtwsVqYhk 2026-02-07 02:28:49 152.53.144.50 8443 0\n";
        let result = r_line(input);

        if let Err(e) = &result {
            println!("Parse error: {e:?}");
        }

        assert!(result.is_ok());

        let (_, (nickname, fingerprint, address, or_port)) = result.expect("parse should succeed");
        assert_eq!(nickname, "lisdex");
        assert_eq!(or_port, 8443);
        assert_eq!(fingerprint.len(), 20);
        assert_eq!(address.to_string(), "152.53.144.50");
    }

    #[test]
    fn test_parse_s_line() {
        let input = "s Fast Guard Running Stable Valid\n";
        let result = s_line(input);
        assert!(result.is_ok());

        let (_, flags) = result.expect("parse should succeed");
        assert!(flags.contains(RelayFlags::FAST));
        assert!(flags.contains(RelayFlags::GUARD));
        assert!(flags.contains(RelayFlags::RUNNING));
        assert!(flags.contains(RelayFlags::STABLE));
        assert!(flags.contains(RelayFlags::VALID));
    }

    #[test]
    fn test_parse_w_line() {
        let input = "w Bandwidth=5000\n";
        let result = w_line(input);
        assert!(result.is_ok());

        let (_, bandwidth) = result.expect("parse should succeed");
        assert_eq!(bandwidth, 5000);
    }

    #[test]
    fn test_parse_full_relay_entry() {
        // Complete relay entry from real Tor consensus
        let input = r"r lisdex AAAErLudKby6FyVrs1ko3b/Iq6k IE+F8M9BVgN0RmHuI0QtwsVqYhk 2026-02-07 02:28:49 152.53.144.50 8443 0
a [2a0a:4cc0:c1:2aac::1]:8443
s Fast Guard Running Stable V2Dir Valid
v Tor 0.4.8.22
pr Conflux=1 Cons=1-2 Desc=1-2 DirCache=2 FlowCtrl=1-2 HSDir=2 HSIntro=4-5 HSRend=1-2 Link=1-5 LinkAuth=1,3 Microdesc=1-2 Padding=2 Relay=1-4
w Bandwidth=83000
p reject 1-65535
";

        let result = debug_parse_relay_entry(input);
        match result {
            Ok((relay, _remaining)) => {
                println!("✅ Parsed relay: {} at {}", relay.nickname, relay.address);
                assert_eq!(relay.nickname, "lisdex");
                assert_eq!(relay.or_port, 8443);
                assert_eq!(relay.bandwidth, 83000);
            }
            Err(e) => {
                panic!("❌ Parse failed: {e}");
            }
        }
    }

    #[test]
    fn test_parse_relay_without_a_line() {
        // Relay entry without IPv6 'a' line
        let input = r"r SharingIsCaring AAB3U5aCNzT5U9IsI48P6F2285A v8XNRBhhYXk+o+4+vHtyNSriAGU 2026-02-06 23:19:15 188.195.48.170 9001 0
s Fast HSDir Running Stable V2Dir Valid
v Tor 0.4.8.21
pr Conflux=1 Cons=1-2 Desc=1-2 DirCache=2 FlowCtrl=1-2 HSDir=2 HSIntro=4-5 HSRend=1-2 Link=1-5 LinkAuth=1,3 Microdesc=1-2 Padding=2 Relay=1-4
w Bandwidth=480
p reject 1-65535
";

        let result = debug_parse_relay_entry(input);
        match result {
            Ok((relay, _)) => {
                println!("✅ Parsed relay: {} at {}", relay.nickname, relay.address);
                assert_eq!(relay.nickname, "SharingIsCaring");
                assert_eq!(relay.or_port, 9001);
            }
            Err(e) => {
                panic!("❌ Parse failed: {e}");
            }
        }
    }

    #[test]
    fn test_parse_multiple_relays() {
        // Test the many0 parser on multiple entries
        // Use valid base64 (27 chars without explicit padding, padding added by our code)
        // The last character must end in 00 bits when padded with =
        // A=0 (000000) works, Q=16 (010000) works
        let input = r"r First AAAAAAAAAAAAAAAAAAAAAAAAAAA AAAAAAAAAAAAAAAAAAAAAAAAAAA 2026-02-07 00:00:00 1.2.3.4 443 0
s Fast Running Valid
w Bandwidth=1000
r Second QQQQQQQQQQQQQQQQQQQQQQQQQQQ QQQQQQQQQQQQQQQQQQQQQQQQQQQ 2026-02-07 00:00:00 5.6.7.8 9001 0
s Guard Running Stable Valid
w Bandwidth=2000
";

        // Skip to first r line
        let (input, _) = take_until::<&str, &str, nom::error::Error<&str>>("r ")(input)
            .expect("input contains 'r '");
        let result = many0(relay_entry)(input);

        match result {
            Ok((_remaining, relays)) => {
                println!("✅ Parsed {} relays", relays.len());
                for relay in &relays {
                    println!("   - {} at {}", relay.nickname, relay.address);
                }
                assert_eq!(relays.len(), 2, "Expected 2 relays");
            }
            Err(e) => {
                panic!("❌ Parse failed: {e:?}");
            }
        }
    }

    #[test]
    fn test_parse_full_consensus_document() {
        // Realistic consensus document with header and multiple relays
        let input = r"network-status-version 3
vote-status consensus
consensus-method 32
valid-after 2026-02-07 10:00:00
fresh-until 2026-02-07 11:00:00
valid-until 2026-02-07 13:00:00
voting-delay 300 300
client-versions 0.4.8.16,0.4.8.17,0.4.8.18
server-versions 0.4.8.16,0.4.8.17,0.4.8.18
known-flags Authority BadExit Exit Fast Guard HSDir Running Stable StaleDesc Valid V2Dir
shared-rand-previous-value 9 somebase64string
shared-rand-current-value 9 anotherbase64string
dir-source gabelmoo 131.188.40.189 131.188.40.189 80 443
contact someone@example.org
vote-digest ABC123
r lisdex AAAErLudKby6FyVrs1ko3b/Iq6k IE+F8M9BVgN0RmHuI0QtwsVqYhk 2026-02-07 02:28:49 152.53.144.50 8443 0
a [2a0a:4cc0:c1:2aac::1]:8443
s Fast Guard Running Stable V2Dir Valid
v Tor 0.4.8.22
pr Conflux=1 Cons=1-2 Desc=1-2 DirCache=2 FlowCtrl=1-2 HSDir=2 HSIntro=4-5 HSRend=1-2 Link=1-5 LinkAuth=1,3 Microdesc=1-2 Padding=2 Relay=1-4
w Bandwidth=83000
p reject 1-65535
r SharingIsCaring AAB3U5aCNzT5U9IsI48P6F2285A v8XNRBhhYXk+o+4+vHtyNSriAGU 2026-02-06 23:19:15 188.195.48.170 9001 0
s Fast HSDir Running Stable V2Dir Valid
v Tor 0.4.8.21
pr Conflux=1 Cons=1-2 Desc=1-2 DirCache=2 FlowCtrl=1-2 HSDir=2 HSIntro=4-5 HSRend=1-2 Link=1-5 LinkAuth=1,3 Microdesc=1-2 Padding=2 Relay=1-4
w Bandwidth=480
p reject 1-65535
r ExampleRelay BQQQQQQQQQQQQQQQQQQQQQQQQQA AAAAAAAAAAAAAAAAAAAAAAAAAAA 2026-02-07 01:00:00 93.115.95.201 9001 0
a [2001:db8::1]:9001
s Exit Fast Running Stable Valid
v Tor 0.4.8.20
pr Conflux=1 Cons=1-2 Desc=1-2 DirCache=2 FlowCtrl=1-2 HSDir=2 HSIntro=4-5 HSRend=1-2 Link=1-5 LinkAuth=1,3 Microdesc=1-2 Padding=2 Relay=1-4
w Bandwidth=25000
p accept 1-65535
directory-footer
";

        let result = parse_consensus(input);

        match result {
            Ok(relays) => {
                println!("✅ Full consensus parse: {} relays", relays.len());
                for relay in &relays {
                    println!(
                        "   - {} at {}:{} (bandwidth: {})",
                        relay.nickname, relay.address, relay.or_port, relay.bandwidth
                    );
                    println!("     flags: {:?}", relay.flags);
                }
                assert_eq!(relays.len(), 3, "Expected 3 relays");

                // Verify specific relay data
                assert_eq!(relays[0].nickname, "lisdex");
                assert_eq!(relays[0].or_port, 8443);
                assert_eq!(relays[0].bandwidth, 83000);
                assert!(relays[0].flags.contains(RelayFlags::GUARD));
                assert!(relays[0].flags.contains(RelayFlags::FAST));

                assert_eq!(relays[1].nickname, "SharingIsCaring");
                assert_eq!(relays[1].or_port, 9001);
                assert_eq!(relays[1].bandwidth, 480);
                assert!(relays[1].flags.contains(RelayFlags::HSDIR));

                assert_eq!(relays[2].nickname, "ExampleRelay");
                assert!(relays[2].flags.contains(RelayFlags::EXIT));
            }
            Err(e) => {
                panic!("❌ Full consensus parse failed: {e}");
            }
        }
    }
}
