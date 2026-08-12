// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::enrollment_crypto::{call_security_provider, load_family_seed_value};
use super::types::EnrollPhase;
use serde_json::{Value, json};

/// Deliver the family seed encrypted to the enrollee's `WireGuard` public key.
///
/// Uses bearDog's `crypto.encrypt` to wrap the seed before transit.
pub(super) async fn deliver_family_seed(wg_pubkey: &str) -> (EnrollPhase, Option<String>) {
    let family_seed = load_family_seed_value();

    let Some(seed) = family_seed else {
        return (
            EnrollPhase {
                name: "seed.deliver".into(),
                ok: false,
                detail: "FAMILY_SEED not available on hub".into(),
            },
            None,
        );
    };

    match call_security_provider(
        "crypto.encrypt",
        json!({
            "plaintext": seed,
            "recipient_key": wg_pubkey,
            "purpose": "gate_enrollment_seed_delivery",
        }),
    )
    .await
    {
        Ok(result) => {
            let ciphertext = result.get("ciphertext").and_then(Value::as_str).map(String::from);

            match ciphertext {
                Some(ct) => (
                    EnrollPhase {
                        name: "seed.deliver".into(),
                        ok: true,
                        detail: "family seed encrypted for enrollee".into(),
                    },
                    Some(ct),
                ),
                None => (
                    EnrollPhase {
                        name: "seed.deliver".into(),
                        ok: false,
                        detail: "crypto.encrypt returned no ciphertext".into(),
                    },
                    None,
                ),
            }
        }
        Err(e) => (
            EnrollPhase {
                name: "seed.deliver".into(),
                ok: false,
                detail: format!("bearDog crypto.encrypt unavailable: {e}"),
            },
            None,
        ),
    }
}
