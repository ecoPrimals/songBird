// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Unit tests for `config::gaming`.

use serde_json::Value;

mod defaults;
mod serde_roundtrip;

pub(super) fn roundtrip<T>(v: &T)
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    let a: Value = serde_json::to_value(v).expect("serialize");
    let back: T = serde_json::from_value(a.clone()).expect("deserialize");
    assert_eq!(serde_json::to_value(&back).expect("serialize again"), a);
}
