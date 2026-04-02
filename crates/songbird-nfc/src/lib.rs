// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Dark Forest compliant NFC genesis protocol
//!
//! # Overview
//!
//! `songbird-nfc` implements a zero-metadata-leakage NFC protocol for genesis ceremonies
//! and secure mobile device pairing. All cryptographic operations are delegated to `BearDog`.
//!
//! # Dark Forest Guarantees
//!
//! - **Zero metadata leakage**: No identifiable information in cleartext
//! - **Ephemeral keys**: Single-use X25519 keys for each exchange
//! - **Timing protection**: Constant-time operations, random delays
//! - **`BearDog` delegation**: All crypto operations via `BearDog` IPC
//! - **Zero unsafe code**: Pure Rust, memory-safe implementation
//!
//! # Protocol
//!
//! ```text
//! Initiator                           Responder
//! ========                           =========
//!
//! 1. Generate ephemeral X25519 keypair
//! 2. Send public key (32 bytes)  --->
//!                                <---  3. Receive public key
//!                                      4. Generate ephemeral keypair
//!                                      5. Compute shared secret
//! 6. Receive public key          <---  6. Send public key
//! 7. Compute shared secret
//! 8. Encrypt genesis             --->  9. Decrypt genesis
//! 10. Destroy ephemeral keys          11. Destroy ephemeral keys
//! ```
//!
//! # Wire Format
//!
//! ```text
//! [1 byte]   Protocol version (0x01)
//! [1 byte]   Message type (0x01 = genesis_request, 0x02 = genesis_response)
//! [2 bytes]  Payload length (big-endian, u16)
//! [32 bytes] Ephemeral public key (X25519)
//! [24 bytes] Nonce (ChaCha20-Poly1305)
//! [N bytes]  Encrypted payload (BearDog ChaCha20-Poly1305, includes 16-byte auth tag)
//! [64 bytes] Signature (ephemeral Ed25519)
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms, unreachable_pub)]
#![allow(
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    reason = "intentional pattern; clippy false positive for this API"
)]
#![cfg_attr(
    test,
    allow(
        deprecated,
        dead_code,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::await_holding_lock,
        clippy::float_cmp,
        clippy::absurd_extreme_comparisons,
        clippy::nonminimal_bool,
        clippy::needless_collect,
        clippy::used_underscore_binding,
        clippy::overly_complex_bool_expr,
        clippy::assertions_on_constants,
        clippy::unreadable_literal,
        clippy::empty_line_after_doc_comments,
        clippy::field_reassign_with_default,
        clippy::unnecessary_wraps,
        clippy::no_effect_underscore_binding,
        clippy::return_self_not_must_use,
        clippy::duplicated_attributes,
        clippy::needless_pass_by_value,
        clippy::must_use_candidate,
        clippy::missing_panics_doc,
        clippy::missing_errors_doc,
        clippy::doc_markdown,
        clippy::wildcard_imports,
        clippy::enum_glob_use,
        unused_imports,
        unused_variables,
        clippy::unused_self,
        clippy::unnecessary_cast,
        clippy::items_after_test_module,
        clippy::clone_on_ref_ptr,
        clippy::default_trait_access,
        clippy::needless_range_loop,
        clippy::similar_names,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::unnecessary_unwrap,
        clippy::ignore_without_reason,
        clippy::case_sensitive_file_extension_comparisons,
    )
)]

pub mod config;
pub mod error;
pub mod genesis;
pub mod platform;
pub mod protocol;
pub mod timing;

pub use config::NfcConfig;
pub use error::{NfcError, Result};
pub use genesis::GenesisExchange;
pub use protocol::{NfcMessage, NfcProtocol};

/// Protocol version
pub const PROTOCOL_VERSION: u8 = 0x01;

/// Message type: Genesis request
pub const MSG_TYPE_GENESIS_REQUEST: u8 = 0x01;

/// Message type: Genesis response
pub const MSG_TYPE_GENESIS_RESPONSE: u8 = 0x02;

/// Ephemeral public key size (X25519)
pub const PUBLIC_KEY_SIZE: usize = 32;

/// Nonce size (ChaCha20-Poly1305)
pub const NONCE_SIZE: usize = 24;

/// Signature size (Ed25519)
pub const SIGNATURE_SIZE: usize = 64;

/// Authentication tag size (Poly1305)
pub const AUTH_TAG_SIZE: usize = 16;

/// Maximum payload size (1KB - prevents memory exhaustion)
pub const MAX_PAYLOAD_SIZE: usize = 1024;

/// Wire format header size
pub const HEADER_SIZE: usize = 1 + 1 + 2; // version + type + length

/// Full frame overhead (header + pubkey + nonce + signature)
pub const FRAME_OVERHEAD: usize = HEADER_SIZE + PUBLIC_KEY_SIZE + NONCE_SIZE + SIGNATURE_SIZE;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn wire_constants_are_consistent() {
        assert_eq!(HEADER_SIZE, 4, "version + type + u16 length");
        assert_eq!(
            FRAME_OVERHEAD,
            HEADER_SIZE + PUBLIC_KEY_SIZE + NONCE_SIZE + SIGNATURE_SIZE,
            "FRAME_OVERHEAD should match header + fixed fields + signature"
        );
        assert_eq!(PROTOCOL_VERSION, 0x01);
        assert_eq!(PUBLIC_KEY_SIZE + NONCE_SIZE + SIGNATURE_SIZE, 32 + 24 + 64);
    }

    #[test]
    fn message_types_differ() {
        assert_ne!(
            MSG_TYPE_GENESIS_REQUEST,
            MSG_TYPE_GENESIS_RESPONSE,
            "request and response must be distinct opcodes"
        );
    }
}
