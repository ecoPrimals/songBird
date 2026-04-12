// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! TLS 1.3 Record Layer
//!
//! Handles record framing, encryption, and decryption per RFC 8446 Section 5.
//!
//! ## Record Format
//!
//! ```text
//! struct {
//!     ContentType type;           // 1 byte
//!     ProtocolVersion legacy_record_version = 0x0303; // 2 bytes (TLS 1.2)
//!     uint16 length;              // 2 bytes
//!     opaque fragment[length];    // variable length (encrypted after handshake)
//! } TLSPlaintext;
//! ```

mod crypto_provider;
mod framing;
mod layer;
mod record_crypto;

pub use crypto_provider::*;
pub use layer::*;
// `framing` / `record_crypto` attach inherent impls to [`RecordLayer`]; the glob has no public
// items but keeps the `mod` + `pub use *` layout consistent with other submodules.
#[allow(unused_imports, reason = "re-exports for downstream module use")]
pub use framing::*;
#[allow(unused_imports, reason = "re-exports for downstream module use")]
pub use record_crypto::*;

#[cfg(test)]
mod fuzz_style_record_parsing_tests;
#[cfg(test)]
mod supplemental_record_layer_tests;
#[cfg(test)]
mod tests;
