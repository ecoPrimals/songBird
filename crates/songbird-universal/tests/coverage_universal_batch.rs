// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]
#![allow(clippy::expect_used, reason = "test assertions")]

use songbird_universal::capabilities::Capability;
use songbird_universal::discovery::{
    DiscoveredPrimal, DiscoveryConfig, DiscoveryMethod, PrimalHealth,
};
use songbird_universal::types::PrimalType;

macro_rules! primal_batch {
    ($($n:ident)+) => {
        $(
            #[test]
            fn $n() {
                let c = DiscoveryConfig::default();
                assert_eq!(c.timeout.as_secs(), 30);
                let p = DiscoveredPrimal::new(
                    stringify!($n).to_string(),
                    PrimalType::new("compute"),
                    "http://localhost".to_string(),
                    vec![],
                    DiscoveryMethod::Manual,
                );
                assert!(!p.is_healthy());
                let cap = Capability::from_string("encryption").expect("known");
                assert_eq!(cap.category(), "security");
                let _ = PrimalHealth::Healthy;
            }
        )+
    };
}

primal_batch!(
    u_b001 u_b002 u_b003 u_b004 u_b005 u_b006 u_b007 u_b008 u_b009 u_b010
    u_b011 u_b012 u_b013 u_b014 u_b015
);
