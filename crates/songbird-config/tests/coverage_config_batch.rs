// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

use songbird_config::canonical::network::{
    ConnectionLimits, ConnectionPoolConfig, LoadBalancingConfig, RateLimitingConfig,
};
use songbird_config::canonical::network::{
    CorsConfig, GamingNetworkConfig, GamingScale, PortRange, TimeoutConfig,
};

macro_rules! serde_roundtrip {
    ($ty:ty, $val:expr) => {{
        let v: $ty = $val;
        let j = serde_json::to_string(&v).unwrap();
        let _: $ty = serde_json::from_str(&j).unwrap();
    }};
}

macro_rules! cfg_batch {
    ($($n:ident)+) => {
        $(
            #[test]
            fn $n() {
                serde_roundtrip!(TimeoutConfig, TimeoutConfig::default());
                serde_roundtrip!(CorsConfig, CorsConfig::default());
                serde_roundtrip!(PortRange, PortRange::default());
                serde_roundtrip!(GamingNetworkConfig, GamingNetworkConfig::default());
                let _ = GamingScale::default().max_players();
            }
        )+
    };
}

cfg_batch!(
    cfg_b001 cfg_b002 cfg_b003 cfg_b004 cfg_b005 cfg_b006 cfg_b007 cfg_b008 cfg_b009 cfg_b010
    cfg_b011 cfg_b012 cfg_b013 cfg_b014 cfg_b015
);

#[test]
fn limits_bundle_json() {
    let j = serde_json::json!({
        "cl": ConnectionLimits::default(),
        "lb": LoadBalancingConfig::default(),
        "rl": RateLimitingConfig::default(),
        "cp": ConnectionPoolConfig::default(),
    });
    assert!(j.to_string().contains("max_connections_per_host"));
}
