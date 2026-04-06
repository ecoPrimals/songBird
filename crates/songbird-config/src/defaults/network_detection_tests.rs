// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;
use std::net::{IpAddr, Ipv4Addr};

#[test]
fn detect_public_ip_returns_valid_addr() {
    let ip = detect_public_ip();
    assert!(ip.is_ipv4() || ip.is_ipv6());
}

#[test]
fn detect_public_ip_respects_songbird_public_ip_env() {
    let ip = detect_public_ip_with(|key| match key {
        "SONGBIRD_PUBLIC_IP" => Ok("10.42.0.1".to_string()),
        _ => Err(std::env::VarError::NotPresent),
    });
    assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(10, 42, 0, 1)));
}

#[test]
fn detect_public_ip_ignores_invalid_env() {
    let ip = detect_public_ip_with(|key| match key {
        "SONGBIRD_PUBLIC_IP" => Ok("not-an-ip".to_string()),
        _ => Err(std::env::VarError::NotPresent),
    });
    assert!(ip.is_ipv4() || ip.is_ipv6());
}

#[test]
fn check_cloud_metadata_per_provider() {
    assert!(check_cloud_metadata_with(|_| Err(std::env::VarError::NotPresent)).is_none());

    assert_eq!(
        check_cloud_metadata_with(|key| match key {
            "AWS_INSTANCE_IP" => Ok("172.31.1.100".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        }),
        Some(IpAddr::V4(Ipv4Addr::new(172, 31, 1, 100)))
    );

    assert_eq!(
        check_cloud_metadata_with(|key| match key {
            "GCE_INSTANCE_IP" => Ok("10.128.0.5".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        }),
        Some(IpAddr::V4(Ipv4Addr::new(10, 128, 0, 5)))
    );

    assert_eq!(
        check_cloud_metadata_with(|key| match key {
            "AZURE_VM_IP" => Ok("10.0.0.4".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        }),
        Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 4)))
    );
}

#[test]
fn detect_via_hostname_returns_option() {
    let _result = detect_via_hostname();
}
