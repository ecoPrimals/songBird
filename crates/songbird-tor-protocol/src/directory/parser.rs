//! Consensus parsing with nom
//!
//! Parses Tor network consensus documents using nom combinator parsers.
//! Format: https://spec.torproject.org/dir-spec/formats.html

use nom::{
    IResult,
    bytes::complete::{tag, take_while1, take_until},
    character::complete::{digit1, space0, space1, line_ending, multispace0},
    combinator::{map_res, opt},
    multi::{many0, separated_list1},
    sequence::preceded,
};
use crate::directory::{RelayInfo, RelayFlags};
use crate::error::{Error, Result};
use std::net::{IpAddr, Ipv4Addr};

/// Parse full consensus document
pub fn parse_consensus(input: &str) -> Result<Vec<RelayInfo>> {
    match consensus_document(input) {
        Ok((_, relays)) => Ok(relays),
        Err(e) => Err(Error::Parse(format!("Consensus parse error: {:?}", e))),
    }
}

/// Parse consensus document
fn consensus_document(input: &str) -> IResult<&str, Vec<RelayInfo>> {
    let (input, _) = multispace0(input)?;
    
    // Parse header (skip to r-lines)
    let (input, _) = take_until("r ")(input)?;
    
    // Parse relay entries
    let (input, relays) = many0(relay_entry)(input)?;
    
    Ok((input, relays))
}

/// Parse single relay entry (r, s, v, w, p lines)
fn relay_entry(input: &str) -> IResult<&str, RelayInfo> {
    let (input, _) = multispace0(input)?;
    
    // r line: nickname identity published IP port
    let (input, (nickname, fingerprint, address, or_port)) = r_line(input)?;
    
    // s line: flags (GUARD, FAST, STABLE, etc.)
    let (input, flags) = s_line(input)?;
    
    // v line: version (optional, skip)
    let (input, _) = opt(v_line)(input)?;
    
    // w line: bandwidth
    let (input, bandwidth) = w_line(input)?;
    
    // p line: exit policy (optional, skip)
    let (input, _) = opt(p_line)(input)?;
    
    Ok((input, RelayInfo {
        nickname,
        fingerprint,
        address,
        or_port,
        dir_port: None,
        flags,
        bandwidth,
        ntor_key: None, // TODO: Parse from microdescriptors
        version: None,  // TODO: Parse from v line
    }))
}

/// Parse r line: r nickname identity published IP ORPort DirPort
fn r_line(input: &str) -> IResult<&str, (String, [u8; 20], IpAddr, u16)> {
    let (input, _) = tag("r ")(input)?;
    
    // Nickname
    let (input, nickname) = map_res(
        take_while1(|c: char| c.is_alphanumeric()),
        |s: &str| Ok::<_, Error>(s.to_string())
    )(input)?;
    let (input, _) = space1(input)?;
    
    // Identity (base64, 27 chars)
    let (input, identity_b64) = take_while1(|c: char| c.is_alphanumeric() || c == '+' || c == '/' || c == '=')(input)?;
    let fingerprint = base64_to_fingerprint(identity_b64)?;
    let (input, _) = space1(input)?;
    
    // Published time (YYYY-MM-DD HH:MM:SS format, skip)
    let (input, _) = take_until(" ")(input)?;
    let (input, _) = space1(input)?;
    let (input, _) = take_until(" ")(input)?;  // Skip time part too
    let (input, _) = space1(input)?;
    
    // IP address
    let (input, ip_str) = take_while1(|c: char| c.is_ascii_digit() || c == '.')(input)?;
    let address = ip_str.parse::<Ipv4Addr>()
        .map(IpAddr::V4)
        .map_err(|_| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Digit)))?;
    let (input, _) = space1(input)?;
    
    // OR port
    let (input, or_port) = map_res(digit1, |s: &str| s.parse::<u16>())(input)?;
    
    // Dir port (optional, skip)
    let (input, _) = opt(preceded(space1, digit1))(input)?;
    
    let (input, _) = line_ending(input)?;
    
    Ok((input, (nickname, fingerprint, address, or_port)))
}

/// Parse s line: s Flag1 Flag2 Flag3...
fn s_line(input: &str) -> IResult<&str, RelayFlags> {
    let (input, _) = tag("s")(input)?;
    let (input, _) = space0(input)?;
    
    let (input, flag_strs) = separated_list1(
        space1,
        take_while1(|c: char| c.is_alphanumeric())
    )(input)?;
    
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

/// Parse v line: v Tor version (skip)
fn v_line(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("v ")(input)?;
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
fn base64_to_fingerprint(b64: &str) -> std::result::Result<[u8; 20], nom::Err<nom::error::Error<&str>>> {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    
    let bytes = STANDARD.decode(b64)
        .map_err(|_| nom::Err::Failure(nom::error::Error::new(b64, nom::error::ErrorKind::Digit)))?;
    
    if bytes.len() != 20 {
        return Err(nom::Err::Failure(nom::error::Error::new(b64, nom::error::ErrorKind::Digit)));
    }
    
    let mut fingerprint = [0u8; 20];
    fingerprint.copy_from_slice(&bytes);
    
    Ok(fingerprint)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_r_line() {
        // Valid base64 that decodes to exactly 20 bytes (SHA1 fingerprint)
        let input = "r Test AAAAAAAAAAAAAAAAAAAAAAAAAAA= 2026-02-04 00:00:00 1.2.3.4 443 80\n";
        let result = r_line(input);
        
        if let Err(e) = &result {
            println!("Parse error: {:?}", e);
        }
        
        assert!(result.is_ok());
        
        let (_, (nickname, fingerprint, _address, or_port)) = result.unwrap();
        assert_eq!(nickname, "Test");
        assert_eq!(or_port, 443);
        assert_eq!(fingerprint.len(), 20);
    }
    
    #[test]
    fn test_parse_s_line() {
        let input = "s Fast Guard Running Stable Valid\n";
        let result = s_line(input);
        assert!(result.is_ok());
        
        let (_, flags) = result.unwrap();
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
        
        let (_, bandwidth) = result.unwrap();
        assert_eq!(bandwidth, 5000);
    }
}
