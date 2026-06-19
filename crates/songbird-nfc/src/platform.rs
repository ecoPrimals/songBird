// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Platform-specific NFC device abstraction
//!
//! Provides a unified interface across targets. **Capability discovery** is explicit:
//! [`NfcDevice::new`] returns [`NfcError::PlatformUnsupported`]
//! until a native backend is wired for that OS.
//!
//! ## Discovery / integration paths (by target)
//!
//! - **Android** — JNI bridge to `android.nfc.NfcAdapter`, `NfcManager`, and tag technologies
//!   (`IsoDep`, `NfcA`, etc.). Discover adapters from Java/Kotlin (`NfcAdapter.getDefault()`),
//!   then surface session lifecycle + I/O into Rust via `jni-rs` (or equivalent).
//! - **iOS** — [CoreNFC](https://developer.apple.com/documentation/corenfc): `NFCNDEFReaderSession` /
//!   `NFCTagReaderSession` in Swift/Objective-C, bridged to Rust; capability is
//!   `NFCReaderUsageDescription` + device hardware, queried at runtime from the app bundle.
//! - **Linux** — [libnfc](https://github.com/nfc-tools/libnfc) or kernel NFC sockets (`AF_NFC`),
//!   typically after listing readers via `nfc-list` / udev; PC/SC may apply for some hardware.

use crate::error::{NfcError, Result};
use crate::protocol::NfcMessage;
use std::time::Duration;
use tracing::{debug, info};

/// Platform-specific backend (enum — no `dyn` dispatch).
#[cfg(test)]
pub(crate) enum NfcBackend {
    /// Scripted I/O for unit tests.
    Scripted(test_support::ScriptedBackend),
}

/// Empty until a native backend is compiled in for this target.
#[cfg(not(test))]
#[derive(Debug)]
pub(crate) enum NfcBackend {}

#[cfg_attr(
    not(test),
    allow(
        clippy::needless_pass_by_ref_mut,
        clippy::uninhabited_references,
        clippy::unused_async,
        reason = "non-test builds use an empty backend enum; methods are stubs until wired"
    )
)]
impl NfcBackend {
    async fn connect(&mut self, timeout: Duration) -> Result<()> {
        #[cfg(test)]
        {
            match self {
                Self::Scripted(b) => b.connect(timeout).await,
            }
        }
        #[cfg(not(test))]
        {
            let _ = timeout;
            match *self {}
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        #[cfg(test)]
        {
            match self {
                Self::Scripted(b) => b.disconnect().await,
            }
        }
        #[cfg(not(test))]
        {
            match *self {}
        }
    }

    async fn send(&mut self, data: &[u8]) -> Result<()> {
        #[cfg(test)]
        {
            match self {
                Self::Scripted(b) => b.send(data).await,
            }
        }
        #[cfg(not(test))]
        {
            let _ = data;
            match *self {}
        }
    }

    async fn receive(&mut self, expected_len: usize) -> Result<Vec<u8>> {
        #[cfg(test)]
        {
            match self {
                Self::Scripted(b) => b.receive(expected_len).await,
            }
        }
        #[cfg(not(test))]
        {
            let _ = expected_len;
            match *self {}
        }
    }
}

/// NFC device abstraction
///
/// Platform-agnostic interface for NFC operations.
pub struct NfcDevice {
    /// Platform-specific backend
    backend: NfcBackend,

    /// Connection timeout
    timeout: Duration,
}

impl std::fmt::Debug for NfcDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NfcDevice").field("timeout", &self.timeout).finish_non_exhaustive()
    }
}

impl NfcDevice {
    /// Create new NFC device
    ///
    /// # Errors
    ///
    /// Returns [`NfcError::PlatformUnsupported`] when
    /// no backend is linked for this target yet, or the host stack cannot be opened.
    pub fn new(timeout: Duration) -> Result<Self> {
        let backend = Self::create_platform_backend()?;

        Ok(Self {
            backend,
            timeout,
        })
    }

    /// Connect to peer device
    ///
    /// # Errors
    ///
    /// Returns an error if connection fails or times out.
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to NFC peer");
        self.backend.connect(self.timeout).await
    }

    /// Disconnect from peer
    ///
    /// # Errors
    ///
    /// Returns an error if disconnection fails.
    pub async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting from NFC peer");
        self.backend.disconnect().await
    }

    /// Send raw bytes
    ///
    /// # Errors
    ///
    /// Returns an error if the send operation fails.
    pub async fn send_raw(&mut self, data: &[u8]) -> Result<()> {
        debug!("Sending {} bytes", data.len());
        self.backend.send(data).await
    }

    /// Receive raw bytes
    ///
    /// # Errors
    ///
    /// Returns an error if receive fails or times out.
    pub async fn receive_raw(&mut self, expected_len: usize) -> Result<Vec<u8>> {
        debug!("Receiving {} bytes", expected_len);
        self.backend.receive(expected_len).await
    }

    /// Send NFC message
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or send fails.
    pub async fn send_message(&mut self, message: &NfcMessage) -> Result<()> {
        let bytes = message.to_bytes()?;
        self.send_raw(&bytes).await
    }

    /// Receive NFC message
    ///
    /// # Errors
    ///
    /// Returns an error if receive or deserialization fails.
    pub async fn receive_message(&mut self) -> Result<NfcMessage> {
        // First receive header to get payload length
        let header = self.receive_raw(crate::HEADER_SIZE).await?;

        let payload_len = u16::from_be_bytes([header[2], header[3]]) as usize;
        let total_len = crate::FRAME_OVERHEAD + payload_len;

        // Receive full frame
        let mut full_frame = header;
        let remaining = self.receive_raw(total_len - crate::HEADER_SIZE).await?;
        full_frame.extend_from_slice(&remaining);

        NfcMessage::from_bytes(&full_frame)
    }

    /// Create platform-specific backend
    fn create_platform_backend() -> Result<NfcBackend> {
        #[cfg(target_os = "android")]
        {
            // Capability path: JNI → `NfcAdapter` / tag tech; see module rustdoc.
            Err(NfcError::PlatformUnsupported(
                "Android NFC requires a JNI bridge to android.nfc (NfcAdapter, Tag, IsoDep, etc.)"
                    .to_string(),
            ))
        }

        #[cfg(target_os = "ios")]
        {
            // Capability path: CoreNFC in Swift + FFI; see module rustdoc.
            Err(NfcError::PlatformUnsupported(
                "iOS NFC requires CoreNFC (NFCNDEFReaderSession / tag sessions) via a Swift bridge"
                    .to_string(),
            ))
        }

        #[cfg(target_os = "linux")]
        {
            // Capability path: libnfc or AF_NFC; see module rustdoc.
            Err(NfcError::PlatformUnsupported(
                "Linux NFC requires libnfc or kernel NFC (AF_NFC) with a discovered USB/PCMCIA reader"
                    .to_string(),
            ))
        }

        #[cfg(not(any(target_os = "android", target_os = "ios", target_os = "linux")))]
        {
            Err(NfcError::PlatformUnsupported(
                "NFC backend not available for this target OS".to_string(),
            ))
        }
    }
}

#[cfg(test)]
impl NfcDevice {
    /// Construct with a custom backend for unit tests (same crate only).
    pub(crate) fn from_backend_for_test(backend: NfcBackend, timeout: Duration) -> Self {
        Self {
            backend,
            timeout,
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::redundant_pub_crate,
    reason = "test_support is crate-private; pub(crate) is intentional"
)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
pub(crate) mod test_support {
    //! In-crate test doubles for [`NfcBackend`].

    use super::NfcBackend;
    use super::NfcDevice;
    use crate::error::{NfcError, Result};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    /// Records sends and serves scripted receive bytes in FIFO order.
    pub(crate) struct ScriptedBackend {
        recv_buf: Arc<Mutex<Vec<u8>>>,
        sent_frames: Arc<Mutex<Vec<Vec<u8>>>>,
    }

    #[allow(
        clippy::needless_pass_by_ref_mut,
        clippy::unused_async,
        reason = "async shape matches future native backends; bodies are sync for the test double"
    )]
    impl ScriptedBackend {
        pub(crate) fn new(recv_buf: Vec<u8>) -> Self {
            Self {
                recv_buf: Arc::new(Mutex::new(recv_buf)),
                sent_frames: Arc::new(Mutex::new(Vec::new())),
            }
        }

        /// Clone of the shared send log for assertions after [`Self::into_device`].
        pub(crate) fn sent_frames_handle(&self) -> Arc<Mutex<Vec<Vec<u8>>>> {
            Arc::clone(&self.sent_frames)
        }

        pub(crate) fn into_device(self, timeout: Duration) -> NfcDevice {
            NfcDevice::from_backend_for_test(NfcBackend::Scripted(self), timeout)
        }

        pub(super) async fn connect(&mut self, _timeout: Duration) -> Result<()> {
            Ok(())
        }

        pub(super) async fn disconnect(&mut self) -> Result<()> {
            Ok(())
        }

        pub(super) async fn send(&mut self, data: &[u8]) -> Result<()> {
            self.sent_frames.lock().expect("scripted backend lock").push(data.to_vec());
            Ok(())
        }

        pub(super) async fn receive(&mut self, expected_len: usize) -> Result<Vec<u8>> {
            let mut buf = self.recv_buf.lock().expect("scripted backend lock");
            if buf.len() < expected_len {
                return Err(NfcError::ConnectionLost);
            }
            let chunk: Vec<u8> = buf.drain(..expected_len).collect();
            Ok(chunk)
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::NfcDevice;
    use super::test_support::ScriptedBackend;
    use crate::protocol::NfcMessage;
    use crate::{
        FRAME_OVERHEAD, HEADER_SIZE, MAX_PAYLOAD_SIZE, MSG_TYPE_GENESIS_REQUEST, PROTOCOL_VERSION,
        PUBLIC_KEY_SIZE, SIGNATURE_SIZE,
    };
    use std::time::Duration;

    #[test]
    fn new_reports_platform_unsupported() {
        let res = NfcDevice::new(Duration::from_secs(1));
        let err = res.expect_err("native NFC backend is not wired in this crate yet");
        match err {
            crate::NfcError::PlatformUnsupported(msg) => {
                assert!(
                    !msg.is_empty(),
                    "unsupported message should explain why NFC is unavailable"
                );
            }
            other => panic!("expected PlatformUnsupported, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn send_and_receive_message_roundtrip_on_scripted_backend() {
        let msg = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [0x11u8; PUBLIC_KEY_SIZE],
            [0x22u8; crate::NONCE_SIZE],
            vec![0x33u8; 7],
            [0x44u8; SIGNATURE_SIZE],
        );
        let wire = msg.to_bytes().expect("valid test frame");

        let backend = ScriptedBackend::new(Vec::new());
        let sent = backend.sent_frames_handle();
        let mut device = backend.into_device(Duration::from_secs(2));

        device.send_message(&msg).await.expect("send_message should serialize and send");
        let frames = sent.lock().expect("sent_frames lock");
        assert_eq!(frames.len(), 1, "send_message should produce exactly one raw frame");
        assert_eq!(frames[0], wire, "raw bytes should match NfcMessage::to_bytes");

        drop(frames);
        let mut device2 = ScriptedBackend::new(wire.clone()).into_device(Duration::from_secs(2));
        let got =
            device2.receive_message().await.expect("receive_message should parse scripted wire");
        assert_eq!(got.version, PROTOCOL_VERSION);
        assert_eq!(got.msg_type, MSG_TYPE_GENESIS_REQUEST);
        assert_eq!(got.encrypted_payload.len(), 7);
    }

    #[tokio::test]
    async fn receive_raw_errors_when_buffer_underruns() {
        let mut device = ScriptedBackend::new(vec![0u8; PUBLIC_KEY_SIZE - 1])
            .into_device(Duration::from_secs(1));
        let err = device
            .receive_raw(PUBLIC_KEY_SIZE)
            .await
            .expect_err("short buffer should surface connection loss");
        assert!(
            matches!(err, crate::NfcError::ConnectionLost),
            "expected ConnectionLost on underrun, got {err:?}"
        );
    }

    #[tokio::test]
    async fn connect_succeeds_on_scripted_backend() {
        let mut device = ScriptedBackend::new(Vec::new()).into_device(Duration::from_secs(2));
        device.connect().await.expect("scripted connect should succeed");
    }

    #[tokio::test]
    async fn disconnect_succeeds_on_scripted_backend() {
        let mut device = ScriptedBackend::new(Vec::new()).into_device(Duration::from_secs(2));
        device.disconnect().await.expect("scripted disconnect should succeed");
    }

    #[tokio::test]
    async fn send_raw_records_exact_bytes() {
        let backend = ScriptedBackend::new(Vec::new());
        let sent = backend.sent_frames_handle();
        let mut device = backend.into_device(Duration::from_secs(2));
        let payload = vec![0xde, 0xad, 0xbe, 0xef];

        device.send_raw(&payload).await.expect("send_raw should succeed");

        let frames = sent.lock().expect("sent_frames lock");
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0], payload);
    }

    #[tokio::test]
    async fn send_and_receive_raw_roundtrip() {
        let wire = vec![0x01, 0x02, 0x03, 0x04];
        let mut device = ScriptedBackend::new(wire.clone()).into_device(Duration::from_secs(2));
        let got =
            device.receive_raw(wire.len()).await.expect("receive_raw should return scripted bytes");
        assert_eq!(got, wire);
    }

    #[tokio::test]
    async fn receive_message_rejects_truncated_header() {
        let mut device =
            ScriptedBackend::new(vec![0u8; HEADER_SIZE - 1]).into_device(Duration::from_secs(1));
        let err = device
            .receive_message()
            .await
            .expect_err("truncated header should fail before frame parse");
        assert!(
            matches!(err, crate::NfcError::ConnectionLost),
            "expected ConnectionLost for short header, got {err:?}"
        );
    }

    #[tokio::test]
    async fn receive_message_rejects_length_field_beyond_buffer() {
        let mut wire = vec![PROTOCOL_VERSION, MSG_TYPE_GENESIS_REQUEST, 0x03, 0xe8];
        wire.extend(std::iter::repeat_n(0u8, 8));
        let mut device = ScriptedBackend::new(wire).into_device(Duration::from_secs(1));
        let err = device
            .receive_message()
            .await
            .expect_err("declared payload length larger than buffer should fail");
        assert!(
            matches!(err, crate::NfcError::ConnectionLost),
            "expected ConnectionLost when body underruns declared length, got {err:?}"
        );
    }

    #[tokio::test]
    async fn receive_message_rejects_unsupported_protocol_version() {
        let msg = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [0x11u8; PUBLIC_KEY_SIZE],
            [0x22u8; crate::NONCE_SIZE],
            vec![0x33u8; 4],
            [0x44u8; SIGNATURE_SIZE],
        );
        let mut wire = msg.to_bytes().expect("valid frame for mutation");
        wire[0] = 0x99;
        let mut device = ScriptedBackend::new(wire).into_device(Duration::from_secs(2));
        let err = device
            .receive_message()
            .await
            .expect_err("unsupported version should fail deserialization");
        assert!(
            matches!(err, crate::NfcError::UnsupportedVersion(0x99)),
            "expected UnsupportedVersion(0x99), got {err:?}"
        );
    }

    #[tokio::test]
    async fn receive_message_roundtrips_empty_payload() {
        let msg = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [0x55u8; PUBLIC_KEY_SIZE],
            [0x66u8; crate::NONCE_SIZE],
            Vec::new(),
            [0x77u8; SIGNATURE_SIZE],
        );
        let wire = msg.to_bytes().expect("empty payload frame");
        assert_eq!(wire.len(), FRAME_OVERHEAD);

        let mut device = ScriptedBackend::new(wire).into_device(Duration::from_secs(2));
        let got = device.receive_message().await.expect("empty payload should parse");
        assert!(got.encrypted_payload.is_empty());
        assert_eq!(got.msg_type, MSG_TYPE_GENESIS_REQUEST);
    }

    #[tokio::test]
    async fn receive_message_roundtrips_max_size_payload() {
        let payload = vec![0xabu8; MAX_PAYLOAD_SIZE];
        let msg = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [0x88u8; PUBLIC_KEY_SIZE],
            [0x99u8; crate::NONCE_SIZE],
            payload.clone(),
            [0xaau8; SIGNATURE_SIZE],
        );
        let wire = msg.to_bytes().expect("max payload frame");
        assert_eq!(wire.len(), FRAME_OVERHEAD + MAX_PAYLOAD_SIZE);

        let mut device = ScriptedBackend::new(wire).into_device(Duration::from_secs(2));
        let got = device.receive_message().await.expect("max payload should parse");
        assert_eq!(got.encrypted_payload.len(), MAX_PAYLOAD_SIZE);
        assert_eq!(got.encrypted_payload, payload);
    }

    #[tokio::test]
    async fn send_message_roundtrips_empty_payload() {
        let msg = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [0xbbu8; PUBLIC_KEY_SIZE],
            [0xccu8; crate::NONCE_SIZE],
            Vec::new(),
            [0xddu8; SIGNATURE_SIZE],
        );
        let backend = ScriptedBackend::new(Vec::new());
        let sent = backend.sent_frames_handle();
        let mut device = backend.into_device(Duration::from_secs(2));

        device.send_message(&msg).await.expect("empty payload send should succeed");

        let frames = sent.lock().expect("sent_frames lock");
        let wire = frames[0].clone();
        drop(frames);

        let mut reader = ScriptedBackend::new(wire).into_device(Duration::from_secs(2));
        let got = reader.receive_message().await.expect("empty payload receive should succeed");
        assert!(got.encrypted_payload.is_empty());
    }
}
